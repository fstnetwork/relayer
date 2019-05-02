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
use ethabi::{Contract, Event, EventParam, Function, Param, ParamType};

use super::erc20::ERC20_TOKEN_INTERFACE;

lazy_static! {
    pub static ref ERC1376_TOKEN_INTERFACE: Contract = {
        let mut events = ERC20_TOKEN_INTERFACE.events.clone();
        let mut functions = ERC20_TOKEN_INTERFACE.functions.clone();

        // event SetDelegate(bool isDelegateEnable);
        events.insert("SetDelegate".into(),
            Event {
                name: "SetDelegate".to_owned(),
                anonymous: false,
                inputs: vec![EventParam {
                    name: "isDelegateEnable".to_owned(),
                    kind: ParamType::Bool,
                    indexed: false,
                }]
            });

        // function setDelegate(bool delegate) public
        functions.insert("setDelegate".into(),
            Function {
                name: "setDelegate".to_owned(),
                constant: false,
                inputs: vec![
                    Param {
                        name: "delegate".to_owned(),
                        kind: ParamType::Bool,
                    }
                ],
                outputs: vec![],
            });

        // function transfer(uint256[] data) public returns (bool)
        functions.insert("transfer1376".into(),
            Function{
                name:"transfer".into(),
                inputs : vec![
                    Param {
                        name: "data".to_owned(),
                        kind: ParamType::Array(Box::new(ParamType::Uint(256))),
                    },
                ],
                outputs: vec![
                    Param {
                        name: "".to_owned(),
                        kind: ParamType::Bool,
                    }
                ],
                constant: false,
            }
        );

        // function transferAndCall(address to, uint256 value, bytes data) public payable returns (bool)
        functions.insert("transferAndCall".into(),
            Function{
                name:"transferAndCall".into(),
                inputs : vec![
                    Param {
                        name: "to".to_owned(),
                        kind: ParamType::Address,
                    },
                    Param {
                        name: "value".to_owned(),
                        kind: ParamType::Uint(256),
                    },
                    Param {
                        name: "data".to_owned(),
                        kind: ParamType::Bytes,
                    },
                ],
                outputs: vec![
                    Param {
                        name: "".to_owned(),
                        kind: ParamType::Bool,
                    }
                ],
                constant: false,
            }
        );

        // function delegateTransferAndCall(
        //      uint256 nonce, uint256 fee, uint256 gasAmount, address to,
        //      uint256 value, bytes data, uint8 mode,
        //      uint8 v, bytes32 r, bytes32 s)
        //      public returns (bool)
        functions.insert(
            "delegateTransferAndCall".into(),
            Function {
                name: "delegateTransferAndCall".into(),
                inputs: vec![
                    Param {
                        name: "nonce".to_owned(),
                        kind: ParamType::Uint(256),
                    },
                    Param {
                        name: "fee".to_owned(),
                        kind: ParamType::Uint(256),
                    },
                    Param {
                        name: "gasAmount".to_owned(),
                        kind: ParamType::Uint(256),
                    },
                    Param {
                        name: "to".to_owned(),
                        kind: ParamType::Address,
                    },
                    Param {
                        name: "value".to_owned(),
                        kind: ParamType::Uint(256),
                    },
                    Param {
                        name: "data".to_owned(),
                        kind: ParamType::Bytes,
                    },
                    Param {
                        name: "mode".to_owned(),
                        kind: ParamType::Uint(8),
                    },
                    Param {
                        name: "v".to_owned(),
                        kind: ParamType::Uint(8),
                    },
                    Param {
                        name: "r".to_owned(),
                        kind: ParamType::FixedBytes(32),
                    },
                    Param {
                        name: "s".to_owned(),
                        kind: ParamType::FixedBytes(32),
                    },
                ],
                outputs: vec![Param {
                    name: "".to_owned(),
                    kind: ParamType::Bool,
                }],
                constant: false,
            },
        );

        // function nonceOf(address owner) public view returns (uint256)
        functions.insert("nonceOf".into(),
            Function {
                name: "nonceOf".to_owned(),
                inputs: vec![Param {
                    name: "owner".to_owned(),
                    kind: ParamType::Address,
                }],
                outputs: vec![Param {
                    name: "".to_owned(),
                    kind: ParamType::Uint(256),
                }],
                constant: true,
            },
        );

        // function isDelegateEnable() public view returns (bool)
        functions.insert("isDelegateEnable".into(),
            Function {
                name: "isDelegateEnable".to_owned(),
                inputs: vec![],
                outputs: vec![Param {
                    name: "".to_owned(),
                    kind: ParamType::Bool,
                }],
                constant: true,
            },
        );

        Contract {
            constructor: None,
            fallback: false,
            events,
            functions,
        }
    };
}
