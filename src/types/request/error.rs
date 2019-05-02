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

use ethereum_types::U256;
use ethkey;

#[derive(Debug, PartialEq, Clone)]
/// Errors concerning request processing.
pub enum Error {
    /// Request is already imported to the queue
    AlreadyImported,
    /// Request is not valid anymore (state already has higher nonce)
    Old,
    /// Request has too low fee
    /// (there is already a request with the same sender-nonce but higher gas price)
    TooCheapToReplace,
    /// Request was not imported to the queue because limit has been reached.
    LimitReached,
    /// Request's gas price is below threshold.
    InsufficientFee {
        /// Minimal expected gas price
        minimal: U256,
        /// Got fee
        got: U256,
    },
    /// Sender doesn't have enough funds to pay for this request
    InsufficientBalance {
        /// Senders balance
        balance: U256,
        /// Request cost
        cost: U256,
    },
    // /// Request's gas limit (aka gas) is invalid.
    // // InvalidGasLimit(OutOfBounds<U256>),
    /// Request sender is banned.
    SenderBanned,
    /// Request receipient is banned.
    RecipientBanned,
    /// Data code is banned.
    DataBanned,
    // /// Invalid chain ID given.
    NotAllowed,
    /// Signature error
    InvalidSignature(String),
}

impl From<ethkey::Error> for Error {
    fn from(err: ethkey::Error) -> Self {
        Error::InvalidSignature(format!("{}", err))
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "Token Transfer Request error"
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Error::*;
        let msg = match *self {
            AlreadyImported => "Already imported".into(),
            Old => "No longer valid".into(),
            TooCheapToReplace => "Gas price too low to replace".into(),
            LimitReached => "Request limit reached".into(),
            InsufficientFee { minimal, got } => {
                format!("Insufficient gas. Min={}, Given={}", minimal, got)
            }
            InsufficientBalance { balance, cost } => format!(
                "Insufficient balance for request. Balance={}, Cost={}",
                balance, cost
            ),
            SenderBanned => "Sender is temporarily banned.".into(),
            RecipientBanned => "Recipient is temporarily banned.".into(),
            DataBanned => "Data is temporarily banned.".into(),
            // InvalidChainId => "Request of this chain ID is not allowed on this chain.".into(),
            InvalidSignature(ref err) => format!("Request has invalid signature: {}.", err),
            NotAllowed => {
                "Sender does not have permissions to execute this type of transction".into()
            }
        };

        f.write_fmt(format_args!("Request error ({})", msg))
    }
}
