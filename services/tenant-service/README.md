# Tenant Service

> 目的：说明 `tenant-service` 在当前仓库里作为复杂 service 参考样例时，负责什么、适合何时参考，以及哪些复杂度不该被默认复制。

## 状态

- status: `reference`
- 角色：多实体、多租户、workflow 与补偿语义的参考 service
- 说明：当新能力涉及租户隔离、长事务或补偿时，再把它作为参考样例

## 责任

1. 维护 `tenant` 与 `tenant-member` 的 ownership 边界。
2. 提供 onboarding、member invitation、compensation-aware mutation 的参考语义。
3. 对齐 `platform/model/workflows/tenant-onboarding.yaml` 这类 durable workflow 路径。

## 入口

1. `model.yaml`：service-local semantics 真理源。
2. `src/domain/`：tenant 与 membership 规则。
3. `src/application/`：命令入口与 workflow 编排。
4. `src/ports/`：持久化与外部依赖抽象。
5. `platform/model/workflows/tenant-onboarding.yaml`：与平台模型挂接的 workflow 参考。

## 验证

```bash
cargo check -p tenant-service
cargo test -p tenant-service
```

## 不要这样用

1. 不要把它当成所有新 service 的默认起点；简单单聚合能力应先看 `counter-service`。
2. 不要把 workflow、补偿和多实体边界机械复制到并不需要这些复杂度的能力上。
3. 不要跳过 `model.yaml` 直接从实现代码倒推出语义边界。
