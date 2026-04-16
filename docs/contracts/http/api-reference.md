# HTTP Contract Docs

> 状态：按需参考，不是默认上下文。
>
> 真理源优先级：`packages/contracts/**`、相关 handler/route 代码、可执行验证，高于本文档。

## 1. 这份文档负责什么

这份文档只负责说明：

1. `docs/contracts/http/` 为什么存在。
2. HTTP 合约应以什么为准。
3. 为什么这里不再维护大而全的手工 API 参考。

## 2. 当前真理源

判断 HTTP 合约时，优先看：

1. `packages/contracts/**`
2. `servers/**` 中真实 handler、route、OpenAPI 相关代码
3. `verification/contract/**`
4. `docs/contracts/api-routes.yaml` 这类局部参考文件

如果这些来源和本文冲突，以前者为准。

## 3. 为什么不再维护大而全 API 手册

旧版手工 API 参考有几个问题：

1. 很容易把未来端点、示例响应、认证行为写成现状。
2. 很容易和真实 contracts、handlers、routes 漂移。
3. 对默认后端主链帮助不大，反而会污染上下文。

因此这里不再保留大段端点清单、示例 payload 和环境 URL 表。

## 4. 当前建议用法

需要理解 HTTP 合约时，建议顺序：

1. 看 `packages/contracts/**` 中相关 DTO / error types
2. 看对应 `servers/**` handler 与 route 注册
3. 必要时再看 `docs/contracts/api-routes.yaml` 作为辅助参考

## 5. 一句话结论

HTTP 合约的真理源在 contracts 和代码，这份文档只保留为最小导航页，不再尝试手工维护完整 API 参考。
