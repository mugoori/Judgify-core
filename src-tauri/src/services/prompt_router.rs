//! 프롬프트 라우터 - 퓨어웰 음료 AI Agent
//!
//! 사용자의 간단한 질문을 확장된 전문 프롬프트로 변환합니다.
//! 차트 분석 요청 시 적절한 템플릿을 매칭하여 SQL, 판단기준, 차트 설정을 포함한
//! 구조화된 프롬프트를 생성합니다.
//!
//! ## Phase 2 개선사항 (2024-12)
//! - WeightedKeyword: 키워드별 가중치 및 정확 매칭 지원
//! - 다중 조건 정렬: 점수 > 매칭 개수 > 우선순위
//! - 동점 처리를 위한 결정적 알고리즘

use std::collections::HashMap;

/// 가중치 키워드 구조체
///
/// 키워드별로 가중치와 매칭 방식을 지정합니다.
/// - weight: 1(일반), 2(중요), 3(핵심)
/// - exact_match: true면 단어 경계를 고려한 정확 매칭
#[derive(Debug, Clone)]
pub struct WeightedKeyword {
    /// 키워드 문자열
    pub keyword: String,
    /// 가중치 (1=일반, 2=중요, 3=핵심)
    pub weight: u32,
    /// 정확 매칭 여부 (true: 단어 경계 체크, false: 포함 매칭)
    pub exact_match: bool,
}

impl WeightedKeyword {
    /// 새 가중치 키워드 생성
    pub fn new(keyword: &str, weight: u32, exact_match: bool) -> Self {
        Self {
            keyword: keyword.to_string(),
            weight,
            exact_match,
        }
    }

    /// 핵심 키워드 (가중치 3, 포함 매칭)
    pub fn core(keyword: &str) -> Self {
        Self::new(keyword, 3, false)
    }

    /// 중요 키워드 (가중치 2, 포함 매칭)
    pub fn important(keyword: &str) -> Self {
        Self::new(keyword, 2, false)
    }

    /// 일반 키워드 (가중치 1, 포함 매칭)
    pub fn general(keyword: &str) -> Self {
        Self::new(keyword, 1, false)
    }

    /// 정확 매칭 키워드 (지정 가중치, 단어 경계 체크)
    pub fn exact(keyword: &str, weight: u32) -> Self {
        Self::new(keyword, weight, true)
    }
}

/// 매칭 결과 구조체 (동점 처리용)
#[derive(Debug, Clone)]
pub struct MatchResult {
    /// 매칭된 템플릿 키
    pub template_key: String,
    /// 가중치 합산 점수
    pub total_score: u32,
    /// 매칭된 키워드 개수
    pub match_count: usize,
    /// 템플릿 우선순위 (낮을수록 높은 우선순위)
    pub priority: u32,
}

/// 차트 설정
#[derive(Debug, Clone)]
pub struct ChartConfig {
    /// 차트 유형: bar, line, pie, gauge
    pub chart_type: String,
    /// 스타일 설명
    pub style: String,
    /// 차트 제목
    pub title: String,
}

impl ChartConfig {
    pub fn new(chart_type: &str, style: &str, title: &str) -> Self {
        Self {
            chart_type: chart_type.to_string(),
            style: style.to_string(),
            title: title.to_string(),
        }
    }
}

/// 프롬프트 템플릿
#[derive(Debug, Clone)]
pub struct PromptTemplate {
    /// 가중치 키워드 목록 (Phase 2 개선)
    pub weighted_keywords: Vec<WeightedKeyword>,
    /// 기존 키워드 목록 (하위 호환성)
    #[allow(dead_code)]
    pub keywords: Vec<String>,
    /// 템플릿 우선순위 (동점 시 사용, 낮을수록 높은 우선순위)
    pub priority: u32,
    /// 차트 설정
    pub chart_config: ChartConfig,
    /// 데이터 소스 설명
    pub data_source: String,
    /// 확장 프롬프트 템플릿
    pub template: String,
}

/// 프롬프트 라우팅 결과
#[derive(Debug, Clone)]
pub struct PromptRoute {
    /// 매칭된 템플릿 키
    pub template_key: String,
    /// 매칭된 키워드들
    pub matched_keywords: Vec<String>,
    /// 차트 설정
    pub chart_config: ChartConfig,
    /// 데이터 소스
    pub data_source: String,
    /// 확장된 프롬프트
    pub expanded_prompt: String,
}

/// 프롬프트 라우터 - 간단한 질문을 확장된 프롬프트로 변환
pub struct PromptRouter {
    templates: HashMap<String, PromptTemplate>,
}

