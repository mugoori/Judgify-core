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

// 참고: Bar/Line 차트 데이터는 serde_json::Value로 직접 생성하여 평탄화된 JSON 구조 보장
// 예: { "name": "1라인", "total_output": 12450, "scrap_qty": 250 }

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
    /// Bar/Line 차트 데이터 - 평탄화된 JSON 객체 배열
    /// 예: [{ "name": "1라인", "total_output": 12450 }, ...]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bar_line_data: Option<Vec<serde_json::Value>>,
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
    #[serde(default)]
    pub color: Option<String>,
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

    /// MES/ERP 스키마 정보 반환
    fn get_mes_schema_info(&self) -> &'static str {
        r#"
## MES/ERP 데이터베이스 스키마 (SQLite) - 퓨어웰 음료㈜

### 1. MES 마스터 테이블
- **line_mst** (라인 마스터): line_cd(PK), line_nm, line_type(BATCHING|FILLING|PACKAGING), capacity_per_hour, is_active
- **equipment_mst** (설비 마스터): equip_cd(PK), equip_nm, line_cd(FK), equip_type, is_ccp, ccp_type
- **operation_mst** (공정 마스터): oper_cd(PK), oper_nm, oper_seq, line_cd(FK), is_ccp
- **shift_mst** (교대 마스터): shift_cd(PK), shift_nm, start_time, end_time
- **operator_mst** (작업자 마스터): operator_id(PK), operator_nm, dept, shift_cd(FK)
- **param_mst** (파라미터 마스터): param_cd(PK), param_nm, param_type, unit, equip_cd, min_val, max_val, target_val, is_ccp
- **reason_code_mst** (사유코드 마스터): reason_cd(PK), reason_type(DOWNTIME|DEFECT|ALARM|DEVIATION), reason_nm, category

### 2. 작업 실행 테이블
- **mes_work_order** (작업지시): wo_no(PK), prod_order_no, line_cd, shift_cd, plan_date, plan_start, plan_end, actual_start, actual_end, status(SCHEDULED|READY|RUNNING|PAUSED|COMPLETED|CANCELLED), plan_qty, good_qty, reject_qty, operator_id
- **operation_exec** (공정실행): id(PK), wo_no(FK), oper_cd(FK), batch_lot_no, equip_cd, start_time, end_time, status(RUNNING|COMPLETED|FAILED|PAUSED), result(OK|NG|DEVIATION), operator_id
- **operation_param_target** (공정 파라미터 목표값): id(PK), wo_no(FK), oper_cd(FK), param_cd(FK), target_val, tolerance_min, tolerance_max
- **operation_param_log** (공정 파라미터 실적): id(PK), operation_exec_id(FK), param_cd(FK), recorded_at, value, is_within_spec
- **checklist_result** (체크리스트 결과): id(PK), wo_no(FK), checklist_type(PRE_START|HOURLY|SHIFT_END|CIP|QUALITY), check_time, operator_id, items(JSON), overall_result(OK|NG|NA)

### 3. LOT 추적성 테이블 (불량 분석 핵심!)
- **batch_lot** (배치LOT): batch_lot_no(PK), prod_order_no(FK), bom_cd, line_cd, batch_size, tank_no, start_time, end_time, status(CREATED|PROCESSING|COMPLETED|CANCELLED|HOLD), operator_id
- **filling_lot** (충진LOT): filling_lot_no(PK), batch_lot_no(FK), filling_date, line_cd, pkg_item_cd, plan_qty, good_qty, reject_qty, start_time, end_time, status
  - 불량률 계산: ROUND(SUM(reject_qty) * 100.0 / NULLIF(SUM(plan_qty), 0), 2) as defect_rate
- **fg_lot** (완제품LOT): fg_lot_no(PK), filling_lot_no(FK), fg_item_cd, qty, mfg_date, exp_date, qc_status(PENDING|PASS|FAIL|HOLD), location
- **process_result** (공정실적): id(PK), batch_lot_no(FK), process_type(BATCHING|PASTEURIZATION|COOLING|HOLDING|TRANSFER), equipment_cd, start_time, end_time, target_temp, actual_temp, target_time_sec, actual_time_sec, result(OK|NG|PENDING), operator_id
- **material_issue** (자재 출고): batch_lot_no(FK), seq, item_cd, lot_no, plan_qty, actual_qty, issue_time, operator_id

