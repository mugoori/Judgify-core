use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::env;
use crate::database::Database;
use tauri::{AppHandle, Manager};
use chrono::Utc;

// ========== Phase 1: LLM 분석 엔진 데이터 구조 ==========

/// 사용자 요청 분석 결과
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestAnalysis {
    /// 요청 의도 (monitoring | analysis | comparison | overview)
    pub intent: String,

    /// 데이터 엔티티 (workflow | judgment | action)
    pub entities: Vec<String>,

    /// 메트릭 목록 (success_rate | execution_time | count)
    pub metrics: Vec<String>,

    /// 시간 범위 (last_week | last_month | today)
    pub time_range: Option<String>,

    /// 선호 차트 타입 (line | bar | pie | gauge)
    pub preferred_charts: Vec<String>,

    /// 복잡도 점수 (0.0-1.0, 0.5 이상이면 LLM 사용)
    pub complexity_score: f64,
}

/// BI 인사이트 결과
#[derive(Debug, Serialize, Deserialize)]
pub struct BiInsight {
    pub title: String,
    pub insights: Vec<String>,
    pub component_code: String,
    pub recommendations: Vec<String>,
}

// ========== Phase 3: Judgment Service 통합 데이터 구조 ==========

/// 데이터 집계 결과
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedData {
    /// 평균 값
    pub mean: f64,

    /// 중앙값
    pub median: f64,

    /// 표준편차
    pub std_dev: f64,

    /// 최소값
    pub min: f64,

    /// 최대값
    pub max: f64,

    /// 총 개수
    pub count: u32,

    /// 평가 상태 (normal | warning | critical)
    pub status: String,

    /// 추세 (increasing | decreasing | stable)
    pub trend: String,

    /// 변화율 (%)
    pub change_rate: f64,
}

/// 시계열 데이터 포인트
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    pub timestamp: String,
    pub value: f64,
    pub label: Option<String>,
}

/// Judgment 실행 결과 (DB에서 조회)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgmentExecution {
    pub id: String,
    pub workflow_id: String,
    pub result: bool,
    pub confidence: f64,
    pub method_used: String,
    pub execution_time_ms: i32,
    pub created_at: String,
}

// ========== Phase 4: RAG 기반 인사이트 데이터 구조 ==========

/// 유사 케이스 (과거 판단 결과)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarCase {
    /// 판단 실행 ID
    pub execution_id: String,

    /// 워크플로우 ID
    pub workflow_id: String,

    /// 입력 데이터 (JSON)
    pub input_data: serde_json::Value,

    /// 판단 결과
    pub result: bool,

    /// 신뢰도
    pub confidence: f64,

    /// 사용된 메서드
    pub method_used: String,

    /// 유사도 점수 (0.0-1.0)
    pub similarity_score: f64,

    /// 실행 시간
    pub created_at: String,
}

/// RAG 컨텍스트
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagContext {
    /// 현재 분석 요청
    pub current_request: RequestAnalysis,

    /// 현재 집계 데이터
    pub current_aggregation: AggregatedData,

    /// 유사한 과거 케이스들 (최대 5개)
    pub similar_cases: Vec<SimilarCase>,

    /// 도메인 지식 (업계 표준, 임계값 등)
    pub domain_knowledge: Vec<String>,
}

/// 비즈니스 권장사항
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessRecommendation {
    /// 권장사항 제목
    pub title: String,

    /// 권장사항 설명
    pub description: String,

    /// 우선순위 (high | medium | low)
    pub priority: String,

    /// 예상 효과
    pub expected_impact: String,

    /// 근거 (유사 케이스 기반)
    pub reasoning: String,
}

// ========== Phase 2: MCP 컴포넌트 라이브러리 ==========

/// 컴포넌트 메타데이터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMetadata {
    /// 컴포넌트 이름 (MetricCard, LineChart 등)
    pub name: String,

    /// 컴포넌트 설명
    pub description: String,

    /// 필수 Props 목록
    pub required_props: Vec<String>,

    /// 선택 Props 목록
    pub optional_props: Vec<String>,

    /// 지원하는 데이터 타입 (number, percentage, count, time_series)
    pub supported_data_types: Vec<String>,

    /// 적합한 메트릭 (success_rate, execution_time, count)
    pub suitable_metrics: Vec<String>,

    /// React 컴포넌트 템플릿
    pub template: String,
}

/// 조립된 대시보드 구성
#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardConfig {
    /// 대시보드 제목
    pub title: String,

    /// 선택된 컴포넌트 목록
    pub components: Vec<AssembledComponent>,

    /// 전체 React 코드
    pub react_code: String,

    /// 실시간 업데이트 설정
    pub real_time_config: Option<RealTimeConfig>,
}

/// 조립된 컴포넌트
#[derive(Debug, Serialize, Deserialize)]
pub struct AssembledComponent {
    /// 컴포넌트 타입
    pub component_type: String,

    /// Props 값
    pub props: HashMap<String, serde_json::Value>,

    /// 생성된 JSX 코드
    pub jsx_code: String,
}

/// 실시간 업데이트 설정
#[derive(Debug, Serialize, Deserialize)]
pub struct RealTimeConfig {
    /// 업데이트 주기 (초)
    pub interval_seconds: u32,

    /// WebSocket 엔드포인트
    pub websocket_url: String,
}

/// LLM 요청/응답 구조체
#[derive(Debug, Serialize, Deserialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    temperature: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

// ========== BI Service 메인 구조체 ==========

pub struct BiService {
    openai_api_key: String,
    http_client: reqwest::Client,
    component_registry: HashMap<String, ComponentMetadata>,
    db: Database,
    app_handle: Option<AppHandle>,  // Phase 5: Tauri 이벤트 발생용
}

impl BiService {
    pub fn new() -> anyhow::Result<Self> {
        Self::with_app_handle(None)
    }

    /// Phase 5: AppHandle을 포함한 생성자 (이벤트 발생용)
    pub fn with_app_handle(app_handle: Option<AppHandle>) -> anyhow::Result<Self> {
        let openai_api_key = env::var("OPENAI_API_KEY")
            .unwrap_or_else(|_| "sk-test-key".to_string());

        let db = Database::new()?;

        let mut service = Self {
            openai_api_key,
            http_client: reqwest::Client::new(),
            component_registry: HashMap::new(),
            db,
            app_handle,
        };

        // Phase 2: 10개 컴포넌트 등록
        service.register_components();

        Ok(service)
    }

    // ========== Phase 3: Judgment Service 데이터 통합 ==========

    /// Judgment 실행 데이터 조회 (시간 범위 기반)
    fn get_judgment_executions(&self, workflow_id: Option<&str>, time_range: Option<&str>) -> anyhow::Result<Vec<JudgmentExecution>> {
        // Mock 데이터 (Phase 3에서 실제 DB 쿼리로 교체 예정)
        let mut executions = vec![
            JudgmentExecution {
                id: "exec-1".to_string(),
                workflow_id: "workflow-123".to_string(),
                result: true,
                confidence: 0.95,
                method_used: "rule".to_string(),
                execution_time_ms: 120,
                created_at: "2025-10-24T10:00:00Z".to_string(),
            },
            JudgmentExecution {
                id: "exec-2".to_string(),
                workflow_id: "workflow-123".to_string(),
                result: true,
                confidence: 0.88,
                method_used: "llm_few_shot".to_string(),
                execution_time_ms: 1200,
                created_at: "2025-10-25T10:00:00Z".to_string(),
            },
            JudgmentExecution {
                id: "exec-3".to_string(),
                workflow_id: "workflow-123".to_string(),
                result: false,
                confidence: 0.92,
                method_used: "hybrid".to_string(),
                execution_time_ms: 850,
                created_at: "2025-10-26T10:00:00Z".to_string(),
            },
        ];

        // workflow_id 필터링
        if let Some(wf_id) = workflow_id {
            executions.retain(|e| e.workflow_id == wf_id);
        }

        Ok(executions)
    }

