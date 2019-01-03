// Copyright 2017-2018 FST Network Pte. Ltd.
// This file is part of FST Relayer.

// FST Relayer is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// FST Relayer is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with FST Relayer. If not, see <http://www.gnu.org/licenses/>.
#![deny(unused_must_use)]

#[macro_use]
extern crate log;
extern crate env_logger;

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate clap;

extern crate tokio;
extern crate tokio_signal;
extern crate tokio_timer;

extern crate dirs;
extern crate ethereum_types;
extern crate ethkey;
extern crate ethstore;
extern crate futures;
extern crate parking_lot;
extern crate rustc_hex;

extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate toml;

extern crate jsonrpc_core;
#[macro_use]
extern crate jsonrpc_macros;
extern crate jsonrpc_http_server;
extern crate jsonrpc_ipc_server;
extern crate jsonrpc_pubsub;
// extern crate jsonrpc_ws_server;

extern crate collation;
extern crate ethereum;
extern crate fdlimit;
extern crate machine;
extern crate network;
extern crate pool;
extern crate pricer;
extern crate traits;
extern crate types;

mod cli;
mod jsonrpc;
mod rpc_apis;
mod run_cmd;
mod server;
mod service;

use run_cmd::Command;
use server::Server;
use service::config;

fn main() {
    let command = {
        let mut app = cli::build();
        let matches = app.clone().get_matches();

        match matches.subcommand() {
            ("generate-config", Some(_)) => Command::GenerateConfiguration,
            ("service", Some(cmd)) => {
                use std::path::PathBuf;
                let config_file_path =
                    cmd.value_of("config")
                        .map(PathBuf::from)
                        .unwrap_or(match dirs::config_dir() {
                            Some(data_dir) => data_dir.join(crate_name!()).join("config.toml"),
                            None => PathBuf::from("config.toml"),
                        });

                fdlimit::raise_fd_limit();
                Command::Service {
                    config_file_path: config_file_path.to_owned(),
                    daemonize: false,
                }
            }
            ("completions", Some(cmd)) => {
                let shell = match cmd.subcommand() {
                    ("bash", _) => clap::Shell::Bash,
                    ("fish", _) => clap::Shell::Fish,
                    ("zsh", _) => clap::Shell::Zsh,
                    ("powershell", _) => clap::Shell::PowerShell,
                    _ => {
                        app.print_help().unwrap();
                        return;
                    }
                };
                app.gen_completions_to(crate_name!(), shell, &mut std::io::stdout());
                Command::ExitSuccess
            }
            ("help", Some(_)) => {
                app.print_help().unwrap();
                Command::ExitSuccess
            }
            ("version", Some(_)) => {
                println!("{} {}", crate_name!(), crate_version!());
                Command::ExitSuccess
            }
            (_, _) => {
                app.print_help().unwrap();
                Command::ExitFailure
            }
        }
    };

    command.run();
}
