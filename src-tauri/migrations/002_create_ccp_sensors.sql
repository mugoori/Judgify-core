-- CCP 센서 로그 테이블 (CCP Sensor Logs)
--
-- 목적:
--   - 일일 CCP 점검 기록 저장 (온도, 금속검출 등)
--   - OK/NG 판정 결과 추적
--   - 작업자 및 시정조치 내용 기록
--
-- 통계 분석 기능:
--   - 기간별 NG 비율 계산
--   - 측정값 평균/최소/최대 산출
--   - 룰베이스 판단 입력 데이터로 활용

CREATE TABLE IF NOT EXISTS ccp_sensors (
    log_id INTEGER PRIMARY KEY AUTOINCREMENT,
    company_id TEXT NOT NULL,              -- 회사 ID (예: 'COMP_A')
    ccp_id TEXT NOT NULL,                  -- CCP 코드 (예: 'CCP-01')
    log_date TEXT NOT NULL,                -- 점검 날짜 (ISO 8601: YYYY-MM-DD)
    measured_value REAL NOT NULL,          -- 측정값 (예: 온도 75.3℃, 무게 500g)
    result TEXT NOT NULL CHECK(result IN ('OK', 'NG')),  -- 판정 결과
    operator_name TEXT,                    -- 작업자 이름 (선택, 예: '김품질')
    action_taken TEXT,                     -- 시정조치 내용 (NG 시 필수, 예: '재가열 후 재측정 완료')
    created_at TEXT NOT NULL DEFAULT (datetime('now'))  -- 기록 생성 시각
);

-- 복합 인덱스: 통계 조회 최적화 (회사 + CCP + 날짜 범위)
CREATE INDEX IF NOT EXISTS idx_ccp_sensors_date
ON ccp_sensors(company_id, ccp_id, log_date);

-- 설명:
-- 1. log_date: TEXT 타입 사용 (ISO 8601 형식)
--    - SQLite의 date/datetime 함수와 호환 가능
--    - 예: WHERE log_date BETWEEN '2025-11-01' AND '2025-11-14'
--
-- 2. result: CHECK 제약으로 'OK'/'NG'만 허용
--
-- 3. action_taken: NG 발생 시 필수 기록 (애플리케이션 레벨에서 검증)
--
-- 예시 통계 쿼리:
--   SELECT
--       COUNT(*) AS total_logs,
--       SUM(CASE WHEN result = 'NG' THEN 1 ELSE 0 END) AS ng_count,
--       CAST(SUM(CASE WHEN result = 'NG' THEN 1 ELSE 0 END) AS REAL) / COUNT(*) AS ng_rate,
--       AVG(measured_value) AS avg_value,
--       MIN(measured_value) AS min_value,
--       MAX(measured_value) AS max_value
--   FROM ccp_sensors
--   WHERE company_id = 'COMP_A'
--     AND ccp_id = 'CCP-01'
--     AND log_date BETWEEN '2025-11-01' AND '2025-11-14';
