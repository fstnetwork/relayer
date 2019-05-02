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
error_chain! {
    foreign_links {
        Timer(tokio_timer::Error);
    }

    errors {
        InvalidTokenTransferRequest {
            description("Invalid token transfer request")
            display("Invalid token transfer request")
        }

        TokenTransferRequestGasEstimationFailed {
            description("Token transfer request gas estimation failed")
            display("Token transfer request gas estimation failed")
        }


        NotSupportedToken {
            description("Not supported token")
            display("Not supported token")
        }

        AlreadyImported(hash: String) {
            description("request is already in the pool"),
            display("[{}] already imported", hash)
        }
        /// request is too cheap to enter the queue
        TooCheapToEnter(hash: String, min_score: String) {
            description("the pool is full and request is too cheap to replace any request"),
            display("[{}] too cheap to enter the pool. Min score: {}", hash, min_score)
        }
        /// Request is too cheap to replace existing request that occupies the same slot.
        TooCheapToReplace(old_hash: String, hash: String) {
            description("request is too cheap to replace existing request in the pool"),
            display("[{}] too cheap to replace: {}", hash, old_hash)
        }
    }
}