impl PromptRouter {
    /// 새 프롬프트 라우터 생성
    pub fn new() -> Self {
        let mut templates = HashMap::new();

        // ==================== BAR 차트 ====================

        // 1. 라인별 생산량 (priority: 1 - 가장 높은 우선순위)
        templates.insert(
            "라인별_생산량".to_string(),
            PromptTemplate {
                weighted_keywords: vec![
                    WeightedKeyword::core("라인별"),      // 3점 - 핵심 구분자
                    WeightedKeyword::core("생산량"),      // 3점 - 핵심
                    WeightedKeyword::important("라인"),   // 2점 - 중요
                    WeightedKeyword::general("생산현황"), // 1점 - 일반
                ],
                keywords: vec![
                    "라인별".to_string(),
                    "생산량".to_string(),
                    "라인".to_string(),
                    "생산현황".to_string(),
                ],
                priority: 1,
                chart_config: ChartConfig::new("bar", "그라데이션 + 둥근 모서리", "라인별 생산량 현황"),
                data_source: "mes_work_order + operation_exec (라인별 qty_output 합계)".to_string(),
                template: include_str!("../../templates/line_production.txt").to_string(),
            },
        );

        // 2. 월별 매출 (priority: 2)
        templates.insert(
            "월별_매출".to_string(),
            PromptTemplate {
                weighted_keywords: vec![
                    WeightedKeyword::core("매출"),        // 3점 - 핵심
                    WeightedKeyword::important("월별"),   // 2점 - 중요
                    WeightedKeyword::general("매출현황"), // 1점 - 일반
                    WeightedKeyword::general("매출액"),   // 1점 - 일반
                ],
                keywords: vec![
                    "월별".to_string(),
                    "매출".to_string(),
                    "매출현황".to_string(),
                    "매출액".to_string(),
                ],
                priority: 2,
                chart_config: ChartConfig::new("bar", "그라데이션 + 둥근 모서리", "월별 매출 현황"),
                data_source: "sales_order + sales_order_dtl (월별 금액 합계)".to_string(),
                template: include_str!("../../templates/monthly_sales.txt").to_string(),
            },
        );

        // 3. 설비별 비가동 (priority: 3)
        templates.insert(
            "설비별_비가동".to_string(),
            PromptTemplate {
                weighted_keywords: vec![
                    WeightedKeyword::core("비가동"),        // 3점 - 핵심
                    WeightedKeyword::core("설비별"),        // 3점 - 핵심 구분자
                    WeightedKeyword::important("다운타임"), // 2점 - 중요 동의어
                    WeightedKeyword::general("비가동시간"), // 1점 - 일반
                    WeightedKeyword::general("설비"),       // 1점 - 일반
                    WeightedKeyword::general("고장"),       // 1점 - 일반
                ],
                keywords: vec![
                    "설비별".to_string(),
                    "비가동".to_string(),
                    "비가동시간".to_string(),
                    "다운타임".to_string(),
                    "설비".to_string(),
                    "고장".to_string(),
                ],
                priority: 3,
                chart_config: ChartConfig::new("bar", "그라데이션 + 둥근 모서리", "설비별 비가동 시간 분석"),
                data_source: "downtime_event + equipment_mst + reason_code_mst".to_string(),
                template: include_str!("../../templates/equipment_downtime.txt").to_string(),
            },
        );

        // ==================== LINE 차트 ====================

        // 4. 월별 생산량 추이 (priority: 4)
        // 주의: "월별 생산량" 질문 시 "라인별_생산량"과 구분 필요
        // "생산추이" 또는 "생산트렌드" 복합어가 있어야 이 템플릿 선택
        // 단독 "추이"/"트렌드"는 다른 템플릿(출고_추이, 입고_추이 등)과 충돌하므로 제거
        templates.insert(
            "월별_생산량_추이".to_string(),
            PromptTemplate {
                weighted_keywords: vec![
                    WeightedKeyword::core("생산추이"),      // 3점 - 핵심 복합어 (다른 템플릿과 구분!)
                    WeightedKeyword::core("생산트렌드"),    // 3점 - 핵심 복합어
                    WeightedKeyword::important("생산량"),   // 2점 - 중요
                    WeightedKeyword::important("월별생산"), // 2점 - 중요 복합어
                    WeightedKeyword::general("월별"),       // 1점 - 일반 (단독 사용시 낮은 점수)
                ],
                keywords: vec![
                    "월별".to_string(),
                    "생산량".to_string(),
                    "생산추이".to_string(),
                    "생산트렌드".to_string(),
                    "월별생산".to_string(),
                ],
                priority: 4,
                chart_config: ChartConfig::new("line", "글로우 효과 + 부드러운 곡선", "월별 생산량 추이"),
                data_source: "fg_lot + production_order (월별 qty 합계)".to_string(),
                template: include_str!("../../templates/monthly_production_trend.txt").to_string(),
            },
        );

        // 5. 매출 트렌드 (priority: 5)
        // "매출추이" 또는 "매출트렌드" 복합어가 있어야 이 템플릿 선택
        // 단독 "추이"/"트렌드"는 다른 템플릿(출고_추이, 입고_추이 등)과 충돌하므로 제거
        templates.insert(
            "매출_트렌드".to_string(),
            PromptTemplate {
                weighted_keywords: vec![
                    WeightedKeyword::core("매출추이"),      // 3점 - 핵심 복합어
                    WeightedKeyword::core("매출트렌드"),    // 3점 - 핵심 복합어
                    WeightedKeyword::important("매출"),     // 2점 - 중요
                    WeightedKeyword::important("월별매출"), // 2점 - 중요 복합어
                    WeightedKeyword::general("매출현황"),   // 1점 - 일반
                ],
                keywords: vec![
                    "매출".to_string(),
                    "매출추이".to_string(),
                    "매출트렌드".to_string(),
                    "월별매출".to_string(),
                ],
                priority: 5,
                chart_config: ChartConfig::new("line", "글로우 효과 + 부드러운 곡선", "매출 트렌드 분석"),
                data_source: "sales_order + sales_order_dtl (일별/주별 금액)".to_string(),
                template: include_str!("../../templates/sales_trend.txt").to_string(),
            },
        );

        // 6. 온도 변화 (priority: 6)
        templates.insert(
            "온도_변화".to_string(),
            PromptTemplate {
                weighted_keywords: vec![
                    WeightedKeyword::core("온도"),          // 3점 - 핵심
                    WeightedKeyword::important("변화"),     // 2점 - 중요
                    WeightedKeyword::important("살균"),     // 2점 - 중요 (도메인 용어)
                    WeightedKeyword::general("온도추이"),   // 1점 - 복합어
                    WeightedKeyword::general("온도변화"),   // 1점 - 복합어
                    WeightedKeyword::general("살균온도"),   // 1점 - 복합어
                ],
                keywords: vec![
                    "온도".to_string(),
                    "변화".to_string(),
                    "온도추이".to_string(),
                    "온도변화".to_string(),
                    "살균".to_string(),
                    "살균온도".to_string(),
                ],
                priority: 6,
                chart_config: ChartConfig::new("line", "글로우 효과 + 부드러운 곡선", "공정 온도 변화 추이"),
                data_source: "ccp_check_log + operation_param_log".to_string(),
                template: include_str!("../../templates/temperature_trend.txt").to_string(),
            },
        );

        // ==================== PIE 차트 ====================

        // 7. 품질검사 분포 (priority: 7)
        templates.insert(
            "품질검사_분포".to_string(),
            PromptTemplate {
                weighted_keywords: vec![
                    WeightedKeyword::core("품질검사"),      // 3점 - 핵심
                    WeightedKeyword::core("품질"),         // 3점 - 핵심
                    WeightedKeyword::important("분포"),    // 2점 - 중요 (차트 타입 힌트)
                    WeightedKeyword::important("검사결과"), // 2점 - 중요
                    WeightedKeyword::exact("QC", 2),       // 2점 - 정확 매칭 (약어)
                    WeightedKeyword::general("검사"),      // 1점 - 일반
                ],
                keywords: vec![
                    "품질검사".to_string(),
                    "품질".to_string(),
                    "검사결과".to_string(),
                    "QC".to_string(),
                    "검사".to_string(),
                ],
                priority: 7,
                chart_config: ChartConfig::new("pie", "도넛 스타일 + 그림자", "품질검사 결과 분포"),
                data_source: "qc_result + qc_spec_mst".to_string(),
                template: include_str!("../../templates/quality_inspection.txt").to_string(),
            },
        );

        // 8. 창고별 재고 (priority: 8)
        templates.insert(
            "창고별_재고".to_string(),
            PromptTemplate {
                weighted_keywords: vec![
                    WeightedKeyword::core("창고별"),       // 3점 - 핵심 구분자
                    WeightedKeyword::core("재고"),         // 3점 - 핵심
                    WeightedKeyword::important("비율"),    // 2점 - 중요 (차트 타입 힌트)
                    WeightedKeyword::general("재고비율"),  // 1점 - 복합어
                    WeightedKeyword::general("재고현황"),  // 1점 - 일반
                    WeightedKeyword::general("창고"),      // 1점 - 일반
                ],
                keywords: vec![
                    "창고별".to_string(),
                    "재고".to_string(),
                    "재고비율".to_string(),
                    "재고현황".to_string(),
                    "창고".to_string(),
                ],
                priority: 8,
                chart_config: ChartConfig::new("pie", "도넛 스타일 + 그림자", "창고별 재고 비율"),
                data_source: "warehouse_stock + warehouse_mst + item_mst".to_string(),
                template: include_str!("../../templates/warehouse_inventory.txt").to_string(),
            },
        );

        // 9. CCP 유형별 (priority: 9)
        templates.insert(
            "CCP_유형별".to_string(),
            PromptTemplate {
                weighted_keywords: vec![
                    WeightedKeyword::exact("CCP", 3),      // 3점 - 핵심 (정확 매칭으로 ACCEPT 등 방지)
                    WeightedKeyword::core("유형별"),       // 3점 - 핵심 구분자
                    WeightedKeyword::important("유형"),    // 2점 - 중요
                    WeightedKeyword::general("CCP현황"),   // 1점 - 복합어
                    WeightedKeyword::general("중요관리점"), // 1점 - 도메인 용어
                ],
                keywords: vec![
                    "CCP".to_string(),
                    "유형별".to_string(),
                    "CCP현황".to_string(),
                    "중요관리점".to_string(),
                ],
                priority: 9,
                chart_config: ChartConfig::new("pie", "도넛 스타일 + 그림자", "CCP 유형별 현황"),
                data_source: "ccp_check_log + ccp_master".to_string(),
                template: include_str!("../../templates/ccp_types.txt").to_string(),
            },
        );

        // ==================== GAUGE 차트 ====================

        // 10. 전체 가동률 (priority: 10)
        templates.insert(
            "전체_가동률".to_string(),
            PromptTemplate {
                weighted_keywords: vec![
                    WeightedKeyword::core("가동률"),       // 3점 - 핵심
                    WeightedKeyword::exact("OEE", 3),     // 3점 - 핵심 약어 (정확 매칭)
                    WeightedKeyword::important("전체"),   // 2점 - 중요
                    WeightedKeyword::general("전체가동률"), // 1점 - 복합어
                    WeightedKeyword::general("설비효율"),  // 1점 - 동의어
                ],
                keywords: vec![
                    "가동률".to_string(),
                    "전체가동률".to_string(),
                    "OEE".to_string(),
                    "설비효율".to_string(),
                ],
                priority: 10,
                chart_config: ChartConfig::new("gauge", "그라데이션 반원 게이지", "전체 설비종합효율(OEE)"),
                data_source: "operation_exec + downtime_event + line_mst".to_string(),
                template: include_str!("../../templates/oee_gauge.txt").to_string(),
            },
        );

        // 11. CCP 합격률 (priority: 11)
        // 주의: "CCP_유형별"과 구분 필요 - "합격률" 키워드로 구분
        templates.insert(
            "CCP_합격률".to_string(),
            PromptTemplate {
                weighted_keywords: vec![
                    WeightedKeyword::core("합격률"),       // 3점 - 핵심 구분자!
                    WeightedKeyword::exact("CCP", 3),     // 3점 - 핵심 (정확 매칭)
                    WeightedKeyword::exact("HACCP", 2),   // 2점 - 중요 (정확 매칭)
                    WeightedKeyword::general("CCP합격"),   // 1점 - 복합어
                    WeightedKeyword::general("CCP합격률"), // 1점 - 복합어
                ],
                keywords: vec![
                    "CCP".to_string(),
                    "합격률".to_string(),
                    "CCP합격".to_string(),
                    "CCP합격률".to_string(),
                    "HACCP".to_string(),
                ],
                priority: 11,
                chart_config: ChartConfig::new("gauge", "위험도 표시 게이지", "CCP 합격률 (HACCP 핵심관리)"),
                data_source: "ccp_check_log + ccp_master".to_string(),
                template: include_str!("../../templates/ccp_pass_rate.txt").to_string(),
            },
        );

        // 12. 평균 불량률 (priority: 12)
        templates.insert(
            "평균_불량률".to_string(),
            PromptTemplate {
                weighted_keywords: vec![
                    WeightedKeyword::core("불량률"),       // 3점 - 핵심
                    WeightedKeyword::core("불량"),         // 3점 - 핵심
                    WeightedKeyword::important("수율"),   // 2점 - 중요 (반대 개념)
                    WeightedKeyword::general("평균불량률"), // 1점 - 복합어
                    WeightedKeyword::general("스크랩"),    // 1점 - 도메인 용어
                ],
                keywords: vec![
                    "불량률".to_string(),
                    "평균불량률".to_string(),
                    "불량".to_string(),
                    "스크랩".to_string(),
                    "수율".to_string(),
                ],
                priority: 12,
                chart_config: ChartConfig::new("gauge", "역방향 게이지 (낮을수록 좋음)", "평균 불량률 현황"),
                data_source: "operation_exec + mes_work_order + line_mst".to_string(),
                template: include_str!("../../templates/defect_rate.txt").to_string(),
            },
        );

        // ==================== Phase 3: 12개 신규 템플릿 (DB 커버리지 확장) ====================

        // 13. 생산계획 달성률 (priority: 13) - GAUGE
        templates.insert(
            "생산계획_달성률".to_string(),
            PromptTemplate {
                weighted_keywords: vec![
                    WeightedKeyword::core("계획달성률"),     // 3점 - 핵심 복합어
                    WeightedKeyword::core("달성률"),        // 3점 - 핵심
                    WeightedKeyword::core("생산달성률"),     // 3점 - 핵심 복합어
                    WeightedKeyword::important("생산계획"),  // 2점 - 중요
                    WeightedKeyword::important("목표대비"),  // 2점 - 중요
                    WeightedKeyword::general("계획"),       // 1점 - 일반
                    WeightedKeyword::general("실적"),       // 1점 - 일반
                ],
                keywords: vec![
                    "계획달성률".to_string(),
                    "달성률".to_string(),
                    "생산달성률".to_string(),
                    "생산계획".to_string(),
                    "목표대비".to_string(),
                    "계획".to_string(),
                    "실적".to_string(),
                ],
                priority: 13,
                chart_config: ChartConfig::new("gauge", "반원형 게이지 + 목표선", "생산계획 달성률"),
                data_source: "production_order (plan_qty vs actual_qty)".to_string(),
                template: include_str!("../../templates/production_achievement.txt").to_string(),
            },
        );

        // 14. 공급업체별 납품현황 (priority: 14) - BAR (horizontal)
        templates.insert(
            "공급업체별_납품현황".to_string(),
            PromptTemplate {
                weighted_keywords: vec![
                    WeightedKeyword::core("공급업체별"),     // 3점 - 핵심 구분자
                    WeightedKeyword::core("납품현황"),      // 3점 - 핵심
                    WeightedKeyword::core("납품"),         // 3점 - 핵심
                    WeightedKeyword::important("공급업체"),  // 2점 - 중요
                    WeightedKeyword::important("입고현황"),  // 2점 - 중요
                    WeightedKeyword::important("벤더"),     // 2점 - 중요 (영어)
                    WeightedKeyword::general("거래처"),     // 1점 - 일반
                    WeightedKeyword::general("납기"),       // 1점 - 일반
                ],
                keywords: vec![
                    "공급업체별".to_string(),
                    "납품현황".to_string(),
                    "납품".to_string(),
                    "공급업체".to_string(),
                    "입고현황".to_string(),
                    "벤더".to_string(),
                    "거래처".to_string(),
                    "납기".to_string(),
                ],
                priority: 14,
                chart_config: ChartConfig::new("bar", "수평 그라데이션 + 납기 준수율 표시", "공급업체별 납품현황"),
                data_source: "vendor_mst + inbound + inbound_dtl + purchase_order".to_string(),
                template: include_str!("../../templates/vendor_delivery.txt").to_string(),
            },
        );

        // 15. 출고 현황 (priority: 15) - BAR (vertical, grouped)
        templates.insert(
            "출고_현황".to_string(),
            PromptTemplate {
                weighted_keywords: vec![
                    WeightedKeyword::core("출고현황"),       // 3점 - 핵심 복합어
                    WeightedKeyword::core("고객별출고"),     // 3점 - 핵심 복합어
                    WeightedKeyword::important("출고"),      // 2점 - 중요
                    WeightedKeyword::important("배송현황"),   // 2점 - 중요
                    WeightedKeyword::general("배송"),        // 1점 - 일반
                    WeightedKeyword::general("납품"),        // 1점 - 일반 (출고 맥락)
                ],
                keywords: vec![
                    "출고현황".to_string(),
                    "고객별출고".to_string(),
                    "출고".to_string(),
                    "배송현황".to_string(),
                    "배송".to_string(),
                ],
                priority: 15,
                chart_config: ChartConfig::new("bar", "그라데이션 + 그룹화 (상태별)", "출고 현황"),
                data_source: "outbound + outbound_dtl + customer_mst".to_string(),
                template: include_str!("../../templates/outbound_status.txt").to_string(),
            },
        );

        // 16. 알람 발생현황 (priority: 16) - PIE (도넛)
        templates.insert(
            "알람_발생현황".to_string(),
            PromptTemplate {
                weighted_keywords: vec![
                    WeightedKeyword::core("알람현황"),       // 3점 - 핵심 복합어
                    WeightedKeyword::core("알람발생"),       // 3점 - 핵심 복합어
                    WeightedKeyword::core("경보현황"),       // 3점 - 핵심 동의어
                    WeightedKeyword::important("알람"),      // 2점 - 중요
                    WeightedKeyword::important("경고"),      // 2점 - 중요
                    WeightedKeyword::important("이벤트"),    // 2점 - 중요
                    WeightedKeyword::general("알림"),        // 1점 - 일반
                    WeightedKeyword::general("통지"),        // 1점 - 일반
                ],
                keywords: vec![
                    "알람현황".to_string(),
                    "알람발생".to_string(),
                    "경보현황".to_string(),
                    "알람".to_string(),
                    "경고".to_string(),
                    "이벤트".to_string(),
                    "알림".to_string(),
                ],
                priority: 16,
                chart_config: ChartConfig::new("pie", "도넛 스타일 + 심각도별 색상", "알람 발생현황"),
                data_source: "alarm_event (alarm_level, alarm_type, is_resolved)".to_string(),
                template: include_str!("../../templates/alarm_status.txt").to_string(),
            },
        );

        // 17. 점검 이행률 (priority: 17) - GAUGE (스피도미터)
        templates.insert(
            "점검_이행률".to_string(),
            PromptTemplate {
                weighted_keywords: vec![
                    WeightedKeyword::core("점검이행률"),      // 3점 - 핵심 복합어
                    WeightedKeyword::core("체크리스트이행"),   // 3점 - 핵심 복합어
                    WeightedKeyword::important("점검현황"),   // 2점 - 중요
                    WeightedKeyword::important("이행률"),     // 2점 - 중요
                    WeightedKeyword::general("점검"),        // 1점 - 일반
                    WeightedKeyword::general("체크"),        // 1점 - 일반
                ],
                keywords: vec![
                    "점검이행률".to_string(),
                    "체크리스트이행".to_string(),
                    "점검현황".to_string(),
                    "이행률".to_string(),
                    "점검".to_string(),
                    "체크".to_string(),
                ],
                priority: 17,
                chart_config: ChartConfig::new("gauge", "스피도미터 + 100% 목표선", "점검 이행률"),
                data_source: "checklist_result (checklist_type, overall_result)".to_string(),
                template: include_str!("../../templates/checklist_compliance.txt").to_string(),
            },
        );

        // 18. 작업자별 생산성 (priority: 18) - BAR (horizontal)
        templates.insert(
            "작업자별_생산성".to_string(),
            PromptTemplate {
                weighted_keywords: vec![
                    WeightedKeyword::core("작업자별"),       // 3점 - 핵심 구분자
                    WeightedKeyword::core("생산성"),        // 3점 - 핵심
                    WeightedKeyword::core("작업자생산성"),    // 3점 - 핵심 복합어
                    WeightedKeyword::important("작업자"),    // 2점 - 중요
                    WeightedKeyword::important("인력별"),    // 2점 - 중요
                    WeightedKeyword::general("작업실적"),    // 1점 - 일반
                    WeightedKeyword::general("개인실적"),    // 1점 - 일반
                ],
                keywords: vec![
                    "작업자별".to_string(),
                    "생산성".to_string(),
                    "작업자생산성".to_string(),
                    "작업자".to_string(),
                    "인력별".to_string(),
                    "작업실적".to_string(),
                    "개인실적".to_string(),
                ],
                priority: 18,
                chart_config: ChartConfig::new("bar", "수평 그라데이션 + 목표선", "작업자별 생산성"),
                data_source: "operator_mst + operation_exec".to_string(),
                template: include_str!("../../templates/operator_productivity.txt").to_string(),
            },
        );

        // 19. 교대별 실적 (priority: 19) - BAR (grouped, 3색)
        templates.insert(
            "교대별_실적".to_string(),
            PromptTemplate {
                weighted_keywords: vec![
                    WeightedKeyword::core("교대별"),         // 3점 - 핵심 구분자
                    WeightedKeyword::core("교대실적"),       // 3점 - 핵심 복합어
                    WeightedKeyword::core("시프트별"),       // 3점 - 핵심 (영어 용어)
                    WeightedKeyword::important("주간"),      // 2점 - 중요
                    WeightedKeyword::important("야간"),      // 2점 - 중요
                    WeightedKeyword::important("교대"),      // 2점 - 중요
                    WeightedKeyword::general("근무별"),      // 1점 - 일반
                    WeightedKeyword::general("조별"),        // 1점 - 일반
                ],
                keywords: vec![
                    "교대별".to_string(),
                    "교대실적".to_string(),
                    "시프트별".to_string(),
                    "주간".to_string(),
                    "야간".to_string(),
                    "교대".to_string(),
                    "근무별".to_string(),
                    "조별".to_string(),
                ],
                priority: 19,
                chart_config: ChartConfig::new("bar", "3색 그룹 (주간/야간/교대)", "교대별 실적"),
                data_source: "shift_mst + mes_work_order".to_string(),
                template: include_str!("../../templates/shift_performance.txt").to_string(),
            },
        );

        // 20. 입고 추이 (priority: 20) - LINE (이중 축)
        templates.insert(
            "입고_추이".to_string(),
            PromptTemplate {
                weighted_keywords: vec![
                    WeightedKeyword::core("입고추이"),       // 3점 - 핵심 복합어
                    WeightedKeyword::core("입고트렌드"),      // 3점 - 핵심 복합어
                    WeightedKeyword::important("월별입고"),   // 2점 - 중요
                    WeightedKeyword::important("입고현황"),   // 2점 - 중요
                    WeightedKeyword::general("입고"),        // 1점 - 일반
                    WeightedKeyword::general("수입"),        // 1점 - 일반 (입고 동의어)
                ],
                keywords: vec![
                    "입고추이".to_string(),
                    "입고트렌드".to_string(),
                    "월별입고".to_string(),
                    "입고현황".to_string(),
                    "입고".to_string(),
                ],
                priority: 20,
                chart_config: ChartConfig::new("line", "부드러운 곡선 + 면적 채우기 + 이중 축", "입고 추이"),
                data_source: "inbound + inbound_dtl + vendor_mst".to_string(),
                template: include_str!("../../templates/inbound_trend.txt").to_string(),
            },
        );

        // 21. 출고 추이 (priority: 21) - LINE (이중 축)
        templates.insert(
            "출고_추이".to_string(),
            PromptTemplate {
                weighted_keywords: vec![
                    WeightedKeyword::core("출고추이"),       // 3점 - 핵심 복합어
                    WeightedKeyword::core("출고트렌드"),      // 3점 - 핵심 복합어
                    WeightedKeyword::important("월별출고"),   // 2점 - 중요
                    WeightedKeyword::important("배송추이"),   // 2점 - 중요
                    WeightedKeyword::general("물류추이"),    // 1점 - 일반
                ],
                keywords: vec![
                    "출고추이".to_string(),
                    "출고트렌드".to_string(),
                    "월별출고".to_string(),
                    "배송추이".to_string(),
                    "물류추이".to_string(),
                ],
                priority: 21,
                chart_config: ChartConfig::new("line", "이중 축 (수량 + 금액) + 부드러운 곡선", "출고 추이"),
                data_source: "outbound + outbound_dtl + customer_mst".to_string(),
                template: include_str!("../../templates/outbound_trend.txt").to_string(),
            },
        );

        // 22. BOM 원자재 소요량 (priority: 22) - PIE (도넛)
        templates.insert(
            "BOM_원자재_소요량".to_string(),
            PromptTemplate {
                weighted_keywords: vec![
                    WeightedKeyword::core("원자재소요"),      // 3점 - 핵심 복합어
                    WeightedKeyword::exact("BOM", 3),       // 3점 - 핵심 (정확 매칭)
                    WeightedKeyword::core("자재소요"),       // 3점 - 핵심 복합어
                    WeightedKeyword::core("BOM분석"),       // 3점 - 핵심 복합어
                    WeightedKeyword::important("원자재"),    // 2점 - 중요
                    WeightedKeyword::important("소요량"),    // 2점 - 중요
                    WeightedKeyword::general("자재"),        // 1점 - 일반
                    WeightedKeyword::general("배합"),        // 1점 - 일반 (음료 산업)
                ],
                keywords: vec![
                    "원자재소요".to_string(),
                    "BOM".to_string(),
                    "자재소요".to_string(),
                    "BOM분석".to_string(),
                    "원자재".to_string(),
                    "소요량".to_string(),
                    "자재".to_string(),
                    "배합".to_string(),
                ],
                priority: 22,
                chart_config: ChartConfig::new("pie", "도넛 스타일 + 품목별 비율", "BOM 원자재 소요량"),
                data_source: "material_issue + item_mst + bom_mst + bom_dtl + inventory".to_string(),
                template: include_str!("../../templates/bom_material_usage.txt").to_string(),
            },
        );

        // 23. 재고이동 현황 (priority: 23) - BAR (stacked)
        templates.insert(
            "재고이동_현황".to_string(),
            PromptTemplate {
                weighted_keywords: vec![
                    WeightedKeyword::core("재고이동"),       // 3점 - 핵심 복합어
                    WeightedKeyword::core("이동현황"),       // 3점 - 핵심 복합어
                    WeightedKeyword::important("입출고"),    // 2점 - 중요
                    WeightedKeyword::important("재고변동"),   // 2점 - 중요
                    WeightedKeyword::general("이동"),        // 1점 - 일반
                    WeightedKeyword::general("변동"),        // 1점 - 일반
                ],
                keywords: vec![
                    "재고이동".to_string(),
                    "이동현황".to_string(),
                    "입출고".to_string(),
                    "재고변동".to_string(),
                    "이동".to_string(),
                    "변동".to_string(),
                ],
                priority: 23,
                chart_config: ChartConfig::new("bar", "스택 바 + 이동유형별 색상 (IN/OUT/TRANSFER/ADJUST/SCRAP)", "재고이동 현황"),
                data_source: "inventory_movement + warehouse_mst + item_mst".to_string(),
                template: include_str!("../../templates/inventory_movement.txt").to_string(),
            },
        );

        // 24. 금속검출 이력 (priority: 24) - LINE (이벤트 마커)
        templates.insert(
            "금속검출_이력".to_string(),
            PromptTemplate {
                weighted_keywords: vec![
                    WeightedKeyword::core("금속검출"),       // 3점 - 핵심
                    WeightedKeyword::core("검출이력"),       // 3점 - 핵심 복합어
                    WeightedKeyword::important("금검"),      // 2점 - 중요 (약어)
                    WeightedKeyword::important("이물검출"),   // 2점 - 중요
                    WeightedKeyword::general("검출"),        // 1점 - 일반
                    WeightedKeyword::general("이물"),        // 1점 - 일반
                ],
                keywords: vec![
                    "금속검출".to_string(),
                    "검출이력".to_string(),
                    "금검".to_string(),
                    "이물검출".to_string(),
                    "검출".to_string(),
                    "이물".to_string(),
                ],
                priority: 24,
                chart_config: ChartConfig::new("line", "검출 이벤트 마커 + HACCP CCP2 표시", "금속검출 이력"),
                data_source: "metal_detection_log + line_mst + item_mst".to_string(),
                template: include_str!("../../templates/metal_detection_history.txt").to_string(),
            },
        );

        Self { templates }
    }

