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
use super::rpc_apis;

mod errors;
mod service;
mod v1;

type HttpServerBuilder = jsonrpc_http_server::ServerBuilder;
type IpcServerBuilder = jsonrpc_ipc_server::ServerBuilder;

use jsonrpc_core::IoHandler as JsonRpcIoHandler;

pub use jsonrpc_ipc_server::{
    MetaExtractor as IpcMetaExtractor, RequestContext as IpcRequestContext,
};

use super::service as relayer_service;

pub use self::service::Service;
pub use self::service::ServiceParams;
pub use self::service::{HttpConfiguration, IpcConfiguration, WebSocketConfiguration};
