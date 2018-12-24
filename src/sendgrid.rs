// Copyright (C) 2018 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: Apache-2.0

use failure::format_err;
use serde::Deserialize;
use slog::error;
use std::collections::BTreeMap;

#[derive(Deserialize)]
pub(crate) struct SendGrid {
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

impl SendGrid {
    /// Return the known e-mail templates.
    pub(crate) fn email_templates(&self) -> Vec<String> {
        self.email_templates.keys().cloned().collect()
    }

    /// Returns the required fields for a respective e-mail template.
    pub(crate) fn required_fields_for_email(
        &self,
        template: &str,
        logger: &slog::Logger,
    ) -> Result<Vec<String>, failure::Error> {
        // Ensures the email template exists
        if !self.email_templates.contains_key(template) {
            error!(logger, "invalid template"; "template" => template);
            return Err(format_err!("Unknown template: {}", template));
        }

        // Collect all required fields
        Ok(self.email_templates[template]
            .required_fields
            .clone()
            .unwrap_or_else(|| vec![]))
    }
}