    /// 단어 경계를 고려한 정확 매칭 검사
    ///
    /// 단순 포함 매칭과 달리, 단어가 다른 단어의 일부가 아닌지 확인합니다.
    /// 예: "CCP"가 "ACCEPT"의 일부가 아닌 독립적인 단어인지 확인
    fn exact_word_match(text: &str, word: &str) -> bool {
        let text_lower = text.to_lowercase();
        let word_lower = word.to_lowercase();

        // 정확히 같은 경우
        if text_lower == word_lower {
            return true;
        }

        // 단어 경계 패턴: 공백, 문장부호, 시작/끝
        let patterns = [
            format!(" {} ", word_lower),  // 중간
            format!(" {}", word_lower),    // 끝 또는 마지막 단어
            format!("{} ", word_lower),    // 시작 또는 첫 단어
        ];

        patterns.iter().any(|p| text_lower.contains(p))
            || text_lower.starts_with(&format!("{} ", word_lower))
            || text_lower.ends_with(&format!(" {}", word_lower))
            || text_lower.starts_with(&word_lower) && text_lower.len() == word_lower.len()
    }

    /// 입력 질문에서 키워드를 매칭하여 템플릿 키 반환 (개선된 알고리즘)
    ///
    /// ## Phase 2 개선사항
    /// - 가중치 기반 점수 계산 (core: 3점, important: 2점, general: 1점)
    /// - 다중 조건 정렬: 총점 > 매칭 개수 > 우선순위
    /// - 정확 매칭 지원 (exact_match: true)
    /// - 공백 제거 정규화로 "라인별생산량" == "라인별 생산량" 처리
    pub fn find_matching_template(&self, query: &str) -> Option<String> {
        let query_lower = query.to_lowercase();
        let query_normalized = query_lower.replace(' ', "");

        let mut matches: Vec<MatchResult> = Vec::new();

        for (template_key, template) in &self.templates {
            let mut total_score = 0u32;
            let mut match_count = 0usize;

            for wk in &template.weighted_keywords {
                let keyword_lower = wk.keyword.to_lowercase();
                let keyword_normalized = keyword_lower.replace(' ', "");

                let matched = if wk.exact_match {
                    // 정확 매칭: 단어 경계 체크
                    Self::exact_word_match(&query_lower, &keyword_lower)
                } else {
                    // 포함 매칭: 원본, 소문자, 정규화 버전 모두 확인
                    query.contains(&wk.keyword)
                        || query_lower.contains(&keyword_lower)
                        || query_normalized.contains(&keyword_normalized)
                };

                if matched {
                    total_score += wk.weight;
                    match_count += 1;
                }
            }

            if total_score > 0 {
                matches.push(MatchResult {
                    template_key: template_key.clone(),
                    total_score,
                    match_count,
                    priority: template.priority,
                });
            }
        }

        // 다중 조건 정렬: 점수(내림차순) > 매칭 개수(내림차순) > 우선순위(오름차순)
        matches.sort_by(|a, b| {
            b.total_score
                .cmp(&a.total_score)
                .then_with(|| b.match_count.cmp(&a.match_count))
                .then_with(|| a.priority.cmp(&b.priority))
        });

        matches.first().map(|m| m.template_key.clone())
    }

