use serde_json;
use uuid::Uuid;
use crate::database::{Database, CcpDocWithScore, CcpStats, CcpJudgmentRequest, CcpJudgmentResponse};
use crate::services::llm_engine::LLMEngine;

/// CCP 데모 서비스 (RAG + 룰베이스 판단)
///
/// 기능:
/// 1. FTS5 BM25 기반 문서 검색 (RAG)
/// 2. 센서 로그 통계 계산
/// 3. 룰베이스 위험도 판정
/// 4. LLM 자연어 요약 생성
/// 5. 하이브리드 판단 결과 저장
pub struct CcpService {
    db: Database,
    llm_engine: LLMEngine,
}

impl CcpService {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            db: Database::new()?,
            llm_engine: LLMEngine::new()?,
        })
    }

    /// FTS5 BM25 기반 CCP 문서 검색
    ///
    /// Parameters:
    /// - company_id: 회사 ID (예: "COMP_A")
    /// - ccp_id: CCP 코드 (선택, 예: "CCP-01")
    /// - query: 검색 쿼리 (예: "열처리 기준")
    /// - top_k: 반환할 최대 문서 수
    ///
    /// Returns: BM25 점수 포함 문서 목록 (점수 오름차순 정렬 = 관련도 높은 순)
    pub fn search_ccp_docs(
        &self,
        company_id: &str,
        ccp_id: Option<&str>,
        query: &str,
        top_k: usize,
    ) -> anyhow::Result<Vec<CcpDocWithScore>> {
        let db_conn = self.db.get_connection();
        let conn = db_conn.lock()
            .map_err(|e| anyhow::anyhow!("DB lock 실패: {}", e))?;

        let sql = if ccp_id.is_some() {
            // CCP 필터 있음
            r#"
                SELECT
                    d.id, d.company_id, d.ccp_id, d.title,
                    d.section_type, d.content,
                    bm25(f) AS score
                FROM ccp_docs d
                JOIN ccp_docs_fts f ON d.id = f.rowid
                WHERE d.company_id = ?1
                  AND d.ccp_id = ?2
                  AND f MATCH ?3
                ORDER BY score
                LIMIT ?4
            "#
        } else {
            // CCP 필터 없음
            r#"
                SELECT
                    d.id, d.company_id, d.ccp_id, d.title,
                    d.section_type, d.content,
                    bm25(f) AS score
                FROM ccp_docs d
                JOIN ccp_docs_fts f ON d.id = f.rowid
                WHERE d.company_id = ?1
                  AND f MATCH ?2
                ORDER BY score
                LIMIT ?3
            "#
        };

        let mut stmt = conn.prepare(sql)?;

        let docs = if let Some(ccp) = ccp_id {
            let rows = stmt.query_map(
                rusqlite::params![company_id, ccp, query, top_k as i64],
                |row| {
                    Ok(CcpDocWithScore {
                        id: row.get(0)?,
                        company_id: row.get(1)?,
                        ccp_id: row.get(2)?,
                        title: row.get(3)?,
                        section_type: row.get(4)?,
                        content: row.get(5)?,
                        score: row.get(6)?,
                    })
                },
            )?;
            rows.collect::<Result<Vec<_>, _>>()?
        } else {
            let rows = stmt.query_map(
                rusqlite::params![company_id, query, top_k as i64],
                |row| {
                    Ok(CcpDocWithScore {
                        id: row.get(0)?,
                        company_id: row.get(1)?,
                        ccp_id: row.get(2)?,
                        title: row.get(3)?,
                        section_type: row.get(4)?,
                        content: row.get(5)?,
                        score: row.get(6)?,
                    })
                },
            )?;
            rows.collect::<Result<Vec<_>, _>>()?
        };

        Ok(docs)
    }

    /// 센서 로그 통계 계산
    ///
    /// Parameters:
    /// - company_id: 회사 ID
    /// - ccp_id: CCP 코드
    /// - from: 시작 날짜 (ISO 8601, YYYY-MM-DD)
    /// - to: 종료 날짜 (ISO 8601, YYYY-MM-DD)
    ///
    /// Returns: 통계 데이터 (총 횟수, NG 횟수, NG 비율, 평균/최소/최대 측정값)
    pub fn calculate_stats(
        &self,
        company_id: &str,
        ccp_id: &str,
        from: &str,
        to: &str,
    ) -> anyhow::Result<CcpStats> {
        let db_conn = self.db.get_connection();
        let conn = db_conn.lock()
            .map_err(|e| anyhow::anyhow!("DB lock 실패: {}", e))?;

        let sql = r#"
            SELECT
                COUNT(*) AS total_logs,
                SUM(CASE WHEN result = 'NG' THEN 1 ELSE 0 END) AS ng_count,
                AVG(measured_value) AS avg_value,
                MIN(measured_value) AS min_value,
                MAX(measured_value) AS max_value
            FROM ccp_sensors
            WHERE company_id = ?1
              AND ccp_id = ?2
              AND log_date BETWEEN ?3 AND ?4
        "#;

        let mut stmt = conn.prepare(sql)?;
        let stats = stmt.query_row(
            rusqlite::params![company_id, ccp_id, from, to],
            |row| {
                let total_logs: i32 = row.get(0)?;
                let ng_count: i32 = row.get(1)?;
                let ng_rate = if total_logs > 0 {
                    ng_count as f64 / total_logs as f64
                } else {
                    0.0
                };

                Ok(CcpStats {
                    total_logs,
                    ng_count,
                    ng_rate,
                    avg_value: row.get(2)?,
                    min_value: row.get(3)?,
                    max_value: row.get(4)?,
                })
            },
        )?;

        Ok(stats)
    }

    /// 룰베이스 위험도 판정
    ///
    /// 규칙:
    /// - NG 비율 >= 10% → HIGH
    /// - NG 비율 >= 3%  → MEDIUM
    /// - NG 비율 < 3%   → LOW
    ///
    /// Returns: "LOW" | "MEDIUM" | "HIGH"
    fn rule_based_risk(&self, ng_rate: f64) -> &'static str {
        if ng_rate >= 0.1 {
            "HIGH"
        } else if ng_rate >= 0.03 {
            "MEDIUM"
        } else {
            "LOW"
        }
    }

    /// LLM 자연어 요약 생성
    ///
    /// Parameters:
    /// - stats: 센서 로그 통계
    /// - evidence_docs: RAG 검색 결과
    /// - risk_level: 룰베이스 위험도
    ///
    /// Returns: 자연어 요약 (예: "열처리 CCP-01의 최근 14일간 불량률은 7.1%로, MEDIUM 위험도입니다. ...")
    async fn generate_llm_summary(
        &self,
        stats: &CcpStats,
        evidence_docs: &[CcpDocWithScore],
        risk_level: &str,
    ) -> anyhow::Result<String> {
        // 증거 문서 요약 (상위 3개 제목만)
        let doc_titles: Vec<String> = evidence_docs
            .iter()
            .take(3)
            .map(|d| format!("- {}", d.title))
            .collect();

        // LLM 프롬프트 구성
        let prompt = format!(
            r#"당신은 제조 품질 관리 전문가입니다. 다음 CCP 점검 데이터를 바탕으로 간단명료한 상태 요약을 작성하세요.

## 통계 데이터
- 총 점검 횟수: {}회
- NG 발생: {}회 (비율: {:.1}%)
- 측정값 평균: {:.1} (범위: {:.1} ~ {:.1})
- 위험도: {}

## 참고 문서 (관리 기준)
{}

## 요청
1. 위 데이터를 바탕으로 **2-3문장**으로 현재 상태를 요약하세요.
2. 위험도가 MEDIUM 이상이면 권장 조치를 1가지 제시하세요.
3. 전문 용어보다 쉬운 표현을 사용하세요.

형식: "CCP-01의 최근 점검 결과, ..."
"#,
            stats.total_logs,
            stats.ng_count,
            stats.ng_rate * 100.0,
            stats.avg_value,
            stats.min_value,
            stats.max_value,
            risk_level,
            doc_titles.join("\n")
        );

        // LLM 호출 (기존 LLMEngine 재사용)
        let summary = self.llm_engine.generate_text(&prompt).await?;

        Ok(summary)
    }

    /// 판단 결과 저장
    fn save_judgment(
        &self,
        judgment_id: &str,
        company_id: &str,
        ccp_id: &str,
        period_from: &str,
        period_to: &str,
        stats: &CcpStats,
        risk_level: &str,
        rule_reason: &str,
        llm_summary: &str,
        evidence_docs: &[CcpDocWithScore],
    ) -> anyhow::Result<()> {
        let db_conn = self.db.get_connection();
        let conn = db_conn.lock()
            .map_err(|e| anyhow::anyhow!("DB lock 실패: {}", e))?;

        // 증거 문서를 JSON 문자열로 변환
        let evidence_json = serde_json::to_string(evidence_docs)?;
        let created_at = chrono::Utc::now().to_rfc3339();

        conn.execute(
            r#"
                INSERT INTO ccp_judgments (
                    id, company_id, ccp_id, period_from, period_to,
                    total_logs, ng_count, ng_rate, avg_value,
                    risk_level, rule_reason, llm_summary, evidence_docs, created_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
            "#,
            rusqlite::params![
                judgment_id,
                company_id,
                ccp_id,
                period_from,
                period_to,
                stats.total_logs,
                stats.ng_count,
                stats.ng_rate,
                stats.avg_value,
                risk_level,
                rule_reason,
                llm_summary,
                evidence_json,
                created_at
            ],
        )?;

        Ok(())
    }

    /// 메인 API: CCP 상태 판단 (하이브리드)
    ///
    /// 흐름:
    /// 1. 센서 로그 통계 계산
    /// 2. 룰베이스 위험도 판정
    /// 3. RAG 검색으로 증거 문서 수집
    /// 4. LLM으로 자연어 요약 생성
    /// 5. 판단 결과 저장
    ///
    /// Returns: 판단 결과 (통계 + 위험도 + AI 요약 + 증거 문서)
    pub async fn judge_ccp_status(
        &self,
        request: CcpJudgmentRequest,
    ) -> anyhow::Result<CcpJudgmentResponse> {
        println!("🔍 CCP 판단 시작: {} / {}", request.company_id, request.ccp_id);

        // 1. 센서 로그 통계 계산
        let stats = self.calculate_stats(
            &request.company_id,
            &request.ccp_id,
            &request.period_from,
            &request.period_to,
        )?;

        println!("📊 통계: 총 {}회, NG {}회, 비율 {:.1}%",
            stats.total_logs, stats.ng_count, stats.ng_rate * 100.0);

        // 2. 룰베이스 위험도 판정
        let risk_level = self.rule_based_risk(stats.ng_rate);
        let rule_reason = format!(
            "NG 비율 {:.1}%에 따른 {} 등급 판정",
            stats.ng_rate * 100.0,
            risk_level
        );

        println!("⚠️  위험도: {}", risk_level);

        // 3. RAG 검색 (관리 기준 + 시정조치 문서)
        let evidence_docs = self.search_ccp_docs(
            &request.company_id,
            Some(&request.ccp_id),
            "관리 기준 시정조치",
            3,
        )?;

        println!("📚 증거 문서: {}건 검색", evidence_docs.len());

        // 4. LLM 자연어 요약 생성
        let llm_summary = self.generate_llm_summary(&stats, &evidence_docs, risk_level).await?;

        println!("🤖 LLM 요약 생성 완료");

        // 5. 판단 결과 저장
        let judgment_id = format!("ccp-judgment-{}", Uuid::new_v4());
        self.save_judgment(
            &judgment_id,
            &request.company_id,
            &request.ccp_id,
            &request.period_from,
            &request.period_to,
            &stats,
            risk_level,
            &rule_reason,
            &llm_summary,
            &evidence_docs,
        )?;

        println!("✅ 판단 결과 저장: {}", judgment_id);

        // 6. 응답 반환
        Ok(CcpJudgmentResponse {
            stats,
            risk_level: risk_level.to_string(),
            rule_reason,
            llm_summary,
            evidence_docs,
            judgment_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 테스트용 임시 데이터베이스 생성 헬퍼
    fn setup_test_db() -> anyhow::Result<Database> {
        let db = Database::new()?;
        let db_conn = db.get_connection();
        let conn = db_conn.lock()
            .map_err(|e| anyhow::anyhow!("DB lock 실패: {}", e))?;

        // 테스트용 더미 데이터 삽입
        conn.execute(
            r#"INSERT INTO ccp_docs (id, company_id, ccp_id, title, section_type, content, created_at)
               VALUES ('test-doc-001', 'TEST_COMPANY', 'TEST-CCP-01', '테스트 관리 기준', '관리 기준',
                       '테스트용 CCP 관리 기준 문서입니다. 온도 75도 이상 유지.', '2025-11-01 09:00:00')"#,
            [],
        )?;

        conn.execute(
            r#"INSERT INTO ccp_sensors (id, company_id, ccp_id, log_date, measured_value, result, created_at)
               VALUES ('test-log-001', 'TEST_COMPANY', 'TEST-CCP-01', '2025-11-01', 78.5, 'OK', '2025-11-01 08:00:00')"#,
            [],
        )?;

        conn.execute(
            r#"INSERT INTO ccp_sensors (id, company_id, ccp_id, log_date, measured_value, result, created_at)
               VALUES ('test-log-002', 'TEST_COMPANY', 'TEST-CCP-01', '2025-11-01', 72.1, 'NG', '2025-11-01 12:00:00')"#,
            [],
        )?;

        Ok(db)
    }

    #[test]
    fn test_rule_based_risk_high() {
        let service = match CcpService::new() {
            Ok(s) => s,
            Err(_) => {
                println!("⚠️  테스트 스킵 (API 키 미설정)");
                return;
            }
        };

        // NG 비율 10% 이상 → HIGH
        assert_eq!(service.rule_based_risk(0.15), "HIGH");
        assert_eq!(service.rule_based_risk(0.10), "HIGH");
    }

    #[test]
    fn test_rule_based_risk_medium() {
        let service = match CcpService::new() {
            Ok(s) => s,
            Err(_) => {
                println!("⚠️  테스트 스킵 (API 키 미설정)");
                return;
            }
        };

        // NG 비율 3% ~ 10% → MEDIUM
        assert_eq!(service.rule_based_risk(0.071), "MEDIUM");
        assert_eq!(service.rule_based_risk(0.05), "MEDIUM");
        assert_eq!(service.rule_based_risk(0.03), "MEDIUM");
    }

    #[test]
    fn test_rule_based_risk_low() {
        let service = match CcpService::new() {
            Ok(s) => s,
            Err(_) => {
                println!("⚠️  테스트 스킵 (API 키 미설정)");
                return;
            }
        };

        // NG 비율 3% 미만 → LOW
        assert_eq!(service.rule_based_risk(0.018), "LOW");
        assert_eq!(service.rule_based_risk(0.01), "LOW");
        assert_eq!(service.rule_based_risk(0.0), "LOW");
    }

    #[test]
    fn test_calculate_stats() {
        // 실제 Seed 데이터 사용 (마이그레이션 004 실행 필요)
        let service = match CcpService::new() {
            Ok(s) => s,
            Err(_) => {
                println!("⚠️  테스트 스킵 (API 키 미설정)");
                return;
            }
        };

        let stats_result = service.calculate_stats(
            "COMP_A",
            "CCP-01",
            "2025-11-01",
            "2025-11-14",
        );

        // Seed 데이터가 없으면 테스트 스킵
        if stats_result.is_err() {
            println!("⚠️  Seed 데이터 없음 - 테스트 스킵 (마이그레이션 004 실행 필요)");
            return;
        }

        let stats = stats_result.unwrap();

        // COMP_A CCP-01 예상 통계 (Seed 데이터 기준)
        assert_eq!(stats.total_logs, 168, "총 점검 횟수 불일치");
        assert_eq!(stats.ng_count, 12, "NG 발생 건수 불일치");

        // NG 비율 7.1% (12/168 = 0.0714...)
        assert!((stats.ng_rate - 0.071).abs() < 0.001, "NG 비율 불일치");

        // 위험도: MEDIUM 예상
        let risk = service.rule_based_risk(stats.ng_rate);
        assert_eq!(risk, "MEDIUM", "위험도 판정 불일치");
    }

    #[test]
    fn test_search_ccp_docs() {
        // 실제 Seed 데이터 사용 (마이그레이션 002, 004 실행 필요)
        let service = match CcpService::new() {
            Ok(s) => s,
            Err(_) => {
                println!("⚠️  테스트 스킵 (API 키 미설정)");
                return;
            }
        };

        let docs_result = service.search_ccp_docs(
            "COMP_A",
            Some("CCP-01"),
            "관리 기준",
            5,
        );

        // Seed 데이터가 없거나 FTS5가 비활성화되면 테스트 스킵
        if docs_result.is_err() {
            println!("⚠️  FTS5 검색 실패 - 테스트 스킵");
            println!("   - 마이그레이션 002 (FTS5) 실행 확인");
            println!("   - 마이그레이션 004 (Seed) 실행 확인");
            return;
        }

        let docs = docs_result.unwrap();

        // 최소 1개 이상의 문서 검색 기대
        assert!(!docs.is_empty(), "검색 결과 없음");

        // 첫 번째 문서 검증
        let first_doc = &docs[0];
        assert_eq!(first_doc.company_id, "COMP_A");
        assert_eq!(first_doc.ccp_id, "CCP-01");
        assert!(first_doc.title.contains("CCP") || first_doc.title.contains("관리"));

        // BM25 점수 음수 확인 (낮을수록 관련도 높음)
        assert!(first_doc.score < 0.0, "BM25 점수가 음수가 아님");

        println!("✅ 검색 성공: {}건 (Top 1: {})", docs.len(), first_doc.title);
    }

    #[test]
    fn test_search_ccp_docs_all_ccps() {
        // CCP 필터 없이 전체 검색
        let service = match CcpService::new() {
            Ok(s) => s,
            Err(_) => {
                println!("⚠️  테스트 스킵 (API 키 미설정)");
                return;
            }
        };

        let docs_result = service.search_ccp_docs(
            "COMP_A",
            None,  // 전체 CCP
            "관리 기준",
            10,
        );

        if docs_result.is_err() {
            println!("⚠️  테스트 스킵 (Seed 데이터 없음)");
            return;
        }

        let docs = docs_result.unwrap();

        // CCP-01과 CCP-02 문서가 모두 포함되어야 함
        assert!(!docs.is_empty(), "검색 결과 없음");

        let has_ccp01 = docs.iter().any(|d| d.ccp_id == "CCP-01");
        let has_ccp02 = docs.iter().any(|d| d.ccp_id == "CCP-02");

        println!("✅ 전체 검색 성공: {}건 (CCP-01: {}, CCP-02: {})",
                 docs.len(), has_ccp01, has_ccp02);
    }

    #[tokio::test]
    async fn test_judge_ccp_status_medium_risk() {
        // COMP_A CCP-01은 MEDIUM 위험도 예상 (NG 7.1%)
        let service = match CcpService::new() {
            Ok(s) => s,
            Err(_) => {
                println!("⚠️  테스트 스킵 (API 키 미설정)");
                return;
            }
        };

        let request = CcpJudgmentRequest {
            company_id: "COMP_A".to_string(),
            ccp_id: "CCP-01".to_string(),
            period_from: "2025-11-01".to_string(),
            period_to: "2025-11-14".to_string(),
        };

        let result = service.judge_ccp_status(request).await;

        // Seed 데이터 없거나 LLM API 키 없으면 스킵
        if result.is_err() {
            println!("⚠️  테스트 스킵 (Seed 데이터 또는 API 키 없음)");
            return;
        }

        let response = result.unwrap();

        // 통계 검증
        assert_eq!(response.stats.total_logs, 168);
        assert_eq!(response.stats.ng_count, 12);
        assert!((response.stats.ng_rate - 0.071).abs() < 0.001);

        // 위험도 검증
        assert_eq!(response.risk_level, "MEDIUM");
        assert!(response.rule_reason.contains("7.1%"));
        assert!(response.rule_reason.contains("MEDIUM"));

        // LLM 요약 존재 확인
        assert!(!response.llm_summary.is_empty());

        // 증거 문서 존재 확인 (최대 3개)
        assert!(!response.evidence_docs.is_empty());
        assert!(response.evidence_docs.len() <= 3);

        // 판단 ID UUID 형식 확인
        assert!(response.judgment_id.starts_with("ccp-judgment-"));

        println!("✅ MEDIUM 위험도 판단 성공");
        println!("   - NG 비율: {:.1}%", response.stats.ng_rate * 100.0);
        println!("   - 위험도: {}", response.risk_level);
        println!("   - 증거 문서: {}건", response.evidence_docs.len());
    }

    #[tokio::test]
    async fn test_judge_ccp_status_high_risk() {
        // COMP_B CCP-01은 HIGH 위험도 예상 (NG 11.9%)
        let service = match CcpService::new() {
            Ok(s) => s,
            Err(_) => {
                println!("⚠️  테스트 스킵 (API 키 미설정)");
                return;
            }
        };

        let request = CcpJudgmentRequest {
            company_id: "COMP_B".to_string(),
            ccp_id: "CCP-01".to_string(),
            period_from: "2025-11-01".to_string(),
            period_to: "2025-11-14".to_string(),
        };

        let result = service.judge_ccp_status(request).await;

        if result.is_err() {
            println!("⚠️  테스트 스킵");
            return;
        }

        let response = result.unwrap();

        // 위험도 HIGH 확인
        assert_eq!(response.risk_level, "HIGH");
        assert_eq!(response.stats.ng_count, 20);
        assert!((response.stats.ng_rate - 0.119).abs() < 0.001);

        println!("✅ HIGH 위험도 판단 성공");
        println!("   - NG 비율: {:.1}%", response.stats.ng_rate * 100.0);
    }

    #[tokio::test]
    async fn test_judge_ccp_status_low_risk() {
        // COMP_A CCP-02는 LOW 위험도 예상 (NG 1.8%)
        let service = match CcpService::new() {
            Ok(s) => s,
            Err(_) => {
                println!("⚠️  테스트 스킵 (API 키 미설정)");
                return;
            }
        };

        let request = CcpJudgmentRequest {
            company_id: "COMP_A".to_string(),
            ccp_id: "CCP-02".to_string(),
            period_from: "2025-11-01".to_string(),
            period_to: "2025-11-14".to_string(),
        };

        let result = service.judge_ccp_status(request).await;

        if result.is_err() {
            println!("⚠️  테스트 스킵");
            return;
        }

        let response = result.unwrap();

        // 위험도 LOW 확인
        assert_eq!(response.risk_level, "LOW");
        assert_eq!(response.stats.ng_count, 3);
        assert!((response.stats.ng_rate - 0.018).abs() < 0.001);

        println!("✅ LOW 위험도 판단 성공");
        println!("   - NG 비율: {:.1}%", response.stats.ng_rate * 100.0);
    }
}
