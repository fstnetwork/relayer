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
use super::config;

use crate::ethereum::service::Error as EthereumServiceError;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Io error: {}", _0)]
    Io(std::io::Error),

    #[fail(display = "JSON error: {}", _0)]
    Json(serde_json::Error),

    #[fail(display = "Configuration error: {}", _0)]
    Config(config::Error),

    #[fail(display = "EthKey error: {}", _0)]
    EthKey(ethkey::Error),

    #[fail(display = "Ethereum service error: {}", _0)]
    EthereumService(EthereumServiceError),
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Error {
        Error::Io(error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Error {
        Error::Json(error)
    }
}

impl From<config::Error> for Error {
    fn from(error: config::Error) -> Error {
        Error::Config(error)
    }
}

impl From<ethkey::Error> for Error {
    fn from(error: ethkey::Error) -> Error {
        Error::EthKey(error)
    }
}

impl From<EthereumServiceError> for Error {
    fn from(error: EthereumServiceError) -> Error {
        Error::EthereumService(error)
    }
}
