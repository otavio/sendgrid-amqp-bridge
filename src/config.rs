// Copyright (C) 2018 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: MIT

use failure::{format_err, ResultExt};
use serde::Deserialize;
use slog::debug;
use std::{collections::BTreeMap, fs::File, io::Read, path::Path};

#[derive(Deserialize)]
pub(crate) struct Config {
    pub(crate) amqp: AMQP,
    pub(crate) sendgrid: SendGrid,
}

#[derive(Deserialize, Clone)]
pub(crate) struct AMQP {
    pub(crate) addr: String,
    pub(crate) username: String,
    pub(crate) password: String,
    #[serde(default = "empty_vhost")]
    pub(crate) vhost: String,
    pub(crate) exchange_name: String,
    pub(crate) routing_key: String,
    pub(crate) queue_name: String,
    #[serde(default = "empty_consumer_name")]
    pub(crate) consumer_name: String,
    #[serde(default = "num_cpus::get")]
    pub(crate) workers: usize,
}

fn empty_vhost() -> String {
    "/".to_string()
}

fn empty_consumer_name() -> String {
    "sendgrid-amqp-bridge".to_string()
}

#[derive(Deserialize, Clone)]
pub(crate) struct SendGrid {
    pub(crate) api_key: String,
    pub(crate) sender_name: String,
    pub(crate) sender_email: String,
    #[serde(with = "serde_with::rust::maps_duplicate_key_is_error")]
    email_templates: BTreeMap<String, EmailTemplate>,
}

#[derive(Deserialize, Clone)]
struct EmailTemplate {
    template_id: String,
    required_fields: Option<Vec<String>>,
}

impl SendGrid {
    /// Returns the required fields for a respective e-mail template.
    pub(crate) fn required_fields_for_email(
        &self,
        template: &str,
    ) -> Result<Vec<String>, failure::Error> {
        // Ensures the email template exists
        if !self.email_templates.contains_key(template) {
            return Err(format_err!("Unknown template: {}", template));
        }

        // Collect all required fields
        Ok(self.email_templates[template]
            .required_fields
            .clone()
            .unwrap_or_else(|| vec![]))
    }

    pub(crate) fn template_id(&self, template: &str) -> Result<String, failure::Error> {
        // Ensures the email template exists
        if !self.email_templates.contains_key(template) {
            return Err(format_err!("Unknown template: {}", template));
        }

        Ok(self.email_templates[template].template_id.clone())
    }
}

impl Config {
    /// Load configuration from filesystem. The file is expected to be
    /// YAML compatible.
    pub(crate) fn load(path: &Path, logger: &slog::Logger) -> Result<Self, failure::Error> {
        debug!(logger, "Loading configuration file {:?}", &path);

        let mut buf = String::new();
        File::open(path)
            .context(format!("opening configuration file {:?}", &path))?
            .read_to_string(&mut buf)
            .context("reading configuration file")?;

        Ok(serde_yaml::from_str(&buf).context("parsing the YAML configuration")?)
    }
}