### 4. 품질 검사 테이블
- **qc_test** (품질검사-기본): qc_no(PK), test_type(INCOMING|IN_PROCESS|FINAL|HOLD_RELEASE), ref_type(INBOUND|BATCH|FILLING|FG), ref_no, item_cd, lot_no, test_date, tester_id, result(PASS|FAIL|CONDITIONAL), test_items(JSON), remarks
- **qc_inspection** (품질검사-상세): inspection_no(PK), inspection_type(INCOMING|IN_PROCESS|FINAL), lot_no, item_cd, inspection_time, inspector_id, ph_level, acidity, brix, fat_content, protein_content, viscosity, moisture, total_bacteria, coliform, color_l, color_a, color_b, result(PASS|FAIL|HOLD), remark
  - 품질 지표: pH(6.5-6.8), 산도(0.14-0.18%), Brix(8-16), 지방(1.5-4%), 단백질(2.5-3.3%)
- **metal_detection_log** (금속검출 로그): id(PK), detection_time, equip_cd, line_cd, lot_no, metal_detected(0|1), metal_type(FE|SUS|NON_FE), sensitivity_fe, sensitivity_sus, sensitivity_non_fe, reject_action(REJECTED|PASSED|RECHECK), operator_id

### 5. 센서/CCP 테이블
- **sensor_log** (센서로그): id(PK), equip_cd(FK), param_cd(FK), batch_lot_no, recorded_at, value, is_alarm, alarm_type(LOW|HIGH|CRITICAL_LOW|CRITICAL_HIGH)
- **ccp_check_log** (CCP검사): id(PK), batch_lot_no, ccp_type(PASTEURIZATION|METAL_DETECTION|COOLING), check_time, equip_cd, operator_id, target_temp, actual_temp, target_time_sec, actual_time_sec, sensitivity_fe, sensitivity_sus, test_piece_detected, reject_confirmed, result(PASS|FAIL|DEVIATION), corrective_action, verified_by, verified_at
- **process_param_log** (공정 파라미터 로그): id(PK), recorded_at, equip_cd, batch_lot_no, sterilization_temp, holding_time_sec, homogenizer_pressure, tank_temp, cip_status, fill_speed, fill_volume, cooling_temp

### 6. 이벤트 테이블
- **downtime_event** (비가동): id(PK), wo_no(FK), equip_cd(FK), line_cd(FK), start_time, end_time, duration_min, reason_cd, reason_detail, is_planned, reported_by
- **alarm_event** (알람): id(PK), equip_cd(FK), param_cd, batch_lot_no, alarm_time, alarm_level(INFO|WARNING|CRITICAL), alarm_type(PARAM_HIGH|PARAM_LOW|CCP_DEVIATION|EQUIP_FAULT|QUALITY_ISSUE|SAFETY), message, value, threshold, is_acknowledged, acknowledged_by, acknowledged_at, is_resolved, resolved_by, resolved_at, resolution

### 7. ERP 마스터 테이블
- **item_mst** (품목 마스터): item_cd(PK), item_nm, item_type(RM|FG|PKG|WIP), unit, category, spec, shelf_life_days, is_active
- **vendor_mst** (거래처 마스터): vendor_cd(PK), vendor_nm, vendor_type(SUPPLIER|MANUFACTURER|BOTH), contact_nm, phone, email, address, is_active
- **customer_mst** (고객 마스터): cust_cd(PK), cust_nm, cust_type(RETAIL|WHOLESALE|ONLINE|DISTRIBUTOR), contact_nm, phone, email, address, credit_limit, is_active
- **bom_mst** (BOM 마스터): bom_cd(PK), fg_item_cd, bom_nm, batch_size, batch_unit, version, is_active
- **bom_dtl** (BOM 상세): id(PK), bom_cd(FK), item_cd, seq, usage_qty, unit, loss_rate
- **warehouse_mst** (창고 마스터): warehouse_id(PK), warehouse_nm, warehouse_type(RAW|COLD|WIP|FG|FROZEN|SHIPPING), location, temp_min, temp_max

