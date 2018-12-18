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
use ethereum_types::Address;

error_chain! {
    foreign_links {
        Io(std::io::Error);
        Json(serde_json::Error);
        EthKey(ethkey::Error);
    }

    errors {
        EthStore(error: ethstore::Error) {
            description("EthStore error")
            display("EthStore error: {:?}", error)
        }

        InvalidRelayInterval {
            description("Invalid relay interval")
            display("Invalid relay interval")
        }

        RecoverPrivateKeyFailed(error: String, address: Address, keyfile: String, password_file: String) {
            description("Failed to recover private key from keyfile")
            display("Failed to recover private key of {} from key file {} and password file {}, error: {}",
                    address, keyfile, password_file, error)
        }

        ResolveFilePathFailed(file_path: String) {
            description("Failed to resolve file path")
            display("Failed to resolve file path {}", file_path)
        }
    }
}
