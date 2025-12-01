-- ========================================
-- 001_knowledge_base.sql
-- RAG용 지식베이스 테이블 (FTS5 전문검색)
-- 퓨어웰 음료㈜ 기업정보 및 SOP 저장
-- ========================================

-- 지식베이스 메인 테이블
CREATE TABLE IF NOT EXISTS knowledge_base (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    category TEXT NOT NULL CHECK (category IN ('company', 'sop', 'policy', 'manual', 'faq')),
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    tags TEXT,                    -- JSON 배열 형태의 태그
    source_file TEXT,             -- 원본 파일명
    chunk_index INTEGER DEFAULT 0, -- 청킹된 경우 순서
    metadata TEXT,                -- 추가 메타데이터 JSON
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- FTS5 전문검색 가상 테이블
CREATE VIRTUAL TABLE IF NOT EXISTS knowledge_base_fts USING fts5(
    title,
    content,
    tags,
    content=knowledge_base,
    content_rowid=rowid,
    tokenize='unicode61'
);

-- FTS5 동기화 트리거: INSERT
CREATE TRIGGER IF NOT EXISTS knowledge_base_ai AFTER INSERT ON knowledge_base BEGIN
    INSERT INTO knowledge_base_fts(rowid, title, content, tags)
    VALUES (NEW.rowid, NEW.title, NEW.content, NEW.tags);
END;

-- FTS5 동기화 트리거: DELETE
CREATE TRIGGER IF NOT EXISTS knowledge_base_ad AFTER DELETE ON knowledge_base BEGIN
    INSERT INTO knowledge_base_fts(knowledge_base_fts, rowid, title, content, tags)
    VALUES ('delete', OLD.rowid, OLD.title, OLD.content, OLD.tags);
END;

-- FTS5 동기화 트리거: UPDATE
CREATE TRIGGER IF NOT EXISTS knowledge_base_au AFTER UPDATE ON knowledge_base BEGIN
    INSERT INTO knowledge_base_fts(knowledge_base_fts, rowid, title, content, tags)
    VALUES ('delete', OLD.rowid, OLD.title, OLD.content, OLD.tags);
    INSERT INTO knowledge_base_fts(rowid, title, content, tags)
    VALUES (NEW.rowid, NEW.title, NEW.content, NEW.tags);
END;

-- 인덱스
CREATE INDEX IF NOT EXISTS idx_knowledge_base_category ON knowledge_base(category);
CREATE INDEX IF NOT EXISTS idx_knowledge_base_source ON knowledge_base(source_file);
CREATE INDEX IF NOT EXISTS idx_knowledge_base_created ON knowledge_base(created_at DESC);

-- 참고: workflows, workflow_executions 테이블은 sqlite.rs의 init_schema()에서 생성됨
-- 중복 생성을 피하기 위해 여기서는 제외
