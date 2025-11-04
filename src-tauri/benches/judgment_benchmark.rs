// Criterion.rs Benchmark: Judgment 실행 쿼리 성능 측정
// Task 1.2: SQLite 쿼리 벤치마킹 (Phase 1, Week 1-2)
//
// 목표:
// - Judgment 히스토리 조회 (LIMIT 10, 50, 100): <50ms

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use rusqlite::Connection;
use uuid::Uuid;
use chrono::Utc;

/// 벤치마크 전용 In-memory SQLite 데이터베이스 생성
fn setup_test_db() -> Connection {
    let conn = Connection::open_in_memory().expect("Failed to create in-memory database");

    // 테이블 생성
    conn.execute(
        "CREATE TABLE workflows (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            definition TEXT NOT NULL,
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
            created_at TEXT NOT NULL
        )",
        [],
    ).expect("Failed to create judgments table");

    // 인덱스 생성 (실제 DB와 동일)
    conn.execute(
        "CREATE INDEX idx_judgments_workflow ON judgments(workflow_id)",
        [],
    ).expect("Failed to create workflow index");

    conn.execute(
        "CREATE INDEX idx_judgments_created ON judgments(created_at)",
        [],
    ).expect("Failed to create created_at index");

    conn
}

/// Judgment 히스토리 조회 벤치마크 (목표: <50ms)
///
/// 측정 쿼리:
/// ```sql
/// SELECT * FROM judgments
/// WHERE workflow_id = ?
/// ORDER BY created_at DESC
/// LIMIT ?
/// ```
fn bench_get_judgment_history(c: &mut Criterion) {
    let conn = setup_test_db();

    // Workflow 1개 삽입
    let workflow_id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO workflows (id, name, definition, created_at) VALUES (?1, ?2, ?3, ?4)",
        [&workflow_id, "Test Workflow", r#"{"nodes": []}"#, &Utc::now().to_rfc3339()],
    ).expect("Failed to insert workflow");

    // 1,000개 Judgment 데이터 사전 삽입 (현실적인 데이터셋)
    for i in 0..1000 {
        let id = Uuid::new_v4().to_string();
        let temp = 70 + (i % 30); // 70~100 범위
        let vib = 30 + (i % 20);  // 30~50 범위
        let result = if temp > 80 { 1 } else { 0 };

        conn.execute(
            "INSERT INTO judgments (id, workflow_id, input_data, result, confidence, method_used, explanation, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            [
                &id,
                &workflow_id,
                &format!(r#"{{"temperature": {}, "vibration": {}}}"#, temp, vib),
                &result.to_string(),
                "0.85",
                "rule",
                &format!("Explanation {}", i),
                &Utc::now().to_rfc3339(),
            ],
        ).expect("Failed to insert judgment");
    }

    // 벤치마크 그룹: LIMIT 값 변화 (10, 50, 100)
    let mut group = c.benchmark_group("judgment_history");
    for limit in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(limit),
            limit,
            |b, &limit| {
                b.iter(|| {
                    let mut stmt = conn.prepare(
                        "SELECT * FROM judgments
                         WHERE workflow_id = ?1
                         ORDER BY created_at DESC
                         LIMIT ?2"
                    ).unwrap();

                    let mut rows = stmt.query([&workflow_id, &limit.to_string()]).unwrap();

                    let mut count = 0;
                    while let Some(_row) = rows.next().unwrap() {
                        count += 1;
                    }
                    assert_eq!(count, limit as usize);
                })
            }
        );
    }
    group.finish();
}

/// Judgment 단일 조회 벤치마크 (목표: <10ms)
fn bench_get_single_judgment(c: &mut Criterion) {
    let conn = setup_test_db();

    // Workflow 1개 삽입
    let workflow_id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO workflows (id, name, definition, created_at) VALUES (?1, ?2, ?3, ?4)",
        [&workflow_id, "Test Workflow", r#"{"nodes": []}"#, &Utc::now().to_rfc3339()],
    ).expect("Failed to insert workflow");

    // 100개 Judgment 삽입
    let mut judgment_ids = Vec::new();
    for i in 0..100 {
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO judgments (id, workflow_id, input_data, result, confidence, method_used, explanation, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            [
                &id,
                &workflow_id,
                r#"{"test": true}"#,
                "1",
                "0.9",
                "hybrid",
                &format!("Test {}", i),
                &Utc::now().to_rfc3339(),
            ],
        ).expect("Failed to insert judgment");
        judgment_ids.push(id);
    }

    c.bench_function("get_single_judgment", |b| {
        b.iter(|| {
            let id = &judgment_ids[black_box(50)];
            let mut stmt = conn.prepare("SELECT * FROM judgments WHERE id = ?1").unwrap();
            stmt.query_row([id], |row| {
                Ok((
                    row.get::<_, String>(0)?, // id
                    row.get::<_, String>(1)?, // workflow_id
                    row.get::<_, String>(6)?, // explanation
                ))
            }).expect("Failed to query judgment");
        })
    });
}

criterion_group!(
    judgment_benches,
    bench_get_judgment_history,
    bench_get_single_judgment
);
criterion_main!(judgment_benches);
