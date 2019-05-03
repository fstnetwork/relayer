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
use ethereum_types::{Address, H256, U256};

use super::DelegateMode;
use super::EthRpcBytes;

use super::error::Error;

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Request {
    /// Token address
    #[serde(rename = "token")]
    pub token_address: Address,

    /// Nonce
    pub nonce: U256,

    /// Fee
    pub fee: U256,

    /// Gas amount
    #[serde(rename = "gasAmount")]
    pub gas_amount: U256,

    /// Receiver, the "to" field
    #[serde(rename = "to")]
    pub receiver: Address,

    /// Value to send to receiver
    pub value: U256,

    /// Data
    pub data: EthRpcBytes,

    /// Delegate Mode
    #[serde(rename = "mode")]
    pub delegate_mode: U256,

    /// Relayer's address
    #[serde(rename = "relayer")]
    pub relayer_address: Address,

    /// The V field of the signature; helps describe the point on the curve.
    pub v: U256,

    /// The R field of the signature; helps describe the point on the curve.
    pub r: U256,

    /// The S field of the signature; helps describe the point on the curve.
    pub s: U256,

    /// Hash of the request
    pub hash: Option<H256>,
}

impl Request {
    pub fn into_signed_request(self) -> Result<super::SignedRequest, Error> {
        let delegate_mode = if DelegateMode::is_valid_numeric(self.delegate_mode.into()) {
            DelegateMode::from(self.delegate_mode)
        } else {
            return Err(Error::InvalidDelegateMode(self.delegate_mode));
        };

        let sig = ethkey::Signature::from_rsv(
            &self.r.into(),
            &self.s.into(),
            super::signature::check_replay_protection(u64::from(self.v)),
        );

        super::SignedRequest::new(
            super::Request {
                token_address: self.token_address,
                nonce: self.nonce,
                fee: self.fee,
                gas_amount: self.gas_amount,
                receiver: self.receiver,
                value: self.value,
                data: self.data.into_vec(),
                delegate_mode,
                relayer_address: self.relayer_address,
            }
            .with_signature(sig),
        )
        .map(|r| r)
        .map_err(|err| Error::from(err))
    }
}

impl<'a> From<&'a super::SignedRequest> for Request {
    fn from(signed_request: &super::SignedRequest) -> Self {
        let unverified = signed_request.unverified();
        Request {
            token_address: *unverified.token(),
            nonce: *unverified.nonce(),
            fee: *unverified.fee(),
            gas_amount: *unverified.gas_amount(),
            receiver: *unverified.receiver(),
            value: *unverified.value(),
            data: EthRpcBytes::from(unverified.data()),
            delegate_mode: U256::from(unverified.delegate_mode() as u8),
            relayer_address: *unverified.relayer_address(),

            v: U256::from(unverified.original_v()),
            r: *unverified.r(),
            s: *unverified.s(),
            hash: Some(*signed_request.hash()),
        }
    }
}

impl From<super::SignedRequest> for Request {
    fn from(signed_request: super::SignedRequest) -> Self {
        let unverified = signed_request.unverified();
        Request {
            token_address: *unverified.token(),
            nonce: *unverified.nonce(),
            fee: *unverified.fee(),
            gas_amount: *unverified.gas_amount(),
            receiver: *unverified.receiver(),
            value: *unverified.value(),
            data: EthRpcBytes::from(unverified.data()),
            delegate_mode: U256::from(unverified.delegate_mode() as u8),
            relayer_address: *unverified.relayer_address(),

            v: U256::from(unverified.original_v() as u8),
            r: *unverified.r(),
            s: *unverified.s(),
            hash: Some(*signed_request.hash()),
        }
    }
}
