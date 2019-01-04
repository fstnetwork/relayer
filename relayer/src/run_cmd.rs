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
use parking_lot::Mutex;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::runtime::Runtime;

use super::config::{self, Configuration};
use super::service::Service;
use super::Server;

pub enum Command {
    ExitSuccess,
    ExitFailure,
    GenerateConfiguration,
    Service {
        config_file_path: PathBuf,
        daemonize: bool,
    },
}

impl Command {
    pub fn run(self) {
        let exit_code = match self {
            Command::ExitSuccess => 0,
            Command::ExitFailure => -1,
            Command::GenerateConfiguration => Self::generate_config(),
            Command::Service {
                config_file_path,
                daemonize,
            } => Self::run_service(config_file_path, daemonize),
        };

        ::std::process::exit(exit_code);
    }

    fn run_service(config_file_path: PathBuf, _daemonize: bool) -> i32 {
        //  env_logger::init();

        let mut logger_builder = env_logger::Builder::from_default_env();
        // logger_builder.filter(None, log::LevelFilter::Trace).init();
        // logger_builder.filter(None, log::LevelFilter::Info).init();

        let log_level = log::LevelFilter::Info;

        logger_builder
            .filter(Some("system"), log_level)
            .filter(Some("relayer"), log_level)
            .filter(Some("pool"), log_level)
            .filter(Some("network"), log_level)
            .filter(Some("ethereum"), log_level)
            .filter(Some("ethereum_monitor"), log_level)
            .init();

        let config = match config::load_config(&config_file_path) {
            Ok(config) => config,
            Err(err) => {
                println!("{:?}", err);
                return -1;
            }
        };

        let mut runtime = match Runtime::new() {
            Ok(runtime) => runtime,
            Err(err) => {
                error!(target: "system", "Failed to start tokio runtime, error: {:?}", err);
                return -1;
            }
        };

        let service = match Service::new(config_file_path, config) {
            Ok(service) => Arc::new(Mutex::new(service)),
            Err(err) => {
                error!(target: "system", "Failed to initial service error: {}", err);
                return -1;
            }
        };

        let server = Server::new(service);
        match runtime.block_on(server) {
            Ok(exit_status) => {
                info!(target: "system", "Receive exit signal, {}", exit_status.reason);
                return exit_status.code;
            }
            Err(err) => {
                println!("{:?}", err);
                return -1;
            }
        }
    }

    fn generate_config() -> i32 {
        let config_example = Configuration::new_example();
        println!(
            "{}",
            toml::to_string(&config_example).expect("example configuration is serializable; qed")
        );
        0
    }
}
