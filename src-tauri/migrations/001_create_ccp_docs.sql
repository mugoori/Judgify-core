-- CCP 정책 문서 테이블 (CCP Policy Documents)
--
-- 목적:
--   - HACCP/ISO22000 CCP 관리 기준 문서 저장
--   - 시정조치 절차, 모니터링 방법 등 정책 문서 관리
--   - FTS5 전문검색 지원 (BM25 알고리즘)
--
-- FTS5 동기화 전략:
--   현재 (데모): seed 시점에 ccp_docs + ccp_docs_fts 동시 INSERT
--   향후 (프로덕션): content= 옵션 + AFTER INSERT/UPDATE 트리거로 자동 동기화

CREATE TABLE IF NOT EXISTS ccp_docs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    company_id TEXT NOT NULL,              -- 더미 회사 ID (예: 'COMP_A', 'COMP_B')
    ccp_id TEXT NOT NULL,                  -- CCP 코드 (예: 'CCP-01', 'CCP-02')
    title TEXT NOT NULL,                   -- 문서 제목 (예: 'CCP-01 열처리 관리 기준')
    section_type TEXT NOT NULL,            -- 문서 섹션 타입: 'standard' | 'monitoring' | 'action'
    content TEXT NOT NULL,                 -- 문서 내용 (전문검색 대상)
    created_at TEXT NOT NULL DEFAULT (datetime('now'))  -- ISO 8601 형식 (YYYY-MM-DD HH:MM:SS)
);

-- FTS5 전문검색 인덱스
-- tokenize='porter unicode61': 영어 형태소 분석 + 유니코드 지원
-- BM25 알고리즘: TF-IDF 기반 관련도 점수 계산
CREATE VIRTUAL TABLE IF NOT EXISTS ccp_docs_fts
USING fts5(
    title,
    content,
    tokenize='porter unicode61'
);

-- 복합 인덱스: company_id + ccp_id 조합 조회 최적화
CREATE INDEX IF NOT EXISTS idx_ccp_docs_company
ON ccp_docs(company_id, ccp_id);

-- 설명:
-- 1. ccp_docs: 원본 데이터 저장 (관계형 DB)
-- 2. ccp_docs_fts: 전문검색 인덱스 (Virtual Table, 실제 데이터 저장 안 함)
-- 3. 검색 시: JOIN으로 결합하여 BM25 점수와 원본 데이터 함께 조회
--
-- 예시 검색 쿼리:
--   SELECT d.*, bm25(f) AS score
--   FROM ccp_docs d
--   JOIN ccp_docs_fts f ON d.id = f.rowid
--   WHERE d.company_id = 'COMP_A'
--     AND f MATCH '열처리 기준'
--   ORDER BY score
--   LIMIT 5;
--
-- BM25 점수: 낮을수록 관련도가 높음 (음수 값 가능)
