//! 차트 생성 서비스
//!
//! 자연어 → SQL 변환 → 차트 데이터 생성
//! MES 스키마 기반 데이터 시각화 지원

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;

/// 차트 타입
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChartType {
    Bar,
    Line,
    Pie,
    Gauge,
}

/// 차트 데이터 포인트 (Bar/Line 차트용)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartDataPoint {
    pub name: String,
    #[serde(flatten)]
    pub values: std::collections::HashMap<String, f64>,
}

/// 파이 차트 데이터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PieChartData {
    pub name: String,
    pub value: f64,
    pub color: Option<String>,
}

/// 게이지 차트 데이터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaugeChartData {
    pub value: f64,
    pub min: f64,
    pub max: f64,
    pub label: String,
    pub unit: String,
}

/// 차트 응답 데이터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartResponse {
    pub chart_type: ChartType,
    pub title: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bar_line_data: Option<Vec<ChartDataPoint>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pie_data: Option<Vec<PieChartData>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gauge_data: Option<GaugeChartData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_keys: Option<Vec<DataKeyConfig>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x_axis_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insight: Option<String>,  // AI 인사이트 (데이터 해석)
}

/// 차트 데이터 키 설정 (Bar/Line 차트용)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataKeyConfig {
    pub key: String,
    pub color: String,
    pub label: String,
}

/// LLM 응답: SQL 쿼리 및 차트 설정
#[derive(Debug, Clone, Deserialize)]
pub struct LLMChartPlan {
    pub sql: String,
    pub chart_type: String,
    pub title: String,
    pub description: String,
    pub x_axis_key: Option<String>,
    pub data_keys: Option<Vec<DataKeyConfig>>,
}

/// 차트 생성 서비스
pub struct ChartService {
    claude_api_key: String,
    http_client: Client,
}

impl ChartService {
    /// 새 ChartService 인스턴스 생성
    pub fn new() -> Result<Self> {
        let claude_api_key = env::var("ANTHROPIC_API_KEY")
            .or_else(|_| {
                keyring::Entry::new("Judgify", "claude_api_key")
                    .and_then(|e| e.get_password())
                    .map_err(|e| anyhow::anyhow!("Keychain 로드 실패: {}", e))
            })
            .map_err(|_| anyhow::anyhow!("Claude API 키가 설정되지 않았습니다."))?;

        Ok(Self {
            claude_api_key,
            http_client: Client::new(),
        })
    }

