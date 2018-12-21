// Copyright (C) 2018 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]
#![allow(unused_variables)]

use exitfailure::ExitFailure;
use slog::{info, o, Drain};
use std::path::PathBuf;
use structopt::StructOpt;

mod build_info;
mod config;
mod sendgrid;

#[structopt(
    name = "sendgrid-amqp-bridge",
    author = "O.S. Systems Software LTDA. <contact@ossystems.com.br>",
    about = "A SendGrid AMQP Bridge."
)]
#[structopt(raw(version = "build_info::version()"))]
#[derive(StructOpt, Debug)]
struct Cli {
    /// Configuration file to use
    #[structopt(short = "c", long = "config")]
    config: PathBuf,
    /// Increase the verboseness level
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: usize,
}

fn main() -> Result<(), ExitFailure> {
    use crate::config::Config;

    let cli = Cli::from_args();
    let logger = init_logger(cli.verbose);

    info!(logger, "starting"; "version" => build_info::version());
    let config = Config::load(&cli.config, &logger)?;

    Ok(())
}

fn init_logger(verbosity: usize) -> slog::Logger {
    let drain = slog_term::term_compact()
        .filter_level(match verbosity {
            0 => slog::Level::Info,
            1 => slog::Level::Debug,
            _ => slog::Level::Trace,
        })
        .fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    slog::Logger::root(drain, o!())
}
