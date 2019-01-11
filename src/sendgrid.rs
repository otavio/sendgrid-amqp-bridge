// Copyright (C) 2018 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: MIT

use crate::config;

pub(crate) struct SendGrid {
    config: config::SendGrid,
}

impl SendGrid {
    pub(crate) fn from_config(config: &config::Config) -> Self {
        Self {
            config: config.sendgrid.clone(),
        }
    }
}
