# Adapters

> 外部协议适配器。每个 adapter 实现一个 runtime port 或 domain port 的具体后端。

## 当前结构

```
adapters/
├── auth/google/         # OAuth Google 登录适配器
├── hosts/tauri/         # Tauri 桌面客户端宿主（最复杂的组合层）
├── storage/surrealdb/   # SurrealDB 存储实现
├── storage/turso/       # Turso/LibSQL 存储实现
├── telemetry/otel/      # OpenTelemetry metrics 集成
└── telemetry/tracing/   # OpenTelemetry tracing 集成
```

## 已清理项

以下 adapter stub 已删除（空目录或仅 Cargo.toml）：
- `auth/{dpop,oauth,passkey}/`
- `cache/`
- `chains/{base,evm,solana,ton}/`
- `hosts/{base-app,browser-extension,farcaster-miniapp,telegram-miniapp}/`
- `protocols/{atproto,farcaster,nostr}/`
- `storage/{extension-storage,indexeddb,sqlite,tauri-store}/`

## 规则

**不新增**新的 `adapters/*/*` 除非：
1. 有真实的外部协议需要接入
2. 实现了至少一个 port trait 的完整功能
