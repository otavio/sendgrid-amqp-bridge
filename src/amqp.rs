// Copyright (C) 2018 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: MIT

use failure::err_msg;
use futures::{
    future::{self, Either, Future, IntoFuture},
    stream::Stream,
};
use lapin::{
    channel::{BasicConsumeOptions, ConfirmSelectOptions, QueueDeclareOptions},
    client::{Client, ConnectionOptions},
    message::Delivery,
    types::FieldTable,
};
use serde::Deserialize;
use slog::{error, o, trace};
use std::net::SocketAddr;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpStream,
};

#[derive(Deserialize)]
pub(crate) struct AMQP {
    addr: SocketAddr,
    username: String,
    password: String,
    #[serde(default = "empty_vhost")]
    vhost: String,
    queue_name: String,
    #[serde(default = "empty_consumer_name")]
    consumer_name: String,
}

fn empty_vhost() -> String {
    "/".to_string()
}

fn empty_consumer_name() -> String {
    "sendgrid-amqp-bridge".to_string()
}

impl AMQP {
    pub(crate) fn into_future<F>(
        self,
        message_handler: F,
        logger: slog::Logger,
    ) -> impl Future<Item = Client<TcpStream>, Error = failure::Error> + Send + 'static
    where
        F: Fn(&Delivery, &slog::Logger) -> bool + Send + Sync + 'static,
    {
        let logger = logger.clone();
        let connect_logger = logger.clone();
        let err_logger = logger.clone();

        let AMQP {
            addr,
            username,
            password,
            vhost,
            queue_name,
            consumer_name,
        } = self;

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

                tokio::spawn(create_consumer(
                    &customer_client,
                    queue_name.clone(),
                    consumer_name.clone(),
                    message_handler,
                    logger.clone(),
                ));

                Ok(client)
            })
            .map_err(failure::Error::from)
    }
}

fn create_consumer<T, F>(
    client: &Client<T>,
    queue_name: String,
    consumer_name: String,
    message_handler: F,
    logger: slog::Logger,
) -> impl Future<Item = (), Error = ()> + Send + 'static
where
    T: AsyncRead + AsyncWrite + Send + Sync + 'static,
    F: Fn(&Delivery, &slog::Logger) -> bool + Send + Sync + 'static,
{
    let err_logger = logger.clone();

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
        .map_err(move |err| error!(err_logger, "got error in consumer: {:?}", err))
        .and_then(move |(channel, stream, logger)| {
            stream
                .for_each(move |message| {
                    tokio::spawn(future::lazy(|| {
                        if message_handler(&message, &logger) {
                            Either::A(channel.basic_ack(message.delivery_tag, false))
                        } else {
                            Either::B(channel.basic_nack(message.delivery_tag, false, true))
                        }
                        .map_err(move |err| error!(err_logger, "got error in consumer: {}", err))
                    }));

                    Ok(())
                })
                .map_err(move |err| error!(err_logger, "got error in consumer: {:?}", err))
        })
        .map_err(move |err| error!(err_logger, "got error in consumer: {:?}", err))
}
