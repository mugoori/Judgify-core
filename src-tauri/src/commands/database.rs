use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;
use crate::database::Database;

#[derive(Debug, Serialize, Deserialize)]
pub struct TableInfo {
    pub name: String,
    pub display_name: String,  // 사용자에게 보여줄 한글 이름
    pub description: Option<String>,  // 테이블 설명 (선택사항)
    pub row_count: i64,
    pub columns: Vec<ColumnInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<HashMap<String, serde_json::Value>>,
    pub total_count: i64,
}

#[tauri::command]
pub async fn get_database_tables(
    database: State<'_, Database>,
) -> Result<Vec<TableInfo>, String> {
    println!("📊 데이터베이스 테이블 목록 조회 시작");

    let db = database.inner();
    let conn = db.get_connection();
    let conn = conn.lock().map_err(|e| {
        eprintln!("❌ DB 연결 잠금 실패: {}", e);
        format!("데이터베이스 연결 잠금 실패: {}", e)
    })?;

    // 뷰어에 표시할 핵심 테이블 2개만 조회
    let mut tables = Vec::new();

    for display_info in DISPLAY_TABLES {
        // 테이블 존재 여부 확인
        let exists_query = format!(
            "SELECT 1 FROM sqlite_master WHERE type='table' AND name='{}'",
            display_info.name
        );

        let exists = conn.query_row(&exists_query, [], |_| Ok(true))
            .unwrap_or(false);

        if !exists {
            println!("⚠️ 테이블 {} 없음, 건너뜀", display_info.name);
            continue;
        }

        // 테이블 정보 생성
        let mut table_info = TableInfo {
            name: display_info.name.to_string(),
            display_name: display_info.display_name.to_string(),
            description: Some(display_info.description.to_string()),
            row_count: 0,
            columns: Vec::new(),
        };

        // 행 개수 조회
        let count_query = format!("SELECT COUNT(*) FROM {}", display_info.name);
        table_info.row_count = conn.query_row(&count_query, [], |row| {
            row.get(0)
        }).unwrap_or(0);

        // 컬럼 정보 조회 (PRAGMA 사용)
        let pragma_query = format!("PRAGMA table_info({})", display_info.name);
        let mut pragma_stmt = conn.prepare(&pragma_query).map_err(|e| {
            format!("컬럼 정보 조회 실패: {}", e)
        })?;

        let columns = pragma_stmt.query_map([], |row| {
            Ok(ColumnInfo {
                name: row.get(1)?,
                data_type: row.get(2)?,
                is_nullable: row.get::<_, i32>(3)? == 0,
            })
        }).map_err(|e| format!("컬럼 정보 파싱 실패: {}", e))?;

        for column in columns.filter_map(Result::ok) {
            table_info.columns.push(column);
        }

        tables.push(table_info);
    }

    println!("✅ 테이블 정보 조회 완료: {} 개 테이블", tables.len());
    Ok(tables)
}

