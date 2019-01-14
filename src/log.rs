// Copyright (C) 2018-2019 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: MIT

use slog::{o, Drain};
use std::str::FromStr;

#[derive(Debug)]
pub(crate) enum Output {
    Human,
    Json,
}

impl FromStr for Output {
    type Err = failure::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "human" => Ok(Output::Human),
            "json" => Ok(Output::Json),
            v => Err(failure::err_msg(format!(
                "{} is invalid. Supported values are 'human' and 'json'.",
                v
            ))),
        }
    }
}

pub(crate) fn init(verbosity: usize, format: Output) -> slog::Logger {
    let level = match verbosity {
        0 => slog::Level::Info,
        1 => slog::Level::Debug,
        _ => slog::Level::Trace,
    };

    let drain = match format {
        Output::Human => {
            let drain = slog_term::term_compact().filter_level(level).fuse();
            slog_async::Async::new(drain).build().fuse()
        }

        Output::Json => {
            let drain = slog_json::Json::default(std::io::stderr())
                .filter_level(level)
                .fuse();
            slog_async::Async::new(drain).build().fuse()
        }
    };

    slog::Logger::root(drain, o!())
}
