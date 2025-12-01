-- CCP 판단 결과 테이블 (CCP Judgment Results)
--
-- 목적:
--   - RAG + 룰베이스 + LLM 하이브리드 판단 결과 저장
--   - 판단 히스토리 추적 및 감사(Audit) 지원
--   - 심사위원 시연용 결과 조회
--
-- 판단 흐름:
--   1. 센서 로그 통계 계산 (ccp_sensors 테이블)
--   2. 룰베이스 위험도 판정 (NG 비율 기반)
--   3. RAG 검색으로 증거 문서 수집 (ccp_docs_fts)
--   4. LLM으로 자연어 요약 생성 (Claude API)
--   5. 최종 결과 저장 (이 테이블)

CREATE TABLE IF NOT EXISTS ccp_judgments (
    id TEXT PRIMARY KEY,                   -- UUID (예: 'ccp-judgment-uuid-12345')
    company_id TEXT NOT NULL,              -- 회사 ID
    ccp_id TEXT NOT NULL,                  -- CCP 코드
    period_from TEXT NOT NULL,             -- 분석 시작일 (ISO 8601: YYYY-MM-DD)
    period_to TEXT NOT NULL,               -- 분석 종료일 (ISO 8601: YYYY-MM-DD)

    -- 통계 지표
    total_logs INTEGER NOT NULL,           -- 총 점검 횟수
    ng_count INTEGER NOT NULL,             -- NG 발생 횟수
    ng_rate REAL NOT NULL,                 -- NG 비율 (0.0 ~ 1.0)
    avg_value REAL NOT NULL,               -- 측정값 평균

    -- 판단 결과
    risk_level TEXT NOT NULL CHECK(risk_level IN ('LOW', 'MEDIUM', 'HIGH')),  -- 룰베이스 위험도
    rule_reason TEXT,                      -- 룰베이스 판단 근거 (선택)
    llm_summary TEXT,                      -- LLM 생성 자연어 요약 (선택)

    -- RAG 증거 문서
    evidence_docs TEXT,                    -- JSON 배열 형식 (현재 구현: 문자열 저장, 향후: JSONB 타입)
                                            -- 예: '[{"id":1,"title":"CCP-01 기준","score":-2.3},...]'

    created_at TEXT NOT NULL DEFAULT (datetime('now'))  -- 판단 생성 시각
);

-- 인덱스: 회사별 최근 판단 조회 최적화
CREATE INDEX IF NOT EXISTS idx_ccp_judgments_company
ON ccp_judgments(company_id, created_at DESC);

-- 인덱스: CCP별 최근 판단 조회 최적화
CREATE INDEX IF NOT EXISTS idx_ccp_judgments_ccp
ON ccp_judgments(company_id, ccp_id, created_at DESC);

-- 설명:
-- 1. id: UUID 사용 (분산 시스템 확장 대비)
--
-- 2. risk_level 판정 규칙 (룰베이스):
--    - HIGH:   NG 비율 >= 10%
--    - MEDIUM: NG 비율 >= 3%
--    - LOW:    NG 비율 < 3%
--
-- 3. evidence_docs: JSON 문자열로 저장 (데모용 간소화)
--    현재 구현: TEXT 타입에 JSON 문자열 저장
--    향후 확장: PostgreSQL 마이그레이션 시 JSONB 타입 사용
--    예시 값:
--      '[
--        {"id": 1, "title": "CCP-01 열처리 관리 기준", "score": -2.34, "content": "..."},
--        {"id": 5, "title": "CCP-01 시정조치 절차", "score": -1.89, "content": "..."}
--      ]'
--
-- 4. 애플리케이션에서 JSON 파싱:
--    Rust: serde_json::from_str()
--    TypeScript: JSON.parse()
--
-- 예시 조회 쿼리:
--   -- 최근 판단 결과 1건
--   SELECT * FROM ccp_judgments
--   WHERE company_id = 'COMP_A' AND ccp_id = 'CCP-01'
--   ORDER BY created_at DESC
--   LIMIT 1;
--
--   -- 위험도별 통계
--   SELECT risk_level, COUNT(*) AS count
--   FROM ccp_judgments
--   WHERE company_id = 'COMP_A'
--   GROUP BY risk_level;
