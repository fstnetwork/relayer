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
use serde_json;
use std::io::Error as IoError;

use super::ethereum_client::Error as EthereumClientError;

error_chain! {
    foreign_links {
        Io(IoError) #[doc = "IO error."];
        Json(serde_json::Error);
        EthereumClient(EthereumClientError) #[doc = "Ethereum client."];
    }

    errors {
        ParseHex {
            description("Parse Hex Error")
            display("Parse Hex Error")
        }

        EthereumClientGroupEmpty {
            description("Ethereum client group is empty")
            display("Ethereum client group is empty")
        }
    }
}