    /// MES 스키마 정보 반환
    fn get_mes_schema_info(&self) -> &'static str {
        r#"
## MES 데이터베이스 스키마 (SQLite)

### 1. 마스터 테이블
- **line_mst** (라인 마스터): line_cd(PK), line_nm, line_type(BATCHING|FILLING|PACKAGING), capacity_per_hour, is_active
- **equipment_mst** (설비 마스터): equip_cd(PK), equip_nm, line_cd(FK), equip_type, is_ccp, ccp_type
- **operation_mst** (공정 마스터): oper_cd(PK), oper_nm, oper_seq, line_cd(FK), is_ccp
- **shift_mst** (교대 마스터): shift_cd(PK), shift_nm, start_time, end_time
- **operator_mst** (작업자 마스터): operator_id(PK), operator_nm, dept, shift_cd(FK)
- **param_mst** (파라미터 마스터): param_cd(PK), param_nm, param_type, unit, equip_cd, min_val, max_val, target_val, is_ccp

### 2. 작업 실행 테이블
- **mes_work_order** (작업지시): wo_no(PK), prod_order_no, line_cd, shift_cd, plan_date, plan_start, plan_end, actual_start, actual_end, status(SCHEDULED|READY|RUNNING|PAUSED|COMPLETED|CANCELLED), plan_qty, good_qty, reject_qty
- **operation_exec** (공정실행): id(PK), wo_no(FK), oper_cd(FK), batch_lot_no, equip_cd, start_time, end_time, status(RUNNING|COMPLETED|FAILED|PAUSED), result(OK|NG|DEVIATION)

### 3. LOT 추적성 테이블 (불량 분석 핵심!)
- **batch_lot** (배치LOT): batch_lot_no(PK), prod_order_no(FK), wo_no(FK), line_cd, batch_seq, batch_size, start_time, end_time, status(CREATED|PROCESSING|COMPLETED|CANCELLED), good_qty, reject_qty
- **filling_lot** (충진LOT): filling_lot_no(PK), batch_lot_no(FK), filling_date, line_cd, pkg_item_cd, plan_qty, good_qty, reject_qty, start_time, end_time, status
  - 불량률 계산: ROUND(SUM(reject_qty) * 100.0 / NULLIF(SUM(plan_qty), 0), 2) as defect_rate
  - 또는: ROUND(SUM(reject_qty) * 100.0 / NULLIF(SUM(good_qty + reject_qty), 0), 2) as defect_rate
- **fg_lot** (완제품LOT): fg_lot_no(PK), filling_lot_no(FK), prod_date, expiry_date, item_cd, lot_qty, location, status(IN_STOCK|SHIPPED|QUARANTINE|DISPOSED)
- **process_result** (공정실적): id(PK), batch_lot_no(FK), process_type(살균|균질|발효|충진|냉각), equip_cd, start_time, end_time, target_temp, actual_temp, target_time_sec, actual_time_sec, result(PASS|FAIL|WARNING)

### 4. 품질 검사 테이블
- **qc_test** (품질검사): id(PK), test_type(원료입고|공정중|완제품), batch_lot_no, item_cd, test_item(수분|지방|pH|산도|Brix|미생물), test_value, unit, spec_min, spec_max, result(PASS|FAIL|HOLD), test_time

### 5. 센서/CCP 테이블
- **sensor_log** (센서로그): id(PK), equip_cd(FK), param_cd(FK), batch_lot_no, recorded_at, value, is_alarm, alarm_type
- **ccp_check_log** (CCP검사): id(PK), batch_lot_no, ccp_type(PASTEURIZATION|METAL_DETECTION|COOLING), check_time, equip_cd, target_temp, actual_temp, target_time_sec, actual_time_sec, result(PASS|FAIL|DEVIATION)

### 6. 이벤트 테이블
- **downtime_event** (비가동): id(PK), wo_no(FK), equip_cd(FK), line_cd(FK), start_time, end_time, duration_min, reason_cd, is_planned
- **alarm_event** (알람): id(PK), equip_cd(FK), param_cd, alarm_time, alarm_level(INFO|WARNING|CRITICAL), alarm_type, message, is_acknowledged, is_resolved

### 샘플 데이터 정보
- 라인: LINE-A(배합), LINE-B(충진), LINE-C(포장)
- 설비: MIX-001(배합기), PAST-001(살균기), FILL-001(충진기), METAL-001(금속검출기) 등
- 작업지시: WO-2024-001 ~ (2024년 9~11월)
- LOT 데이터: batch_lot 513건, filling_lot 1,236건, fg_lot 1,236건, process_result 2,856건
- 품질검사: qc_test 405건 (원료입고/공정중/완제품 검사)
- 이벤트: downtime_event 382건, alarm_event 296건

### 불량률 분석 팁
- **라인별 불량률**: filling_lot 테이블의 good_qty, reject_qty 활용
- **제품별 불량률**: fg_lot + filling_lot 조인으로 item_cd별 분석
- **불량 원인 분석**: filling_lot.reject_reason 컬럼 활용
- **공정별 품질**: process_result.result (PASS/FAIL/WARNING) 분석
"#
    }

    /// 자연어 요청을 SQL + 차트 설정으로 변환
    pub async fn generate_chart_plan(&self, user_request: &str) -> Result<LLMChartPlan> {
        let schema_info = self.get_mes_schema_info();

        let system_prompt = format!(r#"당신은 MES 데이터 분석 전문가입니다.
사용자의 자연어 요청을 분석하여 SQLite 쿼리와 차트 설정을 JSON으로 반환하세요.

{}

## 응답 형식 (JSON만 반환)
반드시 아래 형식의 JSON만 반환하세요. 마크다운 코드블록 사용 금지!
- sql: SELECT 문만 (INSERT/UPDATE/DELETE 금지)
- chart_type: bar, line, pie, gauge 중 하나
- title: 차트 제목 (한글)
- description: 차트 설명 (한글, 1-2문장)
- x_axis_key: X축 컬럼명 (bar/line만 해당)
- data_keys: 배열, 각 항목은 key(컬럼명), color(헥스코드), label(범례명)

## 차트 타입 선택 기준
- **line**: 시간에 따른 추이, 트렌드 분석 (온도 추이, 생산량 추이)
- **bar**: 비교 분석 (라인별 생산량, 설비별 가동률)
- **pie**: 비율/구성 분석 (CCP 결과 분포, 상태별 비율)
- **gauge**: 단일 수치 (현재 불량률, 달성률)

## 색상 가이드
- 긍정(정상/OK/PASS): #22c55e (녹색)
- 부정(불량/NG/FAIL): #ef4444 (빨강)
- 경고(DEVIATION): #f59e0b (주황)
- 중립: #3b82f6 (파랑), #8b5cf6 (보라), #06b6d4 (청록)
"#, schema_info);

        let request_body = json!({
            "model": "claude-sonnet-4-5-20250929",
            "max_tokens": 1024,
            "system": system_prompt,
            "messages": [
                {"role": "user", "content": user_request}
            ],
            "temperature": 0.3
        });

        println!("📊 [ChartService] Generating chart plan for: {}", user_request);

        let response = self
            .http_client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.claude_api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Claude API error ({}): {}", status, error_text);
        }

        let response_json: serde_json::Value = response.json().await?;
        let content = response_json["content"][0]["text"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing content in Claude response"))?;

        println!("📝 [ChartService] LLM response: {}", content);

        // 마크다운 코드 블록 제거
        let clean_content = content.trim();
        let clean_content = if clean_content.starts_with("```json") {
            clean_content
                .strip_prefix("```json")
                .unwrap_or(clean_content)
                .strip_suffix("```")
                .unwrap_or(clean_content)
                .trim()
        } else if clean_content.starts_with("```") {
            clean_content
                .strip_prefix("```")
                .unwrap_or(clean_content)
                .strip_suffix("```")
                .unwrap_or(clean_content)
                .trim()
        } else {
            clean_content
        };

        let plan: LLMChartPlan = serde_json::from_str(clean_content)
            .map_err(|e| anyhow::anyhow!("Failed to parse chart plan: {} - Raw: {}", e, clean_content))?;

        // SQL 안전성 검증 (SELECT만 허용)
        let sql_upper = plan.sql.to_uppercase();
        if !sql_upper.trim_start().starts_with("SELECT") {
            anyhow::bail!("Only SELECT statements are allowed for safety");
        }
        if sql_upper.contains("INSERT") || sql_upper.contains("UPDATE") ||
           sql_upper.contains("DELETE") || sql_upper.contains("DROP") ||
           sql_upper.contains("ALTER") || sql_upper.contains("CREATE") {
            anyhow::bail!("Dangerous SQL detected");
        }

        println!("✅ [ChartService] Chart plan generated: {} ({})", plan.title, plan.chart_type);
        Ok(plan)
    }

    /// SQL 실행 및 차트 데이터 변환
    pub fn execute_and_transform(
        &self,
        conn: &rusqlite::Connection,
        plan: &LLMChartPlan,
    ) -> Result<ChartResponse> {
        println!("🔍 [ChartService] Executing SQL: {}", plan.sql);

        let mut stmt = conn.prepare(&plan.sql)?;
        let columns: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();

        let chart_type = match plan.chart_type.as_str() {
            "line" => ChartType::Line,
            "pie" => ChartType::Pie,
            "gauge" => ChartType::Gauge,
            _ => ChartType::Bar,
        };

        // 결과를 행 단위로 수집
        let mut rows: Vec<Vec<serde_json::Value>> = Vec::new();
        let mut query_rows = stmt.query([])?;

        while let Some(row) = query_rows.next()? {
            let mut row_data = Vec::new();
            for i in 0..columns.len() {
                let value = if let Ok(val) = row.get::<_, String>(i) {
                    serde_json::Value::String(val)
                } else if let Ok(val) = row.get::<_, i64>(i) {
                    serde_json::Value::Number(serde_json::Number::from(val))
                } else if let Ok(val) = row.get::<_, f64>(i) {
                    if let Some(num) = serde_json::Number::from_f64(val) {
                        serde_json::Value::Number(num)
                    } else {
                        serde_json::Value::String(val.to_string())
                    }
                } else {
                    serde_json::Value::Null
                };
                row_data.push(value);
            }
            rows.push(row_data);
        }

        println!("📊 [ChartService] Query returned {} rows", rows.len());

        // 차트 타입에 따라 데이터 변환
        match chart_type {
            ChartType::Bar | ChartType::Line => {
                // 항상 "name" 키를 사용 (ChartDataPoint 구조체의 필드명과 일치)
                let x_axis_key = "name".to_string();
                let data_keys = plan.data_keys.clone().unwrap_or_else(|| {
                    columns.iter().skip(1).enumerate().map(|(i, col)| {
                        let colors = ["#3b82f6", "#22c55e", "#ef4444", "#f59e0b", "#8b5cf6"];
                        DataKeyConfig {
                            key: col.clone(),
                            color: colors[i % colors.len()].to_string(),
                            label: col.clone(),
                        }
                    }).collect()
                });

                let chart_data: Vec<ChartDataPoint> = rows.iter().map(|row| {
                    let name = row.get(0)
                        .and_then(|v| v.as_str().map(|s| s.to_string()))
                        .or_else(|| row.get(0).and_then(|v| v.as_i64().map(|n| n.to_string())))
                        .unwrap_or_default();

                    let mut values = std::collections::HashMap::new();
                    for (i, col) in columns.iter().enumerate().skip(1) {
                        if let Some(val) = row.get(i) {
                            if let Some(n) = val.as_f64() {
                                values.insert(col.clone(), n);
                            } else if let Some(n) = val.as_i64() {
                                values.insert(col.clone(), n as f64);
                            }
                        }
                    }
                    ChartDataPoint { name, values }
                }).collect();

                Ok(ChartResponse {
                    chart_type,
                    title: plan.title.clone(),
                    description: plan.description.clone(),
                    bar_line_data: Some(chart_data),
                    pie_data: None,
                    gauge_data: None,
                    data_keys: Some(data_keys),
                    x_axis_key: Some(x_axis_key),
                    insight: None,  // 인사이트는 별도로 생성
                })
            }
            ChartType::Pie => {
                let pie_data: Vec<PieChartData> = rows.iter().enumerate().map(|(i, row)| {
                    let colors = ["#3b82f6", "#22c55e", "#ef4444", "#f59e0b", "#8b5cf6", "#06b6d4"];
                    let name = row.get(0)
                        .and_then(|v| v.as_str().map(|s| s.to_string()))
                        .unwrap_or_default();
                    let value = row.get(1)
                        .and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|n| n as f64)))
                        .unwrap_or(0.0);

                    PieChartData {
                        name,
                        value,
                        color: Some(colors[i % colors.len()].to_string()),
                    }
                }).collect();

                Ok(ChartResponse {
                    chart_type,
                    title: plan.title.clone(),
                    description: plan.description.clone(),
                    bar_line_data: None,
                    pie_data: Some(pie_data),
                    gauge_data: None,
                    data_keys: None,
                    x_axis_key: None,
                    insight: None,  // 인사이트는 별도로 생성
                })
            }
            ChartType::Gauge => {
                // 게이지는 단일 값 (첫 번째 행의 첫 번째 숫자 컬럼)
                let value = rows.get(0)
                    .and_then(|row| {
                        for val in row.iter() {
                            if let Some(n) = val.as_f64().or_else(|| val.as_i64().map(|n| n as f64)) {
                                return Some(n);
                            }
                        }
                        None
                    })
                    .unwrap_or(0.0);

                Ok(ChartResponse {
                    chart_type,
                    title: plan.title.clone(),
                    description: plan.description.clone(),
                    bar_line_data: None,
                    pie_data: None,
                    gauge_data: Some(GaugeChartData {
                        value,
                        min: 0.0,
                        max: 100.0,
                        label: plan.title.clone(),
                        unit: "%".to_string(),
                    }),
                    data_keys: None,
                    x_axis_key: None,
                    insight: None,  // 인사이트는 별도로 생성
                })
            }
        }
    }

    /// 차트 데이터 기반 AI 인사이트 생성
    pub async fn generate_insight(&self, chart_response: &ChartResponse, user_request: &str) -> Result<String> {
        // 차트 데이터를 요약 텍스트로 변환
        let data_summary = match &chart_response.chart_type {
            ChartType::Bar | ChartType::Line => {
                if let Some(data) = &chart_response.bar_line_data {
                    let summary: Vec<String> = data.iter().map(|point| {
                        let values: Vec<String> = point.values.iter()
                            .map(|(k, v)| format!("{}={:.1}", k, v))
                            .collect();
                        format!("{}: {}", point.name, values.join(", "))
                    }).collect();
                    summary.join("; ")
                } else {
                    "데이터 없음".to_string()
                }
            }
            ChartType::Pie => {
                if let Some(data) = &chart_response.pie_data {
                    let total: f64 = data.iter().map(|p| p.value).sum();
                    let summary: Vec<String> = data.iter().map(|p| {
                        let pct = if total > 0.0 { p.value / total * 100.0 } else { 0.0 };
                        format!("{}={:.1}({:.1}%)", p.name, p.value, pct)
                    }).collect();
                    summary.join(", ")
                } else {
                    "데이터 없음".to_string()
                }
            }
            ChartType::Gauge => {
                if let Some(data) = &chart_response.gauge_data {
                    format!("현재값: {:.1}{} (범위: {:.0}~{:.0})",
                        data.value, data.unit, data.min, data.max)
                } else {
                    "데이터 없음".to_string()
                }
            }
        };

        let system_prompt = r#"당신은 MES/ERP 데이터 분석 전문가입니다.
주어진 차트 데이터를 분석하여 핵심 인사이트를 1-2문장으로 요약하세요.

요구사항:
- 한국어로 답변
- 수치를 포함한 구체적 분석 (예: "평균 85.3°C로 정상 범위")
- 정상/주의/위험 상태 평가 포함
- 마크다운 사용 금지
- 간결하게 1-2문장만"#;

        let user_content = format!(
            "사용자 요청: {}\n차트 제목: {}\n차트 타입: {:?}\n데이터: {}",
            user_request,
            chart_response.title,
            chart_response.chart_type,
            data_summary
        );

        let request_body = json!({
            "model": "claude-sonnet-4-5-20250929",
            "max_tokens": 256,
            "system": system_prompt,
            "messages": [
                {"role": "user", "content": user_content}
            ],
            "temperature": 0.3
        });

        println!("💡 [ChartService] Generating insight for: {}", chart_response.title);

        let response = self
            .http_client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.claude_api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Claude API error ({}): {}", status, error_text);
        }

        let response_json: serde_json::Value = response.json().await?;
        let insight = response_json["content"][0]["text"]
            .as_str()
            .unwrap_or("데이터 분석 결과를 생성할 수 없습니다.")
            .to_string();

        // 한글 안전하게 자르기 (UTF-8 문자 경계 보호)
        let truncated: String = insight.chars().take(50).collect();
        println!("✅ [ChartService] Insight generated: {}", truncated);
        Ok(insight)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chart_type_serialization() {
        let bar = ChartType::Bar;
        let serialized = serde_json::to_string(&bar).unwrap();
        assert_eq!(serialized, r#""bar""#);
    }
}
