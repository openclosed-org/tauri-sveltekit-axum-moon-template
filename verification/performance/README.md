# Performance Baseline

> HTTP API 性能基线测试。

## 基线指标

| 端点 | P50 | P95 | P99 | 目标 RPS |
|-----|-----|-----|-----|----------|
| GET /healthz | < 5ms | < 10ms | < 20ms | 1000 |
| POST /api/counter/increment | < 20ms | < 50ms | < 100ms | 200 |
| GET /api/counter/value | < 10ms | < 30ms | < 50ms | 500 |
| POST /api/agent/chat | < 100ms | < 500ms | < 1000ms | 50 |

## 工具

- k6 或 wrk 用于 HTTP 压测
- 结果记录在 `verification/performance/results/`

## 实现状态

⚠️ 待实现。