    /// 입력 질문에서 키워드를 매칭하여 템플릿 키 반환 (레거시 - 하위 호환성)
    #[allow(dead_code)]
    pub fn find_matching_template_legacy(&self, query: &str) -> Option<String> {
        let query_lower = query.to_lowercase().replace(' ', "");

        let mut best_match: Option<String> = None;
        let mut best_score = 0;

        for (template_key, template) in &self.templates {
            let mut score = 0;

            for keyword in &template.keywords {
                let keyword_lower = keyword.to_lowercase();
                if query.contains(keyword) || query_lower.contains(&keyword_lower) {
                    score += 1;
                }
            }

            if score > best_score {
                best_score = score;
                best_match = Some(template_key.clone());
            }
        }

        if best_score > 0 {
            best_match
        } else {
            None
        }
    }

    /// 질문을 라우팅하여 확장된 프롬프트 결과 반환
    pub fn route(&self, query: &str) -> Option<PromptRoute> {
        let template_key = self.find_matching_template(query)?;
        let template = self.templates.get(&template_key)?;

        // 매칭된 키워드 추출
        let query_lower = query.to_lowercase();
        let matched_keywords: Vec<String> = template
            .keywords
            .iter()
            .filter(|k| query.contains(k.as_str()) || query_lower.contains(&k.to_lowercase()))
            .cloned()
            .collect();

        Some(PromptRoute {
            template_key,
            matched_keywords,
            chart_config: template.chart_config.clone(),
            data_source: template.data_source.clone(),
            expanded_prompt: template.template.clone(),
        })
    }

