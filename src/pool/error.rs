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

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Tokio timer error: {}", _0)]
    Timer(tokio_timer::Error),

    #[fail(display = "Invalid token transfer request.")]
    InvalidTokenTransferRequest,

    #[fail(display = "Token transfer request gas estimation failed.")]
    TokenTransferRequestGasEstimationFailed,

    #[fail(display = "Not supported token")]
    NotSupportedToken,

    #[fail(display = "[{}] already imported", _0)]
    AlreadyImported(String),

    /// request is too cheap to enter the queue
    #[fail(
        display = "[{}] too cheap to enter the pool. Min score: {}",
        hash, min_score
    )]
    TooCheapToEnter { hash: String, min_score: String },

    /// Request is too cheap to replace existing request that occupies the same slot.
    #[fail(display = "[{}] too cheap to replace: {}", hash, old_hash)]
    TooCheapToReplace { old_hash: String, hash: String },
}

impl From<tokio_timer::Error> for Error {
    fn from(error: tokio_timer::Error) -> Error {
        Error::Timer(error)
    }
}
