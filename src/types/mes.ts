/**
 * MES/ERP RAG 타입 정의
 *
 * Rust 백엔드 타입과 1:1 매핑:
 * - src-tauri/src/commands/mes.rs
 */

/**
 * MES/ERP 데이터 업로드 결과
 */
export interface MesUploadResult {
  success: boolean;
  row_count: number;
  file_name: string;
}

/**
 * MES/ERP 데이터 쿼리 결과
 */
export interface MesQueryResult {
  answer: string | null;
  has_data: boolean;
}

/**
 * MES/ERP 세션 통계
 */
export interface MesSessionStats {
  file_name: string;
  row_count: number;
  uploaded_at: string;
}

/**
 * MES/ERP RAG 세션 상태
 */
export interface MesSessionState {
  sessionId: string;
  uploadedFile: {
    name: string;
    rowCount: number;
    uploadedAt: string;
  } | null;
  isUploading: boolean;
  uploadProgress: number;
}
