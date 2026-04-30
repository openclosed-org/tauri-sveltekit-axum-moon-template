# Tenant Service

> 目的：说明 `tenant-service` 在当前仓库里作为复杂 service 参考样例时，负责什么、适合何时参考，以及哪些复杂度不该被默认复制。

## 状态

- status: `secondary-semantics-reference`
- 角色：多实体、多租户、workflow 与补偿语义的二级参考 service
- 说明：它保留更复杂的语义样例，但不是默认后端参考链，也不是新 service 的默认起点

## 责任

1. 维护 `tenant` 与 `tenant-member` 的 ownership 边界。
2. 提供 onboarding、member invitation、compensation-aware mutation 的参考语义。
3. 对齐 `platform/model/workflows/tenant-onboarding.yaml` 这类 durable workflow 路径。

## 入口

1. `model.yaml`：service-local declared semantics index。
2. `src/domain/`：tenant 与 membership 规则。
3. `src/application/`：命令入口与 workflow 编排。
4. `src/ports/`：持久化与外部依赖抽象。
5. `platform/model/workflows/tenant-onboarding.yaml`：与平台模型挂接的 workflow 参考。

## Contracts Boundary

`src/events/mod.rs` 中的 `TenantEvent` 当前是 service-local orchestration 类型，不是跨进程事件契约。边界规则：

1. 写入 `event_outbox` 的 tenant 事件必须先进入 `packages/contracts/events` 的 `AppEvent`。
2. 通过 HTTP/RPC/message 暴露的 DTO、Event、ErrorCode 必须先进入 `packages/contracts/**`。
3. 只在 `tenant-service` crate 内部编排、测试或 workflow 草图中使用的类型可以留在 service 内。
4. 当前 shared tenant events 只有 `TenantCreated` 与 `TenantMemberAdded` 已在 `contracts_events::AppEvent` 中表达。

## 验证

```bash
cargo check -p tenant-service
cargo test -p tenant-service
```

## 不要这样用

1. 不要把它当成所有新 service 的默认起点；简单单聚合能力应先看 `counter-service`。
2. 不要把 workflow、补偿和多实体边界机械复制到并不需要这些复杂度的能力上。
3. 不要跳过 `model.yaml` 的声明意图，直接从实现代码倒推出跨边界语义。
4. 不要把这里更完整的目标态语义写成当前仓库的默认后端主链。
