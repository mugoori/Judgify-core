// Criterion.rs Benchmark: TrainingSample 검색 쿼리 성능 측정
// Task 1.2: SQLite 쿼리 벤치마킹 (Phase 1, Week 1-2)
//
// 목표:
// - TrainingSample 검색 (accuracy 필터링): <20ms

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use rusqlite::Connection;
use uuid::Uuid;
use chrono::Utc;

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
        "CREATE TABLE training_samples (
            id TEXT PRIMARY KEY,
            workflow_id TEXT NOT NULL,
            input_data TEXT NOT NULL,
            expected_result INTEGER NOT NULL,
            actual_result INTEGER,
            accuracy REAL,
            created_at TEXT NOT NULL
        )",
        [],
    ).expect("Failed to create training_samples table");

    // 인덱스 생성 (기존)
    conn.execute(
        "CREATE INDEX idx_training_workflow ON training_samples(workflow_id)",
        [],
    ).expect("Failed to create workflow index");

    conn
}

/// TrainingSample 검색 벤치마크 (목표: <20ms)
///
/// 측정 쿼리:
/// ```sql
/// SELECT * FROM training_samples
/// WHERE workflow_id = ?
///   AND accuracy >= ?
/// ORDER BY created_at DESC
/// LIMIT 20
/// ```
fn bench_get_training_samples(c: &mut Criterion) {
    let conn = setup_test_db();

    // Workflow 1개 삽입
    let workflow_id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO workflows (id, name, definition, created_at) VALUES (?1, ?2, ?3, ?4)",
        [&workflow_id, "Test Workflow", r#"{"nodes": []}"#, &Utc::now().to_rfc3339()],
    ).expect("Failed to insert workflow");

    // 500개 TrainingSample 삽입 (accuracy 분포: 0.5~1.0)
    for i in 0..500 {
        let id = Uuid::new_v4().to_string();
        let accuracy = 0.5 + (i as f64 % 500.0) / 1000.0; // 0.5~1.0 균등 분포

        conn.execute(
            "INSERT INTO training_samples (id, workflow_id, input_data, expected_result, actual_result, accuracy, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            [
                &id,
                &workflow_id,
                &format!(r#"{{"case": {}}}"#, i),
                &(i % 2).to_string(),        // expected_result
                &((i % 3 != 0) as i32).to_string(), // actual_result
                &accuracy.to_string(),
                &Utc::now().to_rfc3339(),
            ],
        ).expect("Failed to insert training sample");
    }

    // 벤치마크 그룹: accuracy 임계값 변화 (0.7, 0.8, 0.9)
    let mut group = c.benchmark_group("training_sample_search");
    for threshold in [0.7, 0.8, 0.9].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("accuracy_gte_{}", threshold)),
            threshold,
            |b, &threshold| {
                b.iter(|| {
                    let mut stmt = conn.prepare(
                        "SELECT * FROM training_samples
                         WHERE workflow_id = ?1
                           AND accuracy >= ?2
                         ORDER BY created_at DESC
                         LIMIT 20"
                    ).unwrap();

                    let mut rows = stmt.query([&workflow_id, &threshold.to_string()]).unwrap();

                    let mut count = 0;
                    while let Some(row) = rows.next().unwrap() {
                        let acc = row.get::<_, f64>(5).unwrap();
                        assert!(acc >= threshold, "Accuracy should be >= {}", threshold);
                        count += 1;
                    }
                    // accuracy >= threshold인 샘플만 반환되어야 함
                    assert!(count <= 20, "Should not exceed LIMIT 20");
                })
            }
        );
    }
    group.finish();
}

/// TrainingSample 전체 조회 벤치마크 (LIMIT 20, accuracy 필터 없음)
fn bench_get_all_training_samples(c: &mut Criterion) {
    let conn = setup_test_db();

    // Workflow 1개 삽입
    let workflow_id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO workflows (id, name, definition, created_at) VALUES (?1, ?2, ?3, ?4)",
        [&workflow_id, "Test Workflow", r#"{"nodes": []}"#, &Utc::now().to_rfc3339()],
    ).expect("Failed to insert workflow");

    // 500개 TrainingSample 삽입
    for i in 0..500 {
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO training_samples (id, workflow_id, input_data, expected_result, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            [
                &id,
                &workflow_id,
                &format!(r#"{{"case": {}}}"#, i),
                &(i % 2).to_string(),
                &Utc::now().to_rfc3339(),
            ],
        ).expect("Failed to insert training sample");
    }

    c.bench_function("get_all_training_samples", |b| {
        b.iter(|| {
            let mut stmt = conn.prepare(
                "SELECT * FROM training_samples
                 WHERE workflow_id = ?1
                 ORDER BY created_at DESC
                 LIMIT 20"
            ).unwrap();

            let mut rows = stmt.query([&workflow_id]).unwrap();

            let mut count = 0;
            while let Some(_row) = rows.next().unwrap() {
                count += 1;
            }
            assert_eq!(count, 20);
        })
    });
}

criterion_group!(
    training_sample_benches,
    bench_get_training_samples,
    bench_get_all_training_samples
);
criterion_main!(training_sample_benches);
