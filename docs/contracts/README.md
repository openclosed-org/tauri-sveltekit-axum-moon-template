# Contract Docs

> 状态：按需参考，不是默认上下文。
> 协议事实以代码、`packages/contracts/**`、handler/worker 实现和验证输出为准，不在这份文档。

## 协议入口

| 协议面 | 入口 |
|--------|--------|
| HTTP API source | `packages/contracts/api/**` + `servers/bff/web-bff/src/handlers/**` |
| HTTP OpenAPI artifact | `packages/contracts/generated/openapi/web-bff.openapi.yaml` |
| Events | `packages/contracts/events/**` + `services/*/model.yaml` + `workers/**` |
| Error Codes | `packages/contracts/errors/**` + handler error mapping code |
| Tauri RPC | 桌面端命令实现 + 前端调用代码 + `packages/contracts/**` |

## 建议用法

1. HTTP 契约先看 `packages/contracts/generated/openapi/web-bff.openapi.yaml`
2. 再看 `packages/contracts/api/**` 和 `servers/bff/web-bff/src/handlers/**` 的 Rust source
3. 事件和错误契约先看 `packages/contracts/events/**`、`packages/contracts/errors/**`
4. 本文档仅作导航，不作为事实清单

## 一句话结论

协议事实在 contracts、Rust/Axum handler、generated OpenAPI artifact 和验证输出中确认；这里不再维护手工 OpenAPI YAML。
