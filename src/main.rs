// Copyright (C) 2018 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: MIT

#![allow(dead_code)]
#![allow(unused_variables)]

use exitfailure::ExitFailure;
use lapin::message::Delivery;
use slog::{info, o, Drain};
use std::path::PathBuf;
use structopt::StructOpt;

mod amqp;
mod build_info;
mod config;
mod payload;
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
    use tokio::runtime::Runtime;

    let cli = Cli::from_args();
    let logger = init_logger(cli.verbose);

    info!(logger, "starting"; "version" => build_info::version());
    let config = Config::load(&cli.config, &logger)?;

    Runtime::new()?.block_on_all(config.amqp.into_future(SimpleHandler, logger))?;

    Ok(())
}

#[derive(Clone)]
struct SimpleHandler;

impl amqp::Handler for SimpleHandler {
    fn handle(self, message: &Delivery, logger: &slog::Logger) -> bool {
        slog::trace!(
            logger,
            "got '{}'",
            std::str::from_utf8(&message.data).unwrap()
        );

        true
    }
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
