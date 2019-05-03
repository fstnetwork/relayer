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

use crate::contract_abi::Error as ContractAbiError;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "IO error: {}", _0)]
    Io(::std::io::Error),

    #[fail(display = "From hex error: {:?}", _0)]
    FromHex(rustc_hex::FromHexError),

    #[fail(display = "Contract ABI error: {}", _0)]
    ContractAbi(ContractAbiError),

    #[fail(display = "Hyper error: {:?}", _0)]
    Hyper(hyper::Error),

    #[fail(display = "JSON error: {:?}", _0)]
    Json(serde_json::Error),

    #[fail(display = "No such field: {}", _0)]
    NoSuchField(&'static str),

    #[fail(display = "Parse hex error")]
    ParseHex,

    #[fail(display = "JSON-RPC error: {:?}", _0)]
    JsonRpc(jsonrpc_core::Error),

    #[fail(display = "Error: {:?}", _0)]
    Other(String),
}

impl From<ContractAbiError> for Error {
    fn from(error: ContractAbiError) -> Error {
        Error::ContractAbi(error)
    }
}

impl From<hyper::Error> for Error {
    fn from(error: hyper::Error) -> Error {
        Error::Hyper(error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Error {
        Error::Json(error)
    }
}

impl From<jsonrpc_core::Error> for Error {
    fn from(error: jsonrpc_core::Error) -> Error {
        Error::JsonRpc(error)
    }
}

impl From<String> for Error {
    fn from(error: String) -> Error {
        Error::Other(error)
    }
}
