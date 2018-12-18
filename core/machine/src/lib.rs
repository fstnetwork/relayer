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

#[macro_use]
extern crate error_chain;

extern crate serde;
#[macro_use]
extern crate serde_derive;

extern crate futures;
extern crate parking_lot;
extern crate tokio;

extern crate ethcore_transaction;
extern crate ethereum_types;
extern crate ethkey;

extern crate collation;
extern crate ethereum;
extern crate pool;
extern crate pricer;
extern crate traits;
extern crate types;

mod error;
mod event;
mod machine;
mod service;
mod transaction_queue;

#[cfg(test)]
mod tests;

pub use self::error::{Error, ErrorKind};
use self::event::RelayerEvent;
use self::machine::{RelayerInfo, RelayerMachine, RelayerParams, RelayerState};
// use self::transaction_queue::TransactionQueue;

pub use self::machine::RelayerMode;
pub use self::service::{
    Params as MachineServiceParams, Service as MachineService, Status as MachineStatus,
};
