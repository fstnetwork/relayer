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
use std::path::PathBuf;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "IO error: {}", _0)]
    Io(std::io::Error),

    #[fail(display = "JSON error: {}", _0)]
    Json(serde_json::Error),

    #[fail(display = "EthKey error: {}", _0)]
    EthKey(ethkey::Error),

    #[fail(display = "EthStore error: {}", _0)]
    EthStore(ethstore::Error),

    #[fail(display = "Invalid relay interval")]
    InvalidRelayInterval,

    #[fail(
        display = "Failed to recover private key of {} from key file {} and password file {}, error: {}",
        address, keyfile, password_file, error
    )]
    RecoverPrivateKeyFailed {
        error: String,
        address: Address,
        keyfile: String,
        password_file: String,
    },

    #[fail(display = "Failed to resolve file path {}", _0)]
    ResolveFilePathFailed(String),

    #[fail(
        display = "Failed to open configuration file: {:?}, error: {}",
        file_path, error
    )]
    OpenConfigurationFileFailed {
        file_path: PathBuf,
        error: std::io::Error,
    },

    #[fail(
        display = "Failed to read content from file: {:?}, error: {}",
        file_path, error
    )]
    ReadConfigurationContentFailed {
        file_path: PathBuf,
        error: std::io::Error,
    },

    #[fail(
        display = "Failed to deserialize configuration file: {:?}, error: {:?}",
        file_path, error
    )]
    DeserializeConfigurationFailed {
        file_path: PathBuf,
        error: toml::de::Error,
    },
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Error {
        Error::Io(error)
    }
}

impl From<ethstore::Error> for Error {
    fn from(error: ethstore::Error) -> Error {
        Error::EthStore(error)
    }
}

impl From<ethkey::Error> for Error {
    fn from(error: ethkey::Error) -> Error {
        Error::EthKey(error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Error {
        Error::Json(error)
    }
}
