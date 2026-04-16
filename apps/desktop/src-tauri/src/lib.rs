//! native-tauri — Tauri application entry point

mod commands;

/// Client-local schema bootstrapping for the embedded desktop runtime.
pub mod schema {
    /// SQL migration for the local counter table used by desktop runtime.
    pub const COUNTER_MIGRATION: &str = "CREATE TABLE IF NOT EXISTS counter (\
            tenant_id TEXT PRIMARY KEY,\
            value INTEGER NOT NULL DEFAULT 0,\
            version INTEGER NOT NULL DEFAULT 0,\
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))\
        );\
        CREATE TABLE IF NOT EXISTS counter_outbox (\
            id INTEGER PRIMARY KEY AUTOINCREMENT,\
            event_type TEXT NOT NULL,\
            payload TEXT NOT NULL,\
            source_service TEXT NOT NULL DEFAULT 'counter-service',\
            correlation_id TEXT,\
            created_at TEXT NOT NULL DEFAULT (datetime('now')),\
            published INTEGER NOT NULL DEFAULT 0\
        );\
        CREATE INDEX IF NOT EXISTS idx_counter_outbox_pending \
            ON counter_outbox(published, id);\
        CREATE TABLE IF NOT EXISTS counter_idempotency (\
            idempotency_key TEXT PRIMARY KEY,\
            result_value INTEGER NOT NULL,\
            result_version INTEGER NOT NULL,\
            created_at TEXT NOT NULL DEFAULT (datetime('now'))\
        );";
}

use commands::counter;
use schema::COUNTER_MIGRATION;

use data::ports::lib_sql::LibSqlPort;
use std::path::{Path, PathBuf};
use storage_turso::{EmbeddedTurso, embedded::run_tenant_migrations};
use tauri::{Emitter, Manager};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

const DB_PATH_ENV: &str = "NATIVE_TAURI_TURSO_DB_PATH";
const DEFAULT_TURSO_DB_FILENAME: &str = "runtime_tauri.db";

#[derive(Debug, PartialEq, Eq)]
enum DatabasePathSource {
    Default,
    Env,
}

impl DatabasePathSource {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Env => "env",
        }
    }
}

#[derive(Debug)]
struct ResolvedDatabasePath {
    path: PathBuf,
    source: DatabasePathSource,
}

/// Shared application state managed by Tauri.
pub struct AppState {
    pub db: EmbeddedTurso,
}

/// Initialize observability: tracing-subscriber (terminal) + LogTracer (bridges log → tracing).
///
/// tauri-plugin-log will also capture log events and send them to WebView.
/// Together they give you: terminal + WebView + log file from a single tracing tree.
fn init_observability() {
    // Bridge log crate → tracing (must happen before tracing_subscriber)
    let _ = tracing_log::LogTracer::init();

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,native_tauri=debug"));
    let _ = tracing_subscriber::registry()
        .with(fmt::layer().with_target(true).with_thread_ids(true))
        .with(filter)
        .try_init();
}

fn resolve_dotenv_path(start_dir: &std::path::Path) -> Option<std::path::PathBuf> {
    start_dir
        .ancestors()
        .map(|dir| dir.join(".env"))
        .find(|path| path.is_file())
}

fn resolve_database_path(
    app_data_dir: &Path,
    env_override: Option<&str>,
) -> Result<ResolvedDatabasePath, String> {
    let default_path = app_data_dir.join(DEFAULT_TURSO_DB_FILENAME);

    let (selected, source) = match env_override.map(str::trim).filter(|v| !v.is_empty()) {
        Some(value) => {
            if is_disallowed_memory_path(value) {
                return Err("memory path is not allowed for desktop runtime".to_string());
            }
            (PathBuf::from(value), DatabasePathSource::Env)
        }
        None => (default_path, DatabasePathSource::Default),
    };

    let normalized = if selected.is_absolute() {
        selected
    } else {
        app_data_dir.join(selected)
    };

    let parent = normalized
        .parent()
        .ok_or_else(|| "database path parent directory is missing".to_string())?;

    std::fs::create_dir_all(parent)
        .map_err(|error| format!("failed to create database directory: {error}"))?;

    let canonical_parent = parent
        .canonicalize()
        .map_err(|error| format!("failed to canonicalize database directory: {error}"))?;

    let file_name = normalized
        .file_name()
        .ok_or_else(|| "database filename is missing".to_string())?;

    let canonical_path = canonical_parent.join(file_name);
    let canonical_path_str = canonical_path.to_string_lossy();
    if is_disallowed_memory_path(&canonical_path_str) {
        return Err("memory path is not allowed for desktop runtime".to_string());
    }

    Ok(ResolvedDatabasePath {
        path: canonical_path,
        source,
    })
}

