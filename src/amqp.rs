// Copyright (C) 2018 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: MIT

use crate::config;
use failure::err_msg;
use futures::{
    future::{Either, Future, IntoFuture},
    stream::Stream,
};
use lapin::{
    message::Delivery,
    options::{
        BasicConsumeOptions, ConfirmSelectOptions, ExchangeDeclareOptions, QueueBindOptions,
        QueueDeclareOptions,
    },
    types::FieldTable,
    Client, ConnectionProperties, ExchangeKind,
};
use slog::{error, o, trace};

pub trait MessageHandler: Clone + Send + Sync {
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

    pub(crate) fn create_consumers<F>(
        self,
        message_handler: F,
        logger: slog::Logger,
    ) -> impl Future<Item = Client, Error = failure::Error> + Send + 'static
    where
        F: MessageHandler + 'static,
    {
        let logger = logger.clone();
        let connect_logger = logger.clone();

        let config::AMQP {
            dsn,
            queue_name,
            exchange_name,
            routing_key,
            consumer_name,
            workers,
        } = self.config;

        trace!(
            connect_logger, "connecting to AMQP server";
            "dsn" => &dsn,
        );

        Client::connect(&dsn, ConnectionProperties::default())
            .map_err(failure::Error::from)
            .and_then(move |client| {
                let customer_client = client.clone();

                futures::stream::iter_ok(0..workers)
                    .for_each(move |_| {
                        tokio::spawn(create_consumer(
                            &customer_client,
                            queue_name.clone(),
                            exchange_name.clone(),
                            routing_key.clone(),
                            consumer_name.clone(),
                            message_handler.clone(),
                            logger.clone(),
                        ))
                    })
                    .into_future()
                    .map(move |_| client)
                    .map_err(|_| err_msg("Couldn't spawn the consumer task"))
            })
            .map_err(failure::Error::from)
    }
}

fn create_consumer<F>(
    client: &Client,
    queue_name: String,
    exchange_name: String,
    consumer_name: String,
    routing_key: String,
    message_handler: F,
    logger: slog::Logger,
) -> impl Future<Item = (), Error = ()> + Send + 'static
where
    F: MessageHandler + 'static,
{
    let err_logger = logger.clone();
    let exchange_bind = exchange_name.clone();

    client
        .create_channel()
        .and_then(move |channel| {
            let logger = logger.new(o!("channel" => channel.id()));

            channel.confirm_select(ConfirmSelectOptions::default());

            trace!(logger, "creating queue {}", &queue_name);
            channel
                .queue_declare(
                    &queue_name,
                    QueueDeclareOptions {
                        durable: true,
                        ..Default::default()
                    },
                    FieldTable::default(),
                )
                .map(move |queue| (channel, queue, logger))
        })
        .and_then(move |(channel, queue, logger)| {
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
                .map(move |_| (channel, queue, logger))
        })
        .and_then(move |(channel, queue, logger)| {
            channel
                .queue_bind(
                    &queue.name().as_str(),
                    &exchange_bind,
                    &routing_key,
                    QueueBindOptions::default(),
                    FieldTable::default(),
                )
                .map(move |_| (channel, queue, logger))
        })
        .and_then(move |(channel, queue, logger)| {
            let logger = logger.new(o!("queue" => queue.name().as_str().to_owned()));

            trace!(logger, "creating consumer");
            channel
                .basic_consume(
                    &queue,
                    &consumer_name,
                    BasicConsumeOptions::default(),
                    FieldTable::default(),
                )
                .map(move |stream| (channel, stream, logger))
        })
        .and_then(move |(channel, stream, logger)| {
            stream.for_each(move |message| {
                if message_handler.clone().handle(&message, &logger) {
                    Either::A(channel.basic_ack(message.delivery_tag, false))
                } else {
                    Either::B(channel.basic_nack(message.delivery_tag, false, true))
                }
            })
        })
        .map_err(move |err| error!(err_logger, "got error in consumer: {}", err))
}
