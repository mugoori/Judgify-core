// Criterion.rs Benchmark: 복잡한 JOIN 쿼리 성능 측정
// Task 1.2: SQLite 쿼리 벤치마킹 (Phase 1, Week 1-2)
//
// 목표:
// - 3-way JOIN (judgments + workflows + feedbacks): <100ms

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use rusqlite::Connection;
use uuid::Uuid;
use chrono::{Utc, Duration};

/// 벤치마크 전용 In-memory SQLite 데이터베이스 생성
fn setup_test_db() -> Connection {
    let conn = Connection::open_in_memory().expect("Failed to create in-memory database");

    // 테이블 생성 (3개 테이블 JOIN용)
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
            created_at TEXT NOT NULL,
            FOREIGN KEY (workflow_id) REFERENCES workflows(id)
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
            created_at TEXT NOT NULL,
            FOREIGN KEY (judgment_id) REFERENCES judgments(id)
        )",
        [],
    ).expect("Failed to create feedbacks table");

    // 인덱스 생성 (JOIN 성능 최적화용)
    conn.execute(
        "CREATE INDEX idx_judgments_workflow ON judgments(workflow_id)",
        [],
    ).expect("Failed to create judgments workflow index");

    conn.execute(
        "CREATE INDEX idx_judgments_created ON judgments(created_at)",
        [],
    ).expect("Failed to create judgments created_at index");

    conn.execute(
        "CREATE INDEX idx_feedbacks_judgment ON feedbacks(judgment_id)",
        [],
    ).expect("Failed to create feedbacks judgment index");

    conn
}

/// 3-way JOIN 쿼리 벤치마크 (목표: <100ms)
///
/// 측정 쿼리:
/// ```sql
/// SELECT
///     j.id,
///     j.workflow_id,
///     j.result,
///     j.confidence,
///     w.name as workflow_name,
///     COUNT(f.id) as feedback_count,
///     AVG(f.value) as avg_feedback
/// FROM judgments j
/// JOIN workflows w ON j.workflow_id = w.id
/// LEFT JOIN feedbacks f ON j.id = f.judgment_id
/// WHERE j.created_at >= ?
/// GROUP BY j.id
/// ORDER BY j.created_at DESC
/// LIMIT 50
/// ```
fn bench_complex_join_query(c: &mut Criterion) {
    let conn = setup_test_db();

    // Workflow 10개 삽입
    let mut workflow_ids = Vec::new();
    for i in 0..10 {
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

    // Judgment 500개 삽입 (각 workflow에 50개씩)
    let mut judgment_ids = Vec::new();
    for i in 0..500 {
        let id = Uuid::new_v4().to_string();
        let workflow_id = &workflow_ids[i % 10];
        let days_ago = (i % 60) as i64; // 최근 60일 분포

        conn.execute(
            "INSERT INTO judgments (id, workflow_id, input_data, result, confidence, method_used, explanation, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            [
                &id,
                workflow_id,
                r#"{"temperature": 90}"#,
                "1",
                "0.85",
                "rule",
                &format!("Explanation {}", i),
                &(Utc::now() - Duration::days(days_ago)).to_rfc3339(),
            ],
        ).expect("Failed to insert judgment");
        judgment_ids.push(id);
    }

    // Feedback 2,000개 삽입 (각 judgment에 평균 4개)
    for i in 0..2000 {
        let id = Uuid::new_v4().to_string();
        let judgment_id = &judgment_ids[i % 500];
        let value = if i % 3 == 0 { 1 } else { -1 }; // 1/3 긍정, 2/3 부정

        conn.execute(
            "INSERT INTO feedbacks (id, judgment_id, feedback_type, value, comment, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            [
                &id,
                judgment_id,
                "thumbs_up",
                &value.to_string(),
                &format!("Comment {}", i),
                &Utc::now().to_rfc3339(),
            ],
        ).expect("Failed to insert feedback");
    }

    // 벤치마크 그룹: 기간별 조회 (7일, 14일, 30일)
    let mut group = c.benchmark_group("complex_join_query");
    for days in [7, 14, 30].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("last_{}_days", days)),
            days,
            |b, &days| {
                b.iter(|| {
                    let cutoff_date = (Utc::now() - Duration::days(days)).to_rfc3339();

                    let mut stmt = conn.prepare(
                        "SELECT
                            j.id,
                            j.workflow_id,
                            j.result,
                            j.confidence,
                            w.name as workflow_name,
                            COUNT(f.id) as feedback_count,
                            AVG(f.value) as avg_feedback
                         FROM judgments j
                         JOIN workflows w ON j.workflow_id = w.id
                         LEFT JOIN feedbacks f ON j.id = f.judgment_id
                         WHERE j.created_at >= ?1
                         GROUP BY j.id
                         ORDER BY j.created_at DESC
                         LIMIT 50"
                    ).unwrap();

                    let mut rows = stmt.query([&cutoff_date]).unwrap();

                    let mut count = 0;
                    while let Some(row) = rows.next().unwrap() {
                        let _id = row.get::<_, String>(0).unwrap();
                        let _workflow_id = row.get::<_, String>(1).unwrap();
                        let _result = row.get::<_, i32>(2).unwrap();
                        let _confidence = row.get::<_, f64>(3).unwrap();
                        let _workflow_name = row.get::<_, String>(4).unwrap();
                        let _feedback_count = row.get::<_, i64>(5).unwrap();
                        let _avg_feedback = row.get::<_, Option<f64>>(6).unwrap();
                        count += 1;
                    }

                    // 최근 N일 내 최대 50개 반환
                    assert!(count <= 50, "Should not exceed LIMIT 50");
                })
            }
        );
    }
    group.finish();
}

