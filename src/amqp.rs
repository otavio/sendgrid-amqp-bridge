// Copyright (C) 2018 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: MIT

use crate::config;

pub(crate) struct AMQP {
    config: config::AMQP,
}

impl AMQP {
    pub(crate) fn from_config(config: &config::Config) -> Self {
        Self {
            config: config.amqp.clone(),
        }
    }
}
