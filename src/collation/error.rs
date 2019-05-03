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

use crate::types::RequestError;

#[derive(Debug, Fail)]
pub enum RequestImportError {
    #[fail(display = "Request error: {}", _0)]
    Request(RequestError),

    #[fail(display = "Request other error: {}", _0)]
    Other(String),
}

impl From<RequestError> for RequestImportError {
    fn from(e: RequestError) -> Self {
        RequestImportError::Request(e)
    }
}

#[derive(Debug, Fail)]
pub enum CollationError {
    #[fail(display = "IO error: {}", _0)]
    Io(::std::io::Error),

    #[fail(display = "EthKey error: {}", _0)]
    Ethkey(ethkey::Error),

    #[fail(display = "Token transfer request error: {}", _0)]
    Request(RequestError),

    #[fail(display = "Close collation with no transaction.")]
    CloseWithNoTransaction,
}

impl From<std::io::Error> for CollationError {
    fn from(error: std::io::Error) -> CollationError {
        CollationError::Io(error)
    }
}

impl From<ethkey::Error> for CollationError {
    fn from(error: ethkey::Error) -> CollationError {
        CollationError::Ethkey(error)
    }
}

impl From<RequestError> for CollationError {
    fn from(error: RequestError) -> CollationError {
        CollationError::Request(error)
    }
}