#[tauri::command]
pub async fn query_table_data(
    database: State<'_, Database>,
    table_name: String,
    limit: Option<i32>,
    offset: Option<i32>,
) -> Result<QueryResult, String> {
    println!("📊 테이블 데이터 조회: {}", table_name);

    // SQL Injection 방지를 위한 테이블명 검증
    if !is_valid_table_name(&table_name) {
        eprintln!("⚠️ 허용되지 않은 테이블 접근 시도: {}", table_name);
        return Err(format!("'{}' 테이블은 접근이 허용되지 않습니다.", table_name));
    }

    let db = database.inner();
    let conn = db.get_connection();
    let conn = conn.lock().map_err(|e| {
        eprintln!("❌ DB 연결 잠금 실패: {}", e);
        format!("데이터베이스 연결 잠금 실패: {}", e)
    })?;

    // 추가 보안: 실제로 테이블이 존재하는지 확인
    let table_exists: bool = conn
        .prepare("SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1")
        .and_then(|mut stmt| stmt.exists([&table_name]))
        .unwrap_or(false);

    if !table_exists {
        eprintln!("⚠️ 존재하지 않는 테이블 접근 시도: {}", table_name);
        return Err(format!("'{}' 테이블이 존재하지 않습니다.", table_name));
    }

    // 전체 행 개수 조회
    let count_query = format!("SELECT COUNT(*) FROM {}", table_name);
    let total_count: i64 = conn.query_row(&count_query, [], |row| {
        row.get(0)
    }).map_err(|e| format!("행 개수 조회 실패: {}", e))?;

    // 데이터 조회 (페이징 지원)
    let limit_val = limit.unwrap_or(100);
    let offset_val = offset.unwrap_or(0);

    let data_query = format!(
        "SELECT * FROM {} LIMIT {} OFFSET {}",
        table_name, limit_val, offset_val
    );

    let mut stmt = conn.prepare(&data_query).map_err(|e| {
        format!("데이터 조회 준비 실패: {}", e)
    })?;

    // 컬럼 이름 가져오기
    let column_count = stmt.column_count();
    let mut columns = Vec::new();
    for i in 0..column_count {
        match stmt.column_name(i) {
            Ok(name) => columns.push(name.to_string()),
            Err(_) => columns.push(format!("column_{}", i)),
        }
    }

    // 데이터 행 가져오기
    let rows_result = stmt.query_map([], |row| {
        let mut row_data = HashMap::new();
        for (i, col_name) in columns.iter().enumerate() {
            // SQLite의 동적 타입 처리
            let value = match row.get::<_, rusqlite::types::Value>(i) {
                Ok(rusqlite::types::Value::Null) => serde_json::Value::Null,
                Ok(rusqlite::types::Value::Integer(i)) => serde_json::Value::Number(i.into()),
                Ok(rusqlite::types::Value::Real(f)) => {
                    serde_json::Value::Number(serde_json::Number::from_f64(f).unwrap_or(0.into()))
                },
                Ok(rusqlite::types::Value::Text(s)) => serde_json::Value::String(s),
                Ok(rusqlite::types::Value::Blob(b)) => {
                    serde_json::Value::String(format!("[BLOB: {} bytes]", b.len()))
                },
                Err(_) => serde_json::Value::Null,
            };
            row_data.insert(col_name.clone(), value);
        }
        Ok(row_data)
    }).map_err(|e| format!("데이터 조회 실패: {}", e))?;

    let mut rows = Vec::new();
    for row in rows_result {
        if let Ok(r) = row {
            rows.push(r);
        }
    }

    println!("✅ {} 행 조회 완료 (전체: {} 행)", rows.len(), total_count);

    Ok(QueryResult {
        columns,
        rows,
        total_count,
    })
}

#[tauri::command]
pub async fn execute_custom_query(
    database: State<'_, Database>,
    query: String,
) -> Result<QueryResult, String> {
    println!("📊 사용자 정의 쿼리 실행");

    // 읽기 전용 쿼리만 허용 (SELECT로 시작)
    let trimmed_query = query.trim().to_uppercase();
    if !trimmed_query.starts_with("SELECT") {
        return Err("SELECT 쿼리만 실행 가능합니다.".to_string());
    }

    let db = database.inner();
    let conn = db.get_connection();
    let conn = conn.lock().map_err(|e| {
        eprintln!("❌ DB 연결 잠금 실패: {}", e);
        format!("데이터베이스 연결 잠금 실패: {}", e)
    })?;

    let mut stmt = conn.prepare(&query).map_err(|e| {
        format!("쿼리 준비 실패: {}", e)
    })?;

    // 컬럼 이름 가져오기
    let column_count = stmt.column_count();
    let mut columns = Vec::new();
    for i in 0..column_count {
        match stmt.column_name(i) {
            Ok(name) => columns.push(name.to_string()),
            Err(_) => columns.push(format!("column_{}", i)),
        }
    }

    // 데이터 행 가져오기
    let rows_result = stmt.query_map([], |row| {
        let mut row_data = HashMap::new();
        for (i, col_name) in columns.iter().enumerate() {
            let value = match row.get::<_, rusqlite::types::Value>(i) {
                Ok(rusqlite::types::Value::Null) => serde_json::Value::Null,
                Ok(rusqlite::types::Value::Integer(i)) => serde_json::Value::Number(i.into()),
                Ok(rusqlite::types::Value::Real(f)) => {
                    serde_json::Value::Number(serde_json::Number::from_f64(f).unwrap_or(0.into()))
                },
                Ok(rusqlite::types::Value::Text(s)) => serde_json::Value::String(s),
                Ok(rusqlite::types::Value::Blob(b)) => {
                    serde_json::Value::String(format!("[BLOB: {} bytes]", b.len()))
                },
                Err(_) => serde_json::Value::Null,
            };
            row_data.insert(col_name.clone(), value);
        }
        Ok(row_data)
    }).map_err(|e| format!("쿼리 실행 실패: {}", e))?;

    let mut rows = Vec::new();
    for row in rows_result {
        if let Ok(r) = row {
            rows.push(r);
        }
    }

    let total_count = rows.len() as i64;
    println!("✅ 쿼리 실행 완료: {} 행 반환", total_count);

    Ok(QueryResult {
        columns,
        rows,
        total_count,
    })
}

// 데이터베이스 뷰어에 표시할 핵심 테이블 정의
struct DisplayTableInfo {
    name: &'static str,
    display_name: &'static str,
    description: &'static str,
}

