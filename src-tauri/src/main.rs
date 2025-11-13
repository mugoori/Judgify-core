// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod algorithms;
mod commands;
mod engines;
mod services;
mod database;
mod utils;
mod tray;

use commands::*;

fn main() {
    // Load .env file from project root (one level up from src-tauri)
    let env_loaded = match dotenvy::from_path("../.env") {
        Ok(_) => {
            eprintln!("✅ Successfully loaded .env file");

            // Verify critical environment variables
            if let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY") {
                let masked_key = if api_key.len() > 20 {
                    format!("{}...{}", &api_key[..10], &api_key[api_key.len()-10..])
                } else {
                    "***".to_string()
                };
                eprintln!("✅ ANTHROPIC_API_KEY loaded from .env: {}", masked_key);
            } else {
                eprintln!("⚠️  ANTHROPIC_API_KEY not found in .env");
            }
            true
        }
        Err(e) => {
            eprintln!("⚠️  Failed to load .env file: {}", e);
            false
        }
    };

    // .env 파일이 없거나 API 키가 없으면 keyring에서 시도 (프로덕션 빌드용)
    if !env_loaded || std::env::var("ANTHROPIC_API_KEY").is_err() {
        eprintln!("🔑 Attempting to load API key from system keychain...");

        match keyring::Entry::new("Judgify", "claude_api_key") {
            Ok(entry) => {
                match entry.get_password() {
                    Ok(api_key) => {
                        std::env::set_var("ANTHROPIC_API_KEY", &api_key);
                        let masked_key = if api_key.len() > 20 {
                            format!("{}...{}", &api_key[..10], &api_key[api_key.len()-10..])
                        } else {
                            "***".to_string()
                        };
                        eprintln!("✅ ANTHROPIC_API_KEY loaded from keychain: {}", masked_key);
                    }
                    Err(_) => {
                        eprintln!("ℹ️  No API key found in keychain. Please set it in Settings page.");
                    }
                }
            }
            Err(e) => {
                eprintln!("⚠️  Failed to access keychain: {}", e);
            }
        }
    }

    tauri::Builder::default()
        .system_tray(tray::create_tray())
        .on_system_tray_event(tray::handle_tray_event)
        .on_window_event(|event| {
            // 창 닫기 요청시 트레이로 최소화 (백그라운드 실행)
            tray::handle_window_close(event.window(), event.event());
        })
        .invoke_handler(tauri::generate_handler![
            // Judgment Service Commands
            judgment::execute_judgment,
            judgment::get_judgment_history,

            // Learning Service Commands
            learning::save_feedback,
            learning::get_few_shot_samples,
            learning::extract_rules,

            // BI Service Commands
            bi::generate_bi_insight,
            bi::generate_bi_insight_stream,  // Phase 5: 실시간 스트리밍

            // Chat Service Commands
            chat::send_chat_message,
            chat::get_chat_history,
            chat::test_claude_api,

            // Workflow Service Commands
            // workflow::create_workflow,
            // workflow::get_workflow,
            // workflow::get_all_workflows,
            // workflow::update_workflow,
            // workflow::delete_workflow,
            // workflow::validate_workflow,
            // workflow::validate_rule_expression,
            workflow::generate_workflow_with_llm,  // Phase 32: LLM workflow generation
            workflow::simulate_workflow_step,      // Week 5 Task 4: Step-by-step simulation

            // System Commands
            system::get_system_status,
            system::get_system_stats,
            system::get_data_directory,
            system::export_database,
            system::get_token_metrics,
            system::save_api_key,
            system::load_api_key,
            system::delete_api_key,

            // Update Commands
            update::check_for_updates,
            update::install_update,
            update::get_app_version,

            // Backup Commands (Phase 8 Task 8.3)
            backup::create_backup,
            backup::restore_backup,
            backup::list_backups,
            backup::get_backup_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
