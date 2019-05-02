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
extern crate byteorder;
extern crate ethereum_types;

mod token;

pub use self::token::Token;

pub mod encoder {
    use super::byteorder;
    use super::Token;

    pub mod packed {

        use super::byteorder::{BigEndian, ByteOrder};
        use super::Token;

        pub fn encode(tokens: &[Token]) -> Vec<u8> {
            let mediatas: Vec<_> = tokens.iter().map(encode_token).collect();
            mediatas.iter().flat_map(|m| m.to_vec()).collect()
        }

        pub fn encode_token(token: &Token) -> Vec<u8> {
            match *token {
                Token::Bytes(ref bytes) => bytes.clone(),
                Token::Address(ref address) => address.to_vec(),
                Token::String(ref s) => s.clone().into(),
                Token::Bool(b) => vec![if b { 0x01u8 } else { 0x00u8 }],
                Token::U8(u) => vec![u],
                Token::U16(u) => {
                    let mut buf = [0u8; 2];
                    BigEndian::write_u16(&mut buf, u);
                    buf.to_vec()
                }
                Token::U32(u) => {
                    let mut buf = [0u8; 4];
                    BigEndian::write_u32(&mut buf, u);
                    buf.to_vec()
                }
                Token::U64(u) => {
                    let mut buf = [0u8; 8];
                    BigEndian::write_u64(&mut buf, u);
                    buf.to_vec()
                }
                Token::U128(ref u) => {
                    let mut buf = [0u8; 16];
                    u.to_big_endian(&mut buf);
                    buf.to_vec()
                }
                Token::U256(ref u) => {
                    let mut buf = [0u8; 32];
                    u.to_big_endian(&mut buf);
                    buf.to_vec()
                }
                Token::U512(ref u) => {
                    let mut buf = [0u8; 64];
                    u.to_big_endian(&mut buf);
                    buf.to_vec()
                }
            }
        }

        #[cfg(test)]
        mod tests {

            extern crate keccak_hash;
            use self::keccak_hash::keccak as keccak_hash;

            use super::*;

            #[test]
            fn test_empty_bytes() {
                let empty_bytes = Vec::new();
                let empty_slice: &[u8] = &[];
                assert_eq!(empty_bytes, empty_slice.to_vec());
                assert_eq!(
                    keccak_hash(encode_token(&Token::Bytes(empty_bytes.clone()))),
                    "0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470".into(),
                );
            }

            #[test]
            fn test_u8() {
                assert_eq!(encode_token(&Token::U8(0x12u8)), &[0x12]);
            }

            #[test]
            fn test_u16() {
                assert_eq!(encode_token(&Token::U16(0x1234u16)), &[0x12, 0x34]);
            }

            #[test]
            fn test_u32() {
                assert_eq!(
                    encode_token(&Token::U32(0x12345678u32)),
                    &[0x12, 0x34, 0x56, 0x78]
                );
                assert_eq!(
                    keccak_hash(encode(&[Token::U32(0x12345678u32)])),
                    "0x30ca65d5da355227c97ff836c9c6719af9d3835fc6bc72bddc50eeecc1bb2b25".into(),
                );
            }

            #[test]
            fn test_u64() {
                assert_eq!(
                    encode_token(&Token::U64(0x1234567812345678u64)),
                    &[0x12, 0x34, 0x56, 0x78, 0x12, 0x34, 0x56, 0x78]
                );

                assert_eq!(
                    keccak_hash(encode(&[Token::U64(0x1234567812345678u64)])),
                    "0xddc7bfc56824b4b38b976f2e46a7dfb6014206c64fcf772de4bef1984d83584f".into(),
                );
            }

            #[test]
            fn test_combined_u32_and_u64() {
                let data = encode(&[Token::U32(0x12345678u32), Token::U64(0x1234567812345678u64)]);
                assert_eq!(
                    data,
                    &[0x12, 0x34, 0x56, 0x78, 0x12, 0x34, 0x56, 0x78, 0x12, 0x34, 0x56, 0x78,]
                );

                assert_eq!(
                    keccak_hash(data),
                    "0x4370b647685d9b535929e09d89c28de2be6827267d5e1b4b6d080d5f27b4db1a".into(),
                );
            }
        }
    }
}
