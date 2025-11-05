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

    // Sample prompt templates (3 core templates)
    let templates = vec![
        (
            "template-judgment",
            "판단 템플릿",
            "judgment",
            r#"당신은 AI 판단 엔진입니다. 다음 워크플로우 컨텍스트와 입력 데이터를 바탕으로 판단을 수행하세요.

📋 **워크플로우 컨텍스트:**
{{workflow_context}}

📊 **입력 데이터:**
{{input_data}}

🎯 **Few-shot 예시:**
{{few_shot_samples}}

**판단 기준:**
1. 입력 데이터가 모든 조건을 만족하는지 평가
2. 유사 사례 패턴 분석
3. 신뢰도 점수 산출 (0.0-1.0)

**응답 형식 (JSON):**
{
  "result": true/false,
  "confidence": 0.95,
  "reasoning": "판단 근거 설명"
}"#,
            r#"["workflow_context","input_data","few_shot_samples"]"#,
            1500,
        ),
        (
            "template-explanation",
            "설명 템플릿",
            "explanation",
            r#"다음 판단 결과에 대해 사용자에게 친절하고 명확한 설명을 제공하세요.

📋 **워크플로우 이름:** {{workflow_name}}

📊 **입력 데이터:**
{{input_data}}

✅ **판단 결과:** {{result}}
💯 **신뢰도:** {{confidence}}%
🔧 **사용된 방법:** {{method_used}}

🎯 **유사 사례:**
{{similar_cases}}

**설명 요구사항:**
1. 왜 이런 결과가 나왔는지 설명
2. 어떤 조건이 충족되었는지 (또는 충족되지 않았는지) 구체적으로 명시
3. 유사 사례와 비교하여 패턴 설명
4. 사용자가 이해하기 쉬운 언어 사용

**응답 형식:** 자연어 텍스트 (3-5문장)"#,
            r#"["workflow_name","input_data","result","confidence","method_used","similar_cases"]"#,
            1000,
        ),
        (
            "template-insight",
            "인사이트 템플릿",
            "insight",
            r#"다음 데이터를 분석하여 비즈니스 인사이트를 제공하세요.

📊 **데이터 요약:**
{{data_summary}}

📈 **트렌드 분석:**
{{trend_data}}

🎯 **핵심 메트릭:**
{{key_metrics}}

**분석 요구사항:**
1. 주요 트렌드 및 패턴 식별
2. 이상 징후 또는 주목할 만한 변화 발견
3. 실행 가능한 권장사항 제시
4. 비즈니스 영향도 평가

**응답 형식 (JSON):**
{
  "insights": [
    {
      "title": "인사이트 제목",
      "description": "상세 설명",
      "impact": "high/medium/low",
      "recommendation": "권장 조치"
    }
  ],
  "summary": "전체 요약"
}"#,
            r#"["data_summary","trend_data","key_metrics"]"#,
            2000,
        ),
    ];

    for (id, name, template_type, content, variables, token_limit) in &templates {
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT OR IGNORE INTO prompt_templates (id, name, template_type, content, variables, version, is_active, token_limit, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 1, 1, ?6, ?7, ?8)",
            rusqlite::params![
                id,
                name,
                template_type,
                content,
                variables,
                token_limit,
                &now,
                &now,
            ],
        )?;
    }

    println!("✅ Seeded 20 sample judgments, 3 workflows, and 3 prompt templates successfully!");
    Ok(())
}
