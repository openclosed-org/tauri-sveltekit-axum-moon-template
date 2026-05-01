# Contract Docs

> 状态：按需参考，不是默认上下文。
> 协议事实以代码、`packages/contracts/**`、handler/worker 实现和验证输出为准，不在这份文档。

## 协议入口

| 协议面 | 入口 |
|--------|--------|
| HTTP API DTOs | `packages/contracts/api/**` + `servers/**` handler/route |
| Events | `packages/contracts/events/**` + `services/*/model.yaml` + `workers/**` |
| Error Codes | `packages/contracts/errors/**` + handler error mapping code |
| Tauri RPC | 桌面端命令实现 + 前端调用代码 + `packages/contracts/**` |

## 建议用法

1. 先看 `packages/contracts/**` 中相关定义
2. 再看对应 server/worker/service 实现
3. 本文档仅作导航，不作为事实清单

## 一句话结论

协议事实在 contracts、代码和验证输出中确认；这里不再维护大型手工协议手册。
