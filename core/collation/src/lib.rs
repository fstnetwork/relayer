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
extern crate futures;

#[macro_use]
extern crate error_chain;

extern crate ethcore_transaction;
extern crate ethereum_types;
extern crate ethkey;

extern crate contract_abi;
extern crate types;

mod collation;
mod error;
mod request_converter;

pub use self::collation::{ClosedCollation, Collation, OpenCollation, RequestDispatcher};
pub use self::error::CollationError;
pub use self::request_converter::{EmptyRequestConverter, FstRequestConverter, RequestConverter};
