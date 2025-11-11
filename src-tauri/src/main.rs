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
    match dotenvy::from_path("../.env") {
        Ok(_) => {
            eprintln!("✅ Successfully loaded .env file");

            // Verify critical environment variables
            if let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY") {
                let masked_key = if api_key.len() > 20 {
                    format!("{}...{}", &api_key[..10], &api_key[api_key.len()-10..])
                } else {
                    "***".to_string()
                };
                eprintln!("✅ ANTHROPIC_API_KEY loaded: {}", masked_key);
            } else {
                eprintln!("⚠️  ANTHROPIC_API_KEY not found in environment");
            }
        }
        Err(e) => {
            eprintln!("⚠️  Failed to load .env file: {}", e);
            eprintln!("Using system environment variables instead");
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

            // Update Commands
            update::check_for_updates,
            update::install_update,
            update::get_app_version,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
