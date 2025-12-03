use judgify_desktop::database::Database;
use rusqlite::Connection;
use std::path::PathBuf;

fn main() {
    println!("초기화 중: 퓨어웰 음료㈜ 데이터베이스...");

    match Database::new() {
        Ok(_db) => {
            println!("✅ 데이터베이스 기본 스키마 생성 완료!");

            // 새로운 마이그레이션 적용 (ERP/MES/RAG)
            match apply_migrations() {
                Ok(()) => {
                    println!("✅ 마이그레이션 001-014 실행 완료");
                    println!("📁 위치: %APPDATA%\\Judgify\\judgify.db");
                    println!("✅ 퓨어웰 음료㈜ 시드 데이터 삽입 완료");
                    println!("✅ 추가 ERP/MES 테이블 및 시드 데이터 삽입 완료");
                }
                Err(e) => {
                    eprintln!("⚠️  마이그레이션 적용 실패: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("❌ 데이터베이스 초기화 실패: {}", e);
            std::process::exit(1);
        }
    }
}

fn apply_migrations() -> rusqlite::Result<()> {
    // Get database path
    let app_data = std::env::var("APPDATA")
        .or_else(|_| std::env::var("HOME"))
        .map_err(|e| rusqlite::Error::InvalidPath(PathBuf::from(e.to_string())))?;

    let db_path = PathBuf::from(app_data).join("Judgify").join("judgify.db");
    let conn = Connection::open(&db_path)?;

    // 마이그레이션 추적 테이블 생성
    conn.execute(
        "CREATE TABLE IF NOT EXISTS _migrations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )?;

    // 마이그레이션 파일 목록 (순서대로 실행)
    let migration_files = [
        "migrations/001_knowledge_base.sql",
        "migrations/002_erp_schema.sql",
        "migrations/003_mes_schema.sql",
        "migrations/004_seed_knowledge.sql",
        "migrations/005_seed_erp_master.sql",
        "migrations/006_seed_erp_transaction.sql",
        "migrations/007_seed_mes.sql",
        "migrations/008_seed_sales_history.sql",
        "migrations/009_seed_2025_sales.sql",
        "migrations/010_additional_erp_mes.sql",
        "migrations/011_seed_additional.sql",
        "migrations/012_seed_erp_extended.sql",
        "migrations/013_seed_mes_extended.sql",
        "migrations/014_seed_mes_complete.sql",
    ];

    let mut applied_count = 0;
    let mut skipped_count = 0;

    for file in &migration_files {
        // 이미 적용된 마이그레이션인지 확인
        let already_applied: i64 = conn.query_row(
            "SELECT COUNT(*) FROM _migrations WHERE name = ?",
            [file],
            |row| row.get(0),
        ).unwrap_or(0);

        if already_applied > 0 {
            println!("📄 스킵: {} (이미 적용됨)", file);
            skipped_count += 1;
            continue;
        }

        println!("📄 실행 중: {}", file);
        match std::fs::read_to_string(file) {
            Ok(sql) => {
                match conn.execute_batch(&sql) {
                    Ok(()) => {
                        // 마이그레이션 적용 기록
                        conn.execute(
                            "INSERT INTO _migrations (name) VALUES (?)",
                            [file],
                        )?;
                        println!("   ✅ 완료");
                        applied_count += 1;
                    }
                    Err(e) => {
                        eprintln!("   ❌ 실패: {}", e);
                        return Err(e);
                    }
                }
            }
            Err(e) => {
                eprintln!("   ❌ 파일 읽기 실패: {}", e);
                return Err(rusqlite::Error::InvalidPath(PathBuf::from(format!("파일 읽기 실패: {}", e))));
            }
        }
    }

    println!("\n📊 마이그레이션 결과: {}개 적용, {}개 스킵", applied_count, skipped_count);

    // FTS5 인덱스 rebuild (knowledge_base)
    println!("\n📊 FTS5 인덱스 재구축 중...");

    // Step 1: 기존 FTS 데이터 삭제
    conn.execute_batch("DELETE FROM knowledge_base_fts")?;

    // Step 2: knowledge_base에서 FTS 인덱스 데이터 복사
    conn.execute_batch(r#"
        INSERT INTO knowledge_base_fts (rowid, title, content, tags)
        SELECT rowid, title, content, tags FROM knowledge_base
    "#)?;

    let kb_fts_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM knowledge_base_fts",
        [],
        |row| row.get(0),
    )?;

    println!("✅ Knowledge Base FTS5 rebuild 완료: {}건", kb_fts_count);

    // 결과 요약 출력
    print_summary(&conn)?;

    Ok(())
}

fn print_summary(conn: &Connection) -> rusqlite::Result<()> {
    println!("\n========================================");
    println!("📊 데이터베이스 초기화 완료 요약");
    println!("========================================");

    // Knowledge Base
    let kb_count: i64 = conn.query_row("SELECT COUNT(*) FROM knowledge_base", [], |row| row.get(0))?;
    println!("📚 Knowledge Base: {} entries (기업정보 + SOP)", kb_count);

    // ERP Master
    let item_count: i64 = conn.query_row("SELECT COUNT(*) FROM item_mst", [], |row| row.get(0))?;
    let vendor_count: i64 = conn.query_row("SELECT COUNT(*) FROM vendor_mst", [], |row| row.get(0))?;
    let customer_count: i64 = conn.query_row("SELECT COUNT(*) FROM customer_mst", [], |row| row.get(0))?;
    let bom_count: i64 = conn.query_row("SELECT COUNT(*) FROM bom_mst", [], |row| row.get(0))?;
    println!("📦 ERP 마스터: 품목 {}, 거래처 {}, 고객 {}, BOM {}", item_count, vendor_count, customer_count, bom_count);

    // ERP Transactions
    let po_count: i64 = conn.query_row("SELECT COUNT(*) FROM purchase_order", [], |row| row.get(0))?;
    let prod_count: i64 = conn.query_row("SELECT COUNT(*) FROM production_order", [], |row| row.get(0))?;
    let so_count: i64 = conn.query_row("SELECT COUNT(*) FROM sales_order", [], |row| row.get(0))?;
    let fg_count: i64 = conn.query_row("SELECT COUNT(*) FROM fg_lot", [], |row| row.get(0))?;
    println!("📋 ERP 거래: 발주 {}, 생산 {}, 수주 {}, 완제품LOT {}", po_count, prod_count, so_count, fg_count);

    // MES Master
    let line_count: i64 = conn.query_row("SELECT COUNT(*) FROM line_mst", [], |row| row.get(0))?;
    let equip_count: i64 = conn.query_row("SELECT COUNT(*) FROM equipment_mst", [], |row| row.get(0))?;
    let operator_count: i64 = conn.query_row("SELECT COUNT(*) FROM operator_mst", [], |row| row.get(0))?;
    println!("🏭 MES 마스터: 라인 {}, 설비 {}, 작업자 {}", line_count, equip_count, operator_count);

    // MES Execution
    let wo_count: i64 = conn.query_row("SELECT COUNT(*) FROM mes_work_order", [], |row| row.get(0))?;
    let ccp_count: i64 = conn.query_row("SELECT COUNT(*) FROM ccp_check_log", [], |row| row.get(0))?;
    let sensor_count: i64 = conn.query_row("SELECT COUNT(*) FROM sensor_log", [], |row| row.get(0))?;
    let alarm_count: i64 = conn.query_row("SELECT COUNT(*) FROM alarm_event", [], |row| row.get(0))?;
    let material_issue_count: i64 = conn.query_row("SELECT COUNT(*) FROM material_issue", [], |row| row.get(0)).unwrap_or(0);
    let operation_exec_count: i64 = conn.query_row("SELECT COUNT(*) FROM operation_exec", [], |row| row.get(0)).unwrap_or(0);
    println!("⚙️  MES 실행: 작업지시 {}, CCP체크 {}, 센서로그 {}, 알람 {}", wo_count, ccp_count, sensor_count, alarm_count);
    println!("⚙️  MES 확장: 공정실행 {}, 자재출고 {}", operation_exec_count, material_issue_count);

    // 추가된 테이블 (010, 011)
    let qc_insp_count: i64 = conn.query_row("SELECT COUNT(*) FROM qc_inspection", [], |row| row.get(0)).unwrap_or(0);
    let metal_count: i64 = conn.query_row("SELECT COUNT(*) FROM metal_detection_log", [], |row| row.get(0)).unwrap_or(0);
    let process_param_count: i64 = conn.query_row("SELECT COUNT(*) FROM process_param_log", [], |row| row.get(0)).unwrap_or(0);
    let material_input_count: i64 = conn.query_row("SELECT COUNT(*) FROM material_input_log", [], |row| row.get(0)).unwrap_or(0);
    let warehouse_count: i64 = conn.query_row("SELECT COUNT(*) FROM warehouse_mst", [], |row| row.get(0)).unwrap_or(0);
    let inv_mov_count: i64 = conn.query_row("SELECT COUNT(*) FROM inventory_movement", [], |row| row.get(0)).unwrap_or(0);
    let operation_param_count: i64 = conn.query_row("SELECT COUNT(*) FROM operation_param_log", [], |row| row.get(0)).unwrap_or(0);
    let qc_test_count: i64 = conn.query_row("SELECT COUNT(*) FROM qc_test", [], |row| row.get(0)).unwrap_or(0);
    let downtime_count: i64 = conn.query_row("SELECT COUNT(*) FROM downtime_event", [], |row| row.get(0)).unwrap_or(0);
    println!("🔬 MES 품질: 품질검사 {}, 금속검출 {}, 공정파라미터 {}", qc_insp_count, metal_count, process_param_count);
    println!("📊 MES 확장2: 공정파라미터로그 {}, 비가동 {}, QC테스트 {}", operation_param_count, downtime_count, qc_test_count);
    println!("📦 ERP 추가: 창고 {}, 자재투입 {}, 재고이동 {}", warehouse_count, material_input_count, inv_mov_count);

    println!("========================================\n");

    Ok(())
}
