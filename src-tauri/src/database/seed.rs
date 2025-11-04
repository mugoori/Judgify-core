use rusqlite::{Connection, Result};
use chrono::{Utc, Duration};
use uuid::Uuid;

/// Seed sample data for demo purposes (only if database is empty)
pub fn seed_sample_data(conn: &Connection) -> Result<()> {
    // Check if judgments table already has data
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM judgments",
        [],
        |row| row.get(0),
    )?;

    if count > 0 {
        println!("📊 Database already has {} judgment records. Skipping seed.", count);
        return Ok(());
    }

    println!("🌱 Seeding sample data for demo...");

    // Sample workflows
    let workflows = vec![
        ("workflow-quality-check", "품질 검사 워크플로우", "temperature > 90 && vibration < 50"),
        ("workflow-safety-check", "안전 점검 워크플로우", "pressure < 100 && temperature < 120"),
        ("workflow-performance", "성능 평가 워크플로우", "efficiency > 80 && uptime > 95"),
    ];

    // Insert sample workflows
    for (id, name, rule) in &workflows {
        conn.execute(
            "INSERT OR IGNORE INTO workflows (id, name, definition, rule_expression, version, is_active, created_at)
             VALUES (?1, ?2, ?3, ?4, 1, 1, ?5)",
            rusqlite::params![
                id,
                name,
                r#"{"nodes":[],"edges":[]}"#,
                rule,
                Utc::now().to_rfc3339(),
            ],
        )?;
    }

    // Sample judgment data (20 records over the past 7 days)
    let method_types = vec!["rule", "llm", "hybrid"];
    let workflow_ids = vec![
        "workflow-quality-check",
        "workflow-safety-check",
        "workflow-performance",
    ];

    for i in 0..20 {
        let days_ago = (i % 7) as i64; // Distribute over 7 days
        let created_at = Utc::now() - Duration::days(days_ago);

        let workflow_id = workflow_ids[i % workflow_ids.len()];
        let method_used = method_types[i % method_types.len()];

        // Vary result (60% pass, 40% fail for realistic demo)
        let result = i % 5 != 0; // 80% pass rate

        // Vary confidence (0.7-0.95 range)
        let confidence = 0.7 + ((i % 6) as f64 * 0.05);

        let input_data = match i % 3 {
            0 => r#"{"temperature":95,"vibration":45}"#,
            1 => r#"{"pressure":85,"temperature":110}"#,
            _ => r#"{"efficiency":88,"uptime":97}"#,
        };

        let explanation = if result {
            format!("판단 결과: 합격. 모든 조건을 만족합니다. ({})", method_used)
        } else {
            format!("판단 결과: 불합격. 일부 조건이 기준을 초과했습니다. ({})", method_used)
        };

        conn.execute(
            "INSERT INTO judgments (id, workflow_id, input_data, result, confidence, method_used, explanation, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                Uuid::new_v4().to_string(),
                workflow_id,
                input_data,
                result as i32,
                confidence,
                method_used,
                explanation,
                created_at.to_rfc3339(),
            ],
        )?;
    }

    println!("✅ Seeded 20 sample judgments and 3 workflows successfully!");
    Ok(())
}
