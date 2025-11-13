// Criterion.rs Benchmark: Feedback 집계 쿼리 성능 측정
// Task 1.2: SQLite 쿼리 벤치마킹 (Phase 1, Week 1-2)
//
// 목표:
// - Feedback 집계 쿼리: <30ms

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rusqlite::Connection;
use uuid::Uuid;
use chrono::{Utc, Duration};

/// 벤치마크 전용 In-memory SQLite 데이터베이스 생성
fn setup_test_db() -> Connection {
    let conn = Connection::open_in_memory().expect("Failed to create in-memory database");

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

    conn.execute(
        "CREATE TABLE feedbacks (
            id TEXT PRIMARY KEY,
            judgment_id TEXT NOT NULL,
            feedback_type TEXT NOT NULL,
            value INTEGER NOT NULL,
            comment TEXT,
            created_at TEXT NOT NULL
        )",
        [],
    ).expect("Failed to create feedbacks table");

    conn
}

/// Feedback 집계 쿼리 벤치마크 (목표: <30ms)
///
/// 측정 쿼리:
/// ```sql
/// SELECT judgment_id, COUNT(*) as count, AVG(value) as avg_rating
/// FROM feedbacks
/// WHERE created_at >= ?
/// GROUP BY judgment_id
/// ```
fn bench_feedback_aggregation(c: &mut Criterion) {
    let conn = setup_test_db();

    // Workflow 1개 삽입
    let workflow_id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO workflows (id, name, definition, created_at) VALUES (?1, ?2, ?3, ?4)",
        [&workflow_id, "Test Workflow", r#"{"nodes": []}"#, &Utc::now().to_rfc3339()],
    ).expect("Failed to insert workflow");

    // Judgment 50개 삽입
    let mut judgment_ids = Vec::new();
    for i in 0..50 {
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
                "rule",
                &format!("Judgment {}", i),
                &Utc::now().to_rfc3339(),
            ],
        ).expect("Failed to insert judgment");
        judgment_ids.push(id);
    }

    // 1,000개 Feedback 삽입 (다양한 judgment_id)
    for i in 0..1000 {
        let id = Uuid::new_v4().to_string();
        let judgment_id = &judgment_ids[i % 50]; // 각 judgment에 평균 20개 피드백
        let value = if i % 3 == 0 { 1 } else { -1 }; // 1/3 긍정, 2/3 부정
        let days_ago = (i % 30) as i64; // 최근 30일 분포

        let created_at = (Utc::now() - Duration::days(days_ago)).to_rfc3339();

        conn.execute(
            "INSERT INTO feedbacks (id, judgment_id, feedback_type, value, comment, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            [
                &id,
                judgment_id,
                "thumbs_up",
                &value.to_string(),
                &format!("Comment {}", i),
                &created_at,
            ],
        ).expect("Failed to insert feedback");
    }

    c.bench_function("feedback_aggregation", |b| {
        b.iter(|| {
            let seven_days_ago = (Utc::now() - Duration::days(7)).to_rfc3339();

            let mut stmt = conn.prepare(
                "SELECT judgment_id, COUNT(*) as count, AVG(value) as avg_rating
                 FROM feedbacks
                 WHERE created_at >= ?1
                 GROUP BY judgment_id"
            ).unwrap();

            let mut rows = stmt.query([&seven_days_ago]).unwrap();

            let mut total_judgments = 0;
            while let Some(row) = rows.next().unwrap() {
                let _judgment_id = row.get::<_, String>(0).unwrap();
                let _count = row.get::<_, i64>(1).unwrap();
                let _avg_rating = row.get::<_, f64>(2).unwrap();
                total_judgments += 1;
            }

            // 최근 7일 내 피드백이 있는 judgment 개수
            assert!(total_judgments > 0, "Should have feedback aggregations");
        })
    });
}

/// Feedback 단순 조회 벤치마크 (집계 없음)
fn bench_get_feedbacks_by_judgment(c: &mut Criterion) {
    let conn = setup_test_db();

    // Workflow + Judgment 1개 삽입
    let workflow_id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO workflows (id, name, definition, created_at) VALUES (?1, ?2, ?3, ?4)",
        [&workflow_id, "Test Workflow", r#"{"nodes": []}"#, &Utc::now().to_rfc3339()],
    ).expect("Failed to insert workflow");

    let judgment_id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO judgments (id, workflow_id, input_data, result, confidence, method_used, explanation, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        [
            &judgment_id,
            &workflow_id,
            r#"{"test": true}"#,
            "1",
            "0.9",
            "rule",
            "Test judgment",
            &Utc::now().to_rfc3339(),
        ],
    ).expect("Failed to insert judgment");

    // 100개 Feedback 삽입
    for i in 0..100 {
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO feedbacks (id, judgment_id, feedback_type, value, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            [
                &id,
                &judgment_id,
                "thumbs_up",
                "1",
                &Utc::now().to_rfc3339(),
            ],
        ).expect("Failed to insert feedback");
    }

    c.bench_function("get_feedbacks_by_judgment", |b| {
        b.iter(|| {
            let mut stmt = conn.prepare(
                "SELECT * FROM feedbacks WHERE judgment_id = ?1"
            ).unwrap();

            let mut rows = stmt.query([&judgment_id]).unwrap();

            let mut count = 0;
            while let Some(_row) = rows.next().unwrap() {
                count += 1;
            }
            assert_eq!(count, 100);
        })
    });
}

criterion_group!(
    feedback_benches,
    bench_feedback_aggregation,
    bench_get_feedbacks_by_judgment
);
criterion_main!(feedback_benches);
