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
use std::collections::HashMap;

use ethabi::{Contract, Event, EventParam, Function, Param, ParamType};

lazy_static! {
    pub static ref ERC20_TOKEN_INTERFACE: Contract = {
        let mut events: HashMap<String, Event> = HashMap::new();
        let mut functions: HashMap<String, Function> = HashMap::new();

        // event Transfer(address indexed from, address indexed to, uint tokens);
        events.insert(
            "Transfer".into(),
            Event {
                name: "Transfer".into(),
                inputs: vec![
                    EventParam {
                        name: "from".to_owned(),
                        kind: ParamType::Address,
                        indexed: true,
                    },
                    EventParam {
                        name: "to".to_owned(),
                        kind: ParamType::Address,
                        indexed: true,
                    },
                    EventParam {
                        name: "tokens".to_owned(),
                        kind: ParamType::Uint(256),
                        indexed: false,
                    },
                ],
                anonymous: false,
            },
        );

        // event Approval(address indexed tokenOwner, address indexed spender, uint tokens);
        events.insert(
            "Approval".into(),
            Event {
                name: "Approval".into(),
                inputs: vec![
                    EventParam {
                        name: "tokenOwner".to_owned(),
                        kind: ParamType::Address,
                        indexed: true,
                    },
                    EventParam {
                        name: "spender".to_owned(),
                        kind: ParamType::Address,
                        indexed: true,
                    },
                    EventParam {
                        name: "tokens".to_owned(),
                        kind: ParamType::Uint(256),
                        indexed: false,
                    },
                ],
                anonymous: false,
            },
        );

        functions.insert("symbol".into(),
            Function{
                name: "symbol".into(),
                constant: true,
                inputs:vec![],
                outputs: vec![
                    Param {
                        name: "".to_owned(),
                        kind: ParamType::String,
                    },
                ]
            },
        );

        // function totalSupply() public constant returns (uint);
        functions.insert(
            "totalSupply".into(),
            Function {
                name: "totalSupply".into(),
                constant: true,
                inputs: vec![],
                outputs: vec![Param {
                    name: "".to_owned(),
                    kind: ParamType::Uint(256),
                }],
            },
        );

        // function balanceOf(address tokenOwner) public constant returns (uint balance);
        functions.insert(
            "balanceOf".to_owned(),
            Function {
                name: "balanceOf".into(),
                constant: true,
                inputs: vec![Param {
                    name: "tokenOwner".to_owned(),
                    kind: ParamType::Address,
                }],
                outputs: vec![Param {
                    name: "balance".to_owned(),
                    kind: ParamType::Uint(256),
                }],
            },
        );

        // function allowance(address tokenOwner, address spender) public constant returns (uint remaining);
        functions.insert(
            "allowance".to_owned(),
            Function {
                name: "allowance".into(),
                constant: true,
                inputs: vec![
                    Param {
                        name: "tokenOwner".to_owned(),
                        kind: ParamType::Address,
                    },
                    Param {
                        name: "spender".to_owned(),
                        kind: ParamType::Address,
                    },
                ],
                outputs: vec![Param {
                    name: "remaining".to_owned(),
                    kind: ParamType::Uint(256),
                }],
            },
        );

        // function approve(address spender, uint tokens) public returns (bool success);
        functions.insert(
            "approve".to_owned(),
            Function {
                name: "approve".into(),
                constant: false,
                inputs: vec![
                    Param {
                        name: "spender".to_owned(),
                        kind: ParamType::Address,
                    },
                    Param {
                        name: "tokens".to_owned(),
                        kind: ParamType::Uint(256),
                    },
                ],
                outputs: vec![Param {
                    name: "success".to_owned(),
                    kind: ParamType::Bool,
                }],
            },
        );

        // function transferFrom(address from, address to, uint tokens) public returns (bool success);
        functions.insert(
            "transferFrom".to_owned(),
            Function {
                name: "transferFrom".into(),
                constant: false,
                inputs: vec![
                    Param {
                        name: "from".to_owned(),
                        kind: ParamType::Address,
                    },
                    Param {
                        name: "to".to_owned(),
                        kind: ParamType::Address,
                    },
                    Param {
                        name: "tokens".to_owned(),
                        kind: ParamType::Uint(256),
                    },
                ],
                outputs: vec![Param {
                    name: "success".to_owned(),
                    kind: ParamType::Bool,
                }],
            },
        );

        // function transfer(address to, uint tokens) public returns (bool success);
        functions.insert(
            "transfer".to_owned(),
            Function {
                name: "transfer".into(),
                constant: false,
                inputs: vec![
                    Param {
                        name: "to".to_owned(),
                        kind: ParamType::Address,
                    },
                    Param {
                        name: "tokens".to_owned(),
                        kind: ParamType::Uint(256),
                    },
                ],
                outputs: vec![Param {
                    name: "success".to_owned(),
                    kind: ParamType::Bool,
                }],
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
