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

use super::ethereum_client::Error as EthereumClientError;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "IO error: {}", _0)]
    Io(::std::io::Error),

    #[fail(display = "JSON error: {}", _0)]
    Json(serde_json::Error),

    #[fail(display = "Ethereum client error: {}", _0)]
    EthereumClient(EthereumClientError),

    #[fail(display = "Ethereum client group is empty")]
    EthereumClientGroupEmpty,

    #[fail(display = "Parse hex error")]
    ParseHex,
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Error {
        Error::Io(error)
    }
}

impl From<EthereumClientError> for Error {
    fn from(error: EthereumClientError) -> Error {
        Error::EthereumClient(error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Error {
        Error::Json(error)
    }
}
