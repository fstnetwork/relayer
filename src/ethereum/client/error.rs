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

error_chain! {
    foreign_links {
        ContractAbi( crate::contract_abi::Error);
        FromHex(rustc_hex::FromHexError);
        Hyper(hyper::Error);
        Json(serde_json::Error);
        Io(::std::io::Error) #[cfg(unix)];
    }

    errors {
        NoSuchField(field_name: &'static str){
            description("No such field")
            display("No such field: {}", field_name)
        }
        ParseHex {
            description("Parse Hex Error")
            display("Parse Hex Error")
        }
        JsonRpc(t: jsonrpc_core::Error) {
            description("JSON RPC Error")
            display("JSON RPC Error: {:?}", t)
        }
    }
}
