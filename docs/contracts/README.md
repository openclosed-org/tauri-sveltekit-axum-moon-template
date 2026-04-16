# Contract Docs Index

> 目标：说明 `docs/contracts/**` 在当前文档体系里的定位。

## 1. 当前定位

`docs/contracts/**` 是按需阅读的局部协议文档，不属于默认后端上下文。

默认情况下，判断协议现状应优先看：

1. `packages/contracts/**`
2. 相关 `services/*/model.yaml`
3. 相关 `servers/**`、`workers/**`、桌面端实现代码
4. `verification/contract/**`

## 2. 为什么不再维护大型手工协议手册

历史上的 HTTP / events / error codes / Tauri RPC 手册都存在同类问题：

1. 很容易和代码、contracts、service semantics 漂移。
2. 很容易把未来态或示例写成现状。
3. 对默认后端主链帮助有限，却会提高上下文噪音。

因此这里保留的文档主要是：

1. 最小导航页
2. 局部辅助参考
3. 仍有一定检索价值、但不再作为事实真理源的材料

## 3. 当前目录说明

1. `api-routes.yaml`：局部 OpenAPI 风格参考，不高于真实 contracts 与路由代码
2. `error-codes.md`：错误码导航页
3. `events/event-schemas.md`：事件合约导航页
4. `http/api-reference.md`：HTTP 合约导航页
5. `rpc/tauri-commands.md`：桌面 RPC 导航页

## 4. 一句话结论

`docs/contracts/**` 现在的目标不是充当大型手工协议百科，而是为需要深入局部协议面的任务提供最小导航入口。
