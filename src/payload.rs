// Copyright (C) 2019 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: MIT

use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug, PartialEq)]
pub struct Message {
    #[serde(rename = "type")]
    pub kind: String,
    pub destination_email: String,
    pub destination_name: String,
    #[serde(default)]
    pub fields: HashMap<String, String>,
}

#[test]
fn valid_message() {
    use serde_json::json;

    assert_eq!(
        Message {
            kind: "payment_confirm".to_string(),
            destination_email: "robertosilva@gmail.com".to_string(),
            destination_name: "Roberto Silva".to_string(),
            fields: [("owner_uid".to_string(), "ns-1".to_string())]
                .iter()
                .cloned()
                .collect(),
        },
        serde_json::from_str(
            &json!(
                {
                    "destination_email": "robertosilva@gmail.com".to_string(),
                    "destination_name": "Roberto Silva".to_string(),
                    "type": "payment_confirm".to_string(),
                    "fields": {
                        "owner_uid": "ns-1".to_string()
                    }
                }
            )
            .to_string()
        )
        .unwrap()
    );

    assert_eq!(
        Message {
            kind: "payment_confirm".to_string(),
            destination_email: "robertosilva@gmail.com".to_string(),
            destination_name: "Roberto Silva".to_string(),
            fields: HashMap::new(),
        },
        serde_json::from_str(
            &json!(
                {
                    "destination_email": "robertosilva@gmail.com".to_string(),
                    "destination_name": "Roberto Silva".to_string(),
                    "type": "payment_confirm".to_string()
                }
            )
            .to_string()
        )
        .unwrap()
    );
}
