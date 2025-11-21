use serde::{Deserialize, Serialize};
use crate::services::workflow_service::WorkflowService;
use crate::engines::rule_engine::RuleEngine;
use crate::services::judgment_engine::{JudgmentEngine, JudgmentInput};
use serde_json::json;
use rusqlite::{params, Connection};

/// Phase 9 WorkflowBuilderV2용 데이터 구조
///
/// Ver2.0 6개 NodeType:
/// - TRIGGER: 트리거 (임계값, 스케줄, 이벤트, 수동)
/// - QUERY: 데이터 조회 (DB, API, 센서, 파일)
/// - CALC: 계산 (수식, 집계, 변환)
/// - JUDGMENT: AI 판단 (Rule/LLM/Hybrid)
/// - APPROVAL: 승인 (수동, 자동, 조건부)
/// - ALERT: 알림 (Email, Slack, Teams, Webhook)

/// 워크플로우 메타데이터 (WorkflowBuilderV2.tsx와 동기화)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkflowMetadata {
    pub name: String,
    pub description: String,
    #[serde(rename = "isActive")]
    pub is_active: bool,
}

/// 워크플로우 스텝 (WorkflowBuilderV2.tsx와 동기화)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkflowStep {
    pub id: String,
    #[serde(rename = "type")]
    pub step_type: String, // "TRIGGER" | "QUERY" | "CALC" | "JUDGMENT" | "APPROVAL" | "ALERT"
    pub label: String,
    pub config: serde_json::Value, // NodeType별 설정 (Forms에서 생성)
}

/// 워크플로우 저장 요청 (Phase 2 UI → Backend)
#[derive(Debug, Serialize, Deserialize)]
pub struct SaveWorkflowRequest {
    pub metadata: WorkflowMetadata,
    pub steps: Vec<WorkflowStep>,
}

/// 워크플로우 저장 응답
#[derive(Debug, Serialize, Deserialize)]
pub struct SaveWorkflowResponse {
    pub id: String,
    pub version: i32,
    pub message: String,
}

/// 워크플로우 불러오기 응답
#[derive(Debug, Serialize, Deserialize)]
pub struct LoadWorkflowResponse {
    pub id: String,
    pub metadata: WorkflowMetadata,
    pub steps: Vec<WorkflowStep>,
    pub version: i32,
    pub created_at: String,
}

/// 워크플로우 목록 조회 응답
#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowListItem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub is_active: bool,
    pub step_count: usize,
    pub version: i32,
    pub created_at: String,
}

/// 워크플로우 저장 Tauri Command
///
/// Phase 3-2에서 구현될 CRUD API의 일부
/// Frontend: WorkflowBuilderV2.handleSaveWorkflow() → Backend: 이 함수
#[tauri::command]
pub async fn save_workflow_v2(
    request: SaveWorkflowRequest,
) -> Result<SaveWorkflowResponse, String> {
    println!("💾 [WorkflowV2] 워크플로우 저장 요청:");
    println!("   이름: {}", request.metadata.name);
    println!("   스텝 개수: {}", request.steps.len());

    // JSON definition 생성 (Phase 2 구조 그대로 저장)
    let definition = json!({
        "metadata": request.metadata,
        "steps": request.steps,
        "version": "2.0", // Phase 2 버전
        "format": "vertical-list" // Phase 2 UI 형식
    });

    // WorkflowService로 저장
    let service = WorkflowService::new()
        .map_err(|e| format!("WorkflowService 초기화 실패: {}", e))?;

    let workflow = service
        .create_workflow(
            request.metadata.name.clone(),
            definition,
            None, // rule_expression은 JUDGMENT 노드 config에 저장됨
        )
        .map_err(|e| format!("워크플로우 저장 실패: {}", e))?;

    println!("✅ [WorkflowV2] 워크플로우 저장 완료: {}", workflow.id);

    Ok(SaveWorkflowResponse {
        id: workflow.id,
        version: workflow.version,
        message: "워크플로우가 성공적으로 저장되었습니다.".to_string(),
    })
}

/// 워크플로우 불러오기 Tauri Command
#[tauri::command]
pub async fn load_workflow_v2(workflow_id: String) -> Result<LoadWorkflowResponse, String> {
    println!("📂 [WorkflowV2] 워크플로우 불러오기: {}", workflow_id);

    let service = WorkflowService::new()
        .map_err(|e| format!("WorkflowService 초기화 실패: {}", e))?;

    let workflow = service
        .get_workflow(&workflow_id)
        .map_err(|e| format!("워크플로우 조회 실패: {}", e))?
        .ok_or_else(|| format!("워크플로우를 찾을 수 없습니다: {}", workflow_id))?;

    // JSON definition 파싱
    let definition: serde_json::Value = serde_json::from_str(&workflow.definition)
        .map_err(|e| format!("워크플로우 definition 파싱 실패: {}", e))?;

    let metadata: WorkflowMetadata = serde_json::from_value(definition["metadata"].clone())
        .map_err(|e| format!("metadata 파싱 실패: {}", e))?;

    let steps: Vec<WorkflowStep> = serde_json::from_value(definition["steps"].clone())
        .map_err(|e| format!("steps 파싱 실패: {}", e))?;

    println!("✅ [WorkflowV2] 워크플로우 불러오기 완료: {} (스텝 {}개)", workflow.id, steps.len());

    Ok(LoadWorkflowResponse {
        id: workflow.id,
        metadata,
        steps,
        version: workflow.version,
        created_at: workflow.created_at.to_rfc3339(),
    })
}

/// 워크플로우 목록 조회 Tauri Command
#[tauri::command]
pub async fn list_workflows_v2() -> Result<Vec<WorkflowListItem>, String> {
    println!("📋 [WorkflowV2] 워크플로우 목록 조회");

    let service = WorkflowService::new()
        .map_err(|e| format!("WorkflowService 초기화 실패: {}", e))?;

    let workflows = service
        .get_all_workflows()
        .map_err(|e| format!("워크플로우 목록 조회 실패: {}", e))?;

    let list: Vec<WorkflowListItem> = workflows
        .into_iter()
        .filter_map(|w| {
            // JSON definition 파싱
            let definition: serde_json::Value = serde_json::from_str(&w.definition).ok()?;

            // metadata 추출
            let metadata: WorkflowMetadata = serde_json::from_value(definition["metadata"].clone()).ok()?;

            // steps 배열 크기 추출
            let steps_count = definition["steps"].as_array().map(|arr| arr.len()).unwrap_or(0);

            Some(WorkflowListItem {
                id: w.id,
                name: metadata.name,
                description: metadata.description,
                is_active: w.is_active,
                step_count: steps_count,
                version: w.version,
                created_at: w.created_at.to_rfc3339(),
            })
        })
        .collect();

    println!("✅ [WorkflowV2] 워크플로우 목록 조회 완료: {}개", list.len());

    Ok(list)
}

/// 워크플로우 삭제 Tauri Command (Soft Delete)
#[tauri::command]
pub async fn delete_workflow_v2(workflow_id: String) -> Result<String, String> {
    println!("🗑️ [WorkflowV2] 워크플로우 삭제: {}", workflow_id);

    let service = WorkflowService::new()
        .map_err(|e| format!("WorkflowService 초기화 실패: {}", e))?;

    service
        .delete_workflow(&workflow_id)
        .map_err(|e| format!("워크플로우 삭제 실패: {}", e))?;

    println!("✅ [WorkflowV2] 워크플로우 삭제 완료: {}", workflow_id);

    Ok(format!("워크플로우 {}가 성공적으로 삭제되었습니다.", workflow_id))
}

/// 워크플로우 시뮬레이션 요청
#[derive(Debug, Serialize, Deserialize)]
pub struct SimulateWorkflowRequest {
    pub workflow_id: String,
    pub steps: Vec<WorkflowStep>,
    pub test_data: serde_json::Value, // 시뮬레이션 입력 데이터
}

/// 워크플로우 시뮬레이션 응답
#[derive(Debug, Serialize, Deserialize)]
pub struct SimulateWorkflowResponse {
    pub workflow_id: String,
    pub steps_executed: Vec<StepExecutionResult>,
    pub final_result: serde_json::Value,
    pub total_execution_time_ms: u64,
    pub status: String, // "success" | "partial_success" | "error"
}

/// 스텝 실행 결과
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StepExecutionResult {
    pub step_id: String,
    pub step_type: String,
    pub label: String,
    pub status: String, // "success" | "error" | "skipped"
    pub input: serde_json::Value,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
}

/// 워크플로우 실행 이력 목록 항목
#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowExecutionListItem {
    pub id: String,
    pub workflow_id: String,
    pub status: String,
    pub execution_time_ms: i64,
    pub created_at: String,
}

/// 워크플로우 실행 이력 상세
#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowExecutionDetail {
    pub id: String,
    pub workflow_id: String,
    pub status: String,
    pub steps_executed: Vec<StepExecutionResult>,
    pub final_result: serde_json::Value,
    pub execution_time_ms: i64,
    pub created_at: String,
}

/// 워크플로우 시뮬레이션 Tauri Command
///
/// Phase 3-5에서 6개 NodeType 실행 로직 구현
/// 현재는 기본 스켈레톤만 제공
#[tauri::command]
pub async fn simulate_workflow_v2(
    request: SimulateWorkflowRequest,
) -> Result<SimulateWorkflowResponse, String> {
    println!("🎭 [WorkflowV2] 워크플로우 시뮬레이션 시작: {}", request.workflow_id);
    println!("   스텝 개수: {}", request.steps.len());

    let start_time = std::time::Instant::now();
    let mut steps_executed: Vec<StepExecutionResult> = Vec::new();
    let mut global_data = request.test_data.clone();
    let mut overall_status = "success".to_string();

    // 각 스텝 순차 실행
    for step in request.steps.iter() {
        println!("  ▶️ 스텝 실행: {} ({})", step.label, step.step_type);

        let step_start = std::time::Instant::now();
        let result = execute_step_v2(step, &global_data).await;

        let execution_time = step_start.elapsed().as_millis() as u64;

        match result {
            Ok((output, next_data)) => {
                steps_executed.push(StepExecutionResult {
                    step_id: step.id.clone(),
                    step_type: step.step_type.clone(),
                    label: step.label.clone(),
                    status: "success".to_string(),
                    input: global_data.clone(),
                    output: Some(output.clone()),
                    error: None,
                    execution_time_ms: execution_time,
                });

                // 다음 스텝으로 데이터 전달
                global_data = next_data;
            }
            Err(e) => {
                steps_executed.push(StepExecutionResult {
                    step_id: step.id.clone(),
                    step_type: step.step_type.clone(),
                    label: step.label.clone(),
                    status: "error".to_string(),
                    input: global_data.clone(),
                    output: None,
                    error: Some(e.clone()),
                    execution_time_ms: execution_time,
                });

                overall_status = "partial_success".to_string();
                println!("  ❌ 스텝 실행 실패: {}", e);
                break; // 에러 발생시 중단
            }
        }
    }

    let total_time = start_time.elapsed().as_millis() as u64;

    println!("✅ [WorkflowV2] 워크플로우 시뮬레이션 완료: {}ms (상태: {})", total_time, overall_status);

    // DB에 실행 이력 저장
    let execution_id = match get_db_connection() {
        Ok(conn) => {
            match save_workflow_execution(
                &conn,
                &request.workflow_id,
                &overall_status,
                &steps_executed,
                &global_data,
                total_time,
            ) {
                Ok(id) => Some(id),
                Err(e) => {
                    eprintln!("⚠️ [WorkflowV2] DB 저장 실패 (무시): {}", e);
                    None
                }
            }
        }
        Err(e) => {
            eprintln!("⚠️ [WorkflowV2] DB 연결 실패 (무시): {}", e);
            None
        }
    };

    if let Some(id) = &execution_id {
        println!("💾 [WorkflowV2] 실행 ID: {}", id);
    }

    Ok(SimulateWorkflowResponse {
        workflow_id: request.workflow_id,
        steps_executed,
        final_result: global_data,
        total_execution_time_ms: total_time,
        status: overall_status,
    })
}

