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
use ethereum_types::U256;

use super::RequestError;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "EthKey error: {}", _0)]
    EthkeyError(ethkey::Error),

    #[fail(display = "Token Transfer Request error: {}", _0)]
    RequestError(RequestError),

    #[fail(display = "Invalid delegate mode {}", _0)]
    InvalidDelegateMode(U256),
}

impl From<ethkey::Error> for Error {
    fn from(error: ethkey::Error) -> Error {
        Error::EthkeyError(error)
    }
}

impl From<RequestError> for Error {
    fn from(error: RequestError) -> Error {
        Error::RequestError(error)
    }
}
