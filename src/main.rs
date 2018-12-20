// Copyright (C) 2018 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: Apache-2.0

use exitfailure::ExitFailure;
use std::path::PathBuf;
use structopt::StructOpt;

#[structopt(
    name = "sendgrid-amqp-bridge",
    author = "O.S. Systems Software LTDA. <contact@ossystems.com.br>",
    about = "A SendGrid AMQP Bridge."
)]
#[derive(StructOpt, Debug)]
struct Cli {
    /// Configuration file to use
    #[structopt(short = "c", long = "config")]
    config: PathBuf,
}

fn main() -> Result<(), ExitFailure> {
    let _cli = Cli::from_args();

    Ok(())
}
