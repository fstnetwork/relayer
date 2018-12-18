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
use ethereum_types::{U128, U256};

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum DelegateMode {
    PublicMsgSender,
    PublicTxOrigin,
    PrivateMsgSender,
    PrivateTxOrigin,
}

impl DelegateMode {
    pub fn is_valid_numeric(mode: u32) -> bool {
        mode <= 0x03
    }

    pub fn is_public(&self) -> bool {
        *self == DelegateMode::PublicMsgSender || *self == DelegateMode::PublicTxOrigin
    }

    pub fn is_private(&self) -> bool {
        *self == DelegateMode::PrivateMsgSender || *self == DelegateMode::PrivateTxOrigin
    }
}

impl ToString for DelegateMode {
    fn to_string(&self) -> String {
        match self {
            DelegateMode::PublicMsgSender => "pubmsgsender".to_string(),
            DelegateMode::PublicTxOrigin => "pubtxorigin".to_string(),
            DelegateMode::PrivateMsgSender => "primsgsender".to_string(),
            DelegateMode::PrivateTxOrigin => "pritxorigin".to_string(),
        }
    }
}

impl Default for DelegateMode {
    fn default() -> Self {
        DelegateMode::PublicMsgSender
    }
}

impl From<u8> for DelegateMode {
    fn from(mode: u8) -> Self {
        match mode {
            0x00u8 => DelegateMode::PublicMsgSender,
            0x01u8 => DelegateMode::PublicTxOrigin,
            0x02u8 => DelegateMode::PrivateMsgSender,
            0x03u8 => DelegateMode::PrivateTxOrigin,
            _ => DelegateMode::PublicMsgSender,
        }
    }
}

impl From<u16> for DelegateMode {
    fn from(mode: u16) -> Self {
        match mode {
            0x00u16 => DelegateMode::PublicMsgSender,
            0x01u16 => DelegateMode::PublicTxOrigin,
            0x02u16 => DelegateMode::PrivateMsgSender,
            0x03u16 => DelegateMode::PrivateTxOrigin,
            _ => DelegateMode::PublicMsgSender,
        }
    }
}

impl From<u32> for DelegateMode {
    fn from(mode: u32) -> Self {
        match mode {
            0x00u32 => DelegateMode::PublicMsgSender,
            0x01u32 => DelegateMode::PublicTxOrigin,
            0x02u32 => DelegateMode::PrivateMsgSender,
            0x03u32 => DelegateMode::PrivateTxOrigin,
            _ => DelegateMode::PublicMsgSender,
        }
    }
}

impl From<u64> for DelegateMode {
    fn from(mode: u64) -> Self {
        match mode {
            0x00u64 => DelegateMode::PublicMsgSender,
            0x01u64 => DelegateMode::PublicTxOrigin,
            0x02u64 => DelegateMode::PrivateMsgSender,
            0x03u64 => DelegateMode::PrivateTxOrigin,
            _ => DelegateMode::PublicMsgSender,
        }
    }
}

impl From<U256> for DelegateMode {
    fn from(mode: U256) -> Self {
        let mode: u64 = mode.into();
        Self::from(mode)
    }
}

impl Into<u8> for DelegateMode {
    fn into(self) -> u8 {
        match self {
            DelegateMode::PublicMsgSender => 0x00u8,
            DelegateMode::PublicTxOrigin => 0x01u8,
            DelegateMode::PrivateMsgSender => 0x02u8,
            DelegateMode::PrivateTxOrigin => 0x03u8,
        }
    }
}

impl Into<u16> for DelegateMode {
    fn into(self) -> u16 {
        let n: u8 = self.into();
        n as u16
    }
}

impl Into<u32> for DelegateMode {
    fn into(self) -> u32 {
        let n: u8 = self.into();
        n as u32
    }
}

impl Into<u64> for DelegateMode {
    fn into(self) -> u64 {
        let n: u8 = self.into();
        n as u64
    }
}
impl Into<U128> for DelegateMode {
    fn into(self) -> U128 {
        let n: u8 = self.into();
        U128::from(n)
    }
}

impl Into<U256> for DelegateMode {
    fn into(self) -> U256 {
        let n: u8 = self.into();
        U256::from(n)
    }
}
