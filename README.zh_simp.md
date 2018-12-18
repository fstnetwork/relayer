# FST Relayer 预览版本

`FST Relayer`是一个由`FST Network`以 Rust 编程语言实作的 Relayer 服务。

注意: FST Relayer 尚处于测试阶段，我们无法保证所有功能都已健全。

## License

FST Relayer 以 GNU GENERAL PUBLIC LICENSE Version 3 进行授权。
此专案中有部分源码和实作方式取自 Parity Technologies 的 Parity Ethereum，Parity Ethereum 亦采用 GPL v3 进行授权。

## Technical Overview

- `Proof-of-Relay`：`Proof-of-Relay` 是 `FST Network` 的技术之一，其目的在帮助 `ERC-1376 Token` 的持有者在不需要花费 `Ether`作为手续费的情况下传送自己持有的 `ERC-1376 Token`，Token 持有者借由支付自己持有的 `ERC-1376 Token` 作为手续费，透过 `Relayer`的协助发送`ERC-1376 Token` 。简言之，透过`Proof-of-Relay`这种方法，持有`ERC-1376 Token`却未持有`Ether`的使用者也可以传送自己的`ERC-1376 Token`给其他使用者。

- `ERC-1376 Token`：符合[ERC-1376](https://github.com/fstnetwork/EIPs/blob/master/EIPS/eip-1376.md)规范的`Token`称为`ERC-1376 Token`。

- `ERC-1376 Token 持有者`：`ERC-1376 Token`的持有者，`ERC-1376 Token`的持有者不一定持有`Ether`。`ERC-1376 Token`的持有者能透过发送`Token Transfer Request`给`Relayer`，在`Token Transfer Request`中声明`Token`的接收者、`Token`的数量以及支付给`Relayer`作为手续费的`Token`数量等信息。

- `Relayer`: `Relayer`是 `Relay Network`中的特殊节点，`Relayer`的职责在于接收来自`ERC-1376 Token`持有者的`Token Transfer Request`并协助持有者传送`Token`，`Relayer`借由消耗自己的`Ether`完成 Token 持有者交付的`Token Transfer Request`，并抽取 Token 持有者在`Token Transfer Request`中所承诺的一定数量的`ERC-1376 Token`作为报酬。

成为`Relayer`有以下必要条件：

- 持有一定数量的`Ether`，借由这些`Ether`作为支付给矿工的手续费发送合法的`Ethereum Transaction`。
- 至少能以一种方式接收来自 Token 持有者的`Token Transfer Request`。
- 能将收集到的`Token Transfer Request` 发送至区块链网络中运行。

## Build Dependencies

`FST Relayer`需要以最新稳定版本的 Rust 编译。
我们建议你以[rustup](https://www.rustup.rs/)安装 Rust，如果你尚未安装`rustup`请使用以下指令安装`rustup`：

- Linux:

```bash
$ curl https://sh.rustup.rs -sSf | sh
```

## Build from Source Code

- Linux:

```bash
# download FST Relayer source code
$ git clone https://github.com/fstnetwork/fst-relayer
$ cd fst-relayer

# build in release mode
$ cargo build --release
```

## Runtime Dependencies

### On-chain

#### ERC-1376 Token

唯有实作了`ERC-1376`规范的 Token 才能透过`Proof-of-Relay` 完成 Token 传送。

#### Dispatcher Smart Contract

`Dispatcher Smart Contract`是一种在区块链上辅助`Relayer` 的智能合约。`Relayer` 将收集到的多个`Token Transfer Request`压制成一个`Ethereum Transaction`，
传送给 Ethereum 上的`Dispatcher Smart Contract`，透过`Dispatcher Smart Contract`的辅助能让`Relayer`在区块链上依序处理多比`Token Transfer Request`。

`Dispatcher Smart Contract`负责呼叫`ERC-1376 Token`的源码片段如下：

```solidity
// 注意：`data` 当中的数据已经由relayer 在链外处理完毕，能够直接透过call() 呼叫token smart contract

// 当一批Token Transfer Request 只针对单一一个Token 时，我们呼叫此method
function singleTokenDispatch(address token, bytes[] data) public {
    for (uint256 i = 0; i < data.length; ++i) {
        token.call(data[i]);
    }
}

// 当一批Token Transfer Request 混合了多种Token 时，我们呼叫此method
function multipleTokenDispatch(address[] tokens, bytes[] data) public {
    for (uint256 i = 0; i < data.length; ++i) {
        tokens[i].call(data[i]);
    }
}
```

### Off-chain

#### Ethereum Endpoint

`FST Relayer`的运行依赖至少一个`Ethereum Node`，Ethereum Node 必须是个拥有整本帐本的 Full Node，
Ethereum Node 可以透过运行[Go Ethereum](https://github.com/ethereum/go-ethereum/)或[Parity](https://github.com/paritytech/parity-ethereum/)建置。

FST Relayer 借由 Ethereum Node 进行以下工作：

- 发送 Ethereum Transaction 借此完成 relayer 的最终任务
- 取得区块链上的状态
- 测试`Token Transfer Request`是否符合发送条件