    /// 최종 프롬프트 생성 (원본 질문 + 확장 프롬프트)
    pub fn get_final_prompt(&self, query: &str) -> String {
        match self.route(query) {
            Some(route) => {
                format!(
                    r#"[사용자 질문]
{}

[차트 설정]
- 유형: {}
- 스타일: {}
- 제목: {}

[데이터 소스]
{}

{}
"#,
                    query,
                    route.chart_config.chart_type,
                    route.chart_config.style,
                    route.chart_config.title,
                    route.data_source,
                    route.expanded_prompt
                )
            }
            None => {
                format!(
                    r#"[사용자 질문]
{}

[안내]
죄송합니다. 해당 질문에 대한 차트 템플릿을 찾지 못했습니다.
다음과 같은 질문을 시도해보세요:
- "라인별 생산량 보여줘"
- "월별 매출 현황"
- "CCP 합격률"
- "전체 가동률 보여줘"
- "창고별 재고 현황"
"#,
                    query
                )
            }
        }
    }

    /// 템플릿 키 목록 반환
    pub fn get_template_keys(&self) -> Vec<String> {
        self.templates.keys().cloned().collect()
    }

    /// 특정 템플릿의 차트 설정 반환
    pub fn get_chart_config(&self, template_key: &str) -> Option<&ChartConfig> {
        self.templates.get(template_key).map(|t| &t.chart_config)
    }
}

