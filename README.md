> English version is coming soon

# FST Relayer 預覽版本

`FST Relayer`是一個由`FST Network`以 Rust 程式語言實作的 Relayer 服務。

注意: FST Relayer 尚處於測試階段，我們無法保證所有功能都已健全。

## License

FST Relayer 以 GNU GENERAL PUBLIC LICENSE Version 3 進行授權。
此專案中有部分程式碼和實作方式取自 Parity Technologies 的 Parity Ethereum，Parity Ethereum 亦採用 GPL v3 進行授權。

## Technical Overview

- `Proof-of-Relay`：`Proof-of-Relay` 是 `FST Network` 的技術之一，其目的在幫助 `ERC-1376 Token` 的持有者在不需要花費 `Ether`作為手續費的情況下傳送自己持有的 `ERC-1376 Token`，Token 持有者藉由支付自己持有的 `ERC-1376 Token` 作為手續費，透過 `Relayer`的協助發送`ERC-1376 Token` 。簡言之，透過`Proof-of-Relay`這種方法，持有`ERC-1376 Token`卻未持有`Ether`的使用者也可以傳送自己的`ERC-1376 Token`給其他使用者。

- `ERC-1376 Token`：符合[ERC-1376](https://github.com/fstnetwork/EIPs/blob/master/EIPS/eip-1376.md)規範的`Token`稱為`ERC-1376 Token`。

- `ERC-1376 Token 持有者`：`ERC-1376 Token`的持有者，`ERC-1376 Token`的持有者不一定持有`Ether`。`ERC-1376 Token`的持有者能透過發送`Token Transfer Request`給`Relayer`，在`Token Transfer Request`中聲明`Token`的接收者、`Token`的數量以及支付給`Relayer`作為手續費的`Token`數量等資訊。

- `Relayer`: `Relayer`是 `Relay Network`中的特殊節點，`Relayer`的職責在於接收來自`ERC-1376 Token`持有者的`Token Transfer Request`並協助持有者傳送`Token`，`Relayer`藉由消耗自己的`Ether`完成 Token 持有者交付的`Token Transfer Request`，並抽取 Token 持有者在`Token Transfer Request`中所承諾的一定數量的`ERC-1376 Token`作為報酬。

成為`Relayer`有以下必要條件：

- 持有一定數量的`Ether`，藉由這些`Ether`作為支付給礦工的手續費發送合法的`Ethereum Transaction`。
- 至少能以一種方式接收來自 Token 持有者的`Token Transfer Request`。
- 能將收集到的`Token Transfer Request` 發送至區塊鏈網路中執行。

## Build Dependencies

`FST Relayer`需要以最新穩定版本的 Rust 編譯。
我們建議你以[rustup](https://www.rustup.rs/)安裝 Rust，如果你尚未安裝`rustup`請使用以下指令安裝`rustup`：

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

唯有實作了`ERC-1376`規範的 Token 才能透過`Proof-of-Relay` 完成 Token 傳送。

#### Dispatcher Smart Contract

`Dispatcher Smart Contract`是一種在區塊鏈上輔助`Relayer` 的智能合約。`Relayer` 將收集到的多個`Token Transfer Request`壓制成一個`Ethereum Transaction`，
傳送給 Ethereum 上的`Dispatcher Smart Contract`，透過`Dispatcher Smart Contract`的輔助能讓`Relayer`在區塊鏈上依序執行多比`Token Transfer Request`。

`Dispatcher Smart Contract`負責呼叫`ERC-1376 Token`的程式碼片段如下：

```solidity
// 注意：data 當中的資料已經由relayer 在鏈外處理完畢，能夠直接透過call() 呼叫token smart contract

// 當一批Token Transfer Request 只針對單一一個Token 時，我們呼叫此method
function singleTokenDispatch(address token, bytes[] data) public {
    for (uint256 i = 0; i < data.length; ++i) {
        token.call(data[i]);
    }
}

// 當一批Token Transfer Request 混合了多種Token 時，我們呼叫此method
function multipleTokenDispatch(address[] tokens, bytes[] data) public {
    for (uint256 i = 0; i < data.length; ++i) {
        tokens[i].call(data[i]);
    }
}
```

### Off-chain

#### Ethereum Endpoint

`FST Relayer`的運作依賴至少一個`Ethereum Node`，`Ethereum Node`必須是個擁有整本帳本的 Full Node。
Ethereum Node 的架設可參考[Go Ethereum](https://github.com/ethereum/go-ethereum/)或[Parity](https://github.com/paritytech/parity-ethereum/)，

FST Relayer 藉由 Ethereum Node 進行以下工作：

- 發送 Ethereum Transaction，藉此完成 relayer 的最終任務
- 取得區塊鏈上的狀態
- 測試`Token Transfer Request`是否符合發送條件