// 사용자에게 표시할 핵심 테이블 (한글 이름으로 친화적 표시)
const DISPLAY_TABLES: &[DisplayTableInfo] = &[
    // === 생산 관리 ===
    DisplayTableInfo {
        name: "production_order",
        display_name: "생산 지시서",
        description: "일별 생산 지시 내역 (3개월간 91건)",
    },
    DisplayTableInfo {
        name: "batch_lot",
        display_name: "배치 생산 기록",
        description: "배합 공정 실행 기록 (작업자, 시간 포함)",
    },
    DisplayTableInfo {
        name: "filling_lot",
        display_name: "충진 기록",
        description: "충진 공정 실행 내역",
    },
    DisplayTableInfo {
        name: "fg_lot",
        display_name: "완제품 LOT",
        description: "완제품 생산 LOT 정보 (제조일자, 유통기한)",
    },

    // === 품질 관리 ===
    DisplayTableInfo {
        name: "ccp_check_log",
        display_name: "CCP 검사 기록",
        description: "살균 온도 등 핵심 관리점 측정 기록 (3.3% 불량률)",
    },
    DisplayTableInfo {
        name: "qc_test",
        display_name: "품질 검사",
        description: "pH, Brix, 미생물 등 품질 검사 결과",
    },
    DisplayTableInfo {
        name: "operation_param_log",
        display_name: "공정 파라미터",
        description: "공정별 온도, 압력 등 파라미터 측정값",
    },

    // === 재고/자재 관리 ===
    DisplayTableInfo {
        name: "item_mst",
        display_name: "제품/원료 마스터",
        description: "프로바이오틱스, 정제수 등 품목 정보",
    },
    DisplayTableInfo {
        name: "purchase_order",
        display_name: "구매 발주",
        description: "원료 구매 발주서",
    },
    DisplayTableInfo {
        name: "inbound",
        display_name: "입고 기록",
        description: "원료 입고 내역",
    },

    // === MES 실행 ===
    DisplayTableInfo {
        name: "mes_work_order",
        display_name: "MES 작업 지시",
        description: "MES 시스템 작업 지시서",
    },
    DisplayTableInfo {
        name: "operation_exec",
        display_name: "공정 실행",
        description: "배합, 살균, 충진 등 공정 실행 기록",
    },

    // === 기준 정보 ===
    DisplayTableInfo {
        name: "line_mst",
        display_name: "생산 라인",
        description: "L1-유산균라인, L2-단백질라인",
    },
    DisplayTableInfo {
        name: "operation_mst",
        display_name: "공정 마스터",
        description: "배합, 살균(CCP), 충진, 금속검출(CCP) 공정 정보",
    },
    DisplayTableInfo {
        name: "vendor_mst",
        display_name: "거래처 정보",
        description: "원료 공급업체 정보",
    },
    DisplayTableInfo {
        name: "customer_mst",
        display_name: "고객사 정보",
        description: "쿠팡, 마켓컬리 등 고객사",
    },
];

// 뷰어에 표시할 테이블인지 검증 (향후 사용 예정)
#[allow(dead_code)]
fn is_display_table(name: &str) -> Option<&'static DisplayTableInfo> {
    DISPLAY_TABLES.iter().find(|t| t.name == name)
}

// API 접근 가능한 모든 테이블 (보안용)
const ALLOWED_TABLES: &[&str] = &[
    // === ERP/MES 마스터 테이블 ===
    "item_mst",           // 품목 마스터
    "vendor_mst",         // 거래처 마스터
    "customer_mst",       // 고객사 마스터
    "bom_mst",           // BOM 마스터
    "bom_dtl",           // BOM 상세

    // === ERP 트랜잭션 테이블 ===
    "purchase_order",     // 구매 발주
    "purchase_order_dtl", // 구매 발주 상세
    "inbound",           // 입고
    "inbound_dtl",       // 입고 상세
    "production_order",   // 생산 지시
    "batch_lot",         // 배치 LOT
    "filling_lot",       // 충진 LOT
    "fg_lot",            // 완제품 LOT
    "qc_test",           // 품질 검사

    // === MES 마스터 테이블 ===
    "line_mst",          // 라인 마스터
    "equipment_mst",     // 설비 마스터
    "operation_mst",     // 공정 마스터
    "param_mst",         // 파라미터 마스터

    // === MES 실행 테이블 ===
    "mes_work_order",    // MES 작업 지시
    "operation_exec",    // 공정 실행
    "operation_param_log", // 공정 파라미터 로그
    "ccp_check_log",     // CCP 체크 로그

    // === 기존 테이블 (호환성 유지) ===
    "ccp_docs",          // CCP 문서 (향후 사용)
    "mes_data_logs",     // MES 데이터 로그 (향후 사용)
];

// API 접근 권한 검증 (보안용)
fn is_valid_table_name(name: &str) -> bool {
    ALLOWED_TABLES.contains(&name)
}