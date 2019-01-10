// Copyright (C) 2018 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: MIT

use crate::{amqp::AMQP, sendgrid::SendGrid};
use failure::ResultExt;
use serde::Deserialize;
use slog::debug;
use std::{fs::File, io::Read, path::Path};

#[derive(Deserialize)]
pub(crate) struct Config {
    pub(crate) amqp: AMQP,
    pub(crate) sendgrid: SendGrid,
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
