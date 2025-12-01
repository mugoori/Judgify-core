-- Migration 005: Generic MES/ERP RAG Data Storage
-- Purpose: Enable CSV upload from MES/ERP systems with FTS5 full-text search

-- Main data table with session isolation
CREATE TABLE IF NOT EXISTS mes_data_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL,           -- UUID for session isolation
    file_name TEXT NOT NULL,            -- Original CSV file name
    row_index INTEGER NOT NULL,         -- Row number in CSV (0-based)
    raw_json TEXT NOT NULL,             -- Original row data as JSON
    content TEXT NOT NULL,              -- Searchable text: "컬럼명: 값, ..." format
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- FTS5 virtual table for full-text search (External Content mode)
CREATE VIRTUAL TABLE IF NOT EXISTS mes_data_logs_fts USING fts5(
    content,
    content='mes_data_logs',           -- External Content mode (no data duplication)
    content_rowid='id',
    tokenize='porter unicode61'
);

-- Index for session-based queries (critical for performance)
CREATE INDEX IF NOT EXISTS idx_mes_data_logs_session
ON mes_data_logs(session_id);

-- Index for file name lookup
CREATE INDEX IF NOT EXISTS idx_mes_data_logs_file
ON mes_data_logs(file_name);

-- Triggers to keep FTS5 in sync with main table

-- Trigger: After insert
CREATE TRIGGER IF NOT EXISTS mes_data_logs_ai
AFTER INSERT ON mes_data_logs
BEGIN
    INSERT INTO mes_data_logs_fts(rowid, content)
    VALUES (new.id, new.content);
END;

-- Trigger: After delete
CREATE TRIGGER IF NOT EXISTS mes_data_logs_ad
AFTER DELETE ON mes_data_logs
BEGIN
    INSERT INTO mes_data_logs_fts(mes_data_logs_fts, rowid, content)
    VALUES('delete', old.id, old.content);
END;

-- Trigger: After update
CREATE TRIGGER IF NOT EXISTS mes_data_logs_au
AFTER UPDATE ON mes_data_logs
BEGIN
    INSERT INTO mes_data_logs_fts(mes_data_logs_fts, rowid, content)
    VALUES('delete', old.id, old.content);
    INSERT INTO mes_data_logs_fts(rowid, content)
    VALUES (new.id, new.content);
END;
