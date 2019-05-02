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

error_chain! {
    foreign_links {
        TimerError(tokio_timer::Error);
        Collation(CollationError);
        EthereumService(EthereumServiceError);
        EthereumMonitor(EthereumMonitorError);
        PriceService(PricerError);
    }

    errors {
        InvalidStateTransfer(current_state: String, expected_state: String) {
            description("Invalid state transfer")
            display("Invalid state transfer, current state: {:?}, expected state: {:?}", current_state, expected_state)
        }

        InvalidIntervalValue(interval: Duration) {
            description("Invalid interval value")
            display("Invalid interval value: {:?}", interval)
        }

        EmptyTokenTransferRequestTransaction {
            description("Empty token transfer request transaction")
            display("Empty token transfer request transaction")
        }

        FailedToImportTokenTransferRequest {
            description("Failed to import token transfer request")
            display("Failed to import token transfer request")
        }
    }
}
