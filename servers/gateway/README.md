# Gateway Server

> 目的：说明 `gateway` 当前只是轻量 Pingora reverse proxy，而不是完整 edge control plane。

## 状态

- status: `reference-lightweight`
- 角色：当前 web/api 流量入口的轻量反向代理
- 说明：已实现 `/healthz`、`/readyz` 与按路径转发到 `web-bff` / web upstream，但未实现完整 API gateway、policy engine 或 Gateway API 控制面能力

## 责任

1. 将 `/api/*` 转发到 `web-bff`。
2. 将其余 web 流量转发到静态 web upstream。
3. 提供本地 health/readiness 探针。

## 入口

1. `src/main.rs`：Pingora 代理主入口与路由规则。

## 不要这样用

1. 不要把它写成已实现的 edge platform、API gateway product 或 Gateway API 控制面。
2. 不要把未来的 auth, rate limit, policy, canary 等能力当成当前已落地事实。
