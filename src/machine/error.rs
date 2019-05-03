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
use std::time::Duration;

use crate::collation::CollationError;
use crate::ethereum::{
    monitor::Error as EthereumMonitorError, service::Error as EthereumServiceError,
};
use crate::pricer::Error as PricerError;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Collation error: {}", _0)]
    Collation(CollationError),

    #[fail(display = "Tokio Timer error: {}", _0)]
    Timer(tokio_timer::Error),

    #[fail(display = "Ethereum Service error: {}", _0)]
    EthereumService(EthereumServiceError),

    #[fail(display = "Ethereum Monitor error: {}", _0)]
    EthereumMonitor(EthereumMonitorError),

    #[fail(display = "Price Service error: {}", _0)]
    PriceService(PricerError),

    #[fail(
        display = "Invalid state transfer, current state: {:?}, expected state: {:?}",
        current_state, expected_state
    )]
    InvalidStateTransfer {
        current_state: String,
        expected_state: String,
    },

    #[fail(display = "Invalid interval value: {:?}", _0)]
    InvalidIntervalValue(Duration),

    #[fail(display = "Empty token transfer request transaction")]
    EmptyTokenTransferRequestTransaction,

    #[fail(display = "Failed to import token transfer request")]
    FailedToImportTokenTransferRequest,
}

impl From<CollationError> for Error {
    fn from(error: CollationError) -> Error {
        Error::Collation(error)
    }
}

impl From<tokio_timer::Error> for Error {
    fn from(error: tokio_timer::Error) -> Error {
        Error::Timer(error)
    }
}

impl From<PricerError> for Error {
    fn from(error: PricerError) -> Error {
        Error::PriceService(error)
    }
}

impl From<EthereumServiceError> for Error {
    fn from(error: EthereumServiceError) -> Error {
        Error::EthereumService(error)
    }
}

impl From<EthereumMonitorError> for Error {
    fn from(error: EthereumMonitorError) -> Error {
        Error::EthereumMonitor(error)
    }
}
