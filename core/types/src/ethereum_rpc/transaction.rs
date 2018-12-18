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
use super::{Bytes, H160, H256, H512, U256, U64};

/// Transaction
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    /// Hash
    pub hash: H256,
    /// Nonce
    pub nonce: U256,
    /// Block hash
    #[serde(rename = "blockHash")]
    pub block_hash: Option<H256>,
    /// Block number
    #[serde(rename = "blockNumber")]
    pub block_number: Option<U256>,
    /// Transaction Index
    #[serde(rename = "transactionIndex")]
    pub transaction_index: Option<U256>,
    /// Sender
    pub from: H160,
    /// Recipient
    pub to: Option<H160>,
    /// Transfered value
    pub value: U256,
    /// Gas Price
    #[serde(rename = "gasPrice")]
    pub gas_price: U256,
    /// Gas
    pub gas: U256,
    /// Data
    pub input: Bytes,
    /// Creates contract
    pub creates: Option<H160>,
    /// Raw transaction data
    pub raw: Bytes,
    /// Public key of the signer.
    #[serde(rename = "publicKey")]
    pub public_key: Option<H512>,
    /// The network id of the transaction, if any.
    #[serde(rename = "chainId")]
    pub chain_id: Option<U64>,
    /// The standardised V field of the signature (0 or 1).
    #[serde(rename = "standardV")]
    pub standard_v: U256,
    /// The standardised V field of the signature.
    pub v: U256,
    /// The R field of the signature.
    pub r: U256,
    /// The S field of the signature.
    pub s: U256,
}