fn is_disallowed_memory_path(value: &str) -> bool {
    let normalized = value.trim().to_ascii_lowercase();
    normalized == ":memory:" || normalized == "memory" || normalized.contains("mode=memory")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_observability();

    let log_plugin = tauri_plugin_log::Builder::default()
        .skip_logger()
        .level(log::LevelFilter::Debug)
        .target(tauri_plugin_log::Target::new(
            tauri_plugin_log::TargetKind::Webview,
        ))
        .build();

    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(log_plugin)
        .invoke_handler(tauri::generate_handler![
            counter::counter_increment,
            counter::counter_decrement,
            counter::counter_reset,
            counter::counter_get_value,
        ]);

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(win) = app.get_webview_window("main") {
                let _ = win.show();
                let _ = win.set_focus();
            }
        }));
    }

    #[cfg(feature = "e2e-testing")]
    {
        builder = builder.plugin(tauri_plugin_playwright::init());
    }

    #[cfg(debug_assertions)]
    {
        builder = builder.plugin(tauri_plugin_mcp::init_with_config(
            tauri_plugin_mcp::PluginConfig::new("My App".to_string())
                .start_socket_server(true)
                .socket_path("/tmp/tauri-mcp.sock".into()),
        ));
    }

    builder
        .setup(|app| {
            let cwd = std::env::current_dir().unwrap_or_default();
            tracing::debug!(?cwd, "startup");

            if let Some(env_path) = resolve_dotenv_path(&cwd) {
                tracing::debug!(?env_path, "looking for .env");
                match dotenvy::from_path_override(&env_path) {
                    Ok(_) => tracing::info!(?env_path, "loaded .env"),
                    Err(error) => tracing::warn!(?env_path, %error, "failed to load .env"),
                }
            }

            let app_data_dir = app
                .path()
                .app_local_data_dir()
                .expect("Failed to resolve app local data directory");
            let db_env_override = std::env::var(DB_PATH_ENV).ok();
            let db_path = resolve_database_path(&app_data_dir, db_env_override.as_deref())
                .expect("Failed to resolve Turso database path");

            tracing::info!(
                provider = "turso",
                path = %db_path.path.display(),
                source = db_path.source.as_str(),
                "database path resolved"
            );

            let db = tauri::async_runtime::block_on(async {
                let file_path = db_path.path.to_string_lossy().into_owned();
                let db = EmbeddedTurso::new(&file_path)
                    .await
                    .expect("Failed to initialize embedded turso database");

                run_tenant_migrations(&db)
                    .await
                    .expect("Failed to run tenant migrations");

                db.execute(COUNTER_MIGRATION, vec![])
                    .await
                    .expect("Failed to run counter migration");

                db
            });

            app.manage(AppState { db: db.clone() });
            app.manage(db.clone());

            let app_handle = app.handle().clone();
            let default_hook = std::panic::take_hook();
            std::panic::set_hook(Box::new(move |info| {
                let msg = if let Some(s) = info.payload().downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = info.payload().downcast_ref::<String>() {
                    s.clone()
                } else {
                    "An unexpected error occurred".to_string()
                };
                tracing::error!(%msg, "panic occurred");
                let _ = app_handle.emit("app:panic", msg);
                default_hook(info);
            }));

            #[cfg(desktop)]
            {
                use tauri::menu::{MenuBuilder, MenuItemBuilder};
                use tauri::tray::TrayIconBuilder;

                let show_item = MenuItemBuilder::with_id("show", "Show").build(app)?;
                let hide_item = MenuItemBuilder::with_id("hide", "Hide").build(app)?;
                let quit_item = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
                let menu = MenuBuilder::new(app)
                    .items(&[&show_item, &hide_item, &quit_item])
                    .build()?;

                TrayIconBuilder::new()
                    .icon(app.default_window_icon().unwrap().clone())
                    .menu(&menu)
                    .on_menu_event(move |app, event| match event.id().as_ref() {
                        "show" => {
                            if let Some(win) = app.get_webview_window("main") {
                                let _ = win.show();
                                let _ = win.set_focus();
                            }
                        }
                        "hide" => {
                            if let Some(win) = app.get_webview_window("main") {
                                let _ = win.hide();
                            }
                        }
                        "quit" => app.exit(0),
                        _ => {}
                    })
                    .on_tray_icon_event(|tray, event| {
                        if let tauri::tray::TrayIconEvent::Click {
                            button: tauri::tray::MouseButton::Left,
                            button_state: tauri::tray::MouseButtonState::Up,
                            ..
                        } = event
                        {
                            let app = tray.app_handle();
                            if let Some(win) = app.get_webview_window("main") {
                                if win.is_visible().unwrap_or(false) {
                                    let _ = win.hide();
                                } else {
                                    let _ = win.show();
                                    let _ = win.set_focus();
                                }
                            }
                        }
                    })
                    .build(app)?;

                if let Some(win) = app.get_webview_window("main") {
                    let win_for_close = win.clone();
                    win.on_window_event(move |event| {
                        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                            api.prevent_close();
                            let _ = win_for_close.hide();
                        }
                    });
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_default_file_path_with_default_source() {
        let temp = tempfile::tempdir().expect("temp dir");
        let resolved = resolve_database_path(temp.path(), None).expect("resolve default path");

        assert_eq!(resolved.source, DatabasePathSource::Default);
        assert!(resolved.path.is_absolute());
        assert_eq!(
            resolved.path.file_name().and_then(std::ffi::OsStr::to_str),
            Some(DEFAULT_TURSO_DB_FILENAME)
        );
        assert!(!resolved.path.to_string_lossy().contains(":memory:"));
    }

    #[test]
    fn resolves_env_override_with_env_source() {
        let temp = tempfile::tempdir().expect("temp dir");
        let resolved =
            resolve_database_path(temp.path(), Some("custom.db")).expect("resolve env path");

        assert_eq!(resolved.source, DatabasePathSource::Env);
        assert!(resolved.path.ends_with("custom.db"));
    }

    #[test]
    fn creates_missing_parent_directory_for_database_file() {
        let temp = tempfile::tempdir().expect("temp dir");
        let missing_parent = temp.path().join("nested").join("db");
        let env_path = missing_parent.join("data.db");

        let resolved = resolve_database_path(temp.path(), env_path.to_str())
            .expect("resolve and create parent");

        assert!(missing_parent.is_dir());
        assert_eq!(resolved.source, DatabasePathSource::Env);
    }

    #[test]
    fn rejects_memory_like_env_override() {
        let temp = tempfile::tempdir().expect("temp dir");
        let err = resolve_database_path(temp.path(), Some("file:test.db?mode=memory"))
            .expect_err("memory-like path should be rejected");

        assert!(err.contains("memory"));
    }
}
