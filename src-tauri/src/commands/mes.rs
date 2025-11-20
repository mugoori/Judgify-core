use crate::services::mes_data_service::MesDataService;
use serde::{Deserialize, Serialize};

/// MES/ERP 데이터 업로드 결과
#[derive(Debug, Serialize, Deserialize)]
pub struct MesUploadResult {
    pub success: bool,
    pub row_count: usize,
    pub file_name: String,
}

/// MES/ERP 데이터 쿼리 결과
#[derive(Debug, Serialize, Deserialize)]
pub struct MesQueryResult {
    pub answer: Option<String>,
    pub has_data: bool,
}

/// MES/ERP 세션 통계
#[derive(Debug, Serialize, Deserialize)]
pub struct MesSessionStats {
    pub file_name: String,
    pub row_count: usize,
    pub uploaded_at: String,
}

/// Tauri command: MES/ERP CSV 파일 업로드
///
/// Frontend 사용 예시:
/// ```typescript
/// const file = document.querySelector('input[type="file"]').files[0];
/// const arrayBuffer = await file.arrayBuffer();
/// const uint8Array = new Uint8Array(arrayBuffer);
/// const fileContent = Array.from(uint8Array);
///
/// const result = await invoke<MesUploadResult>('upload_mes_data', {
///   sessionId: crypto.randomUUID(),
///   fileName: file.name,
///   fileContent
/// });
/// console.log(`${result.row_count}건 업로드 완료`);
/// ```
#[tauri::command]
pub async fn upload_mes_data(
    session_id: String,
    file_name: String,
    file_content: Vec<u8>,
) -> Result<MesUploadResult, String> {
    println!("📤 [IPC] upload_mes_data called!");
    println!("   session_id: {}", session_id);
    println!("   file_name: {}", file_name);
    println!("   file_size: {} bytes", file_content.len());

    let service = MesDataService::new()
        .map_err(|e| format!("Service 초기화 실패: {}", e))?;

    let row_count = service.upload_mes_data(
        &session_id,
        &file_name,
        &file_content,
    )
    .map_err(|e| format!("업로드 실패: {}", e))?;

    println!("   ✅ 업로드 완료: {}건", row_count);

    Ok(MesUploadResult {
        success: true,
        row_count,
        file_name: file_name.clone(),
    })
}

/// Tauri command: MES/ERP 데이터 자연어 쿼리
///
/// Frontend 사용 예시:
/// ```typescript
/// const result = await invoke<MesQueryResult>('query_mes_data', {
///   sessionId: sessionId,
///   question: '온도가 90도 이상인 데이터는?',
///   topK: 5
/// });
///
/// if (result.answer) {
///   console.log('LLM 답변:', result.answer);
/// } else {
///   console.log('데이터 없음');
/// }
/// ```
#[tauri::command]
pub async fn query_mes_data(
    session_id: String,
    question: String,
    top_k: Option<usize>,
) -> Result<MesQueryResult, String> {
    println!("🔍 [IPC] query_mes_data called!");
    println!("   session_id: {}", session_id);
    println!("   question: {}", question);
    println!("   top_k: {:?}", top_k);

    let service = MesDataService::new()
        .map_err(|e| format!("Service 초기화 실패: {}", e))?;

    let answer = service.query_mes_data(
        &session_id,
        &question,
        top_k.unwrap_or(5),
    )
    .await
    .map_err(|e| format!("쿼리 실패: {}", e))?;

    let has_data = answer.is_some();

    println!("   ✅ 쿼리 완료: {}", if has_data { "답변 생성" } else { "데이터 없음" });

    Ok(MesQueryResult {
        answer,
        has_data,
    })
}

/// Tauri command: MES/ERP 세션 데이터 삭제
///
/// Frontend 사용 예시:
/// ```typescript
/// await invoke('delete_mes_session', {
///   sessionId: sessionId
/// });
/// console.log('세션 데이터 삭제 완료');
/// ```
#[tauri::command]
pub async fn delete_mes_session(
    session_id: String,
) -> Result<usize, String> {
    println!("🗑️  [IPC] delete_mes_session called!");
    println!("   session_id: {}", session_id);

    let service = MesDataService::new()
        .map_err(|e| format!("Service 초기화 실패: {}", e))?;

    let deleted = service.delete_session_data(&session_id)
        .map_err(|e| format!("삭제 실패: {}", e))?;

    println!("   ✅ 삭제 완료: {}건", deleted);

    Ok(deleted)
}

/// Tauri command: MES/ERP 세션 통계 조회
///
/// Frontend 사용 예시:
/// ```typescript
/// const stats = await invoke<MesSessionStats>('get_mes_session_stats', {
///   sessionId: sessionId
/// });
/// console.log(`${stats.file_name} (${stats.row_count}건)`);
/// ```
#[tauri::command]
pub async fn get_mes_session_stats(
    session_id: String,
) -> Result<Option<MesSessionStats>, String> {
    println!("📊 [IPC] get_mes_session_stats called!");
    println!("   session_id: {}", session_id);

    let service = MesDataService::new()
        .map_err(|e| format!("Service 초기화 실패: {}", e))?;

    let stats = service.get_session_stats(&session_id)
        .map_err(|e| format!("통계 조회 실패: {}", e))?;

    let result = stats.map(|(file_name, row_count, uploaded_at)| {
        MesSessionStats {
            file_name,
            row_count,
            uploaded_at,
        }
    });

    println!("   ✅ 조회 완료: {:?}", result);

    Ok(result)
}
