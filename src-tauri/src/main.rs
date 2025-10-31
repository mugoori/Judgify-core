// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod algorithms;
mod commands;
mod services;
mod database;
mod utils;

use commands::*;

fn main() {
    // Load .env file from project root (one level up from src-tauri)
    if let Err(e) = dotenvy::from_path("../.env") {
        eprintln!("Warning: Failed to load .env file: {}", e);
        eprintln!("Using system environment variables instead");
    }

    tauri::Builder::default()
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
            workflow::create_workflow,
            workflow::get_workflow,
            workflow::get_all_workflows,
            workflow::update_workflow,
            workflow::delete_workflow,
            workflow::validate_workflow,

            // System Commands
            system::get_system_status,
            system::get_system_stats,
            system::get_data_directory,
            system::export_database,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
