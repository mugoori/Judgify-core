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
    // ✅ Phase 1: Keychain-first loading strategy
    // 1단계: Keychain에서 API 키 로드 (프로덕션 + 개발 공통)
    load_secrets_from_keychain();

    // 2단계: 개발 환경에서만 .env 파일 fallback (Keychain에 없을 경우)
    #[cfg(debug_assertions)]
    {
        if std::env::var("ANTHROPIC_API_KEY").is_err() {
            eprintln!("🔍 Development mode: Attempting to load .env file as fallback...");
            match dotenvy::from_path("../.env") {
                Ok(_) => {
                    eprintln!("✅ Successfully loaded .env file");
                    if let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY") {
                        let masked_key = mask_api_key(&api_key);
                        eprintln!("✅ ANTHROPIC_API_KEY loaded from .env: {}", masked_key);
                    }
                }
                Err(e) => {
                    eprintln!("⚠️  Failed to load .env file: {}", e);
                    eprintln!("ℹ️  Please set API key in Settings page or create .env file");
                }
            }
        }
    }

    // 최종 검증
    if std::env::var("ANTHROPIC_API_KEY").is_err() {
        eprintln!("⚠️  ANTHROPIC_API_KEY not set. Please configure in Settings.");
    }

    run();
}

/// Keychain에서 시크릿 로드 (프로덕션 + 개발 공통)
fn load_secrets_from_keychain() {
    eprintln!("🔑 Loading secrets from system keychain...");

    // ANTHROPIC_API_KEY 로드
    match keyring::Entry::new("Judgify", "claude_api_key") {
        Ok(entry) => {
            match entry.get_password() {
                Ok(api_key) => {
                    std::env::set_var("ANTHROPIC_API_KEY", &api_key);
                    let masked_key = mask_api_key(&api_key);
                    eprintln!("✅ ANTHROPIC_API_KEY loaded from keychain: {}", masked_key);
                }
                Err(_) => {
                    eprintln!("ℹ️  No ANTHROPIC_API_KEY found in keychain");
                }
            }
        }
        Err(e) => {
            eprintln!("⚠️  Failed to access keychain for ANTHROPIC_API_KEY: {}", e);
        }
    }

    // OpenAI Embedding Key 로드 (선택사항)
    match keyring::Entry::new("Judgify", "openai_embedding_key") {
        Ok(entry) => {
            match entry.get_password() {
                Ok(api_key) => {
                    std::env::set_var("OPENAI_API_KEY", &api_key);
                    let masked_key = mask_api_key(&api_key);
                    eprintln!("✅ OPENAI_API_KEY (embedding) loaded from keychain: {}", masked_key);
                }
                Err(_) => {
                    // OpenAI는 선택사항이므로 경고만 출력
                    eprintln!("ℹ️  No OPENAI_API_KEY found in keychain (optional for embeddings)");
                }
            }
        }
        Err(e) => {
            eprintln!("⚠️  Failed to access keychain for OPENAI_API_KEY: {}", e);
        }
    }
}

/// API 키 마스킹 헬퍼 함수
fn mask_api_key(api_key: &str) -> String {
    if api_key.len() > 20 {
        format!("{}...{}", &api_key[..10], &api_key[api_key.len()-10..])
    } else {
        "***".to_string()
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Database 인스턴스 생성
    let database = match database::Database::new() {
        Ok(db) => db,
        Err(e) => {
            eprintln!("❌ Failed to initialize database: {}", e);
            panic!("Database initialization failed");
        }
    };

    tauri::Builder::default()
        .manage(database) // Database state 등록
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

            // Workflow Service Commands (v1 - React Flow 기반)
            // workflow::create_workflow,
            // workflow::get_workflow,
            // workflow::get_all_workflows,
            // workflow::update_workflow,
            // workflow::delete_workflow,
            // workflow::validate_workflow,
            // workflow::validate_rule_expression,
            workflow::generate_workflow_with_llm,  // Phase 32: LLM workflow generation
            workflow::simulate_workflow_step,      // Week 5 Task 4: Step-by-step simulation

            // Workflow Service Commands (v2 - Phase 9 Vertical List UI)
            workflow_v2::save_workflow_v2,         // Phase 3-2: 워크플로우 저장
            workflow_v2::load_workflow_v2,         // Phase 3-3: 워크플로우 불러오기
            workflow_v2::list_workflows_v2,        // Phase 3-2: 워크플로우 목록 조회
            workflow_v2::delete_workflow_v2,       // Phase 3-2: 워크플로우 삭제
            workflow_v2::simulate_workflow_v2,     // Phase 3-4: 워크플로우 시뮬레이션
            workflow_v2::get_workflow_executions,  // Phase 4-2: 워크플로우 실행 이력 목록 조회
            workflow_v2::get_workflow_execution_detail, // Phase 4-2: 워크플로우 실행 이력 상세 조회
            workflow_v2::generate_workflow_draft,  // Phase 9-2: AI 워크플로우 생성
            workflow_v2::get_pending_approvals,    // Phase 9-3: 대기 중인 승인 요청 목록
            workflow_v2::process_approval,         // Phase 9-3: 승인/거부 처리
            workflow_v2::get_approval_request,     // Phase 9-3: 승인 요청 상세 조회
            workflow_v2::get_workflow_schedules,   // Phase 9-4: 스케줄 목록 조회
            workflow_v2::create_workflow_schedule, // Phase 9-4: 스케줄 생성
            workflow_v2::toggle_workflow_schedule, // Phase 9-4: 스케줄 활성화/비활성화
            workflow_v2::delete_workflow_schedule, // Phase 9-4: 스케줄 삭제
            workflow_v2::validate_cron_expression, // Phase 9-4: Cron 표현식 검증

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

            // CCP Demo Commands (RAG + Rule-based Judgment)
            ccp::search_ccp_docs,
            ccp::judge_ccp_status,
            ccp::debug_ccp_database,
            ccp::rebuild_fts5_index,

            // MES/ERP RAG Commands (Phase 8: Generic CSV Upload & Query)
            mes::upload_mes_data,
            mes::query_mes_data,
            mes::delete_mes_session,
            mes::get_mes_session_stats,

            // Database Viewer Commands
            commands::database::get_database_tables,
            commands::database::query_table_data,
            commands::database::execute_custom_query,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
