use rusqlite::{Connection, Result, params};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use crate::database::models::*;
use chrono::Utc;
use serde::{Deserialize, Serialize};

pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// Get a cloned Arc to the database connection for use in services
    pub fn get_connection(&self) -> Arc<Mutex<Connection>> {
        Arc::clone(&self.conn)
    }

    pub fn new() -> Result<Self> {
        let db_path = Self::get_db_path()?;
        let conn = Connection::open(db_path)?;

        // Initialize schema
        Self::init_schema(&conn)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    fn get_db_path() -> Result<PathBuf> {
        let app_data = std::env::var("APPDATA")
            .or_else(|_| std::env::var("HOME"))
            .map_err(|e| rusqlite::Error::InvalidPath(PathBuf::from(e.to_string())))?;

        let db_dir = PathBuf::from(app_data).join("Judgify");
        std::fs::create_dir_all(&db_dir)
            .map_err(|e| rusqlite::Error::InvalidPath(PathBuf::from(e.to_string())))?;

        Ok(db_dir.join("judgify.db"))
    }

    fn init_schema(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS judgments (
                id TEXT PRIMARY KEY,
                workflow_id TEXT NOT NULL,
                input_data TEXT NOT NULL,
                result INTEGER NOT NULL,
                confidence REAL NOT NULL,
                method_used TEXT NOT NULL,
                explanation TEXT,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS workflows (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                definition TEXT NOT NULL,
                rule_expression TEXT,
                version INTEGER DEFAULT 1,
                is_active INTEGER DEFAULT 1,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS training_samples (
                id TEXT PRIMARY KEY,
                workflow_id TEXT NOT NULL,
                input_data TEXT NOT NULL,
                expected_result INTEGER NOT NULL,
                actual_result INTEGER,
                accuracy REAL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (workflow_id) REFERENCES workflows(id)
            );

            CREATE TABLE IF NOT EXISTS feedbacks (
                id TEXT PRIMARY KEY,
                judgment_id TEXT NOT NULL,
                feedback_type TEXT NOT NULL,
                value INTEGER NOT NULL,
                comment TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY (judgment_id) REFERENCES judgments(id)
            );

            CREATE TABLE IF NOT EXISTS prompt_templates (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                template_type TEXT NOT NULL,
                content TEXT NOT NULL,
                variables TEXT NOT NULL,
                version INTEGER DEFAULT 1,
                is_active INTEGER DEFAULT 1,
                token_limit INTEGER,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS token_usage (
                id TEXT PRIMARY KEY,
                judgment_id TEXT NOT NULL,
                service TEXT NOT NULL,
                tokens_used INTEGER NOT NULL,
                cost_usd REAL NOT NULL,
                complexity TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (judgment_id) REFERENCES judgments(id)
            );

            CREATE INDEX IF NOT EXISTS idx_judgments_workflow ON judgments(workflow_id);
            CREATE INDEX IF NOT EXISTS idx_judgments_created ON judgments(created_at);
            CREATE INDEX IF NOT EXISTS idx_training_workflow ON training_samples(workflow_id);

            -- Composite indexes for performance optimization (Task 2.2)
            -- 1. Judgment workflow + time index (Dashboard getJudgmentHistory optimization)
            CREATE INDEX IF NOT EXISTS idx_judgments_workflow_created
              ON judgments(workflow_id, created_at DESC);

            -- 2. TrainingSample search index (Learning Service optimization)
            CREATE INDEX IF NOT EXISTS idx_training_workflow_accuracy
              ON training_samples(workflow_id, accuracy DESC, created_at DESC);

            -- 3. Feedback + Judgment JOIN index (complex query optimization)
            CREATE INDEX IF NOT EXISTS idx_feedbacks_judgment_type
              ON feedbacks(judgment_id, feedback_type, value);

            -- 4. Feedback covering index (optimized retrieval with all columns)
            CREATE INDEX IF NOT EXISTS idx_feedbacks_covering
              ON feedbacks(judgment_id, feedback_type, value, created_at);

            -- 5. PromptTemplate type + active index (template selection optimization)
            CREATE INDEX IF NOT EXISTS idx_templates_type_active
              ON prompt_templates(template_type, is_active, version DESC);

            -- 6. Token Usage indexes (MCP cost tracking optimization)
            CREATE INDEX IF NOT EXISTS idx_token_usage_created
              ON token_usage(created_at DESC);

            CREATE INDEX IF NOT EXISTS idx_token_usage_service_created
              ON token_usage(service, created_at DESC);

            CREATE INDEX IF NOT EXISTS idx_token_usage_judgment
              ON token_usage(judgment_id);

            -- ============================================================
            -- CCP 데모용 테이블 (RAG + 룰베이스 판단)
            -- ============================================================

            -- CCP 정책 문서 테이블
            CREATE TABLE IF NOT EXISTS ccp_docs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                company_id TEXT NOT NULL,
                ccp_id TEXT NOT NULL,
                title TEXT NOT NULL,
                section_type TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            -- FTS5 전문검색 인덱스 (BM25 알고리즘)
            CREATE VIRTUAL TABLE IF NOT EXISTS ccp_docs_fts
            USING fts5(title, content, tokenize='porter unicode61');

            CREATE INDEX IF NOT EXISTS idx_ccp_docs_company
            ON ccp_docs(company_id, ccp_id);

            -- CCP 센서 로그 테이블
            CREATE TABLE IF NOT EXISTS ccp_sensors (
                log_id INTEGER PRIMARY KEY AUTOINCREMENT,
                company_id TEXT NOT NULL,
                ccp_id TEXT NOT NULL,
                log_date TEXT NOT NULL,
                measured_value REAL NOT NULL,
                result TEXT NOT NULL CHECK(result IN ('OK', 'NG')),
                operator_name TEXT,
                action_taken TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE INDEX IF NOT EXISTS idx_ccp_sensors_date
            ON ccp_sensors(company_id, ccp_id, log_date);

            -- CCP 판단 결과 테이블
            CREATE TABLE IF NOT EXISTS ccp_judgments (
                id TEXT PRIMARY KEY,
                company_id TEXT NOT NULL,
                ccp_id TEXT NOT NULL,
                period_from TEXT NOT NULL,
                period_to TEXT NOT NULL,
                total_logs INTEGER NOT NULL,
                ng_count INTEGER NOT NULL,
                ng_rate REAL NOT NULL,
                avg_value REAL NOT NULL,
                risk_level TEXT NOT NULL CHECK(risk_level IN ('LOW', 'MEDIUM', 'HIGH')),
                rule_reason TEXT,
                llm_summary TEXT,
                evidence_docs TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE INDEX IF NOT EXISTS idx_ccp_judgments_company
            ON ccp_judgments(company_id, created_at DESC);

            CREATE INDEX IF NOT EXISTS idx_ccp_judgments_ccp
            ON ccp_judgments(company_id, ccp_id, created_at DESC);

            -- ============================================================
            -- MES/ERP RAG 테이블 (Phase 8: Generic CSV Upload & Query)
            -- ============================================================

            -- MES 데이터 로그 테이블
            CREATE TABLE IF NOT EXISTS mes_data_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL,
                file_name TEXT NOT NULL,
                row_index INTEGER NOT NULL,
                raw_json TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            -- FTS5 전문검색 인덱스 (BM25 알고리즘)
            CREATE VIRTUAL TABLE IF NOT EXISTS mes_data_logs_fts
            USING fts5(content, tokenize='porter unicode61');

            -- FTS5 자동 동기화 트리거
            CREATE TRIGGER IF NOT EXISTS mes_data_logs_ai AFTER INSERT ON mes_data_logs
            BEGIN
                INSERT INTO mes_data_logs_fts(rowid, content) VALUES (new.id, new.content);
            END;

            CREATE TRIGGER IF NOT EXISTS mes_data_logs_ad AFTER DELETE ON mes_data_logs
            BEGIN
                DELETE FROM mes_data_logs_fts WHERE rowid = old.id;
            END;

            CREATE TRIGGER IF NOT EXISTS mes_data_logs_au AFTER UPDATE ON mes_data_logs
            BEGIN
                UPDATE mes_data_logs_fts SET content = new.content WHERE rowid = new.id;
            END;

            CREATE INDEX IF NOT EXISTS idx_mes_data_logs_session
            ON mes_data_logs(session_id, created_at DESC);

            -- ============================================================
            -- Workflow 승인 요청 테이블 (Phase 9: APPROVAL Node)
            -- ============================================================

            CREATE TABLE IF NOT EXISTS approval_requests (
                id TEXT PRIMARY KEY,
                workflow_id TEXT NOT NULL,
                workflow_name TEXT NOT NULL,
                step_id TEXT NOT NULL,
                step_name TEXT NOT NULL,
                approval_type TEXT NOT NULL CHECK(approval_type IN ('manual', 'conditional')),
                status TEXT NOT NULL DEFAULT 'pending' CHECK(status IN ('pending', 'approved', 'rejected', 'expired')),
                approvers TEXT NOT NULL,
                input_data TEXT NOT NULL,
                condition TEXT,
                timeout_minutes INTEGER DEFAULT 60,
                decided_by TEXT,
                decided_at TEXT,
                comment TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                expires_at TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_approval_requests_status
            ON approval_requests(status, created_at DESC);

            CREATE INDEX IF NOT EXISTS idx_approval_requests_workflow
            ON approval_requests(workflow_id, status);

            -- ============================================================
            -- Workflow 스케줄러 테이블 (Phase 9: Cron-based Scheduler)
            -- ============================================================

            CREATE TABLE IF NOT EXISTS workflow_schedules (
                id TEXT PRIMARY KEY,
                workflow_id TEXT NOT NULL,
                workflow_name TEXT NOT NULL,
                cron_expression TEXT NOT NULL,
                timezone TEXT NOT NULL DEFAULT 'Asia/Seoul',
                is_active INTEGER NOT NULL DEFAULT 1,
                input_data TEXT NOT NULL DEFAULT '{}',
                last_run_at TEXT,
                next_run_at TEXT,
                run_count INTEGER NOT NULL DEFAULT 0,
                last_status TEXT CHECK(last_status IN ('success', 'failed', 'running')),
                last_error TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE INDEX IF NOT EXISTS idx_workflow_schedules_active
            ON workflow_schedules(is_active, next_run_at);

            CREATE INDEX IF NOT EXISTS idx_workflow_schedules_workflow
            ON workflow_schedules(workflow_id);

            -- Workflow 실행 이력 테이블 (Phase 9: Execution History)
            CREATE TABLE IF NOT EXISTS workflow_executions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                workflow_id TEXT NOT NULL,
                status TEXT NOT NULL,
                steps_executed TEXT NOT NULL,
                final_result TEXT NOT NULL,
                execution_time_ms INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE INDEX IF NOT EXISTS idx_workflow_executions_workflow
            ON workflow_executions(workflow_id, created_at DESC);"
        )?;

        // Seed sample data for demo (only if database is empty)
        crate::database::seed::seed_sample_data(conn)?;

        // 🔥 앱 시작시 자동 마이그레이션 실행 (ERP/MES 테이블 + 시드 데이터)
        // 마이그레이션 SQL은 컴파일 시점에 바이너리에 포함됨 (include_str!)
        eprintln!("🔄 앱 시작: ERP/MES 마이그레이션 확인 중...");
        if let Err(e) = crate::database::migrations::apply_migrations(conn) {
            eprintln!("⚠️  마이그레이션 경고 (무시 가능): {}", e);
            // 마이그레이션 실패는 치명적이지 않음 - 기존 데이터가 있을 수 있음
        }

        Ok(())
    }

    // Judgment operations
    pub fn save_judgment(&self, judgment: &Judgment) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO judgments (id, workflow_id, input_data, result, confidence, method_used, explanation, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                &judgment.id,
                &judgment.workflow_id,
                &judgment.input_data,
                judgment.result as i32,
                judgment.confidence,
                &judgment.method_used,
                &judgment.explanation,
                judgment.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn get_judgment(&self, id: &str) -> Result<Option<Judgment>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, workflow_id, input_data, result, confidence, method_used, explanation, created_at
             FROM judgments WHERE id = ?1"
        )?;

        let result = stmt.query_row([id], |row| {
            Ok(Judgment {
                id: row.get(0)?,
                workflow_id: row.get(1)?,
                input_data: row.get(2)?,
                result: row.get::<_, i32>(3)? != 0,
                confidence: row.get(4)?,
                method_used: row.get(5)?,
                explanation: row.get(6)?,
                created_at: row.get::<_, String>(7)?.parse().unwrap_or(Utc::now()),
            })
        });

        match result {
            Ok(judgment) => Ok(Some(judgment)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn get_judgment_history(&self, workflow_id: Option<String>, limit: u32) -> Result<Vec<Judgment>> {
        let conn = self.conn.lock().unwrap();

        let (query, params_vec): (String, Vec<Box<dyn rusqlite::ToSql>>) = if let Some(wid) = workflow_id {
            (
                "SELECT id, workflow_id, input_data, result, confidence, method_used, explanation, created_at
                 FROM judgments WHERE workflow_id = ?1 ORDER BY created_at DESC LIMIT ?2".to_string(),
                vec![Box::new(wid), Box::new(limit as i32)]
            )
        } else {
            (
                "SELECT id, workflow_id, input_data, result, confidence, method_used, explanation, created_at
                 FROM judgments ORDER BY created_at DESC LIMIT ?1".to_string(),
                vec![Box::new(limit as i32)]
            )
        };

        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();

        let mut stmt = conn.prepare(&query)?;
        let rows = stmt.query_map(params_refs.as_slice(), |row| {
            Ok(Judgment {
                id: row.get(0)?,
                workflow_id: row.get(1)?,
                input_data: row.get(2)?,
                result: row.get::<_, i32>(3)? != 0,
                confidence: row.get(4)?,
                method_used: row.get(5)?,
                explanation: row.get(6)?,
                created_at: row.get::<_, String>(7)?.parse().unwrap_or(Utc::now()),
            })
        })?;

        let mut judgments = Vec::new();
        for judgment in rows {
            judgments.push(judgment?);
        }
        Ok(judgments)
    }

    // Workflow operations
    pub fn save_workflow(&self, workflow: &Workflow) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO workflows (id, name, definition, rule_expression, version, is_active, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                definition = excluded.definition,
                rule_expression = excluded.rule_expression,
                version = excluded.version,
                is_active = excluded.is_active",
            params![
                &workflow.id,
                &workflow.name,
                &workflow.definition,
                &workflow.rule_expression,
                workflow.version,
                workflow.is_active as i32,
                workflow.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn get_workflow(&self, id: &str) -> Result<Option<Workflow>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, definition, rule_expression, version, is_active, created_at
             FROM workflows WHERE id = ?1"
        )?;

        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Workflow {
                id: row.get(0)?,
                name: row.get(1)?,
                definition: row.get(2)?,
                rule_expression: row.get(3)?,
                version: row.get(4)?,
                is_active: row.get::<_, i32>(5)? != 0,
                created_at: row.get::<_, String>(6)?.parse().unwrap_or(Utc::now()),
            }))
        } else {
            Ok(None)
        }
    }

    pub fn get_all_workflows(&self) -> Result<Vec<Workflow>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, definition, rule_expression, version, is_active, created_at
             FROM workflows WHERE is_active = 1 ORDER BY created_at DESC"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(Workflow {
                id: row.get(0)?,
                name: row.get(1)?,
                definition: row.get(2)?,
                rule_expression: row.get(3)?,
                version: row.get(4)?,
                is_active: row.get::<_, i32>(5)? != 0,
                created_at: row.get::<_, String>(6)?.parse().unwrap_or(Utc::now()),
            })
        })?;

        let mut workflows = Vec::new();
        for workflow in rows {
            workflows.push(workflow?);
        }
        Ok(workflows)
    }

    // Training sample operations
    pub fn save_training_sample(&self, sample: &TrainingSample) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO training_samples (id, workflow_id, input_data, expected_result, actual_result, accuracy, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                &sample.id,
                &sample.workflow_id,
                &sample.input_data,
                sample.expected_result as i32,
                sample.actual_result.map(|r| r as i32),
                sample.accuracy,
                sample.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn get_training_samples(&self, workflow_id: &str, limit: u32) -> Result<Vec<TrainingSample>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, workflow_id, input_data, expected_result, actual_result, accuracy, created_at
             FROM training_samples WHERE workflow_id = ?1 ORDER BY created_at DESC LIMIT ?2"
        )?;

        let rows = stmt.query_map(params![workflow_id, limit as i32], |row| {
            Ok(TrainingSample {
                id: row.get(0)?,
                workflow_id: row.get(1)?,
                input_data: row.get(2)?,
                expected_result: row.get::<_, i32>(3)? != 0,
                actual_result: row.get::<_, Option<i32>>(4)?.map(|v| v != 0),
                accuracy: row.get(5)?,
                created_at: row.get::<_, String>(6)?.parse().unwrap_or(Utc::now()),
            })
        })?;

        let mut samples = Vec::new();
        for sample in rows {
            samples.push(sample?);
        }
        Ok(samples)
    }

    // Feedback operations
    pub fn save_feedback(&self, feedback: &Feedback) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO feedbacks (id, judgment_id, feedback_type, value, comment, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                &feedback.id,
                &feedback.judgment_id,
                &feedback.feedback_type,
                feedback.value,
                &feedback.comment,
                feedback.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    // PromptTemplate operations
    pub fn save_prompt_template(&self, template: &PromptTemplate) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO prompt_templates (id, name, template_type, content, variables, version, is_active, token_limit, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                template_type = excluded.template_type,
                content = excluded.content,
                variables = excluded.variables,
                version = excluded.version,
                is_active = excluded.is_active,
                token_limit = excluded.token_limit,
                updated_at = excluded.updated_at",
            params![
                &template.id,
                &template.name,
                &template.template_type,
                &template.content,
                &template.variables,
                template.version,
                template.is_active as i32,
                template.token_limit,
                template.created_at.to_rfc3339(),
                template.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn get_prompt_template(&self, id: &str) -> Result<Option<PromptTemplate>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, template_type, content, variables, version, is_active, token_limit, created_at, updated_at
             FROM prompt_templates WHERE id = ?1"
        )?;

        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(PromptTemplate {
                id: row.get(0)?,
                name: row.get(1)?,
                template_type: row.get(2)?,
                content: row.get(3)?,
                variables: row.get(4)?,
                version: row.get(5)?,
                is_active: row.get::<_, i32>(6)? != 0,
                token_limit: row.get(7)?,
                created_at: row.get::<_, String>(8)?.parse().unwrap_or(Utc::now()),
                updated_at: row.get::<_, String>(9)?.parse().unwrap_or(Utc::now()),
            }))
        } else {
            Ok(None)
        }
    }

    pub fn get_active_template_by_type(&self, template_type: &str) -> Result<Option<PromptTemplate>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, template_type, content, variables, version, is_active, token_limit, created_at, updated_at
             FROM prompt_templates
             WHERE template_type = ?1 AND is_active = 1
             ORDER BY version DESC LIMIT 1"
        )?;

        let mut rows = stmt.query(params![template_type])?;

        if let Some(row) = rows.next()? {
            Ok(Some(PromptTemplate {
                id: row.get(0)?,
                name: row.get(1)?,
                template_type: row.get(2)?,
                content: row.get(3)?,
                variables: row.get(4)?,
                version: row.get(5)?,
                is_active: row.get::<_, i32>(6)? != 0,
                token_limit: row.get(7)?,
                created_at: row.get::<_, String>(8)?.parse().unwrap_or(Utc::now()),
                updated_at: row.get::<_, String>(9)?.parse().unwrap_or(Utc::now()),
            }))
        } else {
            Ok(None)
        }
    }

    pub fn get_all_prompt_templates(&self) -> Result<Vec<PromptTemplate>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, template_type, content, variables, version, is_active, token_limit, created_at, updated_at
             FROM prompt_templates ORDER BY template_type, version DESC"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(PromptTemplate {
                id: row.get(0)?,
                name: row.get(1)?,
                template_type: row.get(2)?,
                content: row.get(3)?,
                variables: row.get(4)?,
                version: row.get(5)?,
                is_active: row.get::<_, i32>(6)? != 0,
                token_limit: row.get(7)?,
                created_at: row.get::<_, String>(8)?.parse().unwrap_or(Utc::now()),
                updated_at: row.get::<_, String>(9)?.parse().unwrap_or(Utc::now()),
            })
        })?;

        let mut templates = Vec::new();
        for template in rows {
            templates.push(template?);
        }
        Ok(templates)
    }

    pub fn delete_prompt_template(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM prompt_templates WHERE id = ?1", params![id])?;
        Ok(())
    }

    // Token Usage operations (MCP cost tracking)
    pub fn save_token_usage(&self, token_usage: &TokenUsage) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO token_usage (id, judgment_id, service, tokens_used, cost_usd, complexity, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                &token_usage.id,
                &token_usage.judgment_id,
                &token_usage.service,
                token_usage.tokens_used,
                token_usage.cost_usd,
                &token_usage.complexity,
                token_usage.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn get_token_usage_by_judgment(&self, judgment_id: &str) -> Result<Vec<TokenUsage>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, judgment_id, service, tokens_used, cost_usd, complexity, created_at
             FROM token_usage
             WHERE judgment_id = ?1
             ORDER BY created_at DESC"
        )?;

        let rows = stmt.query_map(params![judgment_id], |row| {
            Ok(TokenUsage {
                id: row.get(0)?,
                judgment_id: row.get(1)?,
                service: row.get(2)?,
                tokens_used: row.get(3)?,
                cost_usd: row.get(4)?,
                complexity: row.get(5)?,
                created_at: row.get::<_, String>(6)?.parse().unwrap(),
            })
        })?;

        let mut usages = Vec::new();
        for usage in rows {
            usages.push(usage?);
        }
        Ok(usages)
    }

    pub fn get_token_usage_by_date_range(
        &self,
        start_date: &str,
        end_date: &str,
    ) -> Result<Vec<TokenUsage>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, judgment_id, service, tokens_used, cost_usd, complexity, created_at
             FROM token_usage
             WHERE created_at >= ?1 AND created_at <= ?2
             ORDER BY created_at DESC"
        )?;

        let rows = stmt.query_map(params![start_date, end_date], |row| {
            Ok(TokenUsage {
                id: row.get(0)?,
                judgment_id: row.get(1)?,
                service: row.get(2)?,
                tokens_used: row.get(3)?,
                cost_usd: row.get(4)?,
                complexity: row.get(5)?,
                created_at: row.get::<_, String>(6)?.parse().unwrap(),
            })
        })?;

        let mut usages = Vec::new();
        for usage in rows {
            usages.push(usage?);
        }
        Ok(usages)
    }

    pub fn get_token_usage_by_service(
        &self,
        service: &str,
        limit: Option<u32>,
    ) -> Result<Vec<TokenUsage>> {
        let conn = self.conn.lock().unwrap();
        let query = if let Some(lim) = limit {
            format!(
                "SELECT id, judgment_id, service, tokens_used, cost_usd, complexity, created_at
                 FROM token_usage
                 WHERE service = ?1
                 ORDER BY created_at DESC
                 LIMIT {}",
                lim
            )
        } else {
            "SELECT id, judgment_id, service, tokens_used, cost_usd, complexity, created_at
             FROM token_usage
             WHERE service = ?1
             ORDER BY created_at DESC"
                .to_string()
        };

        let mut stmt = conn.prepare(&query)?;
        let rows = stmt.query_map(params![service], |row| {
            Ok(TokenUsage {
                id: row.get(0)?,
                judgment_id: row.get(1)?,
                service: row.get(2)?,
                tokens_used: row.get(3)?,
                cost_usd: row.get(4)?,
                complexity: row.get(5)?,
                created_at: row.get::<_, String>(6)?.parse().unwrap(),
            })
        })?;

        let mut usages = Vec::new();
        for usage in rows {
            usages.push(usage?);
        }
        Ok(usages)
    }

    /// Get aggregated token usage statistics for a given time range
    pub fn get_token_usage_summary(
        &self,
        start_date: &str,
        end_date: &str,
    ) -> Result<TokenUsageSummary> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT
                SUM(tokens_used) as total_tokens,
                SUM(cost_usd) as total_cost,
                COUNT(*) as total_requests,
                AVG(tokens_used) as avg_tokens_per_request,
                service
             FROM token_usage
             WHERE created_at >= ?1 AND created_at <= ?2
             GROUP BY service"
        )?;

        let mut rows = stmt.query(params![start_date, end_date])?;

        let mut summary = TokenUsageSummary {
            total_tokens: 0,
            total_cost_usd: 0.0,
            total_requests: 0,
            by_service: std::collections::HashMap::new(),
        };

        while let Some(row) = rows.next()? {
            let total_tokens: i32 = row.get(0)?;
            let total_cost: f64 = row.get(1)?;
            let total_requests: i32 = row.get(2)?;
            let avg_tokens: f64 = row.get(3)?;
            let service: String = row.get(4)?;

            summary.total_tokens += total_tokens;
            summary.total_cost_usd += total_cost;
            summary.total_requests += total_requests;

            summary.by_service.insert(service.clone(), ServiceUsageStats {
                total_tokens,
                total_cost_usd: total_cost,
                total_requests,
                avg_tokens_per_request: avg_tokens,
            });
        }

        Ok(summary)
    }

    /// Get simplified token metrics for Cost Dashboard
    ///
    /// Returns overall token usage metrics including cache savings
    pub fn get_token_metrics(&self) -> Result<TokenMetrics> {
        let conn = self.conn.lock().unwrap();

        // Get total tokens and cost
        let mut stmt = conn.prepare(
            "SELECT
                COALESCE(SUM(tokens_used), 0) as total_tokens,
                COALESCE(SUM(cost_usd), 0.0) as total_cost,
                COUNT(*) as total_requests
             FROM token_usage"
        )?;

        let (total_tokens, total_cost, total_requests) = stmt.query_row([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, f64>(1)?,
                row.get::<_, i64>(2)?
            ))
        })?;

        // Calculate average tokens per request
        let avg_tokens = if total_requests > 0 {
            total_tokens as f64 / total_requests as f64
        } else {
            0.0
        };

        // Estimate cache savings (assume 70% token reduction when cache hit)
        // For now, use a simplified heuristic: if Context7 is in service, estimate savings
        let mut cache_stmt = conn.prepare(
            "SELECT COUNT(*) FROM token_usage WHERE service = 'context7' AND complexity = 'simple'"
        )?;
        let cache_hits: i64 = cache_stmt.query_row([], |row| row.get(0))?;

        let tokens_saved = (cache_hits as f64 * avg_tokens * 0.7) as i64;
        let cost_saved = tokens_saved as f64 * 0.000002; // Approximate $0.002 per 1K tokens

        // Calculate cache hit rate (rough estimate)
        let cache_hit_rate = if total_requests > 0 {
            (cache_hits as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        };

        Ok(TokenMetrics {
            total_tokens_used: total_tokens,
            total_cost_usd: total_cost,
            tokens_saved_by_cache: tokens_saved,
            cost_saved_usd: cost_saved,
            cache_hit_rate,
            avg_tokens_per_request: avg_tokens,
        })
    }
}

// Token Usage summary structs
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenUsageSummary {
    pub total_tokens: i32,
    pub total_cost_usd: f64,
    pub total_requests: i32,
    pub by_service: std::collections::HashMap<String, ServiceUsageStats>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceUsageStats {
    pub total_tokens: i32,
    pub total_cost_usd: f64,
    pub total_requests: i32,
    pub avg_tokens_per_request: f64,
}

/// Simplified token metrics for Cost Dashboard
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenMetrics {
    pub total_tokens_used: i64,
    pub total_cost_usd: f64,
    pub tokens_saved_by_cache: i64,
    pub cost_saved_usd: f64,
    pub cache_hit_rate: f64,
    pub avg_tokens_per_request: f64,
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Self {
            conn: Arc::clone(&self.conn),
        }
    }
}
