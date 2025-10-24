use rusqlite::{Connection, Result, params};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use crate::database::models::*;
use chrono::Utc;

pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
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

            CREATE INDEX IF NOT EXISTS idx_judgments_workflow ON judgments(workflow_id);
            CREATE INDEX IF NOT EXISTS idx_judgments_created ON judgments(created_at);
            CREATE INDEX IF NOT EXISTS idx_training_workflow ON training_samples(workflow_id);"
        )?;

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
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Self {
            conn: Arc::clone(&self.conn),
        }
    }
}
