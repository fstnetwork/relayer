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
#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate error_chain;

extern crate ethabi;
extern crate ethereum_types;
extern crate ethkey;
extern crate rustc_hex;

extern crate types;

mod abi_decoder;
mod abi_encoder;
mod error;
mod interfaces;

pub use self::error::Error;

pub use self::abi_decoder::{
    ERC1376AbiDecoder, ERC20AbiDecoder, FstTokenTransferRequestDispatcherAbiDecoder,
};
pub use self::abi_encoder::{
    ERC1376AbiEncoder, ERC20AbiEncoder, FstTokenTransferRequestDispatcherAbiEncoder,
};

pub use self::interfaces::FST_TOKEN_TRANSFER_REQUEST_DISPATCHER_INTERFACE;
pub use self::interfaces::{ERC1376_TOKEN_INTERFACE, ERC20_TOKEN_INTERFACE};