### 8. ERP 거래 테이블
- **purchase_order** (발주서): po_no(PK), vendor_cd(FK), order_date, expected_date, status(DRAFT|CONFIRMED|PARTIAL|COMPLETED|CANCELLED), total_amount, remarks, created_by
- **purchase_order_dtl** (발주상세): id(PK), po_no(FK), item_cd, qty, unit, unit_price, amount, received_qty, status
- **inbound** (입고): inbound_no(PK), inbound_date, po_no(FK), vendor_cd, status(SCHEDULED|IN_PROGRESS|COMPLETED|CANCELLED), total_amount
- **inbound_dtl** (입고상세): id(PK), inbound_no(FK), po_dtl_id(FK), item_cd, lot_no, qty, unit, unit_price, amount, warehouse_id, qc_status
- **production_order** (생산지시): prod_order_no(PK), bom_cd, plan_qty, actual_qty, plan_date, status(PLANNED|RELEASED|IN_PROGRESS|COMPLETED|CANCELLED), priority, remarks, created_by
- **sales_order** (수주): so_no(PK), cust_cd(FK), order_date, request_date, status(DRAFT|CONFIRMED|ALLOCATED|SHIPPED|COMPLETED|CANCELLED), total_amount, ship_to, remarks, created_by
- **sales_order_dtl** (수주상세): id(PK), so_no(FK), item_cd, qty, unit, unit_price, amount, allocated_qty, shipped_qty, status
- **outbound** (출고): outbound_no(PK), outbound_date, so_no(FK), cust_cd, status(SCHEDULED|PICKING|COMPLETED|CANCELLED), ship_to
- **outbound_dtl** (출고상세): id(PK), outbound_no(FK), so_dtl_id(FK), item_cd, lot_no, qty, unit, warehouse_id
- **inventory** (재고): id(PK), item_cd, lot_no, location(창고코드, 예: WH01), qty, reserved_qty, unit, exp_date, last_move_date, updated_at
- **inventory_movement** (재고 이동): movement_no(PK), movement_type(IN|OUT|TRANSFER), movement_date, item_cd, lot_no, qty, unit, from_warehouse, to_warehouse, ref_type(PO|SO|PROD|INBOUND|OUTBOUND|ADJUST), ref_no
- **material_input_log** (자재 투입 이력): id(PK), input_time, batch_lot_no, wo_no, material_lot_no, item_cd, item_nm, plan_qty, input_qty, remain_qty, unit, operator_id, is_verified

### 샘플 데이터 정보 (2024년 8월~11월, 4개월치)
- 라인: LINE-A(배합), LINE-B(충진), LINE-C(포장)
- 설비: EQ-MIX-01(배합기), EQ-PAST-01(살균기), EQ-FILL-01(충진기), EQ-MD-01(금속검출기) 등
- 작업지시: WO-240805-A01 ~ (2024년 8~11월)
- LOT 데이터: batch_lot, filling_lot, fg_lot
- 품질검사: qc_inspection, qc_test, metal_detection_log
- CCP 체크: ccp_check_log (살균/금속검출/냉각)
- 이벤트: downtime_event, alarm_event
- 자재: material_issue, material_input_log
- 재고: inventory, inventory_movement, warehouse_mst

### 분석 쿼리 예시
1. **라인별 불량률**:
   SELECT line_cd, ROUND(SUM(reject_qty)*100.0/NULLIF(SUM(good_qty+reject_qty),0),2) as defect_rate
   FROM filling_lot GROUP BY line_cd

2. **월별 생산량 추이** (fg_lot 테이블 사용):
   SELECT strftime('%Y-%m', production_dt) as month, SUM(qty) as production
   FROM fg_lot GROUP BY month ORDER BY month

3. **CCP 합격률**:
   SELECT ccp_type, ROUND(SUM(CASE WHEN result='PASS' THEN 1 ELSE 0 END)*100.0/COUNT(*),1) as pass_rate
   FROM ccp_check_log GROUP BY ccp_type

4. **설비별 비가동 시간**:
   SELECT equip_cd, SUM(duration_min) as total_downtime, COUNT(*) as event_count
   FROM downtime_event GROUP BY equip_cd ORDER BY total_downtime DESC

5. **품질검사 결과 분포**:
   SELECT result, COUNT(*) as cnt FROM qc_inspection GROUP BY result

6. **월별 매출**:
   SELECT strftime('%Y-%m', order_date) as month, SUM(total_amount) as sales
   FROM sales_order WHERE status IN ('CONFIRMED','SHIPPED','DELIVERED') GROUP BY month

