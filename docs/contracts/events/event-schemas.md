# Event Contract Docs

> 状态：按需参考，不是默认上下文。
>
> 真理源：`packages/contracts/events/**` 与相关 service semantics / worker code。

## 1. 这份文档负责什么

这份文档只保留事件合约导航职责，不再手工维护完整 event catalog。

## 2. 当前真理源

判断事件形状、来源和兼容策略时，优先看：

1. `packages/contracts/events/**`
2. `services/*/model.yaml` 中声明的 published events
3. `workers/**` 中对消息 envelope、replay、projection 的真实消费方式

## 3. 为什么收口

旧版事件文档的问题是：

1. 它很容易把样例 payload 写成已实现 schema。
2. 它很容易把未落地 topic、事件名、source service 写成现状。
3. 它会和 contracts、service semantics、worker reality 三边同时漂移。

## 4. 当前建议用法

理解事件合约时，建议顺序：

1. 看 `packages/contracts/events/**`
2. 对照对应 service 的 `model.yaml`
3. 再看 relay / projector / downstream worker 的消费代码

## 5. 一句话结论

事件合约必须由 contracts、service semantics 和 worker reality 共同取证，不能再依赖手工维护的大型事件手册。
