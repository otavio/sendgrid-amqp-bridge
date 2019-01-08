// Copyright (C) 2018 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: MIT

use serde::Deserialize;
use std::net::SocketAddr;

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