/// 개별 스텝 실행 로직
///
/// Phase 3-5에서 6개 NodeType별 실행 로직 상세 구현
/// 현재는 기본 스켈레톤만 제공
async fn execute_step_v2(
    step: &WorkflowStep,
    input_data: &serde_json::Value,
) -> Result<(serde_json::Value, serde_json::Value), String> {
    match step.step_type.as_str() {
        "TRIGGER" => execute_trigger_step(step, input_data).await,
        "QUERY" => execute_query_step(step, input_data).await,
        "CALC" => execute_calc_step(step, input_data).await,
        "JUDGMENT" => execute_judgment_step(step, input_data).await,
        "APPROVAL" => execute_approval_step(step, input_data).await,
        "ALERT" => execute_alert_step(step, input_data).await,
        _ => Err(format!("지원하지 않는 스텝 타입: {}", step.step_type)),
    }
}

/// TRIGGER 스텝 실행
async fn execute_trigger_step(
    step: &WorkflowStep,
    input_data: &serde_json::Value,
) -> Result<(serde_json::Value, serde_json::Value), String> {
    let config = &step.config;
    let trigger_type = config["triggerType"].as_str().unwrap_or("manual");

    match trigger_type {
        "threshold" => {
            // 임계값 초과 트리거
            let condition = config["condition"].as_str().ok_or("condition 필드 필요")?;
            let threshold = config["threshold"].as_f64().ok_or("threshold 필드 필요")?;

            // condition 파싱 (예: "temperature > 90")
            let parts: Vec<&str> = condition.split_whitespace().collect();
            if parts.len() < 3 {
                return Err("condition 형식 오류 (예: temperature > 90)".to_string());
            }

            let field_name = parts[0];
            let operator = parts[1];
            let field_value = input_data[field_name].as_f64().unwrap_or(0.0);

            let triggered = match operator {
                ">" => field_value > threshold,
                ">=" => field_value >= threshold,
                "<" => field_value < threshold,
                "<=" => field_value <= threshold,
                "==" => (field_value - threshold).abs() < 0.0001,
                _ => return Err(format!("지원하지 않는 연산자: {}", operator)),
            };

            Ok((
                json!({
                    "step_type": "TRIGGER",
                    "trigger_type": "threshold",
                    "triggered": triggered,
                    "condition": condition,
                    "threshold": threshold,
                    "actual_value": field_value,
                    "message": if triggered {
                        format!("{} {} {} 조건 충족", field_name, operator, threshold)
                    } else {
                        format!("{} {} {} 조건 미충족 (현재: {})", field_name, operator, threshold, field_value)
                    }
                }),
                input_data.clone(),
            ))
        }
        "scheduled" => {
            // 스케줄 트리거 (시뮬레이션에서는 항상 true)
            let schedule = config["schedule"].as_str().unwrap_or("* * * * *");
            Ok((
                json!({
                    "step_type": "TRIGGER",
                    "trigger_type": "scheduled",
                    "triggered": true,
                    "schedule": schedule,
                    "message": format!("스케줄 트리거 실행 ({})", schedule)
                }),
                input_data.clone(),
            ))
        }
        "event" => {
            // 이벤트 트리거 (시뮬레이션에서는 항상 true)
            Ok((
                json!({
                    "step_type": "TRIGGER",
                    "trigger_type": "event",
                    "triggered": true,
                    "message": "이벤트 트리거 감지"
                }),
                input_data.clone(),
            ))
        }
        "manual" => {
            // 수동 트리거 (시뮬레이션에서는 항상 true)
            Ok((
                json!({
                    "step_type": "TRIGGER",
                    "trigger_type": "manual",
                    "triggered": true,
                    "message": "수동 트리거 실행"
                }),
                input_data.clone(),
            ))
        }
        _ => Err(format!("지원하지 않는 트리거 타입: {}", trigger_type)),
    }
}

/// QUERY 스텝 실행
async fn execute_query_step(
    step: &WorkflowStep,
    input_data: &serde_json::Value,
) -> Result<(serde_json::Value, serde_json::Value), String> {
    let config = &step.config;
    let data_source = config["dataSource"].as_str().unwrap_or("database");
    let query = config["query"].as_str().unwrap_or("");

    match data_source {
        "database" => {
            // 실제 SQLite 데이터베이스 조회
            let query_type = config["queryType"].as_str().unwrap_or("sql");

            // DB 경로 가져오기
            let app_data = std::env::var("APPDATA")
                .or_else(|_| std::env::var("HOME"))
                .map_err(|e| format!("환경변수 오류: {}", e))?;
            let db_path = std::path::PathBuf::from(app_data).join("Judgify").join("judgify.db");

            // DB 연결
            let conn = Connection::open(&db_path)
                .map_err(|e| format!("DB 연결 실패: {}", e))?;

            // 쿼리 실행
            let query_result = if query.is_empty() {
                // 기본 쿼리: 최근 judgments 조회
                execute_default_query(&conn)?
            } else {
                // 사용자 지정 쿼리 실행 (SELECT만 허용)
                execute_custom_query(&conn, query)?
            };

            let row_count = query_result.as_array().map(|a| a.len()).unwrap_or(0);

            let mut output_data = input_data.clone();
            if let Some(obj) = output_data.as_object_mut() {
                obj.insert("query_result".to_string(), query_result.clone());
            }

            Ok((
                json!({
                    "step_type": "QUERY",
                    "data_source": "database",
                    "query_type": query_type,
                    "query": if query.is_empty() { "SELECT * FROM judgments LIMIT 10" } else { query },
                    "data": query_result,
                    "message": format!("데이터베이스 조회 완료 ({}개 결과)", row_count)
                }),
                output_data,
            ))
        }
        "api" => {
            // 외부 API 호출 (Mock 응답)
            let mock_response = json!({
                "status": "success",
                "data": {
                    "sensor_id": "SENS-001",
                    "readings": [85.2, 86.1, 87.5]
                }
            });

            let mut output_data = input_data.clone();
            if let Some(obj) = output_data.as_object_mut() {
                obj.insert("api_response".to_string(), mock_response.clone());
            }

            Ok((
                json!({
                    "step_type": "QUERY",
                    "data_source": "api",
                    "endpoint": query,
                    "response": mock_response,
                    "message": "API 호출 성공"
                }),
                output_data,
            ))
        }
        "sensor" => {
            // 센서 데이터 조회 (Mock)
            let mock_sensor_data = json!({
                "timestamp": "2025-11-20T10:30:00Z",
                "temperature": 88.5,
                "vibration": 42.3,
                "pressure": 120.5
            });

            let mut output_data = input_data.clone();
            if let Some(obj) = output_data.as_object_mut() {
                obj.insert("sensor_data".to_string(), mock_sensor_data.clone());
            }

            Ok((
                json!({
                    "step_type": "QUERY",
                    "data_source": "sensor",
                    "sensor_data": mock_sensor_data,
                    "message": "센서 데이터 수집 완료"
                }),
                output_data,
            ))
        }
        "file" => {
            // 파일 시스템 조회 (Mock)
            let mock_file_data = json!({
                "filename": "production_data.csv",
                "rows": 150,
                "sample": [
                    {"date": "2025-11-19", "output": 1250, "defects": 15},
                    {"date": "2025-11-20", "output": 1180, "defects": 22}
                ]
            });

            let mut output_data = input_data.clone();
            if let Some(obj) = output_data.as_object_mut() {
                obj.insert("file_data".to_string(), mock_file_data.clone());
            }

            Ok((
                json!({
                    "step_type": "QUERY",
                    "data_source": "file",
                    "file_data": mock_file_data,
                    "message": "파일 조회 완료"
                }),
                output_data,
            ))
        }
        _ => Err(format!("지원하지 않는 데이터 소스: {}", data_source)),
    }
}

/// CALC 스텝 실행
async fn execute_calc_step(
    step: &WorkflowStep,
    input_data: &serde_json::Value,
) -> Result<(serde_json::Value, serde_json::Value), String> {
    let config = &step.config;
    let calc_type = config["calcType"].as_str().unwrap_or("formula");
    let output_field = config["outputField"].as_str().unwrap_or("result");

    match calc_type {
        "formula" => {
            // 수식 계산
            let formula = config["formula"].as_str().ok_or("formula 필드 필요")?;

            // 간단한 수식 평가 (예: "(defect_count / total_count) * 100")
            // input_data의 변수를 치환
            let mut eval_formula = formula.to_string();

            if let Some(obj) = input_data.as_object() {
                for (key, value) in obj {
                    if let Some(num) = value.as_f64() {
                        eval_formula = eval_formula.replace(key, &num.to_string());
                    }
                }
            }

            // 간단한 수식 평가 (evalexpr 크레이트 사용 권장, 여기서는 간단 구현)
            let result = evaluate_simple_formula(&eval_formula)
                .map_err(|e| format!("수식 평가 실패: {}", e))?;

            let mut output_data = input_data.clone();
            if let Some(obj) = output_data.as_object_mut() {
                obj.insert(output_field.to_string(), json!(result));
            }

            Ok((
                json!({
                    "step_type": "CALC",
                    "calc_type": "formula",
                    "formula": formula,
                    "result": result,
                    "output_field": output_field,
                    "message": format!("수식 계산 완료: {} = {}", output_field, result)
                }),
                output_data,
            ))
        }
        "aggregate" => {
            // 집계 함수 (avg, sum, min, max, count)
            let agg_func = config["aggregateFunction"].as_str().unwrap_or("avg");
            let target_field = config["targetField"].as_str().ok_or("targetField 필드 필요")?;

            // input_data에서 배열 데이터 추출
            let values: Vec<f64> = if let Some(arr) = input_data[target_field].as_array() {
                arr.iter()
                    .filter_map(|v| v.as_f64())
                    .collect()
            } else if let Some(num) = input_data[target_field].as_f64() {
                vec![num]
            } else {
                return Err(format!("{} 필드가 숫자 배열이 아닙니다", target_field));
            };

            if values.is_empty() {
                return Err("집계할 데이터가 없습니다".to_string());
            }

            let result = match agg_func {
                "sum" => values.iter().sum(),
                "avg" => values.iter().sum::<f64>() / values.len() as f64,
                "min" => values.iter().cloned().fold(f64::INFINITY, f64::min),
                "max" => values.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
                "count" => values.len() as f64,
                _ => return Err(format!("지원하지 않는 집계 함수: {}", agg_func)),
            };

            let mut output_data = input_data.clone();
            if let Some(obj) = output_data.as_object_mut() {
                obj.insert(output_field.to_string(), json!(result));
            }

            Ok((
                json!({
                    "step_type": "CALC",
                    "calc_type": "aggregate",
                    "aggregate_function": agg_func,
                    "target_field": target_field,
                    "result": result,
                    "output_field": output_field,
                    "message": format!("집계 완료: {}({}) = {}", agg_func, target_field, result)
                }),
                output_data,
            ))
        }
        "transform" => {
            // 데이터 변환 (간단 구현)
            Ok((
                json!({
                    "step_type": "CALC",
                    "calc_type": "transform",
                    "message": "데이터 변환 완료 (Mock)"
                }),
                input_data.clone(),
            ))
        }
        _ => Err(format!("지원하지 않는 계산 타입: {}", calc_type)),
    }
}

/// 간단한 수식 평가 함수 (사칙연산만 지원)
fn evaluate_simple_formula(formula: &str) -> Result<f64, String> {
    // 공백 제거
    let formula = formula.replace(" ", "");

    // 간단한 파서 (괄호, 사칙연산)
    // 실제 프로덕션에서는 evalexpr 크레이트 사용 권장
    meval::eval_str(&formula).map_err(|e| format!("수식 평가 오류: {}", e))
}