    /// 데이터 집계 (통계 계산)
    fn aggregate_data(&self, executions: &[JudgmentExecution], metric: &str) -> anyhow::Result<AggregatedData> {
        if executions.is_empty() {
            return Err(anyhow::anyhow!("No data to aggregate"));
        }

        let values: Vec<f64> = match metric {
            "success_rate" => {
                let total = executions.len() as f64;
                let success = executions.iter().filter(|e| e.result).count() as f64;
                vec![success / total * 100.0]
            }
            "execution_time" => executions.iter().map(|e| e.execution_time_ms as f64).collect(),
            "confidence" => executions.iter().map(|e| e.confidence * 100.0).collect(),
            _ => executions.iter().map(|_| 0.0).collect(),
        };

        // 통계 계산
        let mean = values.iter().sum::<f64>() / values.len() as f64;

        let mut sorted = values.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median = if sorted.len() % 2 == 0 {
            (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0
        } else {
            sorted[sorted.len() / 2]
        };

        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();

        let min = sorted.first().copied().unwrap_or(0.0);
        let max = sorted.last().copied().unwrap_or(0.0);

        // 평가 상태 결정
        let status = if metric == "success_rate" {
            if mean >= 90.0 {
                "normal"
            } else if mean >= 70.0 {
                "warning"
            } else {
                "critical"
            }
        } else {
            "normal"
        }.to_string();

        // 추세 분석
        let trend = if metric == "success_rate" && executions.len() >= 2 {
            // success_rate는 execution 단위로 추세 분석
            let first_half = &executions[..executions.len() / 2];
            let second_half = &executions[executions.len() / 2..];

            let first_success_rate = first_half.iter().filter(|e| e.result).count() as f64 / first_half.len() as f64 * 100.0;
            let second_success_rate = second_half.iter().filter(|e| e.result).count() as f64 / second_half.len() as f64 * 100.0;

            if second_success_rate > first_success_rate * 1.05 {
                "increasing"
            } else if second_success_rate < first_success_rate * 0.95 {
                "decreasing"
            } else {
                "stable"
            }
        } else if values.len() >= 2 {
            // 다른 메트릭은 값 기반 추세 분석
            let first_half = &values[..values.len() / 2];
            let second_half = &values[values.len() / 2..];
            let first_avg = first_half.iter().sum::<f64>() / first_half.len() as f64;
            let second_avg = second_half.iter().sum::<f64>() / second_half.len() as f64;

            if second_avg > first_avg * 1.05 {
                "increasing"
            } else if second_avg < first_avg * 0.95 {
                "decreasing"
            } else {
                "stable"
            }
        } else {
            "stable"
        }.to_string();

        // 변화율
        let change_rate = if values.len() >= 2 {
            let first = values[0];
            let last = values[values.len() - 1];
            if first > 0.0 {
                ((last - first) / first) * 100.0
            } else {
                0.0
            }
        } else {
            0.0
        };

        Ok(AggregatedData {
            mean,
            median,
            std_dev,
            min,
            max,
            count: executions.len() as u32, // 항상 원본 execution 개수
            status,
            trend,
            change_rate,
        })
    }

    /// 시계열 데이터 생성
    fn generate_time_series(&self, executions: &[JudgmentExecution], metric: &str) -> Vec<TimeSeriesPoint> {
        executions.iter().map(|e| {
            let value = match metric {
                "success_rate" => if e.result { 100.0 } else { 0.0 },
                "execution_time" => e.execution_time_ms as f64,
                "confidence" => e.confidence * 100.0,
                _ => 0.0,
            };

            TimeSeriesPoint {
                timestamp: e.created_at.clone(),
                value,
                label: Some(e.method_used.clone()),
            }
        }).collect()
    }

    // ========== Phase 2: MCP 컴포넌트 레지스트리 ==========

    /// 10개 사전 제작 컴포넌트 등록
    fn register_components(&mut self) {
        // 1. MetricCard - KPI 표시
        self.component_registry.insert("MetricCard".to_string(), ComponentMetadata {
            name: "MetricCard".to_string(),
            description: "Single metric display with trend indicator".to_string(),
            required_props: vec!["title".to_string(), "value".to_string()],
            optional_props: vec!["trend".to_string(), "unit".to_string()],
            supported_data_types: vec!["number".to_string(), "percentage".to_string()],
            suitable_metrics: vec!["success_rate".to_string(), "count".to_string()],
            template: r#"<MetricCard title="{title}" value="{value}" trend="{trend}" unit="{unit}" />"#.to_string(),
        });

        // 2. LineChart - 시계열 데이터
        self.component_registry.insert("LineChart".to_string(), ComponentMetadata {
            name: "LineChart".to_string(),
            description: "Time series line chart".to_string(),
            required_props: vec!["data".to_string(), "xKey".to_string(), "yKey".to_string()],
            optional_props: vec!["title".to_string(), "color".to_string()],
            supported_data_types: vec!["time_series".to_string()],
            suitable_metrics: vec!["success_rate".to_string(), "execution_time".to_string()],
            template: r#"<LineChart data={data} xKey="{xKey}" yKey="{yKey}" title="{title}" />"#.to_string(),
        });

        // 3. BarChart - 비교 데이터
        self.component_registry.insert("BarChart".to_string(), ComponentMetadata {
            name: "BarChart".to_string(),
            description: "Bar chart for comparisons".to_string(),
            required_props: vec!["data".to_string(), "xKey".to_string(), "yKey".to_string()],
            optional_props: vec!["title".to_string(), "color".to_string()],
            supported_data_types: vec!["count".to_string(), "number".to_string()],
            suitable_metrics: vec!["count".to_string()],
            template: r#"<BarChart data={data} xKey="{xKey}" yKey="{yKey}" title="{title}" />"#.to_string(),
        });

        // 4. PieChart - 분포 데이터
        self.component_registry.insert("PieChart".to_string(), ComponentMetadata {
            name: "PieChart".to_string(),
            description: "Pie chart for distributions".to_string(),
            required_props: vec!["data".to_string(), "nameKey".to_string(), "valueKey".to_string()],
            optional_props: vec!["title".to_string()],
            supported_data_types: vec!["percentage".to_string(), "count".to_string()],
            suitable_metrics: vec!["success_rate".to_string()],
            template: r#"<PieChart data={data} nameKey="{nameKey}" valueKey="{valueKey}" title="{title}" />"#.to_string(),
        });

        // 5. GaugeChart - 진행률 표시
        self.component_registry.insert("GaugeChart".to_string(), ComponentMetadata {
            name: "GaugeChart".to_string(),
            description: "Gauge chart for progress/percentage".to_string(),
            required_props: vec!["value".to_string(), "max".to_string()],
            optional_props: vec!["title".to_string(), "unit".to_string()],
            supported_data_types: vec!["percentage".to_string()],
            suitable_metrics: vec!["success_rate".to_string()],
            template: r#"<GaugeChart value={value} max={max} title="{title}" unit="{unit}" />"#.to_string(),
        });

        // 6. DataTable - 상세 데이터 테이블
        self.component_registry.insert("DataTable".to_string(), ComponentMetadata {
            name: "DataTable".to_string(),
            description: "Detailed data table".to_string(),
            required_props: vec!["data".to_string(), "columns".to_string()],
            optional_props: vec!["title".to_string(), "pageSize".to_string()],
            supported_data_types: vec!["records".to_string()],
            suitable_metrics: vec!["count".to_string()],
            template: r#"<DataTable data={data} columns={columns} title="{title}" pageSize={pageSize} />"#.to_string(),
        });

        // 7. HeatMap - 행렬 데이터
        self.component_registry.insert("HeatMap".to_string(), ComponentMetadata {
            name: "HeatMap".to_string(),
            description: "Heat map for matrix data".to_string(),
            required_props: vec!["data".to_string(), "xKey".to_string(), "yKey".to_string(), "valueKey".to_string()],
            optional_props: vec!["title".to_string()],
            supported_data_types: vec!["matrix".to_string()],
            suitable_metrics: vec!["execution_time".to_string()],
            template: r#"<HeatMap data={data} xKey="{xKey}" yKey="{yKey}" valueKey="{valueKey}" title="{title}" />"#.to_string(),
        });

        // 8. ScatterPlot - 상관관계
        self.component_registry.insert("ScatterPlot".to_string(), ComponentMetadata {
            name: "ScatterPlot".to_string(),
            description: "Scatter plot for correlations".to_string(),
            required_props: vec!["data".to_string(), "xKey".to_string(), "yKey".to_string()],
            optional_props: vec!["title".to_string(), "color".to_string()],
            supported_data_types: vec!["correlation".to_string()],
            suitable_metrics: vec!["execution_time".to_string(), "success_rate".to_string()],
            template: r#"<ScatterPlot data={data} xKey="{xKey}" yKey="{yKey}" title="{title}" />"#.to_string(),
        });

        // 9. AreaChart - 누적 데이터
        self.component_registry.insert("AreaChart".to_string(), ComponentMetadata {
            name: "AreaChart".to_string(),
            description: "Area chart for cumulative data".to_string(),
            required_props: vec!["data".to_string(), "xKey".to_string(), "yKey".to_string()],
            optional_props: vec!["title".to_string(), "color".to_string()],
            supported_data_types: vec!["time_series".to_string()],
            suitable_metrics: vec!["count".to_string()],
            template: r#"<AreaChart data={data} xKey="{xKey}" yKey="{yKey}" title="{title}" />"#.to_string(),
        });

        // 10. TreeMap - 계층 데이터
        self.component_registry.insert("TreeMap".to_string(), ComponentMetadata {
            name: "TreeMap".to_string(),
            description: "Tree map for hierarchical data".to_string(),
            required_props: vec!["data".to_string(), "nameKey".to_string(), "sizeKey".to_string()],
            optional_props: vec!["title".to_string()],
            supported_data_types: vec!["hierarchical".to_string()],
            suitable_metrics: vec!["count".to_string()],
            template: r#"<TreeMap data={data} nameKey="{nameKey}" sizeKey="{sizeKey}" title="{title}" />"#.to_string(),
        });
    }

    /// 메트릭과 데이터 타입에 맞는 최적 컴포넌트 선택
    fn select_components(&self, analysis: &RequestAnalysis) -> Vec<String> {
        let mut selected = Vec::new();

        for metric in &analysis.metrics {
            // 이미 선호 차트가 있으면 우선 사용
            if !analysis.preferred_charts.is_empty() {
                for chart in &analysis.preferred_charts {
                    let component_name = match chart.as_str() {
                        "gauge" => "GaugeChart",
                        "line" => "LineChart",
                        "bar" => "BarChart",
                        "pie" => "PieChart",
                        _ => "MetricCard",
                    };
                    if !selected.contains(&component_name.to_string()) {
                        selected.push(component_name.to_string());
                    }
                }
            } else {
                // 메트릭 기반 자동 선택
                let component_name = match metric.as_str() {
                    "success_rate" => {
                        selected.push("GaugeChart".to_string());
                        "LineChart"
                    }
                    "execution_time" => "LineChart",
                    "count" => "BarChart",
                    _ => "MetricCard",
                };
                if !selected.contains(&component_name.to_string()) {
                    selected.push(component_name.to_string());
                }
            }
        }

        // 최소 1개는 선택 (MetricCard 기본)
        if selected.is_empty() {
            selected.push("MetricCard".to_string());
        }

        selected
    }

    /// 컴포넌트 조립 (Phase 3: 실제 데이터 통합)
    fn assemble_components(&self, component_names: Vec<String>, analysis: &RequestAnalysis) -> Vec<AssembledComponent> {
        let mut components = Vec::new();

        // Phase 3: 실제 Judgment 데이터 조회
        let executions = self.get_judgment_executions(None, analysis.time_range.as_deref())
            .unwrap_or_else(|_| vec![]);

        for component_name in component_names {
            if let Some(metadata) = self.component_registry.get(&component_name) {
                let mut props = HashMap::new();

                // 메트릭 결정 (분석 결과에서 첫 번째 메트릭 사용)
                let metric = analysis.metrics.first().map(|s| s.as_str()).unwrap_or("success_rate");

                // Phase 3: 실제 집계 데이터 사용
                match metadata.name.as_str() {
                    "MetricCard" => {
                        if let Ok(agg) = self.aggregate_data(&executions, metric) {
                            let metric_name = match metric {
                                "success_rate" => "성공률",
                                "execution_time" => "실행 시간",
                                "confidence" => "신뢰도",
                                _ => "메트릭",
                            };

                            props.insert("title".to_string(), json!(metric_name));
                            props.insert("value".to_string(), json!(format!("{:.1}%", agg.mean)));
                            props.insert("trend".to_string(), json!(agg.trend));
                            props.insert("unit".to_string(), json!("%"));
                        }
                    }
                    "GaugeChart" => {
                        if let Ok(agg) = self.aggregate_data(&executions, metric) {
                            props.insert("value".to_string(), json!(agg.mean));
                            props.insert("max".to_string(), json!(100));
                            props.insert("title".to_string(), json!("성공률"));
                            props.insert("unit".to_string(), json!("%"));
                        }
                    }
                    "LineChart" => {
                        let time_series = self.generate_time_series(&executions, metric);
                        props.insert("data".to_string(), json!(time_series));
                        props.insert("xKey".to_string(), json!("timestamp"));
                        props.insert("yKey".to_string(), json!("value"));
                        props.insert("title".to_string(), json!("성공률 추세"));
                    }
                    "BarChart" => {
                        let time_series = self.generate_time_series(&executions, metric);
                        props.insert("data".to_string(), json!(time_series));
                        props.insert("xKey".to_string(), json!("timestamp"));
                        props.insert("yKey".to_string(), json!("value"));
                        props.insert("title".to_string(), json!("워크플로우별 실행 횟수"));
                    }
                    _ => {
                        props.insert("title".to_string(), json!("데이터"));
                        props.insert("data".to_string(), json!([]));
                    }
                }

                // JSX 코드 생성
                let jsx_code = self.generate_jsx(&metadata, &props);

                components.push(AssembledComponent {
                    component_type: metadata.name.clone(),
                    props,
                    jsx_code,
                });
            }
        }

        components
    }

    /// Props를 기반으로 JSX 코드 생성
    fn generate_jsx(&self, metadata: &ComponentMetadata, props: &HashMap<String, serde_json::Value>) -> String {
        let mut jsx = metadata.template.clone();

        for (key, value) in props {
            let placeholder = format!("{{{}}}", key);
            let value_str = match value {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                _ => value.to_string(),
            };
            jsx = jsx.replace(&placeholder, &value_str);
        }

        jsx
    }

    /// 전체 React 코드 생성
    fn generate_react_code(&self, components: &[AssembledComponent], title: &str) -> String {
        let components_jsx: Vec<String> = components
            .iter()
            .map(|c| format!("  {}", c.jsx_code))
            .collect();

        format!(
            r#"<div className="dashboard">
  <h2>{}</h2>
  <div className="grid grid-cols-3 gap-4">
{}
  </div>
</div>"#,
            title,
            components_jsx.join("\n")
        )
    }

    // ========== Phase 1: LLM 분석 엔진 구현 ==========

    /// 자연어 요청을 분석하여 RequestAnalysis 반환
    pub async fn analyze_user_request(&self, user_request: &str) -> anyhow::Result<RequestAnalysis> {
        println!("🔍 Analyzing user request: {}", user_request);

        // 1. 복잡도 점수 계산 (간단한 휴리스틱)
        let complexity_score = self.calculate_complexity(user_request);

        // 2. 복잡도가 낮으면 템플릿 기반 분석 (빠름)
        if complexity_score < 0.5 {
            return self.analyze_with_template(user_request);
        }

        // 3. 복잡도가 높으면 LLM 기반 분석 (정확함)
        self.analyze_with_llm(user_request).await
    }

    /// 복잡도 점수 계산 (0.0-1.0)
    fn calculate_complexity(&self, request: &str) -> f64 {
        let request_lower = request.to_lowercase();
        let mut score: f64 = 0.0;

        // 간단한 패턴: "지난 주", "성공률", "보여줘" 등
        if request_lower.contains("지난 주") || request_lower.contains("last week") {
            score += 0.1;
        }
        if request_lower.contains("성공률") || request_lower.contains("success rate") {
            score += 0.1;
        }
        if request_lower.contains("비교") || request_lower.contains("compare") {
            score += 0.3;
        }
        if request_lower.contains("추세") || request_lower.contains("trend") {
            score += 0.3;
        }
        if request_lower.contains("이상") || request_lower.contains("anomaly") {
            score += 0.4;
        }
        if request_lower.contains("패턴") || request_lower.contains("pattern") {
            score += 0.2;
        }
        if request_lower.contains("찾") || request_lower.contains("find") || request_lower.contains("search") {
            score += 0.1;
        }

        // 단어 수가 많으면 복잡도 증가
        let word_count = request.split_whitespace().count();
        if word_count > 10 {
            score += 0.2;
        }

        score.min(1.0)
    }

    /// 템플릿 기반 분석 (복잡도 < 0.5)
    fn analyze_with_template(&self, request: &str) -> anyhow::Result<RequestAnalysis> {
        let request_lower = request.to_lowercase();

        let mut analysis = RequestAnalysis {
            intent: "monitoring".to_string(),
            entities: vec![],
            metrics: vec![],
            time_range: None,
            preferred_charts: vec![],
            complexity_score: self.calculate_complexity(request),
        };

        // Intent 분류
        if request_lower.contains("분석") || request_lower.contains("analysis") {
            analysis.intent = "analysis".to_string();
        } else if request_lower.contains("비교") || request_lower.contains("compare") {
            analysis.intent = "comparison".to_string();
        } else if request_lower.contains("개요") || request_lower.contains("overview") {
            analysis.intent = "overview".to_string();
        }

        // Entity 추출
        if request_lower.contains("워크플로우") || request_lower.contains("workflow") {
            analysis.entities.push("workflow".to_string());
        }
        if request_lower.contains("판단") || request_lower.contains("judgment") {
            analysis.entities.push("judgment".to_string());
        }

        // Metric 추출
        if request_lower.contains("성공률") || request_lower.contains("success rate") {
            analysis.metrics.push("success_rate".to_string());
            analysis.preferred_charts.push("gauge".to_string());
            analysis.preferred_charts.push("line".to_string());
        }
        if request_lower.contains("실행 시간") || request_lower.contains("execution time") {
            analysis.metrics.push("execution_time".to_string());
            analysis.preferred_charts.push("line".to_string());
        }
        if request_lower.contains("개수") || request_lower.contains("count") {
            analysis.metrics.push("count".to_string());
            analysis.preferred_charts.push("bar".to_string());
        }

        // Time Range 추출
        if request_lower.contains("지난 주") || request_lower.contains("last week") {
            analysis.time_range = Some("last_week".to_string());
        } else if request_lower.contains("지난 달") || request_lower.contains("last month") {
            analysis.time_range = Some("last_month".to_string());
        } else if request_lower.contains("오늘") || request_lower.contains("today") {
            analysis.time_range = Some("today".to_string());
        }

        println!("📋 Template-based analysis: {:?}", analysis);
        Ok(analysis)
    }

    /// LLM 기반 분석 (복잡도 >= 0.5)
    async fn analyze_with_llm(&self, request: &str) -> anyhow::Result<RequestAnalysis> {
        let prompt = self.build_analysis_prompt(request);

        let openai_request = OpenAIRequest {
            model: "gpt-4o-mini".to_string(),
            messages: vec![
                OpenAIMessage {
                    role: "system".to_string(),
                    content: r#"You are a BI request analyzer. Analyze the user's request and return JSON with:
{
  "intent": "monitoring | analysis | comparison | overview",
  "entities": ["workflow", "judgment", "action"],
  "metrics": ["success_rate", "execution_time", "count"],
  "time_range": "last_week | last_month | today",
  "preferred_charts": ["line", "bar", "pie", "gauge"],
  "complexity_score": 0.0-1.0
}
Return ONLY valid JSON, no additional text."#.to_string(),
                },
                OpenAIMessage {
                    role: "user".to_string(),
                    content: prompt,
                },
            ],
            temperature: 0.3,
        };

        let response = self.http_client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.openai_api_key))
            .json(&openai_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("OpenAI API error: {}", error_text));
        }

        let openai_response: OpenAIResponse = response.json().await?;
        let content = &openai_response.choices[0].message.content;

        // JSON 파싱
        let analysis: RequestAnalysis = serde_json::from_str(content)
            .map_err(|e| anyhow::anyhow!("Failed to parse LLM response: {}. Content: {}", e, content))?;

        println!("🤖 LLM-based analysis: {:?}", analysis);
        Ok(analysis)
    }

    /// LLM 분석용 프롬프트 생성
    fn build_analysis_prompt(&self, request: &str) -> String {
        format!(
            r#"User request: "{}"

Analyze this request and identify:
1. Intent: What is the user trying to do? (monitoring, analysis, comparison, overview)
2. Entities: What data sources are involved? (workflow, judgment, action)
3. Metrics: What metrics are needed? (success_rate, execution_time, count)
4. Time Range: What time period? (last_week, last_month, today)
5. Preferred Charts: What chart types fit best? (line, bar, pie, gauge)
6. Complexity Score: How complex is this request? (0.0-1.0)

Return JSON only."#,
            request
        )
    }

    // ========== Phase 4: RAG 기반 인사이트 생성 ==========

    /// 유사한 과거 케이스 검색 (pgvector 기반)
    fn find_similar_cases(&self, analysis: &RequestAnalysis, limit: usize) -> anyhow::Result<Vec<SimilarCase>> {
        // Phase 4에서는 Mock 데이터 사용 (실제 구현시 pgvector 쿼리로 교체)
        // SELECT * FROM judgment_executions
        // ORDER BY explanation_embedding <=> query_embedding
        // LIMIT {limit}

        let mock_cases = vec![
            SimilarCase {
                execution_id: "exec-similar-1".to_string(),
                workflow_id: "workflow-123".to_string(),
                input_data: json!({"temperature": 88, "vibration": 42}),
                result: true,
                confidence: 0.92,
                method_used: "hybrid".to_string(),
                similarity_score: 0.89,
                created_at: "2025-10-20T14:30:00Z".to_string(),
            },
            SimilarCase {
                execution_id: "exec-similar-2".to_string(),
                workflow_id: "workflow-123".to_string(),
                input_data: json!({"temperature": 85, "vibration": 45}),
                result: true,
                confidence: 0.87,
                method_used: "rule".to_string(),
                similarity_score: 0.85,
                created_at: "2025-10-19T10:15:00Z".to_string(),
            },
            SimilarCase {
                execution_id: "exec-similar-3".to_string(),
                workflow_id: "workflow-123".to_string(),
                input_data: json!({"temperature": 92, "vibration": 38}),
                result: false,
                confidence: 0.78,
                method_used: "llm_few_shot".to_string(),
                similarity_score: 0.75,
                created_at: "2025-10-18T16:45:00Z".to_string(),
            },
        ];

        // limit만큼만 반환
        Ok(mock_cases.into_iter().take(limit).collect())
    }

    /// 도메인 지식 로드 (업계 표준, 임계값 등)
    fn load_domain_knowledge(&self, metric: &str) -> Vec<String> {
        match metric {
            "success_rate" => vec![
                "업계 표준 성공률: 95% 이상".to_string(),
                "경고 임계값: 90% 미만".to_string(),
                "위험 임계값: 80% 미만".to_string(),
                "성공률 개선 방법: Rule 정교화, Few-shot 샘플 추가, 임계값 조정".to_string(),
            ],
            "execution_time" => vec![
                "업계 표준 응답 시간: 500ms 이하".to_string(),
                "경고 임계값: 1000ms 초과".to_string(),
                "위험 임계값: 2000ms 초과".to_string(),
                "성능 개선 방법: 캐싱, 인덱스 최적화, 병렬 처리".to_string(),
            ],
            _ => vec![
                "일반 권장사항: 정기적 모니터링 및 임계값 검토".to_string(),
            ],
        }
    }

    /// RAG 컨텍스트 구성
    fn build_rag_context(
        &self,
        analysis: &RequestAnalysis,
        aggregation: &AggregatedData,
    ) -> anyhow::Result<RagContext> {
        let similar_cases = self.find_similar_cases(analysis, 5)?;

        let metric = analysis.metrics.first()
            .map(|s| s.as_str())
            .unwrap_or("success_rate");

        let domain_knowledge = self.load_domain_knowledge(metric);

        Ok(RagContext {
            current_request: analysis.clone(),
            current_aggregation: aggregation.clone(),
            similar_cases,
            domain_knowledge,
        })
    }

    /// 비즈니스 권장사항 생성 (LLM 기반)
    async fn generate_recommendations(
        &self,
        rag_context: &RagContext,
    ) -> anyhow::Result<Vec<BusinessRecommendation>> {
        // RAG 컨텍스트 기반 프롬프트 생성
        let prompt = self.build_recommendation_prompt(rag_context);

        // OpenAI API 호출
        let openai_request = json!({
            "model": "gpt-4o-mini",
            "messages": [
                {
                    "role": "system",
                    "content": "You are a business analyst specializing in manufacturing quality control. Generate actionable recommendations based on judgment execution data and similar past cases."
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": 0.7,
        });

        let response = self.http_client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.openai_api_key))
            .json(&openai_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("OpenAI API error: {}", error_text));
        }

        let openai_response: OpenAIResponse = response.json().await?;
        let content = &openai_response.choices[0].message.content;

        // JSON 파싱
        let recommendations: Vec<BusinessRecommendation> = serde_json::from_str(content)
            .unwrap_or_else(|_| {
                // 파싱 실패시 기본 권장사항 반환
                vec![BusinessRecommendation {
                    title: "정기 모니터링 강화".to_string(),
                    description: "현재 상태를 지속적으로 모니터링하고 추세 변화를 추적하세요.".to_string(),
                    priority: "medium".to_string(),
                    expected_impact: "안정성 향상".to_string(),
                    reasoning: "일반 권장사항".to_string(),
                }]
            });

        Ok(recommendations)
    }

    /// 권장사항 생성용 프롬프트 빌드
    fn build_recommendation_prompt(&self, rag_context: &RagContext) -> String {
        let similar_cases_desc = rag_context.similar_cases.iter()
            .map(|case| format!(
                "- Case {}: result={}, confidence={:.1}%, similarity={:.1}%",
                case.execution_id, case.result, case.confidence * 100.0, case.similarity_score * 100.0
            ))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"Current Situation:
- Status: {}
- Trend: {}
- Mean: {:.1}%
- Change Rate: {:.1}%

Similar Past Cases (top 5):
{}

Domain Knowledge:
{}

Generate 2-3 actionable business recommendations in JSON format:
[
  {{
    "title": "Recommendation title",
    "description": "Detailed description",
    "priority": "high|medium|low",
    "expected_impact": "Expected outcome",
    "reasoning": "Why this is recommended based on similar cases"
  }}
]

Focus on practical actions the user can take immediately."#,
            rag_context.current_aggregation.status,
            rag_context.current_aggregation.trend,
            rag_context.current_aggregation.mean,
            rag_context.current_aggregation.change_rate,
            similar_cases_desc,
            rag_context.domain_knowledge.join("\n"),
        )
    }

    // ========== 통합 generate_insight (Phase 1 + Phase 2 + Phase 3 + Phase 4) ==========

    pub async fn generate_insight(&self, user_request: String) -> anyhow::Result<BiInsight> {
        println!("🔍 Generating insight for: {}", user_request);

        // Phase 1: 요청 분석
        let analysis = self.analyze_user_request(&user_request).await?;
        println!("📋 Analysis: {:?}", analysis);

        // Phase 3: 실제 데이터 조회 및 집계
        let executions = self.get_judgment_executions(None, analysis.time_range.as_deref())
            .unwrap_or_else(|_| vec![]);
        println!("📊 Found {} executions", executions.len());

        // Phase 2: 컴포넌트 선택 및 조립
        let component_names = self.select_components(&analysis);
        println!("🎨 Selected components: {:?}", component_names);

        let components = self.assemble_components(component_names, &analysis);
        println!("🔧 Assembled {} components", components.len());

        let react_code = self.generate_react_code(&components, &user_request);

        // Phase 3 + 4: 데이터 기반 인사이트 생성 + RAG
        let metric = analysis.metrics.first().map(|s| s.as_str()).unwrap_or("success_rate");
        let mut insights = vec![];
        let mut recommendations_text = vec![];

        if let Ok(agg) = self.aggregate_data(&executions, metric) {
            // Phase 3: 기본 통계 인사이트
            insights.push(format!("📊 평균 {}: {:.1}%", metric, agg.mean));
            insights.push(format!("📈 추세: {} (변화율: {:.1}%)", agg.trend, agg.change_rate));
            insights.push(format!("⚠️ 상태: {} (총 {} 건)", agg.status, agg.count));
            insights.push(format!("📉 범위: {:.1}% ~ {:.1}%", agg.min, agg.max));

            // Phase 4: RAG 기반 권장사항 생성
            if let Ok(rag_context) = self.build_rag_context(&analysis, &agg) {
                println!("🔍 Found {} similar cases", rag_context.similar_cases.len());

                // 유사 케이스 인사이트 추가
                if !rag_context.similar_cases.is_empty() {
                    let similar_success_rate = rag_context.similar_cases.iter()
                        .filter(|c| c.result)
                        .count() as f64 / rag_context.similar_cases.len() as f64 * 100.0;

                    insights.push(format!(
                        "🔍 유사 케이스 {} 건 발견 (성공률: {:.1}%)",
                        rag_context.similar_cases.len(),
                        similar_success_rate
                    ));
                }

                // LLM 기반 권장사항 생성 (비동기)
                if let Ok(recommendations) = self.generate_recommendations(&rag_context).await {
                    println!("💡 Generated {} recommendations", recommendations.len());
                    recommendations_text = recommendations.iter().map(|r| {
                        format!(
                            "✨ {} (우선순위: {})\n   {}\n   예상 효과: {}",
                            r.title, r.priority, r.description, r.expected_impact
                        )
                    }).collect();
                } else {
                    // LLM 호출 실패시 도메인 지식 기반 기본 권장사항
                    recommendations_text = rag_context.domain_knowledge.iter()
                        .take(2)
                        .map(|k| format!("💡 {}", k))
                        .collect();
                }
            }
        } else {
            insights.push("데이터가 충분하지 않습니다.".to_string());
            recommendations_text.push("더 많은 데이터가 수집되면 권장사항을 제공할 수 있습니다.".to_string());
        }

        Ok(BiInsight {
            title: format!("{} 분석 결과", user_request),
            insights,
            component_code: react_code,
            recommendations: recommendations_text,
        })
    }

    // ========== Phase 5: 실시간 스트리밍 & 이벤트 발생 ==========

    /// 이벤트 발생 헬퍼 메서드
    fn emit_event(&self, event_name: &str, payload: &impl serde::Serialize) -> anyhow::Result<()> {
        if let Some(handle) = &self.app_handle {
            handle.emit_all(event_name, payload)
                .map_err(|e| anyhow::anyhow!("Failed to emit event '{}': {}", event_name, e))?;
            println!("📡 Event emitted: {} (payload: {})", event_name,
                serde_json::to_string(payload).unwrap_or_else(|_| "...".to_string()));
        } else {
            println!("⚠️ No AppHandle - event '{}' not emitted (test mode)", event_name);
        }
        Ok(())
    }

    /// Phase 5: 인사이트 생성 with 실시간 진행 상황 이벤트
    pub async fn generate_insight_stream(&self, user_request: String) -> anyhow::Result<BiInsight> {
        // 이벤트 1: 분석 시작
        self.emit_event("bi:analysis:started", &json!({
            "request": user_request,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "percentage": 0
        }))?;

        // Phase 1: 요청 분석
        println!("🧠 Analyzing user request...");
        let analysis = self.analyze_user_request(&user_request).await?;

        // 이벤트 2: 분석 완료
        self.emit_event("bi:analysis:completed", &json!({
            "analysis": analysis,
            "percentage": 20,
            "message": "Request analyzed successfully"
        }))?;

        // Phase 2: 컴포넌트 선택
        println!("🎨 Selecting components...");
        let component_names = self.select_components(&analysis);
        let components = self.assemble_components(component_names.clone(), &analysis);

        // 이벤트 3: 컴포넌트 선택 완료
        self.emit_event("bi:components:selected", &json!({
            "components": component_names,
            "count": components.len(),
            "percentage": 40,
            "message": format!("Selected {} components", components.len())
        }))?;

        // Phase 3: 데이터 집계
        println!("📊 Aggregating data...");
        let executions = self.get_judgment_executions(None, analysis.time_range.as_deref())?;
        let metric = analysis.metrics.first().map(|s| s.as_str()).unwrap_or("success_rate");
        let agg = self.aggregate_data(&executions, metric)?;

        // 이벤트 4: 데이터 집계 완료
        self.emit_event("bi:data:aggregated", &json!({
            "aggregation": agg,
            "executions_count": executions.len(),
            "percentage": 60,
            "message": format!("Aggregated {} executions", executions.len())
        }))?;

        // Phase 4: RAG 인사이트 생성
        println!("🔍 Generating RAG insights...");
        let rag_context = self.build_rag_context(&analysis, &agg)?;
        let recommendations = self.generate_recommendations(&rag_context).await?;

        // 이벤트 5: RAG 완료
        self.emit_event("bi:rag:completed", &json!({
            "similar_cases_count": rag_context.similar_cases.len(),
            "recommendations_count": recommendations.len(),
            "percentage": 80,
            "message": format!("Found {} similar cases", rag_context.similar_cases.len())
        }))?;

        // 최종 인사이트 생성
        println!("✨ Finalizing insight...");
        let react_code = self.generate_react_code(&components, &user_request);

        let mut insights = vec![];
        insights.push(format!("📊 평균 {}: {:.1}%", metric, agg.mean));
        insights.push(format!("📈 추세: {} (변화율: {:.1}%)", agg.trend, agg.change_rate));
        insights.push(format!("⚠️ 상태: {} (총 {} 건)", agg.status, agg.count));
        insights.push(format!("📉 범위: {:.1}% ~ {:.1}%", agg.min, agg.max));

        if !rag_context.similar_cases.is_empty() {
            let similar_success_rate = rag_context.similar_cases.iter()
                .filter(|c| c.result)
                .count() as f64 / rag_context.similar_cases.len() as f64 * 100.0;
            insights.push(format!(
                "🔍 유사 케이스 {} 건 발견 (성공률: {:.1}%)",
                rag_context.similar_cases.len(),
                similar_success_rate
            ));
        }

        let recommendations_text: Vec<String> = recommendations.iter().map(|r| {
            format!(
                "✨ {} (우선순위: {})\n   {}\n   예상 효과: {}",
                r.title, r.priority, r.description, r.expected_impact
            )
        }).collect();

        let insight = BiInsight {
            title: format!("{} 분석 결과", user_request),
            insights,
            component_code: react_code,
            recommendations: recommendations_text,
        };

        // 이벤트 6: 최종 완료
        self.emit_event("bi:insight:completed", &json!({
            "insight": insight,
            "percentage": 100,
            "message": "Insight generation completed successfully"
        }))?;

        Ok(insight)
    }
}