7. **금속검출 현황**:
   SELECT line_cd, COUNT(*) as total, SUM(metal_detected) as detected
   FROM metal_detection_log GROUP BY line_cd

8. **재고 이동 현황**:
   SELECT movement_type, COUNT(*) as cnt, SUM(qty) as total_qty
   FROM inventory_movement GROUP BY movement_type

9. **창고별 재고 현황** (inventory 테이블은 location 컬럼 사용):
   SELECT i.location as warehouse, w.warehouse_nm, SUM(i.qty) as total_qty, COUNT(DISTINCT i.item_cd) as item_count
   FROM inventory i LEFT JOIN warehouse_mst w ON i.location = w.warehouse_id
   GROUP BY i.location ORDER BY total_qty DESC

10. **품목별 재고 현황**:
   SELECT item_cd, SUM(qty) as total_qty, SUM(reserved_qty) as reserved, COUNT(DISTINCT location) as locations
   FROM inventory GROUP BY item_cd ORDER BY total_qty DESC
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
            "max_tokens": 8192,
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
                let colors = ["#3b82f6", "#22c55e", "#ef4444", "#f59e0b", "#8b5cf6", "#06b6d4"];
                let data_keys: Vec<DataKeyConfig> = plan.data_keys.clone()
                    .map(|keys| {
                        // LLM이 color를 생성하지 않은 경우 기본값 할당
                        keys.into_iter().enumerate().map(|(i, mut dk)| {
                            if dk.color.is_none() {
                                dk.color = Some(colors[i % colors.len()].to_string());
                            }
                            dk
                        }).collect()
                    })
                    .unwrap_or_else(|| {
                        columns.iter().skip(1).enumerate().map(|(i, col)| {
                            DataKeyConfig {
                                key: col.clone(),
                                color: Some(colors[i % colors.len()].to_string()),
                                label: col.clone(),
                            }
                        }).collect()
                    });

                // serde_json::Value로 직접 평탄화된 JSON 객체 생성
                // Recharts 기대 형식: { "name": "1라인", "total_output": 12450, "scrap_qty": 250 }
                println!("[CHART_DEBUG] columns: {:?}", columns);
                println!("[CHART_DEBUG] rows count: {}", rows.len());
                println!("[CHART_DEBUG] data_keys: {:?}", data_keys.iter().map(|dk| dk.key.clone()).collect::<Vec<String>>());

                let chart_data: Vec<serde_json::Value> = rows.iter().map(|row| {
                    let name = row.get(0)
                        .and_then(|v| v.as_str().map(|s| s.to_string()))
                        .or_else(|| row.get(0).and_then(|v| v.as_i64().map(|n| n.to_string())))
                        .unwrap_or_default();

                    let mut obj = serde_json::Map::new();
                    obj.insert("name".to_string(), serde_json::Value::String(name));

                    for (i, col) in columns.iter().enumerate().skip(1) {
                        if let Some(val) = row.get(i) {
                            if let Some(n) = val.as_f64() {
                                if let Some(num) = serde_json::Number::from_f64(n) {
                                    obj.insert(col.clone(), serde_json::Value::Number(num));
                                }
                            } else if let Some(n) = val.as_i64() {
                                obj.insert(col.clone(), serde_json::Value::Number(serde_json::Number::from(n)));
                            }
                        }
                    }
                    serde_json::Value::Object(obj)
                }).collect();

                // 디버그: 생성된 chart_data 출력
                if let Ok(json_str) = serde_json::to_string_pretty(&chart_data) {
                    println!("[CHART_DEBUG] chart_data: {}", json_str);
                }

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
                    let summary: Vec<String> = data.iter().filter_map(|point| {
                        // serde_json::Value에서 필드 추출
                        let obj = point.as_object()?;
                        let name = obj.get("name")?.as_str().unwrap_or("unknown");
                        let values: Vec<String> = obj.iter()
                            .filter(|(k, _)| *k != "name")
                            .filter_map(|(k, v)| {
                                v.as_f64().map(|n| format!("{}={:.1}", k, n))
                            })
                            .collect();
                        Some(format!("{}: {}", name, values.join(", ")))
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
            "max_tokens": 8192,
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