/// JUDGMENT 스텝 실행 (Phase 4: 하이브리드 판단 통합)
async fn execute_judgment_step(
    step: &WorkflowStep,
    input_data: &serde_json::Value,
) -> Result<(serde_json::Value, serde_json::Value), String> {
    let config = &step.config;
    let judgment_method = config["judgmentMethod"].as_str().unwrap_or("rule");

    match judgment_method {
        "rule" => {
            // Rule Engine만 사용
            let rule_expr = config["ruleExpression"]
                .as_str()
                .ok_or("Rule 표현식이 설정되지 않았습니다.")?;

            let engine = RuleEngine::new();
            let result = engine
                .evaluate(rule_expr, input_data)
                .map_err(|e| format!("Rule 평가 실패: {}", e))?;

            Ok((
                json!({
                    "step_type": "JUDGMENT",
                    "judgment": result,
                    "method": "rule",
                    "confidence": 1.0,
                    "explanation": "Rule Engine 기반 판단"
                }),
                input_data.clone(),
            ))
        }
        "llm" | "hybrid" => {
            // JudgmentEngine 서비스 사용 (LLM + Few-shot 학습)
            let workflow_id = format!("workflow-{}", step.id);

            let judgment_input = JudgmentInput {
                workflow_id: workflow_id.clone(),
                input_data: input_data.clone(),
            };

            let engine = JudgmentEngine::new()
                .map_err(|e| format!("JudgmentEngine 초기화 실패: {}", e))?;

            let result = engine
                .judge_with_few_shot(judgment_input)
                .await
                .map_err(|e| format!("하이브리드 판단 실패: {}", e))?;

            Ok((
                json!({
                    "step_type": "JUDGMENT",
                    "judgment": result.result,
                    "method": result.method_used,
                    "confidence": result.confidence,
                    "explanation": result.explanation
                }),
                input_data.clone(),
            ))
        }
        _ => Err(format!("지원하지 않는 판단 방식: {}", judgment_method))
    }
}

/// APPROVAL 스텝 실행
///
/// 시뮬레이션 모드 vs 실제 모드:
/// - 시뮬레이션: 항상 즉시 승인 처리 (테스트용)
/// - 실제: DB에 승인 요청 저장 후 대기 상태 반환
async fn execute_approval_step(
    step: &WorkflowStep,
    input_data: &serde_json::Value,
) -> Result<(serde_json::Value, serde_json::Value), String> {
    let config = &step.config;
    let approval_type = config["approvalType"].as_str().unwrap_or("manual");

    // 시뮬레이션 모드 체크 (기본값: true = 시뮬레이션)
    let is_simulation = config["isSimulation"].as_bool().unwrap_or(true);

    // 워크플로우 정보 (실제 승인 요청 생성시 사용)
    let workflow_id = config["workflowId"].as_str().unwrap_or("unknown");
    let workflow_name = config["workflowName"].as_str().unwrap_or("Unknown Workflow");

    match approval_type {
        "auto" => {
            // 자동 승인 - 항상 즉시 통과
            Ok((
                json!({
                    "step_type": "APPROVAL",
                    "approval_type": "auto",
                    "approved": true,
                    "message": "자동 승인 완료"
                }),
                input_data.clone(),
            ))
        }
        "conditional" => {
            // 조건부 승인
            let auto_approve_condition = config["autoApproveCondition"].as_str();

            if let Some(condition) = auto_approve_condition {
                // 간단한 조건 평가 (예: "amount < 100000")
                let parts: Vec<&str> = condition.split_whitespace().collect();
                if parts.len() >= 3 {
                    let field_name = parts[0];
                    let operator = parts[1];
                    let threshold = parts[2].parse::<f64>().unwrap_or(0.0);

                    let field_value = input_data[field_name].as_f64().unwrap_or(0.0);

                    let auto_approved = match operator {
                        ">" => field_value > threshold,
                        ">=" => field_value >= threshold,
                        "<" => field_value < threshold,
                        "<=" => field_value <= threshold,
                        "==" | "=" => (field_value - threshold).abs() < 0.0001,
                        "!=" => (field_value - threshold).abs() >= 0.0001,
                        _ => false,
                    };

                    if auto_approved {
                        // 조건 충족 → 자동 승인
                        Ok((
                            json!({
                                "step_type": "APPROVAL",
                                "approval_type": "conditional",
                                "approved": true,
                                "auto_approved": true,
                                "condition": condition,
                                "message": format!("조건 충족으로 자동 승인: {}", condition)
                            }),
                            input_data.clone(),
                        ))
                    } else if is_simulation {
                        // 시뮬레이션 모드: 조건 미충족이어도 자동 승인
                        let approvers = config["approvers"]
                            .as_str()
                            .unwrap_or("admin@example.com")
                            .to_string();

                        Ok((
                            json!({
                                "step_type": "APPROVAL",
                                "approval_type": "conditional",
                                "approved": true,
                                "auto_approved": false,
                                "approvers": approvers,
                                "condition": condition,
                                "is_simulation": true,
                                "message": format!("조건 미충족 → 수동 승인 처리 (시뮬레이션 모드): {}", condition)
                            }),
                            input_data.clone(),
                        ))
                    } else {
                        // 실제 모드: DB에 승인 요청 생성
                        let approval_request = create_approval_request(
                            workflow_id,
                            workflow_name,
                            step,
                            input_data,
                            "conditional",
                            Some(condition),
                        )?;

                        Ok((
                            json!({
                                "step_type": "APPROVAL",
                                "approval_type": "conditional",
                                "approved": false,
                                "pending": true,
                                "request_id": approval_request.id,
                                "approvers": approval_request.approvers,
                                "condition": condition,
                                "timeout_minutes": approval_request.timeout_minutes,
                                "expires_at": approval_request.expires_at,
                                "message": format!("조건 미충족 → 승인 대기 중 (ID: {})", approval_request.id)
                            }),
                            input_data.clone(),
                        ))
                    }
                } else {
                    Err("조건 형식 오류 (예: amount < 100000)".to_string())
                }
            } else {
                Err("autoApproveCondition 필드 필요".to_string())
            }
        }
        "manual" => {
            // 수동 승인
            let approvers = config["approvers"]
                .as_str()
                .unwrap_or("admin@example.com")
                .to_string();
            let timeout_minutes = config["timeoutMinutes"].as_u64().unwrap_or(60);

            if is_simulation {
                // 시뮬레이션 모드: 항상 즉시 승인
                Ok((
                    json!({
                        "step_type": "APPROVAL",
                        "approval_type": "manual",
                        "approved": true,
                        "approvers": approvers,
                        "timeout_minutes": timeout_minutes,
                        "is_simulation": true,
                        "message": format!("수동 승인 대기 중 (시뮬레이션: 자동 승인) - 승인자: {}", approvers)
                    }),
                    input_data.clone(),
                ))
            } else {
                // 실제 모드: DB에 승인 요청 생성
                let approval_request = create_approval_request(
                    workflow_id,
                    workflow_name,
                    step,
                    input_data,
                    "manual",
                    None,
                )?;

                Ok((
                    json!({
                        "step_type": "APPROVAL",
                        "approval_type": "manual",
                        "approved": false,
                        "pending": true,
                        "request_id": approval_request.id,
                        "approvers": approval_request.approvers,
                        "timeout_minutes": approval_request.timeout_minutes,
                        "expires_at": approval_request.expires_at,
                        "message": format!("승인 대기 중 (ID: {}) - 승인자: {}", approval_request.id, approval_request.approvers)
                    }),
                    input_data.clone(),
                ))
            }
        }
        _ => Err(format!("지원하지 않는 승인 타입: {}", approval_type)),
    }
}

/// ALERT 스텝 실행
async fn execute_alert_step(
    step: &WorkflowStep,
    input_data: &serde_json::Value,
) -> Result<(serde_json::Value, serde_json::Value), String> {
    let config = &step.config;
    let channels = config["channels"].as_array().map(|arr| {
        arr.iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
    }).unwrap_or_else(|| vec!["email".to_string()]);

    let recipients = config["recipients"].as_str().unwrap_or("admin@example.com");
    let subject = config["subject"].as_str().unwrap_or("알림");
    let message_template = config["messageTemplate"].as_str().unwrap_or("워크플로우 알림");
    let priority = config["priority"].as_str().unwrap_or("medium");
    let include_data = config["includeData"].as_bool().unwrap_or(false);

    // 메시지 템플릿에 변수 치환 (예: {equipment_id} → EQ-001)
    let mut message = message_template.to_string();
    if let Some(obj) = input_data.as_object() {
        for (key, value) in obj {
            let placeholder = format!("{{{}}}", key);
            if message.contains(&placeholder) {
                let replacement = match value {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    _ => format!("{}", value),
                };
                message = message.replace(&placeholder, &replacement);
            }
        }
    }

    // 실제 발송 로직 (Slack Webhook, Email SMTP Proxy, Notion API)
    eprintln!("📧 ALERT 발송:");
    eprintln!("  채널: {}", channels.join(", "));
    eprintln!("  수신자: {}", recipients);
    eprintln!("  우선순위: {}", priority);
    eprintln!("  제목: {}", subject);
    eprintln!("  메시지: {}", message);

    if include_data {
        eprintln!("  워크플로우 데이터: {}", serde_json::to_string_pretty(input_data).unwrap_or_default());
    }

    let mut sent_channels = Vec::new();
    let http_client = reqwest::Client::new();

    for channel in &channels {
        match channel.as_str() {
            "email" => {
                // 이메일: 환경변수에서 SMTP 프록시 URL 확인
                let result = match std::env::var("JUDGIFY_EMAIL_WEBHOOK") {
                    Ok(webhook_url) => {
                        send_email_webhook(&http_client, &webhook_url, recipients, subject, &message).await
                    }
                    Err(_) => {
                        eprintln!("  ⚠️ JUDGIFY_EMAIL_WEBHOOK 미설정 - Mock 모드");
                        Ok("mock".to_string())
                    }
                };
                match result {
                    Ok(status) => {
                        eprintln!("  ✅ 이메일 발송: {} → {} ({})", subject, recipients, status);
                        sent_channels.push(json!({"channel": "email", "status": "sent", "recipient": recipients}));
                    }
                    Err(e) => {
                        eprintln!("  ❌ 이메일 발송 실패: {}", e);
                        sent_channels.push(json!({"channel": "email", "status": "failed", "error": e}));
                    }
                }
            }
            "slack" => {
                // Slack: 환경변수에서 Webhook URL 확인
                let result = match std::env::var("JUDGIFY_SLACK_WEBHOOK") {
                    Ok(webhook_url) => {
                        send_slack_webhook(&http_client, &webhook_url, subject, &message, priority).await
                    }
                    Err(_) => {
                        eprintln!("  ⚠️ JUDGIFY_SLACK_WEBHOOK 미설정 - Mock 모드");
                        Ok("mock".to_string())
                    }
                };
                match result {
                    Ok(status) => {
                        eprintln!("  ✅ Slack 발송: {} ({})", message, status);
                        sent_channels.push(json!({"channel": "slack", "status": "sent", "recipient": recipients}));
                    }
                    Err(e) => {
                        eprintln!("  ❌ Slack 발송 실패: {}", e);
                        sent_channels.push(json!({"channel": "slack", "status": "failed", "error": e}));
                    }
                }
            }
            "notion" => {
                // Notion: 환경변수에서 API 키 및 Database ID 확인
                let result = match (std::env::var("NOTION_API_KEY"), std::env::var("NOTION_DATABASE_ID")) {
                    (Ok(api_key), Ok(db_id)) => {
                        send_notion_page(&http_client, &api_key, &db_id, subject, &message, priority).await
                    }
                    _ => {
                        eprintln!("  ⚠️ NOTION_API_KEY/NOTION_DATABASE_ID 미설정 - Mock 모드");
                        Ok("mock".to_string())
                    }
                };
                match result {
                    Ok(status) => {
                        eprintln!("  ✅ Notion 발송: {} ({})", message, status);
                        sent_channels.push(json!({"channel": "notion", "status": "sent", "recipient": recipients}));
                    }
                    Err(e) => {
                        eprintln!("  ❌ Notion 발송 실패: {}", e);
                        sent_channels.push(json!({"channel": "notion", "status": "failed", "error": e}));
                    }
                }
            }
            _ => {
                eprintln!("  ⚠️  알 수 없는 채널: {}", channel);
            }
        }
    }

    Ok((
        json!({
            "step_type": "ALERT",
            "channels": channels,
            "recipients": recipients,
            "subject": subject,
            "message": message,
            "priority": priority,
            "sent_channels": sent_channels,
            "sent": true,
            "summary": format!("알림 발송 완료 ({}개 채널)", channels.len())
        }),
        input_data.clone(),
    ))
}

// ================== DB 저장 헬퍼 함수 ==================

/// DB 연결 가져오기
fn get_db_connection() -> Result<Connection, String> {
    let app_data_dir = dirs::data_dir()
        .ok_or("AppData 디렉토리를 찾을 수 없습니다")?
        .join("Judgify");

    let db_path = app_data_dir.join("judgify.db");

    Connection::open(&db_path)
        .map_err(|e| format!("DB 연결 실패: {}", e))
}