impl Default for PromptRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_matching_template() {
        let router = PromptRouter::new();

        // BAR 차트 매칭 테스트
        assert_eq!(
            router.find_matching_template("라인별 생산량 보여줘"),
            Some("라인별_생산량".to_string())
        );
        assert_eq!(
            router.find_matching_template("월별 매출 현황"),
            Some("월별_매출".to_string())
        );

        // LINE 차트 매칭 테스트
        assert_eq!(
            router.find_matching_template("매출 트렌드 분석해줘"),
            Some("매출_트렌드".to_string())
        );

        // PIE 차트 매칭 테스트
        assert_eq!(
            router.find_matching_template("창고별 재고 비율"),
            Some("창고별_재고".to_string())
        );

        // GAUGE 차트 매칭 테스트
        assert_eq!(
            router.find_matching_template("전체 가동률 보여줘"),
            Some("전체_가동률".to_string())
        );
        assert_eq!(
            router.find_matching_template("CCP 합격률"),
            Some("CCP_합격률".to_string())
        );

        // 매칭 없는 경우
        assert_eq!(router.find_matching_template("안녕하세요"), None);
    }

    #[test]
    fn test_route() {
        let router = PromptRouter::new();

        let result = router.route("라인별 생산량 보여줘");
        assert!(result.is_some());

        let route = result.unwrap();
        assert_eq!(route.template_key, "라인별_생산량");
        assert_eq!(route.chart_config.chart_type, "bar");
        assert!(!route.matched_keywords.is_empty());
    }

    #[test]
    fn test_get_final_prompt() {
        let router = PromptRouter::new();

        let prompt = router.get_final_prompt("라인별 생산량 보여줘");
        assert!(prompt.contains("[사용자 질문]"));
        assert!(prompt.contains("라인별 생산량"));
        assert!(prompt.contains("[차트 설정]"));

        // 매칭 실패 시 안내 메시지
        let prompt_no_match = router.get_final_prompt("안녕하세요");
        assert!(prompt_no_match.contains("템플릿을 찾지 못했습니다"));
    }

    // ========== Phase 2 개선 테스트 케이스 ==========

    /// P0-1: 동점 처리 - 우선순위에 따른 결정적 결과
    #[test]
    fn test_tiebreaker_priority() {
        let router = PromptRouter::new();

        // "생산량" 키워드 하나만 있을 때 - 더 높은 점수의 라인별_생산량이 선택되어야 함
        // 라인별_생산량: 생산량(3점) = 3점, priority=1
        // 월별_생산량_추이: 생산량(2점) = 2점, priority=4
        let result = router.find_matching_template("생산량 현황");
        assert!(result.is_some());
        // 점수가 더 높은 라인별_생산량 선택
        assert_eq!(result.unwrap(), "라인별_생산량");
    }

    /// P1-1: 가중치 기반 점수 계산 테스트
    #[test]
    fn test_weighted_scoring() {
        let router = PromptRouter::new();

        // "라인별 생산량 보여줘" 테스트
        // 라인별_생산량: 라인별(3) + 생산량(3) + 라인(2) = 8점, priority=1
        // → 라인별_생산량 선택 (가장 높은 점수)
        let result = router.find_matching_template("라인별 생산량 보여줘");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "라인별_생산량");

        // "생산추이 분석" 테스트 - 명확하게 월별_생산량_추이 선택
        // 월별_생산량_추이: 생산추이(3) = 3점, priority=4
        // 라인별_생산량: 매칭 없음
        // → 월별_생산량_추이 선택
        let result2 = router.find_matching_template("생산추이 분석해줘");
        assert!(result2.is_some());
        assert_eq!(result2.unwrap(), "월별_생산량_추이");

        // "월별생산 생산량 분석" 테스트
        // 월별_생산량_추이: 월별생산(2) + 생산량(2) = 4점, priority=4
        // 라인별_생산량: 생산량(3) = 3점, priority=1
        // → 월별_생산량_추이 선택 (점수 더 높음)
        let result3 = router.find_matching_template("월별생산 생산량 분석");
        assert!(result3.is_some());
        assert_eq!(result3.unwrap(), "월별_생산량_추이");
    }

    /// P1-1: 핵심 키워드(weight=3)가 일반 키워드(weight=1)보다 우선
    #[test]
    fn test_core_keyword_priority() {
        let router = PromptRouter::new();

        // "가동률" 핵심 키워드(3점) vs 다른 일반 키워드
        let result = router.find_matching_template("전체 가동률 현황 알려줘");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "전체_가동률");

        // "CCP" 핵심 키워드 테스트
        let result2 = router.find_matching_template("CCP 유형별 분포");
        assert!(result2.is_some());
        assert_eq!(result2.unwrap(), "CCP_유형별");
    }

    /// P0-1: 동일 점수일 때 match_count와 priority로 결정
    #[test]
    fn test_match_count_tiebreaker() {
        let router = PromptRouter::new();

        // 여러 키워드가 매칭될 때 - match_count가 더 많은 것 선택
        // "온도 변화 추이" - 온도_변화 템플릿이 매칭되어야 함
        let result = router.find_matching_template("온도 변화 추이 보여줘");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "온도_변화");
    }

    /// 다양한 차트 유형 매칭 검증
    #[test]
    fn test_chart_type_matching() {
        let router = PromptRouter::new();

        // BAR 차트 (3개)
        let bar1 = router.route("라인별 생산량");
        assert!(bar1.is_some());
        assert_eq!(bar1.unwrap().chart_config.chart_type, "bar");

        let bar2 = router.route("월별 매출액 현황");
        assert!(bar2.is_some());
        assert_eq!(bar2.unwrap().chart_config.chart_type, "bar");

        let bar3 = router.route("설비별 비가동 현황");
        assert!(bar3.is_some());
        assert_eq!(bar3.unwrap().chart_config.chart_type, "bar");

        // LINE 차트 (3개)
        // "생산추이"는 월별_생산량_추이의 핵심 키워드
        let line1 = router.route("생산추이 분석");
        assert!(line1.is_some());
        assert_eq!(line1.unwrap().chart_config.chart_type, "line");

        // "매출추이"는 매출_트렌드의 핵심 키워드
        let line2 = router.route("매출추이 분석");
        assert!(line2.is_some());
        assert_eq!(line2.unwrap().chart_config.chart_type, "line");

        let line3 = router.route("온도 변화 추이");
        assert!(line3.is_some());
        assert_eq!(line3.unwrap().chart_config.chart_type, "line");

        // PIE 차트 (3개)
        let pie1 = router.route("품질검사 분포");
        assert!(pie1.is_some());
        assert_eq!(pie1.unwrap().chart_config.chart_type, "pie");

        let pie2 = router.route("창고별 재고 비율");
        assert!(pie2.is_some());
        assert_eq!(pie2.unwrap().chart_config.chart_type, "pie");

        let pie3 = router.route("CCP 유형별 분포");
        assert!(pie3.is_some());
        assert_eq!(pie3.unwrap().chart_config.chart_type, "pie");

        // GAUGE 차트 (3개)
        let gauge1 = router.route("전체 가동률");
        assert!(gauge1.is_some());
        assert_eq!(gauge1.unwrap().chart_config.chart_type, "gauge");

        let gauge2 = router.route("CCP 합격률");
        assert!(gauge2.is_some());
        assert_eq!(gauge2.unwrap().chart_config.chart_type, "gauge");

        let gauge3 = router.route("불량률 현황");
        assert!(gauge3.is_some());
        assert_eq!(gauge3.unwrap().chart_config.chart_type, "gauge");
    }

    /// 공백 처리 및 정규화 테스트
    #[test]
    fn test_query_normalization() {
        let router = PromptRouter::new();

        // 공백 포함 쿼리
        assert_eq!(
            router.find_matching_template("라 인 별  생 산 량"),
            Some("라인별_생산량".to_string())
        );

        // 대소문자 혼용 (한글은 대소문자 없지만 영어 키워드 테스트)
        assert_eq!(
            router.find_matching_template("CCP 합격률"),
            Some("CCP_합격률".to_string())
        );
        assert_eq!(
            router.find_matching_template("ccp 합격률"),
            Some("CCP_합격률".to_string())
        );
    }

    /// 매칭되지 않는 쿼리 테스트
    #[test]
    fn test_no_match_queries() {
        let router = PromptRouter::new();

        // 완전히 관련 없는 쿼리
        assert_eq!(router.find_matching_template("안녕하세요"), None);
        assert_eq!(router.find_matching_template("오늘 날씨 어때?"), None);
        assert_eq!(router.find_matching_template("점심 뭐 먹을까"), None);
        assert_eq!(router.find_matching_template(""), None);
        assert_eq!(router.find_matching_template("   "), None);
    }

    /// WeightedKeyword 생성자 테스트
    #[test]
    fn test_weighted_keyword_constructors() {
        // core: weight=3
        let core = WeightedKeyword::core("테스트");
        assert_eq!(core.weight, 3);
        assert!(!core.exact_match);

        // important: weight=2
        let important = WeightedKeyword::important("테스트");
        assert_eq!(important.weight, 2);
        assert!(!important.exact_match);

        // general: weight=1
        let general = WeightedKeyword::general("테스트");
        assert_eq!(general.weight, 1);
        assert!(!general.exact_match);

        // exact: 지정된 weight + exact_match=true
        let exact = WeightedKeyword::exact("테스트", 3);
        assert_eq!(exact.weight, 3);
        assert!(exact.exact_match);
    }

    /// exact_word_match 함수 테스트
    #[test]
    fn test_exact_word_match() {
        // 정확한 단어 매칭
        assert!(PromptRouter::exact_word_match("ccp 합격률", "ccp"));
        assert!(PromptRouter::exact_word_match("ccp", "ccp"));
        assert!(PromptRouter::exact_word_match("합격률 ccp", "ccp"));

        // 부분 문자열은 매칭되지 않아야 함
        assert!(!PromptRouter::exact_word_match("accept", "ccp"));
        assert!(!PromptRouter::exact_word_match("ccplus", "ccp"));
    }

    /// MatchResult 정렬 테스트
    #[test]
    fn test_match_result_sorting() {
        let mut results = vec![
            MatchResult {
                template_key: "low_score".to_string(),
                total_score: 2,
                match_count: 1,
                priority: 1,
            },
            MatchResult {
                template_key: "high_score".to_string(),
                total_score: 5,
                match_count: 2,
                priority: 3,
            },
            MatchResult {
                template_key: "same_score_lower_priority".to_string(),
                total_score: 5,
                match_count: 2,
                priority: 1,
            },
        ];

        // 다중 조건 정렬: 점수(내림) > 매칭 개수(내림) > 우선순위(오름)
        results.sort_by(|a, b| {
            b.total_score
                .cmp(&a.total_score)
                .then_with(|| b.match_count.cmp(&a.match_count))
                .then_with(|| a.priority.cmp(&b.priority))
        });

        // 동일 점수, 동일 매칭 개수일 때 priority가 낮은 것이 먼저
        assert_eq!(results[0].template_key, "same_score_lower_priority");
        assert_eq!(results[1].template_key, "high_score");
        assert_eq!(results[2].template_key, "low_score");
    }

    /// 레거시 함수 호환성 테스트
    #[test]
    fn test_legacy_function_compatibility() {
        let router = PromptRouter::new();

        // 레거시 함수도 기본적인 매칭이 동작해야 함
        let legacy_result = router.find_matching_template_legacy("라인별 생산량 보여줘");
        assert!(legacy_result.is_some());
        // 레거시 함수는 HashMap 순서에 의존하므로 정확한 결과는 보장하지 않음
        // 단, 매칭 자체는 되어야 함
    }

    // ========== Phase 3 신규 템플릿 테스트 케이스 (12개) ==========

    /// 생산계획 달성률 (GAUGE) 테스트
    #[test]
    fn test_production_achievement_template() {
        let router = PromptRouter::new();

        assert_eq!(
            router.find_matching_template("생산계획 달성률 보여줘"),
            Some("생산계획_달성률".to_string())
        );
        assert_eq!(
            router.find_matching_template("목표대비 달성률"),
            Some("생산계획_달성률".to_string())
        );

        let route = router.route("계획달성률 현황");
        assert!(route.is_some());
        assert_eq!(route.unwrap().chart_config.chart_type, "gauge");
    }

    /// 공급업체별 납품현황 (BAR) 테스트
    #[test]
    fn test_vendor_delivery_template() {
        let router = PromptRouter::new();

        assert_eq!(
            router.find_matching_template("공급업체별 납품현황 보여줘"),
            Some("공급업체별_납품현황".to_string())
        );
        assert_eq!(
            router.find_matching_template("벤더 납기 현황"),
            Some("공급업체별_납품현황".to_string())
        );

        let route = router.route("공급업체 납품 분석");
        assert!(route.is_some());
        assert_eq!(route.unwrap().chart_config.chart_type, "bar");
    }

    /// 출고 현황 (BAR) 테스트
    #[test]
    fn test_outbound_status_template() {
        let router = PromptRouter::new();

        assert_eq!(
            router.find_matching_template("출고현황 보여줘"),
            Some("출고_현황".to_string())
        );
        assert_eq!(
            router.find_matching_template("고객별출고 현황"),
            Some("출고_현황".to_string())
        );

        let route = router.route("배송현황 분석");
        assert!(route.is_some());
        assert_eq!(route.unwrap().chart_config.chart_type, "bar");
    }

    /// 알람 발생현황 (PIE) 테스트
    #[test]
    fn test_alarm_status_template() {
        let router = PromptRouter::new();

        assert_eq!(
            router.find_matching_template("알람 발생현황 보여줘"),
            Some("알람_발생현황".to_string())
        );
        assert_eq!(
            router.find_matching_template("경보현황 분석"),
            Some("알람_발생현황".to_string())
        );

        let route = router.route("알람현황");
        assert!(route.is_some());
        assert_eq!(route.unwrap().chart_config.chart_type, "pie");
    }

    /// 점검 이행률 (GAUGE) 테스트
    #[test]
    fn test_checklist_compliance_template() {
        let router = PromptRouter::new();

        assert_eq!(
            router.find_matching_template("점검이행률 보여줘"),
            Some("점검_이행률".to_string())
        );
        assert_eq!(
            router.find_matching_template("체크리스트이행 현황"),
            Some("점검_이행률".to_string())
        );

        let route = router.route("점검현황 분석");
        assert!(route.is_some());
        assert_eq!(route.unwrap().chart_config.chart_type, "gauge");
    }

    /// 작업자별 생산성 (BAR) 테스트
    #[test]
    fn test_operator_productivity_template() {
        let router = PromptRouter::new();

        assert_eq!(
            router.find_matching_template("작업자별 생산성 보여줘"),
            Some("작업자별_생산성".to_string())
        );
        assert_eq!(
            router.find_matching_template("인력별 실적 현황"),
            Some("작업자별_생산성".to_string())
        );

        let route = router.route("작업자생산성 분석");
        assert!(route.is_some());
        assert_eq!(route.unwrap().chart_config.chart_type, "bar");
    }

    /// 교대별 실적 (BAR) 테스트
    #[test]
    fn test_shift_performance_template() {
        let router = PromptRouter::new();

        assert_eq!(
            router.find_matching_template("교대별 실적 보여줘"),
            Some("교대별_실적".to_string())
        );
        assert_eq!(
            router.find_matching_template("주간 야간 실적 비교"),
            Some("교대별_실적".to_string())
        );

        let route = router.route("시프트별 현황");
        assert!(route.is_some());
        assert_eq!(route.unwrap().chart_config.chart_type, "bar");
    }

    /// 입고 추이 (LINE) 테스트
    #[test]
    fn test_inbound_trend_template() {
        let router = PromptRouter::new();

        assert_eq!(
            router.find_matching_template("입고추이 보여줘"),
            Some("입고_추이".to_string())
        );
        assert_eq!(
            router.find_matching_template("월별입고 트렌드"),
            Some("입고_추이".to_string())
        );

        let route = router.route("입고트렌드 분석");
        assert!(route.is_some());
        assert_eq!(route.unwrap().chart_config.chart_type, "line");
    }

    /// 출고 추이 (LINE) 테스트
    #[test]
    fn test_outbound_trend_template() {
        let router = PromptRouter::new();

        // "출고추이"(3점) - 핵심 키워드로 명확하게 매칭
        assert_eq!(
            router.find_matching_template("출고추이 보여줘"),
            Some("출고_추이".to_string())
        );

        // "출고트렌드"(3점) - 또 다른 핵심 키워드
        assert_eq!(
            router.find_matching_template("출고트렌드 분석"),
            Some("출고_추이".to_string())
        );

        // "월별출고"(2점) + "배송추이"(2점) = 4점
        assert_eq!(
            router.find_matching_template("월별출고 배송추이 분석"),
            Some("출고_추이".to_string())
        );

        // 차트 타입 검증
        let route = router.route("출고추이 그래프");
        assert!(route.is_some());
        assert_eq!(route.unwrap().chart_config.chart_type, "line");
    }

    /// BOM 원자재 소요량 (PIE) 테스트
    #[test]
    fn test_bom_material_usage_template() {
        let router = PromptRouter::new();

        assert_eq!(
            router.find_matching_template("BOM 분석 보여줘"),
            Some("BOM_원자재_소요량".to_string())
        );
        assert_eq!(
            router.find_matching_template("원자재소요 현황"),
            Some("BOM_원자재_소요량".to_string())
        );

        let route = router.route("자재소요 분석");
        assert!(route.is_some());
        assert_eq!(route.unwrap().chart_config.chart_type, "pie");
    }

    /// 재고이동 현황 (BAR) 테스트
    #[test]
    fn test_inventory_movement_template() {
        let router = PromptRouter::new();

        assert_eq!(
            router.find_matching_template("재고이동 현황 보여줘"),
            Some("재고이동_현황".to_string())
        );
        assert_eq!(
            router.find_matching_template("입출고 변동 현황"),
            Some("재고이동_현황".to_string())
        );

        let route = router.route("재고변동 분석");
        assert!(route.is_some());
        assert_eq!(route.unwrap().chart_config.chart_type, "bar");
    }

    /// 금속검출 이력 (LINE) 테스트
    #[test]
    fn test_metal_detection_history_template() {
        let router = PromptRouter::new();

        assert_eq!(
            router.find_matching_template("금속검출 이력 보여줘"),
            Some("금속검출_이력".to_string())
        );
        assert_eq!(
            router.find_matching_template("금검 현황"),
            Some("금속검출_이력".to_string())
        );

        let route = router.route("이물검출 분석");
        assert!(route.is_some());
        assert_eq!(route.unwrap().chart_config.chart_type, "line");
    }

    /// 전체 24개 템플릿 개수 검증
    #[test]
    fn test_total_template_count() {
        let router = PromptRouter::new();
        let template_keys = router.get_template_keys();

        // 기존 12개 + 신규 12개 = 24개
        assert_eq!(template_keys.len(), 24);
    }

    /// Phase 3 템플릿 차트 타입 검증
    #[test]
    fn test_phase3_chart_types() {
        let router = PromptRouter::new();

        // GAUGE 타입 (2개 추가: 생산계획_달성률, 점검_이행률)
        assert_eq!(
            router.get_chart_config("생산계획_달성률").unwrap().chart_type,
            "gauge"
        );
        assert_eq!(
            router.get_chart_config("점검_이행률").unwrap().chart_type,
            "gauge"
        );

        // BAR 타입 (5개 추가)
        assert_eq!(
            router.get_chart_config("공급업체별_납품현황").unwrap().chart_type,
            "bar"
        );
        assert_eq!(
            router.get_chart_config("출고_현황").unwrap().chart_type,
            "bar"
        );
        assert_eq!(
            router.get_chart_config("작업자별_생산성").unwrap().chart_type,
            "bar"
        );
        assert_eq!(
            router.get_chart_config("교대별_실적").unwrap().chart_type,
            "bar"
        );
        assert_eq!(
            router.get_chart_config("재고이동_현황").unwrap().chart_type,
            "bar"
        );

        // LINE 타입 (3개 추가)
        assert_eq!(
            router.get_chart_config("입고_추이").unwrap().chart_type,
            "line"
        );
        assert_eq!(
            router.get_chart_config("출고_추이").unwrap().chart_type,
            "line"
        );
        assert_eq!(
            router.get_chart_config("금속검출_이력").unwrap().chart_type,
            "line"
        );

        // PIE 타입 (2개 추가)
        assert_eq!(
            router.get_chart_config("알람_발생현황").unwrap().chart_type,
            "pie"
        );
        assert_eq!(
            router.get_chart_config("BOM_원자재_소요량").unwrap().chart_type,
            "pie"
        );
    }
}
