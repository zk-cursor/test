# zkchain: 一个最小可运行的 ZK 区块链样例（Rust）

zkchain 是一个教学用途的最小实现，用 Rust 构建，演示了如何在交易层引入零知识区间证明（Bulletproofs）。项目包含：
- 一个 `zk` 库：封装 Bulletproofs 区间证明与交易签名验证
- 一个 `zkchain-node` HTTP 节点：接收 ZK 交易、验证、出块
- 一个 `zkchain-client` CLI 客户端：生成密钥、构造并发送 ZK 交易

本项目重点演示 ZK 交易的生成与验证流程，而非完整的共识、UTXO/账户模型或网络传播。

---

## 目录结构

```
gpt5cursor/
├── Cargo.toml              # 工作区配置
├── README.md
├── zk/                     # ZK 与交易/账本最小实现
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── zkps.rs        # Bulletproofs 区间证明封装
│       ├── tx.rs          # 地址/交易/签名与校验
│       └── ledger.rs      # 极简“链”和区块结构
├── zkchain-node/           # 节点（Axum HTTP 服务）
│   ├── Cargo.toml
│   └── src/main.rs
└── zkchain-client/         # 客户端 CLI
    ├── Cargo.toml
    └── src/main.rs
```

---

## 功能概览

- 使用 Bulletproofs 生成/验证交易金额的区间证明（0 ≤ amount < 2^bits，默认 bits=32）
- 金额承诺采用 Ristretto Pedersen 承诺
- 交易使用 Ed25519 对消息哈希签名
- 节点提供 HTTP API：`/submit_tx`、`/mine`、`/height`、`/health`
- 客户端支持：生成密钥、发送 ZK 交易

注意：当前实现仅做验证演示：
- `/submit_tx` 仅验证交易有效性并返回结果，不保存内存池
- `/mine` 需显式提交一组交易（JSON），节点将其打包为新区块
- 未实现余额模型与花费检查，仅演示 ZK 证明与签名校验

---

## 构建与运行

### 先决条件
- Rust 工具链（建议 stable 最新版）：`https://www.rust-lang.org/tools/install`
- macOS / Linux（Windows 亦可，但命令稍有差异）

### 构建

```bash
# 在仓库根目录
cargo build
```

### 启动节点

```bash
# 终端 1（建议带日志）
RUST_LOG=info cargo run -p zkchain-node
# 启动后监听 127.0.0.1:8080
```

健康检查：

```bash
curl -s http://127.0.0.1:8080/health
# ok
```

### 使用客户端

打开另一个终端：

```bash
# 终端 2
# 生成密钥与地址
cargo run -p zkchain-client -- gen-key
# 输出示例：
# pub: <32字节公钥hex>
# addr: <32字节地址hex>
# sk: <32字节私钥base64>

# 发送一笔 ZK 交易到指定收款地址（hex），金额 42，默认 bits=32
cargo run -p zkchain-client -- send-tx <to_addr_hex> 42
# 返回节点验证结果 JSON，例如：{"ok":true}
```

出块（演示）：
- 由于 `/submit_tx` 仅做验证，不会缓存交易。你可以通过自定义脚本或在应用层保存交易 JSON 后，向 `/mine` 提交交易数组（`Vec<Tx>` 的 JSON）。
- `Tx` 的序列化结构较原始（如 `[u8; 32]` 序列化为 32 个整数组成的数组），不推荐手写；建议在业务中由应用层管理交易对象并直接 POST。

示例（仅示意，不可直接复制）：

```bash
curl -X POST http://127.0.0.1:8080/mine \
  -H 'Content-Type: application/json' \
  -d '[ /* 这里放多个通过客户端或程序生成的 Tx JSON */ ]'
```

查询区块高度：

```bash
curl -s http://127.0.0.1:8080/height
# {"height": 0}  # 创世块高度为 0，出第一个块后返回 1
```

---

## HTTP API

- GET `/health`
  - 返回：`"ok"`

- POST `/submit_tx`
  - Body：`Tx` 的 JSON（参见 `zk::tx::Tx` 的序列化结构）
  - 返回：`{"ok": true}` 或 `{"ok": false, "error": "..."}`

- POST `/mine`
  - Body：`[Tx, ...]` 的 JSON 数组
  - 返回：`{"ok": true, "height": <u64>, "hash": <hex>}` 或错误 JSON

- GET `/height`
  - 返回：`{"height": <u64>}`

---

## 设计与加密细节（简要）

- 区间证明：
  - 使用 `bulletproofs` v4 + `curve25519-dalek-ng`（Ristretto 曲线）
  - `prove_single/verify_single` 生成/验证金额的范围 0..2^bits
  - 承诺与证明通过 `serde_json` 序列化为字节数组

- 交易签名：
  - 使用 Ed25519 (`ed25519-dalek`)
  - 签名消息为 `SHA-256(from || to || amount_commitment || range_proof_bytes)`

- 区块：
  - 区块哈希：对 `{height, prev_hash, txs_json}` 计算 SHA-256
  - 共识/难度/时间戳：未实现（演示用）

---

## 限制与安全提示

- 未实现余额模型、账户状态与双花检查
- 未实现网络传播、共识、持久化存储
- 交易/区块数据结构简化，序列化形式偏底层，不适合直接对外开放
- 本项目仅供学习与 PoC，勿用于生产环境

---

## 常见问题

- 端口被占用：修改 `zkchain-node/src/main.rs` 中监听地址或释放 8080 端口
- 构建失败：请更新 Rust 工具链与依赖缓存 `cargo update`
- 运行无输出：加上 `RUST_LOG=info`

---

## 许可证

MIT