/// 워크플로우 실행 결과를 DB에 저장
fn save_workflow_execution(
    conn: &Connection,
    workflow_id: &str,
    status: &str,
    steps_executed: &[StepExecutionResult],
    final_result: &serde_json::Value,
    execution_time_ms: u64,
) -> Result<String, String> {
    // JSON 직렬화
    let steps_json = serde_json::to_string(steps_executed)
        .map_err(|e| format!("steps_executed 직렬화 실패: {}", e))?;

    let final_result_json = serde_json::to_string(final_result)
        .map_err(|e| format!("final_result 직렬화 실패: {}", e))?;

    // INSERT 실행
    conn.execute(
        r#"
        INSERT INTO workflow_executions (workflow_id, status, steps_executed, final_result, execution_time_ms)
        VALUES (?1, ?2, ?3, ?4, ?5)
        "#,
        params![workflow_id, status, steps_json, final_result_json, execution_time_ms as i64],
    )
    .map_err(|e| format!("DB 저장 실패: {}", e))?;

    // 생성된 ID 가져오기
    let execution_id = conn.last_insert_rowid().to_string();

    println!("💾 [WorkflowV2] 실행 이력 저장 완료: {}", execution_id);

    Ok(execution_id)
}

// ================== 실행 이력 조회 API ==================

/// 특정 workflow의 실행 이력 목록 조회
#[tauri::command]
pub async fn get_workflow_executions(
    workflow_id: String,
    limit: Option<i64>,
) -> Result<Vec<WorkflowExecutionListItem>, String> {
    let conn = get_db_connection()?;

    let limit_value = limit.unwrap_or(50);

    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, workflow_id, status, execution_time_ms, created_at
            FROM workflow_executions
            WHERE workflow_id = ?1
            ORDER BY created_at DESC
            LIMIT ?2
            "#,
        )
        .map_err(|e| format!("쿼리 준비 실패: {}", e))?;

    let executions = stmt
        .query_map(params![workflow_id, limit_value], |row| {
            let id: i64 = row.get(0)?;
            Ok(WorkflowExecutionListItem {
                id: id.to_string(),
                workflow_id: row.get(1)?,
                status: row.get(2)?,
                execution_time_ms: row.get(3)?,
                created_at: row.get(4)?,
            })
        })
        .map_err(|e| format!("쿼리 실행 실패: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("결과 수집 실패: {}", e))?;

    println!("📋 [WorkflowV2] 실행 이력 조회: {} ({}건)", workflow_id, executions.len());

    Ok(executions)
}

/// 특정 실행 이력 상세 조회
#[tauri::command]
pub async fn get_workflow_execution_detail(
    execution_id: String,
) -> Result<WorkflowExecutionDetail, String> {
    let conn = get_db_connection()?;

    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, workflow_id, status, steps_executed, final_result, execution_time_ms, created_at
            FROM workflow_executions
            WHERE id = ?1
            "#,
        )
        .map_err(|e| format!("쿼리 준비 실패: {}", e))?;

    let result = stmt
        .query_row(params![execution_id], |row| {
            let steps_json: String = row.get(3)?;
            let final_result_json: String = row.get(4)?;

            let steps_executed: Vec<StepExecutionResult> = serde_json::from_str(&steps_json)
                .map_err(|e| rusqlite::Error::InvalidQuery)?;

            let final_result: serde_json::Value = serde_json::from_str(&final_result_json)
                .map_err(|e| rusqlite::Error::InvalidQuery)?;

            let id: i64 = row.get(0)?;
            Ok(WorkflowExecutionDetail {
                id: id.to_string(),
                workflow_id: row.get(1)?,
                status: row.get(2)?,
                steps_executed,
                final_result,
                execution_time_ms: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => "실행 이력을 찾을 수 없습니다".to_string(),
            _ => format!("조회 실패: {}", e),
        })?;

    println!("🔍 [WorkflowV2] 실행 이력 상세 조회: {}", execution_id);

    Ok(result)
}

// ================== QUERY 노드 헬퍼 함수 ==================

/// 기본 쿼리 실행 (judgments 테이블 조회)
fn execute_default_query(conn: &Connection) -> Result<serde_json::Value, String> {
    let mut stmt = conn
        .prepare("SELECT id, workflow_id, input_data, result, confidence, method_used, explanation, created_at FROM judgments ORDER BY created_at DESC LIMIT 10")
        .map_err(|e| format!("쿼리 준비 실패: {}", e))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(json!({
                "id": row.get::<_, String>(0)?,
                "workflow_id": row.get::<_, String>(1)?,
                "input_data": row.get::<_, String>(2)?,
                "result": row.get::<_, i32>(3)?,
                "confidence": row.get::<_, f64>(4)?,
                "method_used": row.get::<_, String>(5)?,
                "explanation": row.get::<_, Option<String>>(6)?,
                "created_at": row.get::<_, String>(7)?
            }))
        })
        .map_err(|e| format!("쿼리 실행 실패: {}", e))?;

    let results: Vec<serde_json::Value> = rows
        .filter_map(|r| r.ok())
        .collect();

    Ok(json!(results))
}

/// 사용자 지정 쿼리 실행 (SELECT만 허용)
fn execute_custom_query(conn: &Connection, query: &str) -> Result<serde_json::Value, String> {
    // 보안: SELECT 문만 허용
    let query_upper = query.trim().to_uppercase();
    if !query_upper.starts_with("SELECT") {
        return Err("보안상 SELECT 쿼리만 허용됩니다".to_string());
    }

    // 위험한 키워드 차단
    let dangerous_keywords = ["DROP", "DELETE", "UPDATE", "INSERT", "ALTER", "CREATE", "TRUNCATE"];
    for keyword in dangerous_keywords {
        if query_upper.contains(keyword) {
            return Err(format!("보안상 {} 키워드는 허용되지 않습니다", keyword));
        }
    }

    let mut stmt = conn
        .prepare(query)
        .map_err(|e| format!("쿼리 준비 실패: {}", e))?;

    // 컬럼 정보 가져오기
    let column_count = stmt.column_count();
    let column_names: Vec<String> = (0..column_count)
        .map(|i| stmt.column_name(i).unwrap_or("unknown").to_string())
        .collect();

    let rows = stmt
        .query_map([], |row| {
            let mut obj = serde_json::Map::new();
            for (i, col_name) in column_names.iter().enumerate() {
                // 타입 추론하여 적절한 JSON 값으로 변환
                let value: serde_json::Value = match row.get_ref(i) {
                    Ok(rusqlite::types::ValueRef::Null) => serde_json::Value::Null,
                    Ok(rusqlite::types::ValueRef::Integer(i)) => json!(i),
                    Ok(rusqlite::types::ValueRef::Real(f)) => json!(f),
                    Ok(rusqlite::types::ValueRef::Text(t)) => {
                        json!(String::from_utf8_lossy(t).to_string())
                    }
                    Ok(rusqlite::types::ValueRef::Blob(b)) => {
                        json!(format!("[BLOB: {} bytes]", b.len()))
                    }
                    Err(_) => serde_json::Value::Null,
                };
                obj.insert(col_name.clone(), value);
            }
            Ok(serde_json::Value::Object(obj))
        })
        .map_err(|e| format!("쿼리 실행 실패: {}", e))?;

    let results: Vec<serde_json::Value> = rows
        .filter_map(|r| r.ok())
        .collect();

    Ok(json!(results))
}

// ================== ALERT 노드 발송 헬퍼 함수 ==================

