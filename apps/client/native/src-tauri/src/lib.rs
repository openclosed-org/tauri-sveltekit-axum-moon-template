//! native-tauri — Tauri application entry point

mod commands;
mod sync;

use runtime_tauri::commands::{admin, agent, auth, config, counter};

use domain::ports::lib_sql::LibSqlPort;
use storage_turso::{EmbeddedTurso, embedded::run_tenant_migrations};
use sync::SyncEngine;
use sync::engine::init_sync_tables;
use tauri::{Emitter, Manager};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_observability();

    // Initialize the embedded libsql database
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    let db = rt.block_on(async {
        let db = EmbeddedTurso::new(":memory:")
            .await
            .expect("Failed to initialize embedded libsql");
        run_tenant_migrations(&db)
            .await
            .expect("Failed to run tenant migrations");
        db
    });

    // Run counter table migration
    let _ = rt.block_on(async {
        db.execute(usecases::counter_service::COUNTER_MIGRATION, vec![])
            .await
            .expect("Failed to run counter migration")
    });

    for migration in usecases::agent_service::AGENT_MIGRATIONS {
        let _ = rt.block_on(async {
            db.execute(migration, vec![])
                .await
                .expect("Failed to run agent migration")
        });
    }

    // Register EmbeddedLibSql for runtime_tauri commands (counter, admin)
    let db_for_commands = db.clone();
    let app_state = AppState { db };

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
        .manage(app_state)
        .manage(db_for_commands)
        .invoke_handler(tauri::generate_handler![
            auth::start_oauth,
            auth::handle_oauth_callback,
            auth::get_session,
            auth::logout,
            auth::quit_app,
            config::get_config,
            counter::counter_increment,
            counter::counter_decrement,
            counter::counter_reset,
            counter::counter_get_value,
            admin::admin_get_dashboard_stats,
            agent::agent_create_conversation,
            agent::agent_list_conversations,
            agent::agent_get_messages,
            agent::agent_chat,
            commands::sync::sync_start,
            commands::sync::sync_stop,
            commands::sync::sync_once,
            commands::sync::sync_get_stats,
            commands::sync::sync_configure,
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

            let client_id = std::env::var("GOOGLE_CLIENT_ID").unwrap_or_default();
            let client_secret = std::env::var("GOOGLE_CLIENT_SECRET").unwrap_or_default();
            let api_url = std::env::var("API_URL").unwrap_or_default();
            tracing::info!(
                client_id_len = client_id.len(),
                client_secret_len = client_secret.len(),
                %api_url,
                "config loaded"
            );

            let handle = app.handle().clone();
            auth::start_refresh_timer(handle);

            // Initialize sync tables and start background sync if configured
            let remote_url = std::env::var("TURSO_SYNC_URL").ok();
            let auth_token = std::env::var("TURSO_AUTH_TOKEN").ok();

            if let (Some(url), Some(token)) = (remote_url, auth_token) {
                let db = app.state::<AppState>().db.clone();
                let app_handle = app.handle().clone();

                // Spawn async initialization for sync
                tauri::async_runtime::spawn(async move {
                    // Initialize sync metadata tables
                    if let Err(e) = init_sync_tables(&db).await {
                        tracing::error!(%e, "failed to initialize sync tables");
                    }

                    // Create and manage sync engine
                    let sync_engine = SyncEngine::new(db, url, token, sync::SyncConfig::default());
                    let sync_state = commands::sync::SyncState {
                        engine: std::sync::Arc::new(tokio::sync::Mutex::new(sync_engine)),
                    };
                    app_handle.manage(sync_state);

                    // Start background sync
                    let state = app_handle.state::<commands::sync::SyncState>();
                    let mut engine = state.engine.lock().await;
                    let app_handle_for_sync = app_handle.clone();
                    engine.start_background_sync(app_handle_for_sync);
                    drop(engine);

                    tracing::info!("background sync initialized and started");
                });
            } else {
                tracing::info!("sync not configured (TURSO_SYNC_URL and TURSO_AUTH_TOKEN not set)");
            }

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
