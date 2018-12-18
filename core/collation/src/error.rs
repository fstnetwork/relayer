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
use std::{error, fmt};

use ethkey::Error as EthkeyError;

use types::RequestError;

#[derive(Debug, Clone)]
pub enum RequestImportError {
    Request(RequestError),
    Other(String),
}

impl From<RequestError> for RequestImportError {
    fn from(e: RequestError) -> Self {
        RequestImportError::Request(e)
    }
}

impl From<Error> for RequestImportError {
    fn from(e: Error) -> Self {
        match e {
            Error(ErrorKind::RequestError(request_error), _) => {
                RequestImportError::Request(request_error)
            }
            _ => RequestImportError::Other(format!("other collaction import error: {:?}", e)),
        }
    }
}

error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    foreign_links {
        StdIo(::std::io::Error) #[doc = "Error concerning the Rust standard library's IO subsystem."];
        Request(RequestError) #[doc = "Error concerning request processing."];
        Ethkey(EthkeyError) #[doc = "Ethkey error."];
    }

    errors {
        RequestError(err: RequestError) {
            description("Request error.")
            display("Request error {}", err)
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum CollationError {
    CloseWithNoTransaction,
}

impl fmt::Display for CollationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // use self::CollationError::*;

        let msg = "";
        f.write_fmt(format_args!("Collation error ({})", msg))
    }
}

impl error::Error for CollationError {
    fn description(&self) -> &str {
        "Collation error"
    }
}