/// Slack Webhook으로 메시지 발송
async fn send_slack_webhook(
    client: &reqwest::Client,
    webhook_url: &str,
    title: &str,
    message: &str,
    priority: &str,
) -> Result<String, String> {
    let emoji = match priority {
        "high" => "🚨",
        "medium" => "⚠️",
        "low" => "ℹ️",
        _ => "📌",
    };

    let payload = json!({
        "blocks": [
            {
                "type": "header",
                "text": {"type": "plain_text", "text": format!("{} {}", emoji, title)}
            },
            {
                "type": "section",
                "text": {"type": "mrkdwn", "text": message}
            },
            {
                "type": "context",
                "elements": [
                    {"type": "mrkdwn", "text": format!("*Priority:* {} | *From:* Judgify Workflow", priority)}
                ]
            }
        ]
    });

    let response = client
        .post(webhook_url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Slack 요청 실패: {}", e))?;

    if response.status().is_success() {
        Ok("sent".to_string())
    } else {
        Err(format!("Slack 응답 오류: {}", response.status()))
    }
}

/// 이메일 Webhook (SendGrid, Mailgun 등 호환)
async fn send_email_webhook(
    client: &reqwest::Client,
    webhook_url: &str,
    to: &str,
    subject: &str,
    body: &str,
) -> Result<String, String> {
    let payload = json!({
        "to": to,
        "subject": subject,
        "body": body,
        "from": "noreply@judgify.app"
    });

    let response = client
        .post(webhook_url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Email 요청 실패: {}", e))?;

    if response.status().is_success() {
        Ok("sent".to_string())
    } else {
        Err(format!("Email 응답 오류: {}", response.status()))
    }
}

/// Notion Database에 페이지 생성
async fn send_notion_page(
    client: &reqwest::Client,
    api_key: &str,
    database_id: &str,
    title: &str,
    content: &str,
    priority: &str,
) -> Result<String, String> {
    let payload = json!({
        "parent": {"database_id": database_id},
        "properties": {
            "Name": {"title": [{"text": {"content": title}}]},
            "Priority": {"select": {"name": priority}},
            "Status": {"select": {"name": "New"}}
        },
        "children": [
            {
                "object": "block",
                "type": "paragraph",
                "paragraph": {
                    "rich_text": [{"type": "text", "text": {"content": content}}]
                }
            }
        ]
    });

    let response = client
        .post("https://api.notion.com/v1/pages")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Notion-Version", "2022-06-28")
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Notion 요청 실패: {}", e))?;

    if response.status().is_success() {
        Ok("sent".to_string())
    } else {
        let error_text = response.text().await.unwrap_or_default();
        Err(format!("Notion 응답 오류: {}", error_text))
    }
}

// ============================================================
// APPROVAL 노드 실제 승인 플로우 (Phase 9-3)
// ============================================================

/// 승인 요청 상태
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApprovalRequest {
    pub id: String,
    pub workflow_id: String,
    pub workflow_name: String,
    pub step_id: String,
    pub step_name: String,
    pub approval_type: String,
    pub status: String, // pending, approved, rejected, expired
    pub approvers: String,
    pub input_data: serde_json::Value,
    pub condition: Option<String>,
    pub timeout_minutes: i64,
    pub decided_by: Option<String>,
    pub decided_at: Option<String>,
    pub comment: Option<String>,
    pub created_at: String,
    pub expires_at: Option<String>,
}

/// 승인/거부 요청
#[derive(Debug, Serialize, Deserialize)]
pub struct ApprovalDecision {
    pub request_id: String,
    pub decision: String, // "approved" or "rejected"
    pub decided_by: String,
    pub comment: Option<String>,
}

/// 승인 요청 생성 (내부 헬퍼)
fn create_approval_request(
    workflow_id: &str,
    workflow_name: &str,
    step: &WorkflowStep,
    input_data: &serde_json::Value,
    approval_type: &str,
    condition: Option<&str>,
) -> Result<ApprovalRequest, String> {
    let config = &step.config;
    let approvers = config["approvers"].as_str().unwrap_or("admin@example.com").to_string();
    let timeout_minutes = config["timeoutMinutes"].as_i64().unwrap_or(60);

    let request_id = format!("apr-{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap_or("000"));
    let now = chrono::Utc::now();
    let expires_at = now + chrono::Duration::minutes(timeout_minutes);

    // DB에 승인 요청 저장
    let app_data = std::env::var("APPDATA")
        .or_else(|_| std::env::var("HOME"))
        .map_err(|e| format!("환경변수 오류: {}", e))?;
    let db_path = std::path::PathBuf::from(app_data).join("Judgify").join("judgify.db");

    let conn = Connection::open(&db_path)
        .map_err(|e| format!("DB 연결 실패: {}", e))?;

    conn.execute(
        "INSERT INTO approval_requests (id, workflow_id, workflow_name, step_id, step_name, approval_type, status, approvers, input_data, condition, timeout_minutes, created_at, expires_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'pending', ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            &request_id,
            workflow_id,
            workflow_name,
            &step.id,
            &step.label,
            approval_type,
            &approvers,
            serde_json::to_string(input_data).unwrap_or_default(),
            condition,
            timeout_minutes,
            now.to_rfc3339(),
            expires_at.to_rfc3339(),
        ],
    ).map_err(|e| format!("승인 요청 저장 실패: {}", e))?;

    println!("📋 [APPROVAL] 승인 요청 생성: {} (만료: {}분)", request_id, timeout_minutes);

    Ok(ApprovalRequest {
        id: request_id,
        workflow_id: workflow_id.to_string(),
        workflow_name: workflow_name.to_string(),
        step_id: step.id.clone(),
        step_name: step.label.clone(),
        approval_type: approval_type.to_string(),
        status: "pending".to_string(),
        approvers,
        input_data: input_data.clone(),
        condition: condition.map(|s| s.to_string()),
        timeout_minutes,
        decided_by: None,
        decided_at: None,
        comment: None,
        created_at: now.to_rfc3339(),
        expires_at: Some(expires_at.to_rfc3339()),
    })
}

/// 대기 중인 승인 요청 목록 조회
#[tauri::command]
pub async fn get_pending_approvals() -> Result<Vec<ApprovalRequest>, String> {
    println!("📋 [APPROVAL] 대기 중인 승인 요청 조회");

    let app_data = std::env::var("APPDATA")
        .or_else(|_| std::env::var("HOME"))
        .map_err(|e| format!("환경변수 오류: {}", e))?;
    let db_path = std::path::PathBuf::from(app_data).join("Judgify").join("judgify.db");

    let conn = Connection::open(&db_path)
        .map_err(|e| format!("DB 연결 실패: {}", e))?;

    // 만료된 요청 자동 처리
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE approval_requests SET status = 'expired' WHERE status = 'pending' AND expires_at < ?1",
        params![&now],
    ).map_err(|e| format!("만료 처리 실패: {}", e))?;

    // 대기 중인 요청 조회
    let mut stmt = conn.prepare(
        "SELECT id, workflow_id, workflow_name, step_id, step_name, approval_type, status, approvers, input_data, condition, timeout_minutes, decided_by, decided_at, comment, created_at, expires_at
         FROM approval_requests WHERE status = 'pending' ORDER BY created_at DESC"
    ).map_err(|e| format!("쿼리 준비 실패: {}", e))?;

    let requests = stmt.query_map([], |row| {
        let input_data_str: String = row.get(8)?;
        let input_data: serde_json::Value = serde_json::from_str(&input_data_str).unwrap_or(json!({}));

        Ok(ApprovalRequest {
            id: row.get(0)?,
            workflow_id: row.get(1)?,
            workflow_name: row.get(2)?,
            step_id: row.get(3)?,
            step_name: row.get(4)?,
            approval_type: row.get(5)?,
            status: row.get(6)?,
            approvers: row.get(7)?,
            input_data,
            condition: row.get(9)?,
            timeout_minutes: row.get(10)?,
            decided_by: row.get(11)?,
            decided_at: row.get(12)?,
            comment: row.get(13)?,
            created_at: row.get(14)?,
            expires_at: row.get(15)?,
        })
    }).map_err(|e| format!("쿼리 실행 실패: {}", e))?;

    let result: Vec<ApprovalRequest> = requests.filter_map(|r| r.ok()).collect();
    println!("📋 [APPROVAL] 대기 중인 요청: {}건", result.len());

    Ok(result)
}

/// 승인/거부 처리
#[tauri::command]
pub async fn process_approval(decision: ApprovalDecision) -> Result<serde_json::Value, String> {
    println!("📋 [APPROVAL] 승인 처리: {} → {}", decision.request_id, decision.decision);

    if decision.decision != "approved" && decision.decision != "rejected" {
        return Err("decision은 'approved' 또는 'rejected'만 가능합니다".to_string());
    }

    let app_data = std::env::var("APPDATA")
        .or_else(|_| std::env::var("HOME"))
        .map_err(|e| format!("환경변수 오류: {}", e))?;
    let db_path = std::path::PathBuf::from(app_data).join("Judgify").join("judgify.db");

    let conn = Connection::open(&db_path)
        .map_err(|e| format!("DB 연결 실패: {}", e))?;

    let now = chrono::Utc::now().to_rfc3339();

    let affected = conn.execute(
        "UPDATE approval_requests SET status = ?1, decided_by = ?2, decided_at = ?3, comment = ?4 WHERE id = ?5 AND status = 'pending'",
        params![&decision.decision, &decision.decided_by, &now, &decision.comment, &decision.request_id],
    ).map_err(|e| format!("승인 처리 실패: {}", e))?;

    if affected == 0 {
        return Err(format!("승인 요청을 찾을 수 없거나 이미 처리되었습니다: {}", decision.request_id));
    }

    println!("✅ [APPROVAL] 승인 처리 완료: {} by {}", decision.decision, decision.decided_by);

    Ok(json!({
        "request_id": decision.request_id,
        "decision": decision.decision,
        "decided_by": decision.decided_by,
        "decided_at": now,
        "message": format!("승인 요청이 {}되었습니다", if decision.decision == "approved" { "승인" } else { "거부" })
    }))
}

/// 승인 요청 상세 조회
#[tauri::command]
pub async fn get_approval_request(request_id: String) -> Result<ApprovalRequest, String> {
    println!("📋 [APPROVAL] 승인 요청 상세 조회: {}", request_id);

    let app_data = std::env::var("APPDATA")
        .or_else(|_| std::env::var("HOME"))
        .map_err(|e| format!("환경변수 오류: {}", e))?;
    let db_path = std::path::PathBuf::from(app_data).join("Judgify").join("judgify.db");

    let conn = Connection::open(&db_path)
        .map_err(|e| format!("DB 연결 실패: {}", e))?;

    let mut stmt = conn.prepare(
        "SELECT id, workflow_id, workflow_name, step_id, step_name, approval_type, status, approvers, input_data, condition, timeout_minutes, decided_by, decided_at, comment, created_at, expires_at
         FROM approval_requests WHERE id = ?1"
    ).map_err(|e| format!("쿼리 준비 실패: {}", e))?;

    stmt.query_row(params![&request_id], |row| {
        let input_data_str: String = row.get(8)?;
        let input_data: serde_json::Value = serde_json::from_str(&input_data_str).unwrap_or(json!({}));

        Ok(ApprovalRequest {
            id: row.get(0)?,
            workflow_id: row.get(1)?,
            workflow_name: row.get(2)?,
            step_id: row.get(3)?,
            step_name: row.get(4)?,
            approval_type: row.get(5)?,
            status: row.get(6)?,
            approvers: row.get(7)?,
            input_data,
            condition: row.get(9)?,
            timeout_minutes: row.get(10)?,
            decided_by: row.get(11)?,
            decided_at: row.get(12)?,
            comment: row.get(13)?,
            created_at: row.get(14)?,
            expires_at: row.get(15)?,
        })
    }).map_err(|e| format!("승인 요청을 찾을 수 없습니다: {}", e))
}

// ============================================================
// 워크플로우 스케줄러 (Phase 9-4: Cron-based Scheduler)
// ============================================================

/// 스케줄 설정
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkflowSchedule {
    pub id: String,
    pub workflow_id: String,
    pub workflow_name: String,
    pub cron_expression: String,
    pub timezone: String,
    pub is_active: bool,
    pub input_data: serde_json::Value,
    pub last_run_at: Option<String>,
    pub next_run_at: Option<String>,
    pub run_count: i64,
    pub last_status: Option<String>,
    pub last_error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// 스케줄 생성 요청
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateScheduleRequest {
    pub workflow_id: String,
    pub workflow_name: String,
    pub cron_expression: String,
    pub timezone: Option<String>,
    pub input_data: Option<serde_json::Value>,
}

/// Row를 WorkflowSchedule로 변환하는 헬퍼
fn row_to_schedule(row: &rusqlite::Row) -> Result<WorkflowSchedule, rusqlite::Error> {
    let input_data_str: String = row.get(6)?;
    let input_data: serde_json::Value = serde_json::from_str(&input_data_str).unwrap_or(json!({}));
    Ok(WorkflowSchedule {
        id: row.get(0)?,
        workflow_id: row.get(1)?,
        workflow_name: row.get(2)?,
        cron_expression: row.get(3)?,
        timezone: row.get(4)?,
        is_active: row.get::<_, i32>(5)? != 0,
        input_data,
        last_run_at: row.get(7)?,
        next_run_at: row.get(8)?,
        run_count: row.get(9)?,
        last_status: row.get(10)?,
        last_error: row.get(11)?,
        created_at: row.get(12)?,
        updated_at: row.get(13)?,
    })
}

/// 스케줄 목록 조회
#[tauri::command]
pub async fn get_workflow_schedules(
    workflow_id: Option<String>,
    active_only: Option<bool>,
) -> Result<Vec<WorkflowSchedule>, String> {
    println!("📅 [SCHEDULER] 스케줄 목록 조회");

    let conn = get_db_connection()?;
    let active_filter = active_only.unwrap_or(false);

    let mut result: Vec<WorkflowSchedule> = Vec::new();

    if let Some(wf_id) = workflow_id {
        let query = if active_filter {
            "SELECT id, workflow_id, workflow_name, cron_expression, timezone, is_active, input_data, last_run_at, next_run_at, run_count, last_status, last_error, created_at, updated_at FROM workflow_schedules WHERE workflow_id = ?1 AND is_active = 1 ORDER BY created_at DESC"
        } else {
            "SELECT id, workflow_id, workflow_name, cron_expression, timezone, is_active, input_data, last_run_at, next_run_at, run_count, last_status, last_error, created_at, updated_at FROM workflow_schedules WHERE workflow_id = ?1 ORDER BY created_at DESC"
        };
        let mut stmt = conn.prepare(query).map_err(|e| format!("쿼리 준비 실패: {}", e))?;
        let schedules = stmt.query_map(params![wf_id], row_to_schedule)
            .map_err(|e| format!("쿼리 실행 실패: {}", e))?;
        result = schedules.filter_map(|r| r.ok()).collect();
    } else {
        let query = if active_filter {
            "SELECT id, workflow_id, workflow_name, cron_expression, timezone, is_active, input_data, last_run_at, next_run_at, run_count, last_status, last_error, created_at, updated_at FROM workflow_schedules WHERE is_active = 1 ORDER BY created_at DESC"
        } else {
            "SELECT id, workflow_id, workflow_name, cron_expression, timezone, is_active, input_data, last_run_at, next_run_at, run_count, last_status, last_error, created_at, updated_at FROM workflow_schedules ORDER BY created_at DESC"
        };
        let mut stmt = conn.prepare(query).map_err(|e| format!("쿼리 준비 실패: {}", e))?;
        let schedules = stmt.query_map([], row_to_schedule)
            .map_err(|e| format!("쿼리 실행 실패: {}", e))?;
        result = schedules.filter_map(|r| r.ok()).collect();
    }

    println!("📅 [SCHEDULER] 조회된 스케줄: {}건", result.len());
    Ok(result)
}

/// 스케줄 생성
#[tauri::command]
pub async fn create_workflow_schedule(
    request: CreateScheduleRequest,
) -> Result<WorkflowSchedule, String> {
    println!("📅 [SCHEDULER] 스케줄 생성: {} ({})", request.workflow_name, request.cron_expression);

    // Cron 표현식 유효성 검사
    use cron::Schedule;
    use std::str::FromStr;

    let _schedule = Schedule::from_str(&request.cron_expression)
        .map_err(|e| format!("잘못된 Cron 표현식: {} - {}", request.cron_expression, e))?;

    let schedule_id = format!("sch-{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap_or("000"));
    let timezone = request.timezone.unwrap_or_else(|| "Asia/Seoul".to_string());
    let input_data = request.input_data.unwrap_or(json!({}));
    let now = chrono::Utc::now().to_rfc3339();

    // 다음 실행 시간 계산
    let next_run = _schedule.upcoming(chrono::Utc).next()
        .map(|dt| dt.to_rfc3339());

    let conn = get_db_connection()?;

    conn.execute(
        "INSERT INTO workflow_schedules (id, workflow_id, workflow_name, cron_expression, timezone, is_active, input_data, next_run_at, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?7, ?8, ?8)",
        params![
            &schedule_id,
            &request.workflow_id,
            &request.workflow_name,
            &request.cron_expression,
            &timezone,
            &serde_json::to_string(&input_data).unwrap_or_default(),
            &next_run,
            &now
        ],
    ).map_err(|e| format!("스케줄 생성 실패: {}", e))?;

    println!("✅ [SCHEDULER] 스케줄 생성 완료: {} (다음 실행: {:?})", schedule_id, next_run);

    Ok(WorkflowSchedule {
        id: schedule_id,
        workflow_id: request.workflow_id,
        workflow_name: request.workflow_name,
        cron_expression: request.cron_expression,
        timezone,
        is_active: true,
        input_data,
        last_run_at: None,
        next_run_at: next_run,
        run_count: 0,
        last_status: None,
        last_error: None,
        created_at: now.clone(),
        updated_at: now,
    })
}

/// 스케줄 활성화/비활성화 토글
#[tauri::command]
pub async fn toggle_workflow_schedule(
    schedule_id: String,
    is_active: bool,
) -> Result<serde_json::Value, String> {
    println!("📅 [SCHEDULER] 스케줄 토글: {} → {}", schedule_id, if is_active { "활성화" } else { "비활성화" });

    let conn = get_db_connection()?;
    let now = chrono::Utc::now().to_rfc3339();

    let affected = conn.execute(
        "UPDATE workflow_schedules SET is_active = ?1, updated_at = ?2 WHERE id = ?3",
        params![is_active as i32, &now, &schedule_id],
    ).map_err(|e| format!("스케줄 업데이트 실패: {}", e))?;

    if affected == 0 {
        return Err(format!("스케줄을 찾을 수 없습니다: {}", schedule_id));
    }

    Ok(json!({
        "schedule_id": schedule_id,
        "is_active": is_active,
        "message": format!("스케줄이 {}되었습니다", if is_active { "활성화" } else { "비활성화" })
    }))
}

/// 스케줄 삭제
#[tauri::command]
pub async fn delete_workflow_schedule(schedule_id: String) -> Result<serde_json::Value, String> {
    println!("📅 [SCHEDULER] 스케줄 삭제: {}", schedule_id);

    let conn = get_db_connection()?;

    let affected = conn.execute(
        "DELETE FROM workflow_schedules WHERE id = ?1",
        params![&schedule_id],
    ).map_err(|e| format!("스케줄 삭제 실패: {}", e))?;

    if affected == 0 {
        return Err(format!("스케줄을 찾을 수 없습니다: {}", schedule_id));
    }

    println!("✅ [SCHEDULER] 스케줄 삭제 완료: {}", schedule_id);

    Ok(json!({
        "schedule_id": schedule_id,
        "message": "스케줄이 삭제되었습니다"
    }))
}

/// Cron 표현식 유효성 검사 및 다음 실행 시간 미리보기
#[tauri::command]
pub async fn validate_cron_expression(
    cron_expression: String,
    count: Option<usize>,
) -> Result<serde_json::Value, String> {
    use cron::Schedule;
    use std::str::FromStr;

    let schedule = Schedule::from_str(&cron_expression)
        .map_err(|e| format!("잘못된 Cron 표현식: {}", e))?;

    let count = count.unwrap_or(5);
    let upcoming: Vec<String> = schedule
        .upcoming(chrono::Utc)
        .take(count)
        .map(|dt| dt.to_rfc3339())
        .collect();

    Ok(json!({
        "valid": true,
        "expression": cron_expression,
        "next_runs": upcoming,
        "message": format!("유효한 Cron 표현식입니다. 다음 {}회 실행 예정", count)
    }))
}

/// 스케줄 실행 기록 업데이트 (내부용)
fn update_schedule_run_status(
    conn: &Connection,
    schedule_id: &str,
    status: &str,
    error: Option<&str>,
) -> Result<(), String> {
    use cron::Schedule;
    use std::str::FromStr;

    let now = chrono::Utc::now().to_rfc3339();

    // 현재 스케줄의 cron expression 가져오기
    let cron_expr: String = conn.query_row(
        "SELECT cron_expression FROM workflow_schedules WHERE id = ?1",
        params![schedule_id],
        |row| row.get(0),
    ).map_err(|e| format!("스케줄 조회 실패: {}", e))?;

    // 다음 실행 시간 계산
    let next_run = Schedule::from_str(&cron_expr)
        .ok()
        .and_then(|s| s.upcoming(chrono::Utc).next())
        .map(|dt| dt.to_rfc3339());

    conn.execute(
        "UPDATE workflow_schedules SET last_run_at = ?1, last_status = ?2, last_error = ?3, next_run_at = ?4, run_count = run_count + 1, updated_at = ?1 WHERE id = ?5",
        params![&now, status, error, &next_run, schedule_id],
    ).map_err(|e| format!("스케줄 상태 업데이트 실패: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_metadata_serialization() {
        let metadata = WorkflowMetadata {
            name: "테스트 워크플로우".to_string(),
            description: "테스트 설명".to_string(),
            is_active: true,
        };

        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains("isActive")); // camelCase 확인
        assert!(json.contains("테스트 워크플로우"));
    }

    #[test]
    fn test_workflow_step_serialization() {
        let step = WorkflowStep {
            id: "step-1".to_string(),
            step_type: "TRIGGER".to_string(),
            label: "트리거 스텝".to_string(),
            config: json!({
                "triggerType": "threshold",
                "condition": "temperature > 90",
                "threshold": 90
            }),
        };

        let json = serde_json::to_string(&step).unwrap();
        assert!(json.contains("\"type\":\"TRIGGER\"")); // type 필드 확인
        assert!(json.contains("triggerType"));
    }

    #[tokio::test]
    async fn test_execute_trigger_step() {
        let step = WorkflowStep {
            id: "step-1".to_string(),
            step_type: "TRIGGER".to_string(),
            label: "트리거".to_string(),
            config: json!({}),
        };

        let input = json!({"test": true});
        let result = execute_trigger_step(&step, &input).await;

        assert!(result.is_ok());
        let (output, _) = result.unwrap();
        assert_eq!(output["step_type"], "TRIGGER");
        assert_eq!(output["triggered"], true);
    }

    #[tokio::test]
    async fn test_execute_judgment_step_with_rule() {
        let step = WorkflowStep {
            id: "step-judgment".to_string(),
            step_type: "JUDGMENT".to_string(),
            label: "AI 판단".to_string(),
            config: json!({
                "judgmentMethod": "rule",
                "ruleExpression": "temperature > 90"
            }),
        };

        let input = json!({"temperature": 95});
        let result = execute_judgment_step(&step, &input).await;

        assert!(result.is_ok());
        let (output, _) = result.unwrap();
        assert_eq!(output["step_type"], "JUDGMENT");
        assert_eq!(output["judgment"], true);
        assert_eq!(output["method"], "rule");
        assert_eq!(output["confidence"], 1.0);
    }

    #[tokio::test]
    async fn test_execute_judgment_step_rule_missing() {
        // Rule 모드인데 ruleExpression이 없으면 에러
        let step = WorkflowStep {
            id: "step-judgment".to_string(),
            step_type: "JUDGMENT".to_string(),
            label: "AI 판단".to_string(),
            config: json!({
                "judgmentMethod": "rule"
                // ruleExpression 누락
            }),
        };

        let input = json!({"temperature": 95});
        let result = execute_judgment_step(&step, &input).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Rule 표현식이 설정되지 않았습니다"));
    }

    // Phase 4: LLM/Hybrid 모드 통합 테스트는 Mock API 환경에서 별도 테스트 필요
    // (실제 Claude API 호출 대신 Mock LLM Engine 사용)
    //
    // TODO: 다음 단계에서 구현
    // - test_execute_judgment_step_llm_mode()
    // - test_execute_judgment_step_hybrid_rule_success()
    // - test_execute_judgment_step_hybrid_llm_fallback()

    #[tokio::test]
    async fn test_e2e_workflow_6_nodetypes() {
        // Phase 4-1: 6개 NodeType End-to-End 워크플로우 테스트
        // TRIGGER → QUERY → CALC → JUDGMENT → APPROVAL → ALERT

        let request = SimulateWorkflowRequest {
            workflow_id: "e2e-test-workflow".to_string(),
            steps: vec![
                WorkflowStep {
                    id: "step-1-trigger".to_string(),
                    step_type: "TRIGGER".to_string(),
                    label: "온도 임계값 트리거".to_string(),
                    config: json!({
                        "triggerType": "threshold",
                        "condition": "temperature > 90",
                    "threshold": 90.0
                    }),
                },
                WorkflowStep {
                    id: "step-2-query".to_string(),
                    step_type: "QUERY".to_string(),
                    label: "설비 데이터 조회".to_string(),
                    config: json!({
                        "queryType": "database",
                        "tableName": "equipment_status"
                    }),
                },
                WorkflowStep {
                    id: "step-3-calc".to_string(),
                    step_type: "CALC".to_string(),
                    label: "평균 온도 계산".to_string(),
                    config: json!({
                        "calcType": "aggregate",
                        "aggregateFunction": "avg",
                        "targetField": "temperature",
                        "outputField": "avg_temperature"
                    }),
                },
                WorkflowStep {
                    id: "step-4-judgment".to_string(),
                    step_type: "JUDGMENT".to_string(),
                    label: "고온 이상 판단".to_string(),
                    config: json!({
                        "judgmentMethod": "rule",
                        "ruleExpression": "avg_temperature > 85"
                    }),
                },
                WorkflowStep {
                    id: "step-5-approval".to_string(),
                    step_type: "APPROVAL".to_string(),
                    label: "자동 승인".to_string(),
                    config: json!({
                        "approvalType": "auto"
                    }),
                },
                WorkflowStep {
                    id: "step-6-alert".to_string(),
                    step_type: "ALERT".to_string(),
                    label: "Slack 알림".to_string(),
                    config: json!({
                        "channels": ["slack"],
                        "recipients": ["#alerts"],
                        "message": "고온 이상 감지: {avg_temperature}도"
                    }),
                },
            ],
            test_data: json!({
                "temperature": 95,
                "equipment_id": "EQ-001"
            }),
        };

        let result = simulate_workflow_v2(request).await;

        assert!(result.is_ok());
        let response = result.unwrap();

        // 전체 워크플로우 실행 성공 확인
        assert_eq!(response.status, "success");

        // 6개 스텝 모두 실행되었는지 확인
        assert_eq!(response.steps_executed.len(), 6);

        // 각 스텝 타입 확인 (Option<JsonValue>이므로 unwrap 필요)
        let output_0 = response.steps_executed[0].output.as_ref().unwrap();
        let output_1 = response.steps_executed[1].output.as_ref().unwrap();
        let output_2 = response.steps_executed[2].output.as_ref().unwrap();
        let output_3 = response.steps_executed[3].output.as_ref().unwrap();
        let output_4 = response.steps_executed[4].output.as_ref().unwrap();
        let output_5 = response.steps_executed[5].output.as_ref().unwrap();

        assert_eq!(output_0["step_type"], "TRIGGER");
        assert_eq!(output_1["step_type"], "QUERY");
        assert_eq!(output_2["step_type"], "CALC");
        assert_eq!(output_3["step_type"], "JUDGMENT");
        assert_eq!(output_4["step_type"], "APPROVAL");
        assert_eq!(output_5["step_type"], "ALERT");

        // TRIGGER 성공 확인
        assert_eq!(output_0["triggered"], true);

        // CALC 집계 결과 확인
        assert!(output_2["result"].is_number());

        // JUDGMENT 판단 결과 확인
        assert!(output_3["judgment"].is_boolean());
        assert_eq!(output_3["method"], "rule");

        // APPROVAL 승인 확인
        assert_eq!(output_4["approved"], true);

        // ALERT 발송 확인
        assert_eq!(output_5["sent"], true);

        // 실행 시간 확인 (0ms일 수도 있음 - 빠른 실행)
        assert!(response.total_execution_time_ms >= 0);

        println!("✅ E2E 워크플로우 테스트 성공!");
        println!("  - 총 실행 시간: {}ms", response.total_execution_time_ms);
        println!("  - 최종 상태: {}", response.status);
    }

    #[tokio::test]
    async fn test_get_workflow_executions() {
        // E2E 테스트 먼저 실행 (DB에 데이터 생성)
        let request = SimulateWorkflowRequest {
            workflow_id: "test-history-workflow".to_string(),
            steps: vec![
                WorkflowStep {
                    id: "trigger-1".to_string(),
                    step_type: "TRIGGER".to_string(),
                    label: "테스트 트리거".to_string(),
                    config: json!({
                        "triggerType": "manual"
                    }),
                },
            ],
            test_data: json!({"test": "data"}),
        };

        let result = simulate_workflow_v2(request).await;
        assert!(result.is_ok());

        // 실행 이력 조회
        let executions = get_workflow_executions("test-history-workflow".to_string(), Some(10)).await;
        assert!(executions.is_ok());

        let list = executions.unwrap();
        assert!(list.len() > 0);

        println!("✅ 실행 이력 조회 테스트 성공!");
        println!("  - 조회된 이력: {}건", list.len());

        // 상세 조회
        let execution_id = list[0].id.clone();
        let detail = get_workflow_execution_detail(execution_id).await;
        assert!(detail.is_ok());

        let detail_data = detail.unwrap();
        assert_eq!(detail_data.workflow_id, "test-history-workflow");
        assert_eq!(detail_data.status, "success");

        println!("✅ 실행 이력 상세 조회 테스트 성공!");
        println!("  - 스텝 개수: {}", detail_data.steps_executed.len());
    }

    #[tokio::test]
    async fn test_query_step_database() {
        let step = WorkflowStep {
            id: "query-1".to_string(),
            step_type: "QUERY".to_string(),
            label: "데이터베이스 조회".to_string(),
            config: json!({
                "dataSource": "database",
                "queryType": "SELECT",
                "query": "SELECT * FROM judgments LIMIT 5"
            }),
        };

        let input_data = json!({"test": "data"});

        let result = execute_query_step(&step, &input_data).await;
        assert!(result.is_ok());

        let (output, updated_data) = result.unwrap();
        assert_eq!(output["step_type"], "QUERY");
        assert_eq!(output["data_source"], "database");
        assert!(output["data"].is_array());
        assert!(updated_data["query_result"].is_array());

        println!("✅ QUERY (database) 유닛 테스트 성공!");
    }

    #[tokio::test]
    async fn test_query_step_api() {
        let step = WorkflowStep {
            id: "query-2".to_string(),
            step_type: "QUERY".to_string(),
            label: "API 호출".to_string(),
            config: json!({
                "dataSource": "api",
                "query": "https://api.example.com/sensors/SENS-001"
            }),
        };

        let input_data = json!({"test": "data"});

        let result = execute_query_step(&step, &input_data).await;
        assert!(result.is_ok());

        let (output, updated_data) = result.unwrap();
        assert_eq!(output["step_type"], "QUERY");
        assert_eq!(output["data_source"], "api");
        assert_eq!(output["response"]["status"], "success");
        assert!(updated_data["api_response"]["data"]["readings"].is_array());

        println!("✅ QUERY (api) 유닛 테스트 성공!");
    }

    #[tokio::test]
    async fn test_query_step_sensor() {
        let step = WorkflowStep {
            id: "query-3".to_string(),
            step_type: "QUERY".to_string(),
            label: "센서 데이터 수집".to_string(),
            config: json!({
                "dataSource": "sensor",
                "sensorId": "SENS-001"
            }),
        };

        let input_data = json!({"test": "data"});

        let result = execute_query_step(&step, &input_data).await;
        assert!(result.is_ok());

        let (output, updated_data) = result.unwrap();
        assert_eq!(output["step_type"], "QUERY");
        assert_eq!(output["data_source"], "sensor");
        assert!(output["sensor_data"]["temperature"].is_number());
        assert!(updated_data["sensor_data"]["vibration"].is_number());

        println!("✅ QUERY (sensor) 유닛 테스트 성공!");
    }

    #[tokio::test]
    async fn test_query_step_file() {
        let step = WorkflowStep {
            id: "query-4".to_string(),
            step_type: "QUERY".to_string(),
            label: "파일 조회".to_string(),
            config: json!({
                "dataSource": "file",
                "filePath": "/data/production_data.csv"
            }),
        };

        let input_data = json!({"test": "data"});

        let result = execute_query_step(&step, &input_data).await;
        assert!(result.is_ok());

        let (output, updated_data) = result.unwrap();
        assert_eq!(output["step_type"], "QUERY");
        assert_eq!(output["data_source"], "file");
        assert!(output["file_data"]["sample"].is_array());
        assert_eq!(output["file_data"]["rows"], 150);
        assert!(updated_data["file_data"].is_object());

        println!("✅ QUERY (file) 유닛 테스트 성공!");
    }

    #[tokio::test]
    async fn test_query_step_invalid_source() {
        let step = WorkflowStep {
            id: "query-5".to_string(),
            step_type: "QUERY".to_string(),
            label: "잘못된 데이터 소스".to_string(),
            config: json!({
                "dataSource": "invalid_source"
            }),
        };

        let input_data = json!({"test": "data"});

        let result = execute_query_step(&step, &input_data).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("지원하지 않는 데이터 소스"));

        println!("✅ QUERY (invalid source) 에러 처리 테스트 성공!");
    }

    #[tokio::test]
    async fn test_alert_step_email() {
        let step = WorkflowStep {
            id: "alert-1".to_string(),
            step_type: "ALERT".to_string(),
            label: "이메일 알림".to_string(),
            config: json!({
                "channels": ["email"],
                "recipients": "manager@example.com",
                "subject": "긴급: 설비 고장",
                "messageTemplate": "설비 {equipment_id}에서 이상 감지",
                "priority": "high",
                "includeData": true
            }),
        };

        let input_data = json!({
            "equipment_id": "EQ-001",
            "temperature": 95.5
        });

        let result = execute_alert_step(&step, &input_data).await;
        assert!(result.is_ok());

        let (output, _) = result.unwrap();
        assert_eq!(output["step_type"], "ALERT");
        assert!(output["channels"].as_array().unwrap().contains(&json!("email")));
        assert_eq!(output["recipients"], "manager@example.com");
        assert!(output["message"].as_str().unwrap().contains("EQ-001"));

        println!("✅ ALERT (email) 유닛 테스트 성공!");
    }

    #[tokio::test]
    async fn test_alert_step_slack() {
        let step = WorkflowStep {
            id: "alert-2".to_string(),
            step_type: "ALERT".to_string(),
            label: "Slack 알림".to_string(),
            config: json!({
                "channels": ["slack"],
                "recipients": "#production-alerts",
                "subject": "품질 경고",
                "messageTemplate": "불량률 {defect_rate}% 초과",
                "priority": "medium"
            }),
        };

        let input_data = json!({
            "defect_rate": 7.5
        });

        let result = execute_alert_step(&step, &input_data).await;
        assert!(result.is_ok());

        let (output, _) = result.unwrap();
        assert_eq!(output["step_type"], "ALERT");
        assert!(output["channels"].as_array().unwrap().contains(&json!("slack")));
        assert!(output["message"].as_str().unwrap().contains("7.5"));

        println!("✅ ALERT (slack) 유닛 테스트 성공!");
    }

    #[tokio::test]
    async fn test_alert_step_teams() {
        let step = WorkflowStep {
            id: "alert-3".to_string(),
            step_type: "ALERT".to_string(),
            label: "Teams 알림".to_string(),
            config: json!({
                "channels": ["teams"],
                "recipients": "Production Team",
                "subject": "생산 지연",
                "messageTemplate": "라인 {line_id}에서 {delay_minutes}분 지연",
                "priority": "low"
            }),
        };

        let input_data = json!({
            "line_id": "LINE-A",
            "delay_minutes": 15
        });

        let result = execute_alert_step(&step, &input_data).await;
        assert!(result.is_ok());

        let (output, _) = result.unwrap();
        assert_eq!(output["step_type"], "ALERT");
        assert!(output["channels"].as_array().unwrap().contains(&json!("teams")));
        assert!(output["message"].as_str().unwrap().contains("LINE-A"));
        assert!(output["message"].as_str().unwrap().contains("15"));

        println!("✅ ALERT (teams) 유닛 테스트 성공!");
    }

    #[tokio::test]
    async fn test_alert_step_webhook() {
        let step = WorkflowStep {
            id: "alert-4".to_string(),
            step_type: "ALERT".to_string(),
            label: "Webhook 알림".to_string(),
            config: json!({
                "channels": ["webhook"],
                "recipients": "https://example.com/webhook",
                "subject": "시스템 알림",
                "messageTemplate": "이벤트 발생: {event_type}",
                "priority": "high"
            }),
        };

        let input_data = json!({
            "event_type": "EQUIPMENT_FAILURE"
        });

        let result = execute_alert_step(&step, &input_data).await;
        assert!(result.is_ok());

        let (output, _) = result.unwrap();
        assert_eq!(output["step_type"], "ALERT");
        assert!(output["channels"].as_array().unwrap().contains(&json!("webhook")));
        assert!(output["message"].as_str().unwrap().contains("EQUIPMENT_FAILURE"));

        println!("✅ ALERT (webhook) 유닛 테스트 성공!");
    }

    #[tokio::test]
    async fn test_alert_step_multiple_channels() {
        let step = WorkflowStep {
            id: "alert-5".to_string(),
            step_type: "ALERT".to_string(),
            label: "다중 채널 알림".to_string(),
            config: json!({
                "channels": ["email", "slack", "teams"],
                "recipients": "admin@example.com",
                "subject": "긴급 알림",
                "messageTemplate": "다중 채널 테스트",
                "priority": "high"
            }),
        };

        let input_data = json!({"test": "data"});

        let result = execute_alert_step(&step, &input_data).await;
        assert!(result.is_ok());

        let (output, _) = result.unwrap();
        assert_eq!(output["step_type"], "ALERT");
        let channels = output["channels"].as_array().unwrap();
        assert_eq!(channels.len(), 3);
        assert!(channels.contains(&json!("email")));
        assert!(channels.contains(&json!("slack")));
        assert!(channels.contains(&json!("teams")));

        println!("✅ ALERT (multiple channels) 유닛 테스트 성공!");
    }

    // ============================================================================
    // Phase 9-2: AI Workflow Generator 테스트
    // ============================================================================

    #[test]
    fn test_system_prompt_contains_all_node_types() {
        // Given: System prompt 생성
        let system_prompt = create_workflow_dsl_prompt();

        // When: 6개 NodeType이 모두 포함되어 있는지 검증
        let expected_types = vec![
            "TRIGGER", "QUERY", "CALC", "JUDGMENT", "APPROVAL", "ALERT"
        ];

        // Then: 모든 NodeType이 시스템 프롬프트에 포함되어야 함
        for node_type in expected_types {
            assert!(
                system_prompt.contains(node_type),
                "System prompt should contain NodeType: {}",
                node_type
            );
        }
        println!("✅ System Prompt NodeType 검증 성공!");
    }

    #[test]
    fn test_system_prompt_contains_few_shot_examples() {
        // Given: System prompt 생성
        let system_prompt = create_workflow_dsl_prompt();

        // When: 5개 Few-shot 예시가 포함되어 있는지 검증
        let expected_examples = vec![
            "불량률 모니터링",      // Example 1
            "설비 가동률 분석",     // Example 2
            "AI 품질 판단",         // Example 3
            "주기적 모니터링",      // Example 4 (실제 prompt 텍스트)
            "다단계 승인 프로세스", // Example 5
        ];

        // Then: 모든 예시가 시스템 프롬프트에 포함되어야 함
        for example in expected_examples {
            assert!(
                system_prompt.contains(example),
                "System prompt should contain example: {}",
                example
            );
        }
        println!("✅ System Prompt Few-shot 예시 검증 성공!");
    }

    #[test]
    fn test_parse_simple_workflow_json() {
        // Given: Claude가 반환한 간단한 워크플로우 JSON
        let json_response = r#"[
            {
                "id": "trigger_1",
                "type": "TRIGGER",
                "label": "불량 감지",
                "config": {
                    "triggerType": "threshold",
                    "metric": "불량률",
                    "condition": "> 3%"
                }
            },
            {
                "id": "alert_1",
                "type": "ALERT",
                "label": "알림 전송",
                "config": {
                    "channel": "slack",
                    "message": "불량률 초과"
                }
            }
        ]"#;

        // When: JSON 파싱
        let result: Result<Vec<WorkflowStep>, _> = serde_json::from_str(json_response);

        // Then: 파싱 성공 및 2개 스텝 확인
        assert!(result.is_ok(), "JSON parsing should succeed");
        let steps = result.unwrap();
        assert_eq!(steps.len(), 2, "Should have 2 steps");
        assert_eq!(steps[0].step_type, "TRIGGER");
        assert_eq!(steps[1].step_type, "ALERT");
        println!("✅ 간단한 워크플로우 JSON 파싱 성공!");
    }

    #[test]
    fn test_parse_complex_workflow_json() {
        // Given: Claude가 반환한 복잡한 워크플로우 JSON (6개 NodeType 모두 포함)
        let json_response = r#"[
            {
                "id": "trigger_1",
                "type": "TRIGGER",
                "label": "매 시간 실행",
                "config": { "cron": "0 * * * *" }
            },
            {
                "id": "query_1",
                "type": "QUERY",
                "label": "불량률 조회",
                "config": { "sql": "SELECT AVG(defect_rate) FROM line_1" }
            },
            {
                "id": "calc_1",
                "type": "CALC",
                "label": "평균 계산",
                "config": { "formula": "SUM(values) / COUNT(values)" }
            },
            {
                "id": "judgment_1",
                "type": "JUDGMENT",
                "label": "판단 실행",
                "config": { "rule": "defect_rate > 3%" }
            },
            {
                "id": "approval_1",
                "type": "APPROVAL",
                "label": "팀장 승인",
                "config": { "approver": "생산팀장" }
            },
            {
                "id": "alert_1",
                "type": "ALERT",
                "label": "알림 전송",
                "config": { "channel": "slack" }
            }
        ]"#;

        // When: JSON 파싱
        let result: Result<Vec<WorkflowStep>, _> = serde_json::from_str(json_response);

        // Then: 파싱 성공 및 6개 NodeType 모두 확인
        assert!(result.is_ok(), "JSON parsing should succeed");
        let steps = result.unwrap();
        assert_eq!(steps.len(), 6, "Should have 6 steps");

        // 각 NodeType 검증
        assert_eq!(steps[0].step_type, "TRIGGER");
        assert_eq!(steps[1].step_type, "QUERY");
        assert_eq!(steps[2].step_type, "CALC");
        assert_eq!(steps[3].step_type, "JUDGMENT");
        assert_eq!(steps[4].step_type, "APPROVAL");
        assert_eq!(steps[5].step_type, "ALERT");
        println!("✅ 복잡한 워크플로우 JSON 파싱 성공 (6개 NodeType)!");
    }

    #[test]
    fn test_parse_invalid_json_should_fail() {
        // Given: 잘못된 JSON (type 필드 누락)
        let invalid_json = r#"[
            {
                "id": "trigger_1",
                "label": "불량 감지",
                "config": {}
            }
        ]"#;

        // When: JSON 파싱 시도
        let result: Result<Vec<WorkflowStep>, _> = serde_json::from_str(invalid_json);

        // Then: 파싱 실패해야 함
        assert!(result.is_err(), "Invalid JSON should fail to parse");
        println!("✅ 잘못된 JSON 파싱 실패 검증 성공!");
    }
}

