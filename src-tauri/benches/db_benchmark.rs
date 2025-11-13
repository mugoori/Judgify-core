// Criterion.rs Benchmark: 기본 DB CRUD 성능 측정
// Task 1.2: SQLite 쿼리 벤치마킹 (Phase 1, Week 1-2)

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rusqlite::Connection;
use uuid::Uuid;
use chrono::Utc;

/// 벤치마크 전용 In-memory SQLite 데이터베이스 생성
fn setup_test_db() -> Connection {
    let conn = Connection::open_in_memory().expect("Failed to create in-memory database");

    // 테이블 생성 (실제 schema와 동일)
    conn.execute(
        "CREATE TABLE workflows (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            definition TEXT NOT NULL,
            rule_expression TEXT,
            version INTEGER DEFAULT 1,
            is_active INTEGER DEFAULT 1,
            created_at TEXT NOT NULL
        )",
        [],
    ).expect("Failed to create workflows table");

    conn.execute(
        "CREATE TABLE judgments (
            id TEXT PRIMARY KEY,
            workflow_id TEXT NOT NULL,
            input_data TEXT NOT NULL,
            result INTEGER NOT NULL,
            confidence REAL NOT NULL,
            method_used TEXT NOT NULL,
            explanation TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY (workflow_id) REFERENCES workflows(id)
        )",
        [],
    ).expect("Failed to create judgments table");

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_judgments_workflow ON judgments(workflow_id)",
        [],
    ).expect("Failed to create index");

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_judgments_created ON judgments(created_at)",
        [],
    ).expect("Failed to create index");

    conn
}

/// Workflow 저장 벤치마크 (목표: <10ms)
fn bench_save_workflow(c: &mut Criterion) {
    let conn = setup_test_db();

    c.bench_function("save_workflow", |b| {
        b.iter(|| {
            let id = Uuid::new_v4().to_string();
            let name = "Test Workflow".to_string();
            let definition = r#"{"nodes": []}"#.to_string();
            let created_at = Utc::now().to_rfc3339();

            conn.execute(
                "INSERT INTO workflows (id, name, definition, created_at) VALUES (?1, ?2, ?3, ?4)",
                [&id, &name, &definition, &created_at],
            ).expect("Failed to insert workflow");
        })
    });
}

/// Workflow 조회 벤치마크 (목표: <5ms)
fn bench_get_workflow(c: &mut Criterion) {
    let conn = setup_test_db();

    // 사전 데이터 삽입 (100개 워크플로우)
    let mut workflow_ids = Vec::new();
    for i in 0..100 {
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO workflows (id, name, definition, created_at) VALUES (?1, ?2, ?3, ?4)",
            [
                &id,
                &format!("Workflow {}", i),
                r#"{"nodes": []}"#,
                &Utc::now().to_rfc3339(),
            ],
        ).expect("Failed to insert workflow");
        workflow_ids.push(id);
    }

    c.bench_function("get_workflow", |b| {
        b.iter(|| {
            let id = &workflow_ids[black_box(50)]; // 중간 데이터 조회
            let mut stmt = conn.prepare("SELECT * FROM workflows WHERE id = ?1").unwrap();
            stmt.query_row([id], |row| {
                Ok((
                    row.get::<_, String>(0)?, // id
                    row.get::<_, String>(1)?, // name
                ))
            }).expect("Failed to query workflow");
        })
    });
}

/// Judgment 저장 벤치마크 (목표: <15ms)
fn bench_save_judgment(c: &mut Criterion) {
    let conn = setup_test_db();

    // Workflow 하나 삽입 (FK 제약조건 만족)
    let workflow_id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO workflows (id, name, definition, created_at) VALUES (?1, ?2, ?3, ?4)",
        [&workflow_id, "Test Workflow", r#"{"nodes": []}"#, &Utc::now().to_rfc3339()],
    ).expect("Failed to insert workflow");

    c.bench_function("save_judgment", |b| {
        b.iter(|| {
            let id = Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO judgments (id, workflow_id, input_data, result, confidence, method_used, explanation, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                [
                    &id,
                    &workflow_id,
                    r#"{"temperature": 90}"#,
                    "1", // true as INTEGER
                    "0.95",
                    "rule",
                    "Temperature exceeds threshold",
                    &Utc::now().to_rfc3339(),
                ],
            ).expect("Failed to insert judgment");
        })
    });
}

criterion_group!(
    db_benches,
    bench_save_workflow,
    bench_get_workflow,
    bench_save_judgment
);
criterion_main!(db_benches);
