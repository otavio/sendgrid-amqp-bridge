// Copyright (C) 2018 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: Apache-2.0

use failure::{format_err, ResultExt};
use serde_derive::Deserialize;
use slog::{debug, error, trace};
use std::{collections::BTreeMap, fs::File, io::Read, path::Path};

#[derive(Deserialize)]
pub struct Config {
    sendgrid: SendGrid,
}

#[derive(Deserialize)]
struct SendGrid {
    api_key: String,
    sender: String,
    #[serde(with = "serde_with::rust::maps_duplicate_key_is_error")]
    email_templates: BTreeMap<String, EmailTemplate>,
}

#[derive(Deserialize)]
struct EmailTemplate {
    template_id: String,
    required_fields: Option<Vec<String>>,
}

impl Config {
    /// Load configuration from filesystem. The file is expected to be
    /// YAML compatible.
    pub fn load(path: &Path, logger: &slog::Logger) -> Result<Self, failure::Error> {
        let mut buf = String::new();
        File::open(path)
            .context(format!("opening configuration file {:?}", &path))?
            .read_to_string(&mut buf)
            .context("reading configuration file")?;

        let config: Self = serde_yaml::from_str(&buf).context("parsing the YAML configuration")?;

        debug!(
            logger,
            "loaded configuration file";
            "templates" => format!("{:?}",
            config.sendgrid.email_templates.keys())
        );

        config.sendgrid.email_templates.iter().for_each(|(k, v)| {
            trace!(
                logger,
                "template: {}, required_fields: {:?}",
                k,
                v.required_fields.clone().unwrap_or_else(|| vec![])
            );
        });

        Ok(config)
    }

    /// Returns the required fields for a respective e-mail template.
    pub fn required_fields_for_email(
        &self,
        template: &str,
        logger: &slog::Logger,
    ) -> Result<Vec<String>, failure::Error> {
        // Ensures the email template exists
        if !self.sendgrid.email_templates.contains_key(template) {
            error!(logger, "invalid template"; "template" => template);
            return Err(format_err!("Unknown template: {}", template));
        }

        // Collect all required fields
        Ok(self.sendgrid.email_templates[template]
            .required_fields
            .clone()
            .unwrap_or_else(|| vec![]))
    }
}
