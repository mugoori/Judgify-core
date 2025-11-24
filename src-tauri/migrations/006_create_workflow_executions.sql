-- 워크플로우 실행 이력 테이블
CREATE TABLE IF NOT EXISTS workflow_executions (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    workflow_id TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('success', 'failed', 'partial')),
    steps_executed TEXT NOT NULL, -- JSON 배열
    final_result TEXT, -- JSON 객체
    execution_time_ms INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 인덱스: workflow_id로 빠른 조회
CREATE INDEX IF NOT EXISTS idx_workflow_executions_workflow_id
ON workflow_executions(workflow_id);

-- 인덱스: created_at으로 최신순 정렬
CREATE INDEX IF NOT EXISTS idx_workflow_executions_created_at
ON workflow_executions(created_at DESC);

-- 인덱스: status로 필터링 (성공/실패 분류)
CREATE INDEX IF NOT EXISTS idx_workflow_executions_status
ON workflow_executions(status);
