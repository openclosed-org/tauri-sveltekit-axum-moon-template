//! Unit tests for agent service tool execution.

use agent_service::infrastructure::libsql_adapter::{execute_tool_by_name, persist_tool_result};
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Default)]
struct MockLibSqlPort {
    counter: Arc<Mutex<i64>>,
    messages: Arc<Mutex<Vec<(String, String, String, String)>>>,
}

#[async_trait]
impl domain::ports::lib_sql::LibSqlPort for MockLibSqlPort {
    async fn health_check(&self) -> Result<(), domain::ports::lib_sql::LibSqlError> {
        Ok(())
    }

    async fn execute(
        &self,
        sql: &str,
        params: Vec<String>,
    ) -> Result<u64, domain::ports::lib_sql::LibSqlError> {
        if sql.contains("INSERT INTO messages") && params.len() >= 4 {
            self.messages.lock().await.push((
                params[0].clone(),
                params[1].clone(),
                params[2].clone(),
                params[3].clone(),
            ));
            return Ok(1);
        }
        Ok(1)
    }

    async fn query<T: DeserializeOwned + Send + Sync>(
        &self,
        sql: &str,
        _params: Vec<String>,
    ) -> Result<Vec<T>, domain::ports::lib_sql::LibSqlError> {
        if sql.contains("SELECT value FROM counter") {
            let v = *self.counter.lock().await;
            let raw = json!([[v]]);
            return serde_json::from_value(raw)
                .map_err(|e| Box::new(e) as domain::ports::lib_sql::LibSqlError);
        }
        if sql.contains("SELECT id, name, created_at FROM tenant") {
            let raw = json!([{"id": "t1", "name": "Alpha", "created_at": "2026-01-01T00:00:00Z"}]);
            return serde_json::from_value(raw)
                .map_err(|e| Box::new(e) as domain::ports::lib_sql::LibSqlError);
        }
        serde_json::from_value(json!([]))
            .map_err(|e| Box::new(e) as domain::ports::lib_sql::LibSqlError)
    }
}

#[tokio::test]
async fn executes_get_counter_value_tool_and_persists_result_message() {
    let port = MockLibSqlPort::default();
    *port.counter.lock().await = 7;

    let result = execute_tool_by_name(&port, "conv-1", "get_counter_value", json!({}))
        .await
        .expect("tool call should succeed");

    assert!(result.contains("counter"));
    assert!(result.contains('7'));
    assert_eq!(port.messages.lock().await.len(), 1);
}

#[tokio::test]
async fn executes_list_tenants_tool_and_returns_summary() {
    let port = MockLibSqlPort::default();

    let result = execute_tool_by_name(&port, "conv-2", "list_tenants", json!({}))
        .await
        .expect("tool call should succeed");

    assert!(result.contains("Alpha"));
    assert_eq!(port.messages.lock().await.len(), 1);
}

#[tokio::test]
async fn executes_system_status_and_handles_unknown_tool_safely() {
    let port = MockLibSqlPort::default();

    let status = execute_tool_by_name(&port, "conv-3", "get_system_status", json!({}))
        .await
        .expect("status tool call should succeed");
    assert!(status.contains("ok"));

    let unknown = execute_tool_by_name(&port, "conv-3", "not_allowed_tool", json!({})).await;
    assert!(unknown.is_err());
}
