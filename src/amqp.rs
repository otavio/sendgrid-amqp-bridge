// Copyright (C) 2018 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: MIT

use crate::config;
use lapin::{
    message::Delivery,
    options::{
        BasicAckOptions, BasicConsumeOptions, BasicNackOptions, ConfirmSelectOptions,
        ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions,
    },
    types::FieldTable,
    Channel, Connection, ConnectionProperties, ExchangeKind,
};
use slog::{error, o, trace};

pub trait MessageHandler: Clone + Send {
    fn handle(self, message: &Delivery, logger: &slog::Logger) -> bool;
}

pub(crate) struct AMQP {
    config: config::AMQP,
}

impl AMQP {
    pub(crate) fn from_config(config: &config::Config) -> Self {
        Self {
            config: config.amqp.clone(),
        }
    }

    pub(crate) async fn create_consumers<F>(
        self,
        message_handler: F,
        logger: slog::Logger,
    ) -> Result<Connection, lapin::Error>
    where
        F: MessageHandler + 'static,
    {
        let config::AMQP {
            dsn,
            queue_name,
            exchange_name,
            routing_key,
            consumer_name,
        } = self.config;

        trace!(
            logger, "connecting to AMQP server";
            "dsn" => &dsn,
        );

        let conn = Connection::connect(&dsn, ConnectionProperties::default()).await?;
        let channel = conn.create_channel().await?;
        tokio::spawn(async move {
            if let Err(err) = create_consumer(
                channel,
                queue_name,
                exchange_name,
                routing_key,
                consumer_name,
                message_handler,
                logger.clone(),
            )
            .await
            {
                error!(logger, "got error in consumer: {}", err);
            }
        });

        Ok(conn)
    }
}

async fn create_consumer<F>(
    channel: Channel,
    queue_name: String,
    exchange_name: String,
    consumer_name: String,
    routing_key: String,
    message_handler: F,
    logger: slog::Logger,
) -> Result<(), lapin::Error>
where
    F: MessageHandler + 'static,
{
    channel
        .confirm_select(ConfirmSelectOptions::default())
        .await?;

    let logger = logger.new(o!("channel" => channel.id()));
    trace!(logger, "creating queue {}", &queue_name);
    let queue = channel
        .queue_declare(
            &queue_name,
            QueueDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;

    channel
        .exchange_declare(
            &exchange_name,
            ExchangeKind::Direct,
            ExchangeDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;

    channel
        .queue_bind(
            &queue.name().as_str(),
            &exchange_name,
            &routing_key,
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await?;

    let logger = logger.new(o!("queue" =>  queue.name().as_str().to_owned()));
    trace!(logger, "creating consumer");
    let stream = channel
        .basic_consume(
            &queue.name().as_str(),
            &consumer_name,
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    for message in stream.into_iter() {
        let (channel, message) = message.expect("error caught in in consumer");
        if message_handler.clone().handle(&message, &logger) {
            channel
                .basic_ack(message.delivery_tag, BasicAckOptions { multiple: false })
                .await?;
        } else {
            channel
                .basic_nack(
                    message.delivery_tag,
                    BasicNackOptions {
                        multiple: false,
                        requeue: true,
                    },
                )
                .await?;
        }
    }

    Ok(())
}