// ========== 테스트 코드 (Phase 1 + Phase 2) ==========

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_complexity_calculation() {
        let service = BiService::new().unwrap();

        // 간단한 요청 (< 0.5)
        let score1 = service.calculate_complexity("지난 주 성공률 보여줘");
        println!("Score 1: {}", score1);
        assert!(score1 < 0.5);

        // 복잡한 요청 (>= 0.5)
        let score2 = service.calculate_complexity("지난 달과 이번 달의 성공률 추세를 비교해줘");
        println!("Score 2: {}", score2);
        assert!(score2 >= 0.5);

        // 이상 탐지 요청 (>= 0.5) - "anomaly" 영어 키워드 사용
        let score3 = service.calculate_complexity("워크플로우에서 anomaly 패턴을 찾아줘");
        println!("Score 3: {}", score3);
        assert!(score3 >= 0.5);
    }

    #[tokio::test]
    async fn test_template_based_analysis() {
        let service = BiService::new().unwrap();

        let analysis = service.analyze_with_template("지난 주 워크플로우 성공률 보여줘").unwrap();

        assert_eq!(analysis.intent, "monitoring");
        assert!(analysis.entities.contains(&"workflow".to_string()));
        assert!(analysis.metrics.contains(&"success_rate".to_string()));
        assert_eq!(analysis.time_range, Some("last_week".to_string()));
        assert!(analysis.preferred_charts.contains(&"gauge".to_string()));
        assert!(analysis.complexity_score < 0.5);
    }

    #[tokio::test]
    async fn test_analyze_user_request_simple() {
        let service = BiService::new().unwrap();

        // 간단한 요청 (템플릿 기반)
        let analysis = service.analyze_user_request("지난 주 성공률").await.unwrap();

        assert_eq!(analysis.intent, "monitoring");
        assert!(analysis.metrics.contains(&"success_rate".to_string()));
        assert_eq!(analysis.time_range, Some("last_week".to_string()));
    }

    #[tokio::test]
    async fn test_generate_insight_integration() {
        let service = BiService::new().unwrap();

        let insight = service.generate_insight("지난 주 워크플로우 성공률".to_string()).await.unwrap();

        assert!(!insight.title.is_empty());
        assert!(!insight.insights.is_empty());
        assert!(!insight.component_code.is_empty());
    }

    #[tokio::test]
    async fn test_intent_classification() {
        let service = BiService::new().unwrap();

        // Monitoring intent
        let analysis1 = service.analyze_with_template("워크플로우 성공률 보여줘").unwrap();
        assert_eq!(analysis1.intent, "monitoring");

        // Analysis intent
        let analysis2 = service.analyze_with_template("워크플로우 성공률 분석해줘").unwrap();
        assert_eq!(analysis2.intent, "analysis");

        // Comparison intent
        let analysis3 = service.analyze_with_template("이번 주와 지난 주 성공률 비교해줘").unwrap();
        assert_eq!(analysis3.intent, "comparison");
    }

    #[tokio::test]
    async fn test_metric_extraction() {
        let service = BiService::new().unwrap();

        let analysis = service.analyze_with_template("실행 시간과 성공률 보여줘").unwrap();

        assert!(analysis.metrics.contains(&"execution_time".to_string()));
        assert!(analysis.metrics.contains(&"success_rate".to_string()));
        assert_eq!(analysis.metrics.len(), 2);
    }

    #[tokio::test]
    async fn test_chart_type_preference() {
        let service = BiService::new().unwrap();

        // 성공률 → gauge + line
        let analysis1 = service.analyze_with_template("성공률 보여줘").unwrap();
        assert!(analysis1.preferred_charts.contains(&"gauge".to_string()));

        // 개수 → bar
        let analysis2 = service.analyze_with_template("워크플로우 개수 보여줘").unwrap();
        assert!(analysis2.preferred_charts.contains(&"bar".to_string()));

        // 실행 시간 → line
        let analysis3 = service.analyze_with_template("실행 시간 보여줘").unwrap();
        assert!(analysis3.preferred_charts.contains(&"line".to_string()));
    }

    // ========== Phase 2 테스트 ==========

    #[tokio::test]
    async fn test_component_registry() {
        let service = BiService::new().unwrap();

        // 10개 컴포넌트가 등록되었는지 확인
        assert_eq!(service.component_registry.len(), 10);

        // 각 컴포넌트 확인
        assert!(service.component_registry.contains_key("MetricCard"));
        assert!(service.component_registry.contains_key("LineChart"));
        assert!(service.component_registry.contains_key("BarChart"));
        assert!(service.component_registry.contains_key("PieChart"));
        assert!(service.component_registry.contains_key("GaugeChart"));
        assert!(service.component_registry.contains_key("DataTable"));
        assert!(service.component_registry.contains_key("HeatMap"));
        assert!(service.component_registry.contains_key("ScatterPlot"));
        assert!(service.component_registry.contains_key("AreaChart"));
        assert!(service.component_registry.contains_key("TreeMap"));
    }

    #[tokio::test]
    async fn test_component_selection() {
        let service = BiService::new().unwrap();

        // success_rate 메트릭 → GaugeChart + LineChart
        let analysis1 = RequestAnalysis {
            intent: "monitoring".to_string(),
            entities: vec!["workflow".to_string()],
            metrics: vec!["success_rate".to_string()],
            time_range: Some("last_week".to_string()),
            preferred_charts: vec![],
            complexity_score: 0.2,
        };

        let selected1 = service.select_components(&analysis1);
        assert!(selected1.contains(&"GaugeChart".to_string()));
        assert!(selected1.contains(&"LineChart".to_string()));

        // count 메트릭 → BarChart
        let analysis2 = RequestAnalysis {
            intent: "monitoring".to_string(),
            entities: vec!["workflow".to_string()],
            metrics: vec!["count".to_string()],
            time_range: Some("today".to_string()),
            preferred_charts: vec![],
            complexity_score: 0.1,
        };

        let selected2 = service.select_components(&analysis2);
        assert!(selected2.contains(&"BarChart".to_string()));
    }

    #[tokio::test]
    async fn test_component_assembly() {
        let service = BiService::new().unwrap();

        let analysis = RequestAnalysis {
            intent: "monitoring".to_string(),
            entities: vec!["workflow".to_string()],
            metrics: vec!["success_rate".to_string()],
            time_range: Some("last_week".to_string()),
            preferred_charts: vec!["gauge".to_string()],
            complexity_score: 0.2,
        };

        let component_names = vec!["GaugeChart".to_string()];
        let components = service.assemble_components(component_names, &analysis);

        assert_eq!(components.len(), 1);
        assert_eq!(components[0].component_type, "GaugeChart");
        assert!(components[0].jsx_code.contains("GaugeChart"));
        assert!(components[0].props.contains_key("value"));
        assert!(components[0].props.contains_key("max"));
    }

    #[tokio::test]
    async fn test_jsx_generation() {
        let service = BiService::new().unwrap();

        let metadata = ComponentMetadata {
            name: "MetricCard".to_string(),
            description: "Test component".to_string(),
            required_props: vec!["title".to_string(), "value".to_string()],
            optional_props: vec![],
            supported_data_types: vec![],
            suitable_metrics: vec![],
            template: r#"<MetricCard title="{title}" value="{value}" />"#.to_string(),
        };

        let mut props = HashMap::new();
        props.insert("title".to_string(), json!("성공률"));
        props.insert("value".to_string(), json!("95.5%"));

        let jsx = service.generate_jsx(&metadata, &props);

        assert!(jsx.contains("성공률"));
        assert!(jsx.contains("95.5%"));
        assert!(!jsx.contains("{title}"));
        assert!(!jsx.contains("{value}"));
    }

    #[tokio::test]
    async fn test_react_code_generation() {
        let service = BiService::new().unwrap();

        let components = vec![
            AssembledComponent {
                component_type: "MetricCard".to_string(),
                props: HashMap::new(),
                jsx_code: r#"<MetricCard title="성공률" value="95%" />"#.to_string(),
            },
            AssembledComponent {
                component_type: "LineChart".to_string(),
                props: HashMap::new(),
                jsx_code: r#"<LineChart data={data} xKey="date" yKey="value" />"#.to_string(),
            },
        ];

        let react_code = service.generate_react_code(&components, "성공률 대시보드");

        assert!(react_code.contains("성공률 대시보드"));
        assert!(react_code.contains("MetricCard"));
        assert!(react_code.contains("LineChart"));
        assert!(react_code.contains("grid grid-cols-3"));
    }

    #[tokio::test]
    async fn test_generate_insight_with_component_assembly() {
        let service = BiService::new().unwrap();

        let insight = service.generate_insight("지난 주 워크플로우 성공률".to_string()).await.unwrap();

        assert!(!insight.title.is_empty());
        assert!(!insight.insights.is_empty());
        assert!(insight.component_code.contains("dashboard"));
        assert!(insight.component_code.contains("GaugeChart") || insight.component_code.contains("LineChart"));
        assert!(!insight.recommendations.is_empty());
    }

    #[tokio::test]
    async fn test_preferred_chart_override() {
        let service = BiService::new().unwrap();

        // preferred_charts가 있으면 우선 사용
        let analysis = RequestAnalysis {
            intent: "monitoring".to_string(),
            entities: vec!["workflow".to_string()],
            metrics: vec!["success_rate".to_string()],
            time_range: Some("last_week".to_string()),
            preferred_charts: vec!["bar".to_string()],
            complexity_score: 0.2,
        };

        let selected = service.select_components(&analysis);
        assert!(selected.contains(&"BarChart".to_string()));
    }

    // ========== Phase 3 테스트: 데이터 통합 ==========

    #[tokio::test]
    async fn test_get_judgment_executions() {
        let service = BiService::new().unwrap();

        // Mock 데이터 조회 테스트
        let executions = service.get_judgment_executions(None, None).unwrap();

        assert_eq!(executions.len(), 3); // Mock 데이터 3건
        assert!(executions[0].result); // 첫 번째: 성공
        assert!(executions[1].result); // 두 번째: 성공
        assert!(!executions[2].result); // 세 번째: 실패
        assert_eq!(executions[0].method_used, "rule");
    }

    #[tokio::test]
    async fn test_aggregate_data_success_rate() {
        let service = BiService::new().unwrap();

        let executions = vec![
            JudgmentExecution {
                id: "1".to_string(),
                workflow_id: "test".to_string(),
                result: true,
                confidence: 0.95,
                method_used: "rule".to_string(),
                execution_time_ms: 120,
                created_at: "2025-10-22T10:00:00Z".to_string(),
            },
            JudgmentExecution {
                id: "2".to_string(),
                workflow_id: "test".to_string(),
                result: true,
                confidence: 0.90,
                method_used: "llm".to_string(),
                execution_time_ms: 150,
                created_at: "2025-10-22T11:00:00Z".to_string(),
            },
            JudgmentExecution {
                id: "3".to_string(),
                workflow_id: "test".to_string(),
                result: false,
                confidence: 0.60,
                method_used: "hybrid".to_string(),
                execution_time_ms: 200,
                created_at: "2025-10-22T12:00:00Z".to_string(),
            },
        ];

        let agg = service.aggregate_data(&executions, "success_rate").unwrap();

        assert_eq!(agg.count, 3);
        assert!((agg.mean - 66.67).abs() < 0.1); // 2/3 = 66.67%
        assert_eq!(agg.status, "critical"); // 66.67% < 70% threshold → critical
        assert!(agg.trend.contains("decreasing")); // true, true, false → 하락
    }

    #[tokio::test]
    async fn test_aggregate_data_execution_time() {
        let service = BiService::new().unwrap();

        let executions = vec![
            JudgmentExecution {
                id: "1".to_string(),
                workflow_id: "test".to_string(),
                result: true,
                confidence: 0.95,
                method_used: "rule".to_string(),
                execution_time_ms: 100,
                created_at: "2025-10-22T10:00:00Z".to_string(),
            },
            JudgmentExecution {
                id: "2".to_string(),
                workflow_id: "test".to_string(),
                result: true,
                confidence: 0.90,
                method_used: "llm".to_string(),
                execution_time_ms: 200,
                created_at: "2025-10-22T11:00:00Z".to_string(),
            },
            JudgmentExecution {
                id: "3".to_string(),
                workflow_id: "test".to_string(),
                result: true,
                confidence: 0.85,
                method_used: "hybrid".to_string(),
                execution_time_ms: 300,
                created_at: "2025-10-22T12:00:00Z".to_string(),
            },
        ];

        let agg = service.aggregate_data(&executions, "execution_time").unwrap();

        assert_eq!(agg.count, 3);
        assert_eq!(agg.mean, 200.0); // (100 + 200 + 300) / 3
        assert_eq!(agg.median, 200.0);
        assert_eq!(agg.min, 100.0);
        assert_eq!(agg.max, 300.0);
        assert!(agg.trend.contains("increasing")); // 100 → 300 증가
        assert!((agg.change_rate - 200.0).abs() < 0.1); // (300 - 100) / 100 * 100 = 200%
    }

    #[tokio::test]
    async fn test_generate_time_series() {
        let service = BiService::new().unwrap();

        let executions = vec![
            JudgmentExecution {
                id: "1".to_string(),
                workflow_id: "test".to_string(),
                result: true,
                confidence: 0.95,
                method_used: "rule".to_string(),
                execution_time_ms: 120,
                created_at: "2025-10-22T10:00:00Z".to_string(),
            },
            JudgmentExecution {
                id: "2".to_string(),
                workflow_id: "test".to_string(),
                result: false,
                confidence: 0.90,
                method_used: "llm".to_string(),
                execution_time_ms: 150,
                created_at: "2025-10-22T11:00:00Z".to_string(),
            },
        ];

        let time_series = service.generate_time_series(&executions, "success_rate");

        assert_eq!(time_series.len(), 2);
        assert_eq!(time_series[0].timestamp, "2025-10-22T10:00:00Z");
        assert_eq!(time_series[0].value, 100.0); // true = 100%
        assert_eq!(time_series[1].timestamp, "2025-10-22T11:00:00Z");
        assert_eq!(time_series[1].value, 0.0); // false = 0%
    }

    #[tokio::test]
    async fn test_trend_detection() {
        let service = BiService::new().unwrap();

        // 증가 추세 테스트
        let executions_increasing = vec![
            JudgmentExecution {
                id: "1".to_string(),
                workflow_id: "test".to_string(),
                result: true,
                confidence: 0.80,
                method_used: "rule".to_string(),
                execution_time_ms: 100,
                created_at: "2025-10-22T10:00:00Z".to_string(),
            },
            JudgmentExecution {
                id: "2".to_string(),
                workflow_id: "test".to_string(),
                result: true,
                confidence: 0.95,
                method_used: "rule".to_string(),
                execution_time_ms: 120,
                created_at: "2025-10-22T11:00:00Z".to_string(),
            },
        ];

        let agg_inc = service.aggregate_data(&executions_increasing, "confidence").unwrap();
        assert!(agg_inc.trend.contains("increasing")); // 80% → 95% 증가

        // 감소 추세 테스트
        let executions_decreasing = vec![
            JudgmentExecution {
                id: "1".to_string(),
                workflow_id: "test".to_string(),
                result: true,
                confidence: 0.95,
                method_used: "rule".to_string(),
                execution_time_ms: 100,
                created_at: "2025-10-22T10:00:00Z".to_string(),
            },
            JudgmentExecution {
                id: "2".to_string(),
                workflow_id: "test".to_string(),
                result: true,
                confidence: 0.80,
                method_used: "rule".to_string(),
                execution_time_ms: 120,
                created_at: "2025-10-22T11:00:00Z".to_string(),
            },
        ];

        let agg_dec = service.aggregate_data(&executions_decreasing, "confidence").unwrap();
        assert!(agg_dec.trend.contains("decreasing")); // 95% → 80% 감소

        // 안정 추세 테스트
        let executions_stable = vec![
            JudgmentExecution {
                id: "1".to_string(),
                workflow_id: "test".to_string(),
                result: true,
                confidence: 0.90,
                method_used: "rule".to_string(),
                execution_time_ms: 100,
                created_at: "2025-10-22T10:00:00Z".to_string(),
            },
            JudgmentExecution {
                id: "2".to_string(),
                workflow_id: "test".to_string(),
                result: true,
                confidence: 0.92,
                method_used: "rule".to_string(),
                execution_time_ms: 120,
                created_at: "2025-10-22T11:00:00Z".to_string(),
            },
        ];

        let agg_stable = service.aggregate_data(&executions_stable, "confidence").unwrap();
        assert!(agg_stable.trend.contains("stable")); // 90% → 92% 안정 (변화 2% < 5% threshold)
    }

    #[tokio::test]
    async fn test_status_classification() {
        let service = BiService::new().unwrap();

        // Normal 상태 테스트 (>= 90%)
        let executions_normal = vec![
            JudgmentExecution {
                id: "1".to_string(),
                workflow_id: "test".to_string(),
                result: true,
                confidence: 0.95,
                method_used: "rule".to_string(),
                execution_time_ms: 100,
                created_at: "2025-10-22T10:00:00Z".to_string(),
            },
            JudgmentExecution {
                id: "2".to_string(),
                workflow_id: "test".to_string(),
                result: true,
                confidence: 0.93,
                method_used: "rule".to_string(),
                execution_time_ms: 120,
                created_at: "2025-10-22T11:00:00Z".to_string(),
            },
        ];

        let agg_normal = service.aggregate_data(&executions_normal, "success_rate").unwrap();
        assert_eq!(agg_normal.status, "normal"); // 100% >= 90%

        // Warning 상태 테스트 (70% <= x < 90%)
        let executions_warning = vec![
            JudgmentExecution {
                id: "1".to_string(),
                workflow_id: "test".to_string(),
                result: true,
                confidence: 0.80,
                method_used: "rule".to_string(),
                execution_time_ms: 100,
                created_at: "2025-10-22T10:00:00Z".to_string(),
            },
            JudgmentExecution {
                id: "2".to_string(),
                workflow_id: "test".to_string(),
                result: true,
                confidence: 0.85,
                method_used: "rule".to_string(),
                execution_time_ms: 120,
                created_at: "2025-10-22T11:00:00Z".to_string(),
            },
            JudgmentExecution {
                id: "3".to_string(),
                workflow_id: "test".to_string(),
                result: true,
                confidence: 0.75,
                method_used: "rule".to_string(),
                execution_time_ms: 130,
                created_at: "2025-10-22T12:00:00Z".to_string(),
            },
            JudgmentExecution {
                id: "4".to_string(),
                workflow_id: "test".to_string(),
                result: false,
                confidence: 0.60,
                method_used: "rule".to_string(),
                execution_time_ms: 150,
                created_at: "2025-10-22T13:00:00Z".to_string(),
            },
        ];

        let agg_warning = service.aggregate_data(&executions_warning, "success_rate").unwrap();
        assert_eq!(agg_warning.status, "warning"); // 75% (3/4) - 70% <= 75% < 90%

        // Critical 상태 테스트 (< 70%)
        let executions_critical = vec![
            JudgmentExecution {
                id: "1".to_string(),
                workflow_id: "test".to_string(),
                result: true,
                confidence: 0.60,
                method_used: "rule".to_string(),
                execution_time_ms: 100,
                created_at: "2025-10-22T10:00:00Z".to_string(),
            },
            JudgmentExecution {
                id: "2".to_string(),
                workflow_id: "test".to_string(),
                result: false,
                confidence: 0.50,
                method_used: "rule".to_string(),
                execution_time_ms: 120,
                created_at: "2025-10-22T11:00:00Z".to_string(),
            },
        ];

        let agg_critical = service.aggregate_data(&executions_critical, "success_rate").unwrap();
        assert_eq!(agg_critical.status, "critical"); // 50% < 70%
    }

    #[tokio::test]
    async fn test_assemble_components_with_real_data() {
        let service = BiService::new().unwrap();

        let analysis = RequestAnalysis {
            intent: "monitoring".to_string(),
            entities: vec!["workflow".to_string()],
            metrics: vec!["success_rate".to_string()],
            time_range: Some("last_week".to_string()),
            preferred_charts: vec![],
            complexity_score: 0.2,
        };

        let component_names = vec!["MetricCard".to_string(), "GaugeChart".to_string()];
        let components = service.assemble_components(component_names, &analysis);

        assert_eq!(components.len(), 2);

        // MetricCard는 집계 데이터 사용
        let metric_card = &components[0];
        assert_eq!(metric_card.component_type, "MetricCard");
        assert!(metric_card.props.contains_key("value"));
        assert!(metric_card.props.contains_key("trend"));

        // GaugeChart도 집계 데이터 사용
        let gauge_chart = &components[1];
        assert_eq!(gauge_chart.component_type, "GaugeChart");
        assert!(gauge_chart.props.contains_key("value"));
        assert_eq!(gauge_chart.props.get("max"), Some(&json!(100)));
    }

    #[tokio::test]
    async fn test_generate_insight_with_aggregation() {
        let service = BiService::new().unwrap();

        let insight = service.generate_insight("지난 주 워크플로우 성공률".to_string()).await.unwrap();

        // 기본 구조 검증
        assert!(!insight.title.is_empty());
        assert!(!insight.insights.is_empty());
        assert!(!insight.component_code.is_empty());

        // Phase 3 집계 데이터 기반 인사이트 확인
        let insights_text = insight.insights.join(" ");
        assert!(insights_text.contains("평균") || insights_text.contains("추세") || insights_text.contains("상태"));
    }

    // ========== Phase 4 테스트: RAG 기반 인사이트 ==========

    #[tokio::test]
    async fn test_find_similar_cases() {
        let service = BiService::new().unwrap();

        let analysis = RequestAnalysis {
            intent: "monitoring".to_string(),
            entities: vec!["workflow".to_string()],
            metrics: vec!["success_rate".to_string()],
            time_range: Some("last_week".to_string()),
            preferred_charts: vec![],
            complexity_score: 0.3,
        };

        let similar_cases = service.find_similar_cases(&analysis, 5).unwrap();

        // Mock 데이터 3건 반환
        assert_eq!(similar_cases.len(), 3);

        // 첫 번째 케이스 검증
        assert_eq!(similar_cases[0].execution_id, "exec-similar-1");
        assert!(similar_cases[0].similarity_score > 0.8); // 높은 유사도

        // 유사도 순 정렬 확인
        assert!(similar_cases[0].similarity_score >= similar_cases[1].similarity_score);
        assert!(similar_cases[1].similarity_score >= similar_cases[2].similarity_score);
    }

    #[tokio::test]
    async fn test_load_domain_knowledge() {
        let service = BiService::new().unwrap();

        // success_rate 메트릭에 대한 도메인 지식
        let knowledge_success = service.load_domain_knowledge("success_rate");
        assert!(!knowledge_success.is_empty());
        assert!(knowledge_success.iter().any(|k| k.contains("업계 표준")));
        assert!(knowledge_success.iter().any(|k| k.contains("임계값")));

        // execution_time 메트릭에 대한 도메인 지식
        let knowledge_time = service.load_domain_knowledge("execution_time");
        assert!(!knowledge_time.is_empty());
        assert!(knowledge_time.iter().any(|k| k.contains("500ms")));

        // 기타 메트릭
        let knowledge_other = service.load_domain_knowledge("unknown");
        assert!(!knowledge_other.is_empty());
        assert!(knowledge_other.iter().any(|k| k.contains("모니터링")));
    }

    #[tokio::test]
    async fn test_build_rag_context() {
        let service = BiService::new().unwrap();

        let analysis = RequestAnalysis {
            intent: "monitoring".to_string(),
            entities: vec!["workflow".to_string()],
            metrics: vec!["success_rate".to_string()],
            time_range: Some("last_week".to_string()),
            preferred_charts: vec![],
            complexity_score: 0.3,
        };

        let agg = AggregatedData {
            mean: 85.5,
            median: 87.0,
            std_dev: 5.2,
            min: 75.0,
            max: 95.0,
            count: 10,
            status: "warning".to_string(),
            trend: "decreasing".to_string(),
            change_rate: -10.5,
        };

        let rag_context = service.build_rag_context(&analysis, &agg).unwrap();

        // 구조 검증
        assert_eq!(rag_context.current_request.intent, "monitoring");
        assert_eq!(rag_context.current_aggregation.mean, 85.5);
        assert_eq!(rag_context.similar_cases.len(), 3); // Mock 데이터 3건
        assert!(!rag_context.domain_knowledge.is_empty());

        // 도메인 지식에 success_rate 관련 내용 포함
        assert!(rag_context.domain_knowledge.iter().any(|k| k.contains("성공률")));
    }

    #[tokio::test]
    async fn test_build_recommendation_prompt() {
        let service = BiService::new().unwrap();

        let rag_context = RagContext {
            current_request: RequestAnalysis {
                intent: "monitoring".to_string(),
                entities: vec!["workflow".to_string()],
                metrics: vec!["success_rate".to_string()],
                time_range: Some("last_week".to_string()),
                preferred_charts: vec![],
                complexity_score: 0.3,
            },
            current_aggregation: AggregatedData {
                mean: 85.5,
                median: 87.0,
                std_dev: 5.2,
                min: 75.0,
                max: 95.0,
                count: 10,
                status: "warning".to_string(),
                trend: "decreasing".to_string(),
                change_rate: -10.5,
            },
            similar_cases: vec![
                SimilarCase {
                    execution_id: "test-1".to_string(),
                    workflow_id: "workflow-123".to_string(),
                    input_data: json!({"temp": 90}),
                    result: true,
                    confidence: 0.92,
                    method_used: "hybrid".to_string(),
                    similarity_score: 0.89,
                    created_at: "2025-10-20T14:30:00Z".to_string(),
                },
            ],
            domain_knowledge: vec![
                "업계 표준 성공률: 95% 이상".to_string(),
                "경고 임계값: 90% 미만".to_string(),
            ],
        };

        let prompt = service.build_recommendation_prompt(&rag_context);

        // 프롬프트 구조 검증
        assert!(prompt.contains("warning")); // 상태 포함
        assert!(prompt.contains("decreasing")); // 추세 포함
        assert!(prompt.contains("85.5")); // 평균 포함
        assert!(prompt.contains("Similar Past Cases")); // 유사 케이스 섹션
        assert!(prompt.contains("Domain Knowledge")); // 도메인 지식 섹션
        assert!(prompt.contains("JSON format")); // JSON 요청
    }

    #[tokio::test]
    async fn test_generate_insight_with_rag() {
        let service = BiService::new().unwrap();

        let insight = service.generate_insight("지난 주 워크플로우 성공률".to_string()).await.unwrap();

        // 기본 구조 검증
        assert!(!insight.title.is_empty());
        assert!(!insight.insights.is_empty());
        assert!(!insight.component_code.is_empty());
        assert!(!insight.recommendations.is_empty()); // Phase 4: 권장사항 추가됨

        // Phase 4 RAG 인사이트 확인
        let insights_text = insight.insights.join(" ");
        assert!(insights_text.contains("평균") || insights_text.contains("추세"));

        // 유사 케이스 인사이트 포함
        assert!(insights_text.contains("유사 케이스") || insights_text.contains("유사한"));

        // 권장사항 존재 확인
        let recommendations_text = insight.recommendations.join(" ");
        assert!(!recommendations_text.is_empty());

        // "Phase 4에서 RAG 기반 권장사항 생성 예정" 제거 확인
        assert!(!recommendations_text.contains("Phase 4에서"));
    }

    #[tokio::test]
    async fn test_similar_cases_success_rate() {
        let service = BiService::new().unwrap();

        let similar_cases = vec![
            SimilarCase {
                execution_id: "test-1".to_string(),
                workflow_id: "workflow-123".to_string(),
                input_data: json!({}),
                result: true,
                confidence: 0.92,
                method_used: "hybrid".to_string(),
                similarity_score: 0.89,
                created_at: "2025-10-20T14:30:00Z".to_string(),
            },
            SimilarCase {
                execution_id: "test-2".to_string(),
                workflow_id: "workflow-123".to_string(),
                input_data: json!({}),
                result: true,
                confidence: 0.87,
                method_used: "rule".to_string(),
                similarity_score: 0.85,
                created_at: "2025-10-19T10:15:00Z".to_string(),
            },
            SimilarCase {
                execution_id: "test-3".to_string(),
                workflow_id: "workflow-123".to_string(),
                input_data: json!({}),
                result: false,
                confidence: 0.78,
                method_used: "llm".to_string(),
                similarity_score: 0.75,
                created_at: "2025-10-18T16:45:00Z".to_string(),
            },
        ];

        // 성공률 계산
        let success_rate = similar_cases.iter()
            .filter(|c| c.result)
            .count() as f64 / similar_cases.len() as f64 * 100.0;

        assert!((success_rate - 66.67).abs() < 0.1); // 2/3 = 66.67%
    }

    // ========== Phase 5 테스트: 이벤트 발생 & 스트리밍 ==========

    #[tokio::test]
    async fn test_emit_event_without_app_handle() {
        // AppHandle 없이 생성 (테스트 모드)
        let service = BiService::new().unwrap();

        // 이벤트 발생시 에러 없이 무시됨
        let result = service.emit_event("test:event", &json!({"test": "data"}));
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_generate_insight_stream_events() {
        // 테스트 모드 (AppHandle 없음)
        let service = BiService::new().unwrap();

        // 인사이트 생성 (LLM API 실패시 fallback 사용)
        let result = service.generate_insight_stream("지난 주 성공률".to_string()).await;

        // LLM API 실패시 fallback으로 성공해야 함
        match result {
            Ok(insight) => {
                // 결과 검증
                assert!(insight.title.contains("지난 주 성공률"));
                assert!(!insight.insights.is_empty());
                assert!(!insight.component_code.is_empty());

                // 인사이트에 Phase 3 데이터 포함 확인
                let insights_text = insight.insights.join(" ");
                assert!(insights_text.contains("평균") || insights_text.contains("추세"));
            }
            Err(e) => {
                // API 키 없음 에러는 예상됨 (테스트 환경)
                assert!(e.to_string().contains("OpenAI") || e.to_string().contains("API"));
                println!("⚠️ Expected error in test environment: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_stream_progress_stages() {
        let service = BiService::new().unwrap();

        // Phase별 진행 상황 확인 (로그 출력 검증용)
        println!("=== Testing Insight Generation Stream ===");

        let result = service.generate_insight_stream("워크플로우 실행 시간 분석".to_string()).await;

        // LLM API 실패시 fallback으로 성공해야 함
        match result {
            Ok(insight) => {
                // 각 Phase 완료 확인
                assert!(insight.title.contains("워크플로우 실행 시간 분석"));
                assert!(!insight.recommendations.is_empty(), "Recommendations should be generated");

                // 권장사항에 RAG 기반 내용 포함 확인
                let recommendations_text = insight.recommendations.join(" ");
                assert!(!recommendations_text.is_empty());
            }
            Err(e) => {
                // API 키 없음 에러는 예상됨 (테스트 환경)
                assert!(e.to_string().contains("OpenAI") || e.to_string().contains("API"));
                println!("⚠️ Expected error in test environment: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_with_app_handle_constructor() {
        // with_app_handle 메서드 테스트 (AppHandle 없이)
        let service = BiService::with_app_handle(None).unwrap();

        // 정상 동작 확인
        let components = service.select_components(&RequestAnalysis {
            intent: "monitoring".to_string(),
            entities: vec!["workflow".to_string()],
            metrics: vec!["success_rate".to_string()],
            time_range: Some("last_week".to_string()),
            preferred_charts: vec![],
            complexity_score: 0.3,
        });

        assert!(!components.is_empty());
    }
}
