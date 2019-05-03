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
extern crate failure;

#[macro_use]
extern crate clap;

#[macro_use]
extern crate lazy_static;

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
#[macro_use]
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
extern crate jsonrpc_ws_server;

mod collation;
mod contract_abi;
mod ethereum;
mod machine;
mod network;
mod pool;
mod pricer;
mod relayer;
mod traits;
mod types;
mod utils;

use relayer::cli::Cli;

fn main() {
    Cli::build().command().run();
}
