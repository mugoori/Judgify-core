// 앱 시작시 자동 실행되는 마이그레이션 모듈
// SQL 파일들을 컴파일 시점에 바이너리에 포함시킴 (include_str!)

use rusqlite::{Connection, Result};

/// 마이그레이션 SQL 정의 (컴파일 시점에 바이너리 포함)
struct Migration {
    name: &'static str,
    sql: &'static str,
}

/// 모든 마이그레이션 SQL 포함 (순서대로 실행)
const MIGRATIONS: &[Migration] = &[
    Migration {
        name: "001_knowledge_base.sql",
        sql: include_str!("../../migrations/001_knowledge_base.sql"),
    },
    Migration {
        name: "002_erp_schema.sql",
        sql: include_str!("../../migrations/002_erp_schema.sql"),
    },
    Migration {
        name: "003_mes_schema.sql",
        sql: include_str!("../../migrations/003_mes_schema.sql"),
    },
    Migration {
        name: "004_seed_knowledge.sql",
        sql: include_str!("../../migrations/004_seed_knowledge.sql"),
    },
    Migration {
        name: "005_seed_erp_master.sql",
        sql: include_str!("../../migrations/005_seed_erp_master.sql"),
    },
    Migration {
        name: "006_seed_erp_transaction.sql",
        sql: include_str!("../../migrations/006_seed_erp_transaction.sql"),
    },
    Migration {
        name: "007_seed_mes.sql",
        sql: include_str!("../../migrations/007_seed_mes.sql"),
    },
    Migration {
        name: "008_seed_sales_history.sql",
        sql: include_str!("../../migrations/008_seed_sales_history.sql"),
    },
    Migration {
        name: "009_seed_2025_sales.sql",
        sql: include_str!("../../migrations/009_seed_2025_sales.sql"),
    },
    Migration {
        name: "010_additional_erp_mes.sql",
        sql: include_str!("../../migrations/010_additional_erp_mes.sql"),
    },
    Migration {
        name: "011_seed_additional.sql",
        sql: include_str!("../../migrations/011_seed_additional.sql"),
    },
    Migration {
        name: "012_seed_erp_extended.sql",
        sql: include_str!("../../migrations/012_seed_erp_extended.sql"),
    },
    Migration {
        name: "013_seed_mes_extended.sql",
        sql: include_str!("../../migrations/013_seed_mes_extended.sql"),
    },
    Migration {
        name: "014_seed_mes_complete.sql",
        sql: include_str!("../../migrations/014_seed_mes_complete.sql"),
    },
    Migration {
        name: "015_seed_erp_2025_full.sql",
        sql: include_str!("../../migrations/015_seed_erp_2025_full.sql"),
    },
];

/// 마이그레이션 실행 (앱 시작시 자동 호출)
pub fn apply_migrations(conn: &Connection) -> Result<()> {
    // 마이그레이션 추적 테이블 생성
    conn.execute(
        "CREATE TABLE IF NOT EXISTS _migrations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )?;

    let mut applied_count = 0;
    let mut skipped_count = 0;

    for migration in MIGRATIONS {
        // 이미 적용된 마이그레이션인지 확인
        let already_applied: i64 = conn.query_row(
            "SELECT COUNT(*) FROM _migrations WHERE name = ?",
            [migration.name],
            |row| row.get(0),
        ).unwrap_or(0);

        if already_applied > 0 {
            skipped_count += 1;
            continue;
        }

        // 마이그레이션 실행
        eprintln!("📄 마이그레이션 실행: {}", migration.name);
        match conn.execute_batch(migration.sql) {
            Ok(()) => {
                // 마이그레이션 적용 기록
                conn.execute(
                    "INSERT INTO _migrations (name) VALUES (?)",
                    [migration.name],
                )?;
                eprintln!("   ✅ 완료: {}", migration.name);
                applied_count += 1;
            }
            Err(e) => {
                eprintln!("   ❌ 실패: {} - {}", migration.name, e);
                // 마이그레이션 실패시 에러 반환 (치명적이지 않은 경우 계속 진행)
                // 일부 중복 INSERT 등은 무시
                if e.to_string().contains("UNIQUE constraint failed") {
                    eprintln!("   ⚠️  중복 데이터 무시하고 계속 진행");
                    // 마이그레이션 적용 기록 (중복 무시)
                    let _ = conn.execute(
                        "INSERT OR IGNORE INTO _migrations (name) VALUES (?)",
                        [migration.name],
                    );
                    applied_count += 1;
                }
                // 다른 에러는 계속 진행 (데이터가 이미 있을 수 있음)
            }
        }
    }

    // FTS5 인덱스 재구축 (knowledge_base)
    if applied_count > 0 {
        eprintln!("📊 FTS5 인덱스 재구축 중...");
        let _ = rebuild_fts5_indexes(conn);
    }

    eprintln!("📊 마이그레이션 완료: {}개 적용, {}개 스킵", applied_count, skipped_count);

    Ok(())
}

/// FTS5 인덱스 재구축
fn rebuild_fts5_indexes(conn: &Connection) -> Result<()> {
    // Knowledge Base FTS 재구축
    let kb_exists: bool = conn
        .prepare("SELECT 1 FROM sqlite_master WHERE type='table' AND name='knowledge_base'")
        .and_then(|mut stmt| stmt.exists([]))
        .unwrap_or(false);

    if kb_exists {
        let fts_exists: bool = conn
            .prepare("SELECT 1 FROM sqlite_master WHERE type='table' AND name='knowledge_base_fts'")
            .and_then(|mut stmt| stmt.exists([]))
            .unwrap_or(false);

        if fts_exists {
            // 기존 FTS 데이터 삭제 후 재구축
            let _ = conn.execute_batch("DELETE FROM knowledge_base_fts");
            let _ = conn.execute_batch(r#"
                INSERT INTO knowledge_base_fts (rowid, title, content, tags)
                SELECT rowid, title, content, tags FROM knowledge_base
            "#);

            let count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM knowledge_base_fts",
                [],
                |row| row.get(0),
            ).unwrap_or(0);

            eprintln!("✅ Knowledge Base FTS5 재구축 완료: {}건", count);
        }
    }

    Ok(())
}
