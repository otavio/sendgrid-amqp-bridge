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
    channel::{
        BasicConsumeOptions, ConfirmSelectOptions, ExchangeDeclareOptions, QueueBindOptions,
        QueueDeclareOptions,
    },
    client::{Client, ConnectionOptions},
    message::Delivery,
    types::FieldTable,
};
use slog::{error, o, trace};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpStream,
};

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
    ) -> impl Future<Item = Client<TcpStream>, Error = failure::Error> + Send + 'static
    where
        F: MessageHandler + 'static,
    {
        let logger = logger.clone();
        let connect_logger = logger.clone();
        let err_logger = logger.clone();

        let config::AMQP {
            addr,
            username,
            password,
            vhost,
            queue_name,
            exchange_name,
            routing_key,
            consumer_name,
        } = self.config;

        TcpStream::connect(&addr)
            .map_err(|e| err_msg(format!("Couldn't connect to {}", e)))
            .and_then(move |stream| {
                trace!(
                    connect_logger, "connecting to AMQP server";
                    "addr" => addr,
                    "vhost" => &vhost,
                );

                Client::connect(
                    stream,
                    ConnectionOptions {
                        username,
                        password,
                        vhost,
                        heartbeat: 20,
                        ..Default::default()
                    },
                )
                .map_err(failure::Error::from)
            })
            .and_then(|(client, heartbeat)| {
                tokio::spawn(
                    heartbeat.map_err(move |e| error!(err_logger, "heartbeat error: {}", e)),
                );

                Ok(client)
            })
            .and_then(move |client| {
                let customer_client = client.clone();

                futures::stream::iter_ok(0..5)
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

fn create_consumer<T, F>(
    client: &Client<T>,
    queue_name: String,
    exchange_name: String,
    consumer_name: String,
    routing_key: String,
    message_handler: F,
    logger: slog::Logger,
) -> impl Future<Item = (), Error = ()> + Send + 'static
where
    F: MessageHandler + 'static,
    T: AsyncRead + AsyncWrite + Send + Sync + 'static,
{
    let err_logger = logger.clone();
    let exchange_bind = exchange_name.clone();

    client
        .create_confirm_channel(ConfirmSelectOptions::default())
        .and_then(move |channel| {
            let logger = logger.new(o!("channel" => channel.id));

            trace!(logger, "creating queue {}", &queue_name);
            channel
                .queue_declare(
                    &queue_name,
                    QueueDeclareOptions {
                        durable: true,
                        ..Default::default()
                    },
                    FieldTable::new(),
                )
                .map(move |queue| (channel, queue, logger))
        })
        .and_then(move |(channel, queue, logger)| {
            channel
                .exchange_declare(
                    &exchange_name,
                    "direct",
                    ExchangeDeclareOptions {
                        durable: true,
                        ..Default::default()
                    },
                    FieldTable::new(),
                )
                .map(move |_| (channel, queue, logger))
        })
        .and_then(move |(channel, queue, logger)| {
            channel
                .queue_bind(
                    &queue.name(),
                    &exchange_bind,
                    &routing_key,
                    QueueBindOptions::default(),
                    FieldTable::new(),
                )
                .map(move |_| (channel, queue, logger))
        })
        .and_then(move |(channel, queue, logger)| {
            let logger = logger.new(o!("queue" => queue.name()));

            trace!(logger, "creating consumer");
            channel
                .basic_consume(
                    &queue,
                    &consumer_name,
                    BasicConsumeOptions::default(),
                    FieldTable::new(),
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
