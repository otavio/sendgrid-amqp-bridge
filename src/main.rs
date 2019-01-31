// Copyright (C) 2018 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: MIT

use crate::{amqp::AMQP, config::Config, sendgrid::SendGrid};
use exitfailure::ExitFailure;
use slog::info;
use std::path::PathBuf;
use structopt::StructOpt;
use tokio::runtime::Runtime;

mod amqp;
mod build_info;
mod config;
mod log;
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
    /// Log output to use ('human' or 'json')
    #[structopt(short = "l", long = "log", default_value = "human")]
    log: log::Output,
}

fn main() -> Result<(), ExitFailure> {
    let cli = Cli::from_args();
    let logger = log::init(cli.verbose, cli.log);

    info!(logger, "starting"; "version" => build_info::version());
    let config = Config::load(&cli.config, &logger)?;
    let amqp = AMQP::from_config(&config);
    let sendgrid = SendGrid::from_config(&config);

    Runtime::new()?.block_on_all(amqp.create_consumers(sendgrid, logger))?;

    Ok(())
}
