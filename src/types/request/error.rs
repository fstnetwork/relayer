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

#[derive(Debug, Clone, Fail, PartialEq)]
/// Errors concerning request processing.
pub enum Error {
    /// Request is already imported to the queue
    #[fail(display = "Already imported")]
    AlreadyImported,

    /// Request is not valid anymore (state already has higher nonce)
    #[fail(display = "No longer valid")]
    Old,

    /// Request has too low fee
    /// (there is already a request with the same sender-nonce but higher gas price)
    #[fail(display = "Gas price too low to replace")]
    TooCheapToReplace,

    /// Request was not imported to the queue because limit has been reached.
    #[fail(display = "Request limit reached")]
    LimitReached,

    /// Request's gas price is below threshold.
    #[fail(display = "Insufficient gas. Min={}, Given={}", minimal, got)]
    InsufficientFee {
        /// Minimal expected gas price
        minimal: U256,
        /// Got fee
        got: U256,
    },

    /// Sender doesn't have enough funds to pay for this request
    #[fail(
        display = "Insufficient balance for request. Balance={}, Cost={}",
        balance, cost
    )]
    InsufficientBalance {
        /// Senders balance
        balance: U256,
        /// Request cost
        cost: U256,
    },

    // /// Request's gas limit (aka gas) is invalid.
    // #[fail(display = "Request's gas limit is invalid: {}.", _0)]
    // InvalidGasLimit(OutOfBounds<U256>),
    /// Request sender is banned.
    #[fail(display = "Sender is temporarily banned.")]
    SenderBanned,

    /// Request receipient is banned.
    #[fail(display = "Recipient is temporarily banned.")]
    RecipientBanned,

    /// Data code is banned.
    #[fail(display = "Data is temporarily banned.")]
    DataBanned,

    /// Invalid chain ID given.
    #[fail(display = "Sender does not have permissions to execute this type of transction.")]
    NotAllowed,

    /// Signature error
    #[fail(display = "Request has invalid signature: {}.", _0)]
    InvalidSignature(String),
}

impl From<ethkey::Error> for Error {
    fn from(err: ethkey::Error) -> Self {
        Error::InvalidSignature(format!("{}", err))
    }
}
