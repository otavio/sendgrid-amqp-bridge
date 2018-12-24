// Copyright (C) 2018 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: Apache-2.0

use crate::sendgrid::SendGrid;
use failure::ResultExt;
use serde::Deserialize;
use slog::{debug, trace};
use std::{fs::File, io::Read, path::Path};

#[derive(Deserialize)]
pub struct Config {
    sendgrid: SendGrid,
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
            config.sendgrid.email_templates())
        );

        for template in &config.sendgrid.email_templates() {
            trace!(
                logger,
                "template: {}, required_fields: {:?}",
                template,
                config
                    .sendgrid
                    .required_fields_for_email(template, logger)
                    .unwrap()
            )
        }

        Ok(config)
    }
}