/// Workflow별 집계 쿼리 벤치마크
fn bench_workflow_aggregation(c: &mut Criterion) {
    let conn = setup_test_db();

    // 동일한 데이터 설정 (간단히 재사용)
    let workflow_id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO workflows (id, name, definition, created_at) VALUES (?1, ?2, ?3, ?4)",
        [&workflow_id, "Test Workflow", r#"{"nodes": []}"#, &Utc::now().to_rfc3339()],
    ).expect("Failed to insert workflow");

    // 200개 Judgment 삽입
    for i in 0..200 {
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO judgments (id, workflow_id, input_data, result, confidence, method_used, explanation, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            [
                &id,
                &workflow_id,
                r#"{"test": true}"#,
                &(i % 2).to_string(),
                "0.9",
                "rule",
                "Test",
                &Utc::now().to_rfc3339(),
            ],
        ).expect("Failed to insert judgment");
    }

    c.bench_function("workflow_aggregation", |b| {
        b.iter(|| {
            let mut stmt = conn.prepare(
                "SELECT
                    w.id,
                    w.name,
                    COUNT(j.id) as total_judgments,
                    SUM(CASE WHEN j.result = 1 THEN 1 ELSE 0 END) as positive_count,
                    AVG(j.confidence) as avg_confidence
                 FROM workflows w
                 LEFT JOIN judgments j ON w.id = j.workflow_id
                 GROUP BY w.id"
            ).unwrap();

            let mut rows = stmt.query([]).unwrap();

            while let Some(row) = rows.next().unwrap() {
                let _id = row.get::<_, String>(0).unwrap();
                let _name = row.get::<_, String>(1).unwrap();
                let _total = row.get::<_, i64>(2).unwrap();
                let _positive = row.get::<_, i64>(3).unwrap();
                let _avg_conf = row.get::<_, Option<f64>>(4).unwrap();
            }
        })
    });
}

criterion_group!(
    complex_query_benches,
    bench_complex_join_query,
    bench_workflow_aggregation
);
criterion_main!(complex_query_benches);
