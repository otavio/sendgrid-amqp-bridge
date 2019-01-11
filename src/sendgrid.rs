// Copyright (C) 2018 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: MIT

use crate::{amqp, config, payload};
use slog::{error, info};

#[derive(Clone)]
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

impl amqp::MessageHandler for SendGrid {
    fn handle(self, message: &lapin::message::Delivery, logger: &slog::Logger) -> bool {
        match serde_json::from_slice::<payload::Message>(&message.data) {
            Ok(msg) => match self.config.required_fields_for_email(&msg.kind) {
                Ok(fields) => {
                    if fields.iter().all(|field| msg.fields.contains_key(field)) {
                        info!(
                            logger, "email delivered";
                            "type" => msg.kind,
                            "destination_name" => msg.destination_name,
                            "destination_email" => msg.destination_email,
                        );

                        return true;
                    } else {
                        error!(
                            logger, "missing required field for email";
                            "type" => msg.kind,
                            "destination_name" => msg.destination_name,
                            "destination_email" => msg.destination_email,
                            "required_fields" => format!("{:?}", fields),
                            "message_fields" => format!("{:?}", msg.fields.keys()),
                        )
                    }
                }
                Err(e) => error!(logger, "fail to parse the payload: {}", e),
            },
            Err(e) => error!(logger, "fail to parse the payload: {}", e),
        }

        false
    }
}
