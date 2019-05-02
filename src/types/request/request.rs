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
use ethereum_types::{Address, H160, H256, U256};
use ethkey;
use keccak_hash::keccak as keccak_hash;

use super::DelegateMode;

pub const UNSIGNED_SENDER: Address = H160([0xff; 20]);

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Request {
    /// Token address
    pub token_address: Address,
    /// Nonce
    pub nonce: U256,
    /// Fee
    pub fee: U256,
    /// Gas amount
    pub gas_amount: U256,
    /// Receiver, the "to" field
    pub receiver: Address,
    /// Value to send to receiver
    pub value: U256,
    /// Data
    pub data: Vec<u8>,
    /// Delegate Mode
    pub delegate_mode: DelegateMode,
    /// Relayer's address
    pub relayer_address: Address,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct UnverifiedRequest {
    inner: Request,
    /// The V field of the signature; the LS bit described which half of the curve our point falls
    /// in. The MS bits describe which chain this request is for. If 27/28, its for all chains.
    v: u64,
    /// The R field of the signature; helps describe the point on the curve.
    r: U256,
    /// The S field of the signature; helps describe the point on the curve.
    s: U256,
    /// Hash of the request
    hash: H256,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct SignedRequest {
    request: UnverifiedRequest,
    sender: Address,
}

/// Replay protection logic for v part of transaction's signature
pub mod signature {
    /// Adds chain id into v
    pub fn add_chain_replay_protection(v: u64) -> u64 {
        v + 27
    }

    /// Returns refined v
    /// 0 if `v` would have been 27 under "Electrum" notation, 1 if 28 or 4 if invalid.
    pub fn check_replay_protection(v: u64) -> u8 {
        match v {
            v if v == 27 => 0,
            v if v == 28 => 1,
            _ => 4,
        }
    }
}

impl Request {
    #[inline]
    pub fn empty() -> Self {
        Request {
            token_address: Address::zero(),
            nonce: U256::from(0),
            fee: U256::from(0),
            gas_amount: U256::from(0),
            receiver: Address::zero(),
            value: U256::from(0),
            data: Vec::with_capacity(68),
            delegate_mode: DelegateMode::PublicMsgSender,
            relayer_address: Address::zero(),
        }
    }

    pub fn with_signature(self, sig: ethkey::Signature) -> UnverifiedRequest {
        UnverifiedRequest {
            inner: self,
            r: sig.r().into(),
            s: sig.s().into(),
            v: signature::add_chain_replay_protection(sig.v() as u64),
            hash: 0.into(),
        }
        .compute_hash()
    }

    pub fn null_sign(self) -> SignedRequest {
        SignedRequest {
            request: UnverifiedRequest {
                inner: self,
                r: U256::zero(),
                s: U256::zero(),
                v: 0,
                hash: 0.into(),
            }
            .compute_hash(),
            sender: UNSIGNED_SENDER,
        }
    }

    // encode packed data
    pub fn pack(&self) -> Vec<u8> {
        use super::solidity::encoder;
        use super::solidity::Token;

        let param_tokens = [
            Token::Address(self.token_address),
            Token::U256(self.nonce),
            Token::U256(self.fee),
            Token::U256(self.gas_amount),
            Token::Address(self.receiver),
            Token::U256(self.value),
            Token::Bytes(self.data.clone()),
            Token::U8(self.delegate_mode.into()),
            Token::Address(self.relayer_address),
        ];

        encoder::packed::encode(&param_tokens)
    }

    pub fn hash(&self) -> H256 {
        keccak_hash(self.pack())
    }

    pub fn sign(self, secret: &ethkey::Secret) -> SignedRequest {
        let sig = ethkey::sign(secret, &self.hash())
            .expect("data is valid and context has signing capabilities; qed");

        SignedRequest::new(self.with_signature(sig)).expect("secret is valid so it's recoverable")
    }
}

impl UnverifiedRequest {
    fn compute_hash(mut self) -> Self {
        let mut packed = self.inner.pack();
        let mut buf = [0u8; 65];
        self.r.to_big_endian(&mut buf[0..32]);
        self.s.to_big_endian(&mut buf[32..64]);
        buf[64] = self.standard_v() as u8;
        packed.append(&mut buf.to_vec());

        self.hash = keccak_hash(packed);
        self
    }

    #[inline]
    pub fn hash(&self) -> &H256 {
        &self.hash
    }

    #[inline]
    pub fn is_unsigned(&self) -> bool {
        self.r.is_zero() && self.s.is_zero()
    }

    #[inline]
    pub fn as_unsigned(&self) -> &Request {
        &self.inner
    }

    #[inline]
    pub fn token(&self) -> &Address {
        &self.inner.token_address
    }

    #[inline]
    pub fn nonce(&self) -> &U256 {
        &self.inner.nonce
    }

    #[inline]
    pub fn fee(&self) -> &U256 {
        &self.inner.fee
    }

    #[inline]
    pub fn gas_amount(&self) -> &U256 {
        &self.inner.gas_amount
    }

    #[inline]
    pub fn receiver(&self) -> &Address {
        &self.inner.receiver
    }

    #[inline]
    pub fn value(&self) -> &U256 {
        &self.inner.value
    }

    #[inline]
    pub fn data(&self) -> Vec<u8> {
        self.inner.data.clone()
    }

    #[inline]
    pub fn delegate_mode(&self) -> DelegateMode {
        self.inner.delegate_mode
    }

    #[inline]
    pub fn relayer_address(&self) -> &Address {
        &self.inner.relayer_address
    }

    #[inline]
    pub fn r(&self) -> &U256 {
        &self.r
    }

    #[inline]
    pub fn s(&self) -> &U256 {
        &self.s
    }

    #[inline]
    pub fn original_v(&self) -> u64 {
        self.v
    }

    pub fn standard_v(&self) -> u8 {
        signature::check_replay_protection(self.v)
    }

    pub fn signature(&self) -> ethkey::Signature {
        ethkey::Signature::from_rsv(&self.r.into(), &self.s.into(), self.standard_v() as u8)
    }

    pub fn recover_public(&self) -> Result<ethkey::Public, ethkey::Error> {
        Ok(ethkey::recover(&self.signature(), &self.inner.hash())?)
    }
}

impl SignedRequest {
    pub fn new(request: UnverifiedRequest) -> Result<SignedRequest, ethkey::Error> {
        if request.is_unsigned() {
            Ok(SignedRequest {
                request,
                sender: UNSIGNED_SENDER,
            })
        } else {
            let public = request.recover_public()?;
            let sender = ethkey::public_to_address(&public);
            Ok(SignedRequest { request, sender })
        }
    }

    #[inline]
    pub fn as_unverified(self) -> UnverifiedRequest {
        self.request
    }

    #[inline]
    pub fn unverified(&self) -> &UnverifiedRequest {
        &self.request
    }

    #[inline]
    pub fn deconstruct(self) -> (UnverifiedRequest, Address) {
        (self.request, self.sender)
    }

    #[inline]
    pub fn sender(&self) -> &Address {
        &self.sender
    }

    #[inline]
    pub fn hash(&self) -> &H256 {
        self.request.hash()
    }
}

impl ::std::hash::Hash for Request {
    fn hash<H: ::std::hash::Hasher>(&self, state: &mut H) {
        self.hash().hash(state)
    }
}

impl ::std::hash::Hash for UnverifiedRequest {
    fn hash<H: ::std::hash::Hasher>(&self, state: &mut H) {
        self.hash().hash(state)
    }
}

impl ::std::hash::Hash for SignedRequest {
    fn hash<H: ::std::hash::Hasher>(&self, state: &mut H) {
        self.hash().hash(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_delegate_mode(mode: DelegateMode) -> Request {
        let mut empty = Request::empty();
        empty.delegate_mode = mode;
        empty
    }

    fn mock_request() -> Request {
        Request {
            token_address: Address::from("0x89cF87c35e69A9B84F7A3e50EAf54bFc3Cabc377"),
            nonce: U256::from(1),
            fee: U256::from(20000),
            gas_amount: U256::from(1200000),
            receiver: Address::from("0x7195eb47570cF0aeCe30893e8e7e56C4Da5f0AC2"),
            value: U256::from(20000000),
            data: Vec::default(),
            delegate_mode: DelegateMode::PublicMsgSender,
            relayer_address: Address::from("0x0000000000000000000000000000000000000000"),
        }
    }

    #[test]
    fn test_len() {
        let empty_len = Request::empty().pack().len();

        assert_eq!(empty_len, mock_request().pack().len());

        assert_eq!(
            empty_len,
            with_delegate_mode(DelegateMode::PublicMsgSender)
                .pack()
                .len(),
        );

        assert_eq!(
            empty_len,
            with_delegate_mode(DelegateMode::PublicTxOrigin)
                .pack()
                .len(),
        );

        assert_eq!(
            empty_len,
            with_delegate_mode(DelegateMode::PrivateMsgSender)
                .pack()
                .len(),
        );

        assert_eq!(
            empty_len,
            with_delegate_mode(DelegateMode::PrivateTxOrigin)
                .pack()
                .len(),
        );
    }

    #[test]
    fn test_empty() {
        let empty = Request::empty();

        assert_eq!(empty.pack(), vec![0u8; 189]);

        assert_eq!(
            empty.hash(),
            "0x785ea77dec5a8f92f2a76716538b2d1763c493f4aa9ede26df1a527eae82c171".into()
        );
    }

    #[test]
    fn test_dummy_request_with_different_delegate_mode() {
        let req = mock_request();

        assert_eq!(
            req.hash(),
            "0x242a95bdb71a264da8bd30585d6ae66114c101329894d4d56018c9444a8aa597".into()
        );

        assert_eq!(
            {
                let mut req = req.clone();
                req.delegate_mode = DelegateMode::PublicTxOrigin;
                req
            }
            .hash(),
            "0x1fd931cc809dbb13f1c7af7d0a7d57be9be73459e8c35e887adfd618245d6b5a".into()
        );

        assert_eq!(
            {
                let mut req = req.clone();
                req.delegate_mode = DelegateMode::PrivateMsgSender;
                req
            }
            .hash(),
            "0x688d3fcdcf2c977d01ac314716a1a95bf28349238fdc51635ecc73181f32a0e2".into()
        );

        assert_eq!(
            {
                let mut req = req.clone();
                req.delegate_mode = DelegateMode::PrivateTxOrigin;
                req
            }
            .hash(),
            "0x23efba2a0f73a5b629d1afa3eff356e84e9c441ec66f99bd702f004a16492246".into()
        );
    }

    #[test]
    fn test_signing() {
        use ethkey::{Generator, Random};
        let key = Random.generate().unwrap();
        let signed_req = mock_request().sign(&key.secret());
        assert_eq!(
            Address::from(keccak_hash(key.public())),
            *signed_req.sender()
        );
    }

    #[test]
    fn test_signed_request() {
        use ethkey::{Secret, Signature};

        let sender_address = Address::from("0x0172bf37b2ff1bc5ff140634d9981011f54ae6aa");
        let sender_secret =
            Secret::from("8eeda46d11c1630bd1d9c4aace189513d3153b739f56ba6dfb5143b13dcb1eab");

        let mut req = mock_request();
        req.delegate_mode = DelegateMode::PublicTxOrigin;

        assert_eq!(
            req.hash(),
            "0x1fd931cc809dbb13f1c7af7d0a7d57be9be73459e8c35e887adfd618245d6b5a".into(),
        );

        let signed_req = req.sign(&sender_secret);
        assert_ne!(*signed_req.sender(), UNSIGNED_SENDER);
        assert_eq!(sender_address, *signed_req.sender());

        let (unverified_request, sender) = signed_req.deconstruct();
        assert_eq!(sender_address, sender);
        assert_eq!(
            U256::from("95c2586dacf49683fc8493b7bde081470473fdaca62b0d495a83e40d8608e2b8"),
            unverified_request.r,
        );
        assert_eq!(
            U256::from("02f034976a6161f125ae6be425f730ec702981f9bed2cb81899071f3d5fb25fa"),
            unverified_request.s,
        );
        assert_eq!(28, unverified_request.v);

        let expected_sig = Signature::from_rsv(
            &"95c2586dacf49683fc8493b7bde081470473fdaca62b0d495a83e40d8608e2b8".into(),
            &"02f034976a6161f125ae6be425f730ec702981f9bed2cb81899071f3d5fb25fa".into(),
            1,
        );
        assert_eq!(expected_sig, unverified_request.signature());
    }
}
