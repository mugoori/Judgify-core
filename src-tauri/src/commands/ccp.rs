use crate::services::ccp_service::CcpService;
use crate::database::{CcpDocWithScore, CcpJudgmentRequest, CcpJudgmentResponse};

/// Tauri command: CCP 문서 검색 (FTS5 BM25)
///
/// Frontend 사용 예시:
/// ```typescript
/// const docs = await invoke('search_ccp_docs', {
///   companyId: 'COMP_A',
///   ccpId: 'CCP-01',
///   query: '열처리 기준',
///   topK: 5
/// });
/// ```
#[tauri::command]
pub async fn search_ccp_docs(
    company_id: String,
    ccp_id: Option<String>,
    query: String,
    top_k: usize,
) -> Result<Vec<CcpDocWithScore>, String> {
    println!("🔍 [IPC] search_ccp_docs called!");
    println!("   company_id: {}", company_id);
    println!("   ccp_id: {:?}", ccp_id);
    println!("   query: {}", query);
    println!("   top_k: {}", top_k);

    let service = CcpService::new()
        .map_err(|e| format!("Service 초기화 실패: {}", e))?;

    let results = service.search_ccp_docs(
        &company_id,
        ccp_id.as_deref(),
        &query,
        top_k,
    )
    .map_err(|e| format!("검색 실패: {}", e))?;

    println!("   결과: {}건", results.len());
    Ok(results)
}

/// Tauri command: 데이터베이스 디버깅 (데이터 존재 여부 확인)
///
/// Frontend 사용 예시:
/// ```typescript
/// const debugInfo = await invoke('debug_ccp_database');
/// console.log('CCP Docs 테이블:', debugInfo.ccp_docs_count);
/// console.log('FTS5 인덱스:', debugInfo.fts_index_count);
/// ```
#[tauri::command]
pub async fn debug_ccp_database() -> Result<serde_json::Value, String> {
    use crate::database::Database;

    let db = Database::new()
        .map_err(|e| format!("DB 초기화 실패: {}", e))?;

    let db_conn = db.get_connection();
    let conn = db_conn.lock()
        .map_err(|e| format!("DB lock 실패: {}", e))?;

    // 1. ccp_docs 테이블 row 개수 확인
    let docs_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM ccp_docs",
        [],
        |row| row.get(0),
    ).map_err(|e| format!("ccp_docs 개수 조회 실패: {}", e))?;

    // 2. FTS5 인덱스 row 개수 확인
    let fts_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM ccp_docs_fts",
        [],
        |row| row.get(0),
    ).map_err(|e| format!("FTS5 개수 조회 실패: {}", e))?;

    // 3. 샘플 데이터 1개 확인 (실제 content 확인)
    let sample_doc: Option<(i64, String, String, String)> = conn.query_row(
        "SELECT id, company_id, ccp_id, title FROM ccp_docs LIMIT 1",
        [],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
    ).ok();

    // 4. "온도" 키워드를 포함한 문서 개수 확인 (LIKE 검색)
    let temp_keyword_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM ccp_docs WHERE content LIKE '%온도%' OR title LIKE '%온도%'",
        [],
        |row| row.get(0),
    ).map_err(|e| format!("'온도' 키워드 개수 조회 실패: {}", e))?;

    // 5. FTS5 직접 테스트 (MATCH 검색)
    let fts_match_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM ccp_docs_fts WHERE ccp_docs_fts MATCH '온도'",
        [],
        |row| row.get(0),
    ).map_err(|e| format!("FTS5 MATCH 테스트 실패: {}", e))?;

    println!("🔍 [DEBUG] Database Status:");
    println!("   ccp_docs 테이블: {}건", docs_count);
    println!("   FTS5 인덱스: {}건", fts_count);
    println!("   '온도' LIKE 검색: {}건", temp_keyword_count);
    println!("   '온도' FTS MATCH: {}건", fts_match_count);
    if let Some((id, company, ccp, title)) = &sample_doc {
        println!("   샘플 문서: ID={}, {}_{}, {}", id, company, ccp, title);
    }

    Ok(serde_json::json!({
        "ccp_docs_count": docs_count,
        "fts_index_count": fts_count,
        "temp_keyword_like_count": temp_keyword_count,
        "temp_keyword_fts_match_count": fts_match_count,
        "sample_doc": sample_doc.map(|(id, company, ccp, title)| {
            serde_json::json!({
                "id": id,
                "company_id": company,
                "ccp_id": ccp,
                "title": title
            })
        })
    }))
}

/// Tauri command: FTS5 인덱스 강제 rebuild
///
/// Frontend 사용 예시:
/// ```typescript
/// const result = await invoke('rebuild_fts5_index');
/// console.log('Rebuild 완료:', result.message);
/// ```
#[tauri::command]
pub async fn rebuild_fts5_index() -> Result<serde_json::Value, String> {
    use crate::database::Database;

    println!("🔄 [FTS5] Rebuild 시작...");

    let db = Database::new()
        .map_err(|e| format!("DB 초기화 실패: {}", e))?;

    let db_conn = db.get_connection();
    let conn = db_conn.lock()
        .map_err(|e| format!("DB lock 실패: {}", e))?;

    // FTS5 rebuild 실행 (External Content 방식)
    // Step 1: 기존 데이터 삭제
    conn.execute_batch("DELETE FROM ccp_docs_fts")
        .map_err(|e| format!("FTS5 기존 데이터 삭제 실패: {}", e))?;

    // Step 2: 원본 테이블에서 데이터 복사
    conn.execute_batch(r#"
        INSERT INTO ccp_docs_fts (rowid, title, content)
        SELECT id, title, content FROM ccp_docs
    "#)
        .map_err(|e| format!("FTS5 데이터 삽입 실패: {}", e))?;

    // Rebuild 후 인덱스 개수 확인
    let fts_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM ccp_docs_fts",
        [],
        |row| row.get(0),
    ).map_err(|e| format!("FTS5 개수 조회 실패: {}", e))?;

    println!("✅ [FTS5] Rebuild 완료: {}건", fts_count);

    Ok(serde_json::json!({
        "success": true,
        "message": format!("FTS5 인덱스가 성공적으로 rebuild되었습니다 ({}건)", fts_count),
        "fts_count": fts_count
    }))
}

/// Tauri command: CCP 상태 판단 (하이브리드)
///
/// Frontend 사용 예시:
/// ```typescript
/// const result = await invoke('judge_ccp_status', {
///   request: {
///     company_id: 'COMP_A',
///     ccp_id: 'CCP-01',
///     period_from: '2025-11-01',
///     period_to: '2025-11-14'
///   }
/// });
/// console.log('위험도:', result.risk_level);
/// console.log('AI 요약:', result.llm_summary);
/// console.log('증거 문서:', result.evidence_docs);
/// ```
#[tauri::command]
pub async fn judge_ccp_status(
    request: CcpJudgmentRequest,
) -> Result<CcpJudgmentResponse, String> {
    let service = CcpService::new()
        .map_err(|e| format!("Service 초기화 실패: {}", e))?;

    service.judge_ccp_status(request)
        .await
        .map_err(|e| format!("판단 실패: {}", e))
}
