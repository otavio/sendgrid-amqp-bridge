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
                        send_email(&self.config, &msg, logger);
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

fn send_email(config: &config::SendGrid, payload: &payload::Message, logger: &slog::Logger) {
    use sendgrid_api::v3::*;

    let mut req = SGMailV3::new();
    req.set_template_id(&config.template_id(&payload.kind).unwrap());

    let mut from = Email::new();
    from.set_email(&config.sender_email);
    from.set_name(&config.sender_name);
    req.set_from(from);

    let mut personalization = Personalization::new();

    let mut to = Email::new();
    to.set_email(&payload.destination_email);
    to.set_name(&payload.destination_name);
    personalization.add_to(to);

    personalization.add_dynamic_template_data(payload.fields.clone());
    req.add_personalization(personalization);

    if let Some(attachment) = &payload.attachment {
        let mut a = Attachment::new();
        a.set_base64_content(&attachment.content);
        a.set_filename(&attachment.name);
        req.add_attachment(a);
    }

    let sender = V3Sender::new(config.api_key.clone());
    match sender.send(&req) {
        Ok(ref res) if res.status().is_success() => info!(
            logger, "email delivered";
            "type" => &payload.kind,
            "destination_name" => &payload.destination_name,
            "destination_email" => &payload.destination_email,
            "status" => res.status().as_str()
        ),
        Ok(ref mut res) => error!(
            logger, "fail to send email: {}", res.text().unwrap();
            "type" => &payload.kind,
            "destination_name" => &payload.destination_name,
            "destination_email" => &payload.destination_email,
            "status" => res.status().as_str(),
        ),
        Err(e) => error!(logger, "fail to send email: {}", e),
    };
}