// ============================================================================
// Phase 9-2: AI Workflow Generator
// ============================================================================

/// AI 워크플로우 생성 (자연어 → WorkflowStep 배열)
///
/// # Arguments
/// * `user_prompt` - 사용자 자연어 입력 (예: "1호선 불량률 3% 초과시 알림")
/// * `app_handle` - Tauri AppHandle (ChatService 초기화용)
///
/// # Returns
/// * `Ok(Vec<WorkflowStep>)` - 생성된 워크플로우 스텝 배열
/// * `Err(String)` - 에러 메시지
///
/// # Example
/// ```rust
/// let steps = generate_workflow_draft(
///     "1호선 불량률 3% 초과시 알림".to_string(),
///     app_handle
/// ).await?;
/// ```
#[tauri::command]
pub async fn generate_workflow_draft(
    user_prompt: String,
    app_handle: tauri::AppHandle,
) -> Result<Vec<WorkflowStep>, String> {
    use crate::services::chat_service::ChatService;

    // 1. ChatService 초기화
    let chat_service = ChatService::with_app_handle(Some(app_handle))
        .map_err(|e| format!("ChatService 초기화 실패: {}", e))?;

    // 2. System Prompt (Manufacturing DSL)
    let system_prompt = create_workflow_dsl_prompt();

    // 3. Claude API 호출
    let response = chat_service
        .generate_workflow_from_prompt(&system_prompt, &user_prompt)
        .await
        .map_err(|e| format!("Claude API 호출 실패: {}", e))?;

    // 4. JSON 파싱 → Vec<WorkflowStep>
    let steps: Vec<WorkflowStep> = serde_json::from_str(&response)
        .map_err(|e| format!("워크플로우 JSON 파싱 실패: {}\n\nReceived: {}", e, response))?;

    // 5. 유효성 검증
    if steps.is_empty() {
        return Err("생성된 워크플로우가 비어있습니다.".to_string());
    }

    Ok(steps)
}

/// Manufacturing DSL System Prompt 생성
///
/// Claude가 한국어 제조업 워크플로우를 생성하도록 가이드하는 프롬프트
fn create_workflow_dsl_prompt() -> String {
    r##"You are a Manufacturing Workflow Architect specializing in Korean smart factory automation.

# Available Node Types (6개):
1. **TRIGGER**: Event-based activation (시간, 센서, Webhook 등)
2. **QUERY**: Data retrieval (DB, API, File 등)
3. **CALC**: Mathematical calculations (통계, 집계 등)
4. **JUDGMENT**: Rule-based or AI-powered decision (하이브리드 판단)
5. **APPROVAL**: Human approval gates (생산팀장, 품질팀장 등)
6. **ALERT**: Notifications (Email, Slack, Teams, Webhook)

# Output Format (JSON Array ONLY - NO MARKDOWN!):
Return ONLY a valid JSON array. Do NOT wrap in markdown code blocks.

[
  {
    "id": "step-{unique-id}",
    "type": "TRIGGER|QUERY|CALC|JUDGMENT|APPROVAL|ALERT",
    "label": "한글 스텝 이름",
    "config": { /* type별 설정 */ }
  }
]

# Rules:
- Always return valid JSON array (no markdown, no explanation)
- Use Korean labels for clarity
- Infer factory/line IDs from context (default: "Plant-A", "L01")
- JUDGMENT rules use structured format: "field operator value" (예: "rate > 3.0")
- ALERT default channel: ["email"] (사용자가 명시하면 slack, teams 추가)
- Each step must have unique ID (step-1, step-2, ...)

# Few-Shot Examples:

## Example 1: 불량률 모니터링
User: "1호선 불량률이 3% 초과하면 알림"
Output:
[
  {
    "id": "step-1",
    "type": "QUERY",
    "label": "1호선 불량률 조회",
    "config": {
      "dataSource": "database",
      "query": "SELECT rate FROM defect_rates WHERE line_id = 'L01' ORDER BY created_at DESC LIMIT 1",
      "queryType": "sql"
    }
  },
  {
    "id": "step-2",
    "type": "JUDGMENT",
    "label": "불량률 3% 초과 판단",
    "config": {
      "judgmentMethod": "rule",
      "ruleExpression": "rate > 3.0"
    }
  },
  {
    "id": "step-3",
    "type": "ALERT",
    "label": "이메일 알림 발송",
    "config": {
      "channels": ["email"],
      "recipients": "production-team@company.com",
      "messageTemplate": "⚠️ 1호선 불량률 {rate}% 초과 발생!"
    }
  }
]

## Example 2: 설비 가동률 분석
User: "A라인 설비 가동률 계산하고 80% 미만이면 팀장 승인 후 알림"
Output:
[
  {
    "id": "step-1",
    "type": "QUERY",
    "label": "A라인 가동 시간 조회",
    "config": {
      "dataSource": "database",
      "query": "SELECT uptime_hours, total_hours FROM equipment_status WHERE line_id = 'A' AND date = CURRENT_DATE",
      "queryType": "sql"
    }
  },
  {
    "id": "step-2",
    "type": "CALC",
    "label": "가동률 계산",
    "config": {
      "formula": "(uptime_hours / total_hours) * 100",
      "outputVariable": "utilization_rate"
    }
  },
  {
    "id": "step-3",
    "type": "JUDGMENT",
    "label": "가동률 80% 미만 판단",
    "config": {
      "judgmentMethod": "rule",
      "ruleExpression": "utilization_rate < 80"
    }
  },
  {
    "id": "step-4",
    "type": "APPROVAL",
    "label": "생산팀장 승인 요청",
    "config": {
      "approvers": ["production-manager@company.com"],
      "approvalType": "single",
      "timeoutMinutes": 30
    }
  },
  {
    "id": "step-5",
    "type": "ALERT",
    "label": "가동률 저하 알림",
    "config": {
      "channels": ["email", "slack"],
      "recipients": "#production-team",
      "messageTemplate": "⚠️ A라인 가동률 {utilization_rate}% (80% 미만)"
    }
  }
]

## Example 3: AI 품질 판단
User: "제품 이미지로 불량 여부 AI 판단"
Output:
[
  {
    "id": "step-1",
    "type": "QUERY",
    "label": "제품 이미지 조회",
    "config": {
      "dataSource": "api",
      "endpoint": "https://api.factory.com/products/latest-image",
      "method": "GET"
    }
  },
  {
    "id": "step-2",
    "type": "JUDGMENT",
    "label": "AI 불량 판단",
    "config": {
      "judgmentMethod": "ai",
      "aiModel": "claude-sonnet-4-5-20250929",
      "prompt": "다음 제품 이미지를 분석하여 불량 여부를 판단하세요. 불량이면 true, 정상이면 false를 반환하세요.",
      "temperature": 0.3
    }
  },
  {
    "id": "step-3",
    "type": "ALERT",
    "label": "불량 감지 알림",
    "config": {
      "channels": ["email"],
      "recipients": "quality-team@company.com",
      "messageTemplate": "🔴 불량 제품 감지! AI 신뢰도: {confidence}%"
    }
  }
]

## Example 4: 주기적 모니터링
User: "매시간 전체 라인 온도 체크"
Output:
[
  {
    "id": "step-1",
    "type": "TRIGGER",
    "label": "매시간 실행 트리거",
    "config": {
      "triggerType": "schedule",
      "schedule": "0 * * * *"
    }
  },
  {
    "id": "step-2",
    "type": "QUERY",
    "label": "전체 라인 온도 조회",
    "config": {
      "dataSource": "database",
      "query": "SELECT line_id, AVG(temperature) as avg_temp FROM sensor_data WHERE timestamp > NOW() - INTERVAL 1 HOUR GROUP BY line_id",
      "queryType": "sql"
    }
  },
  {
    "id": "step-3",
    "type": "JUDGMENT",
    "label": "온도 이상 판단",
    "config": {
      "judgmentMethod": "rule",
      "ruleExpression": "avg_temp > 80 OR avg_temp < 20"
    }
  },
  {
    "id": "step-4",
    "type": "ALERT",
    "label": "온도 이상 알림",
    "config": {
      "channels": ["email"],
      "recipients": "maintenance-team@company.com",
      "messageTemplate": "🌡️ 라인 {line_id} 온도 이상: {avg_temp}°C"
    }
  }
]

## Example 5: 다단계 승인 프로세스
User: "재고 부족시 구매 요청 → 팀장 승인 → 구매팀 알림"
Output:
[
  {
    "id": "step-1",
    "type": "QUERY",
    "label": "재고 수량 조회",
    "config": {
      "dataSource": "database",
      "query": "SELECT item_name, quantity, min_threshold FROM inventory WHERE quantity < min_threshold",
      "queryType": "sql"
    }
  },
  {
    "id": "step-2",
    "type": "JUDGMENT",
    "label": "재고 부족 판단",
    "config": {
      "judgmentMethod": "rule",
      "ruleExpression": "quantity < min_threshold"
    }
  },
  {
    "id": "step-3",
    "type": "APPROVAL",
    "label": "구매팀장 승인 요청",
    "config": {
      "approvers": ["purchase-manager@company.com"],
      "approvalType": "single",
      "timeoutMinutes": 60,
      "requireComment": true
    }
  },
  {
    "id": "step-4",
    "type": "ALERT",
    "label": "구매팀 알림",
    "config": {
      "channels": ["email", "slack"],
      "recipients": "#purchase-team",
      "messageTemplate": "📦 재고 부족 구매 승인됨: {item_name} (현재 {quantity}개)"
    }
  }
]

Now, generate a workflow based on the user's request."##.to_string()
}
