-- ========================================
-- 010_additional_erp_mes.sql
-- 추가 ERP/MES 테이블 (사용자 요청 기반)
-- ========================================

PRAGMA foreign_keys = ON;

-- ========================================
-- 1. MES 품질관리 테이블 (Quality Control)
-- ========================================

-- 품질 검사 데이터 (수입검사, 공정검사, 완제품검사)
CREATE TABLE IF NOT EXISTS qc_inspection (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    inspection_no TEXT NOT NULL UNIQUE,           -- 검사번호
    inspection_type TEXT NOT NULL CHECK (inspection_type IN ('INCOMING', 'IN_PROCESS', 'FINAL')),
    lot_no TEXT NOT NULL,                         -- LOT 번호
    item_cd TEXT NOT NULL,                        -- 품목코드
    inspection_time TEXT NOT NULL DEFAULT (datetime('now')),
    inspector_id TEXT,                            -- 검사자

    -- 품질 측정값들
    ph_level REAL,                                -- pH농도
    acidity REAL,                                 -- 산도
    brix REAL,                                    -- 당도 (Brix)
    fat_content REAL,                             -- 유지방 함량 (%)
    protein_content REAL,                         -- 단백질 함량 (%)
    moisture REAL,                                -- 수분 (%)
    total_bacteria REAL,                          -- 세균수 (CFU/ml)
    coliform INTEGER,                             -- 대장균군 유무 (0/1)

    -- 물성 검사
    viscosity REAL,                               -- 점도
    color_l REAL,                                 -- 색도 L (밝기)
    color_a REAL,                                 -- 색도 a (적-녹)
    color_b REAL,                                 -- 색도 b (황-청)

    -- 판정
    result TEXT CHECK (result IN ('PASS', 'FAIL', 'CONDITIONAL')),
    deviation_items TEXT,                         -- JSON: 규격 벗어난 항목들
    remark TEXT,

    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (item_cd) REFERENCES item_mst(item_cd)
);

-- 금속검출 로그
CREATE TABLE IF NOT EXISTS metal_detection_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    detection_time TEXT NOT NULL DEFAULT (datetime('now')),
    equip_cd TEXT NOT NULL,                       -- 금속검출기 설비코드
    line_cd TEXT NOT NULL,                        -- 라인코드
    lot_no TEXT,                                  -- LOT 번호

    metal_detected INTEGER NOT NULL DEFAULT 0,   -- 금속 검출 여부 (True/False)
    metal_type TEXT CHECK (metal_type IN ('FE', 'SUS', 'NON_FE', NULL)),  -- 검출 금속 종류
    sensitivity_fe REAL,                          -- Fe 감도 (mm)
    sensitivity_sus REAL,                         -- SUS 감도 (mm)
    sensitivity_non_fe REAL,                      -- 비철 감도 (mm)

    reject_action TEXT CHECK (reject_action IN ('REJECTED', 'PASSED', 'RECHECK')),
    operator_id TEXT,
    remark TEXT,

    FOREIGN KEY (equip_cd) REFERENCES equipment_mst(equip_cd),
    FOREIGN KEY (line_cd) REFERENCES line_mst(line_cd)
);

-- ========================================
-- 2. MES 공정 데이터 테이블 (Process Data)
-- ========================================

-- 공정 상세 파라미터 로그 (설비별 상세 데이터)
CREATE TABLE IF NOT EXISTS process_param_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    recorded_at TEXT NOT NULL DEFAULT (datetime('now')),
    equip_cd TEXT NOT NULL,                       -- 설비ID (machine_id)
    batch_lot_no TEXT,                            -- 배치 LOT

    -- 살균 관련
    sterilization_temp REAL,                      -- 살균온도
    holding_time_sec INTEGER,                     -- 살균 유지 시간 (초)

    -- 균질화 관련
    homogenizer_pressure REAL,                    -- 균질압력 (bar)

    -- 탱크 관련
    tank_temp REAL,                               -- 탱크온도
    tank_level REAL,                              -- 탱크 레벨 (%)

    -- CIP 관련
    cip_status INTEGER DEFAULT 0,                 -- CIP 상태 (0:생산, 1:세척)
    cip_step TEXT,                                -- CIP 단계
    cip_temp REAL,                                -- CIP 세정액 온도
    cip_conductivity REAL,                        -- 전도도 (세정액 농도)

    -- 충진 관련
    fill_speed REAL,                              -- 충진 속도 (개/분)
    fill_volume REAL,                             -- 충진량 (ml)
    fill_temp REAL,                               -- 충진 온도

    -- 냉각 관련
    cooling_temp REAL,                            -- 냉각 온도
    glycol_temp REAL,                             -- 글리콜 온도

    is_alarm INTEGER DEFAULT 0,
    alarm_message TEXT

    -- FK 제거: 유연한 데이터 입력 허용
    -- FOREIGN KEY (equip_cd) REFERENCES equipment_mst(equip_cd),
    -- FOREIGN KEY (batch_lot_no) REFERENCES batch_lot(batch_lot_no)
);

-- ========================================
-- 3. MES 자재 투입 이력 (Material Input)
-- ========================================

-- 자재 투입 이력 (생산 현장)
CREATE TABLE IF NOT EXISTS material_input_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    input_time TEXT NOT NULL DEFAULT (datetime('now')),
    batch_lot_no TEXT NOT NULL,                   -- 배치 LOT
    wo_no TEXT,                                   -- 작업지시번호

    material_lot_no TEXT NOT NULL,                -- 투입자재 LOT
    item_cd TEXT NOT NULL,                        -- 품목코드
    item_nm TEXT,                                 -- 품목명 (조회 편의)

    plan_qty REAL,                                -- 계획 투입량
    input_qty REAL NOT NULL,                      -- 실제 투입량
    remain_qty REAL,                              -- 투입 후 잔량
    unit TEXT DEFAULT 'KG',

    operator_id TEXT,                             -- 투입 작업자
    equip_cd TEXT,                                -- 투입 설비

    is_verified INTEGER DEFAULT 0,                -- 검증 여부 (바코드 스캔)
    verification_time TEXT,

    remark TEXT

    -- FK 제거: 유연한 데이터 입력 허용
    -- FOREIGN KEY (batch_lot_no) REFERENCES batch_lot(batch_lot_no),
    -- FOREIGN KEY (item_cd) REFERENCES item_mst(item_cd)
);

-- ========================================
-- 4. ERP 창고/위치 마스터
-- ========================================

-- 창고 마스터
CREATE TABLE IF NOT EXISTS warehouse_mst (
    warehouse_id TEXT PRIMARY KEY,                -- 창고코드
    warehouse_nm TEXT NOT NULL,                   -- 창고명
    warehouse_type TEXT CHECK (warehouse_type IN ('RAW', 'WIP', 'FG', 'COLD', 'FROZEN', 'SHIPPING')),
    location TEXT,                                -- 위치
    capacity REAL,                                -- 용량
    temp_min REAL,                                -- 최소 온도
    temp_max REAL,                                -- 최대 온도
    is_active INTEGER DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 재고 이동 이력
CREATE TABLE IF NOT EXISTS inventory_movement (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    movement_no TEXT NOT NULL UNIQUE,
    movement_type TEXT NOT NULL CHECK (movement_type IN ('IN', 'OUT', 'TRANSFER', 'ADJUST', 'SCRAP')),
    movement_date TEXT NOT NULL DEFAULT (datetime('now')),

    item_cd TEXT NOT NULL,
    lot_no TEXT NOT NULL,
    qty REAL NOT NULL,
    unit TEXT DEFAULT 'EA',

    from_warehouse TEXT,                          -- 출발 창고
    from_location TEXT,                           -- 출발 위치
    to_warehouse TEXT,                            -- 도착 창고
    to_location TEXT,                             -- 도착 위치

    ref_type TEXT,                                -- 참조 유형 (PO, SO, PROD 등)
    ref_no TEXT,                                  -- 참조 번호

    reason_cd TEXT,                               -- 사유코드 (조정/폐기시)
    remark TEXT,
    created_by TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),

    FOREIGN KEY (item_cd) REFERENCES item_mst(item_cd)
);

-- ========================================
-- 5. 인덱스
-- ========================================

-- 품질검사
CREATE INDEX IF NOT EXISTS idx_qc_insp_type ON qc_inspection(inspection_type);
CREATE INDEX IF NOT EXISTS idx_qc_insp_lot ON qc_inspection(lot_no);
CREATE INDEX IF NOT EXISTS idx_qc_insp_time ON qc_inspection(inspection_time);
CREATE INDEX IF NOT EXISTS idx_qc_insp_result ON qc_inspection(result);

-- 금속검출
CREATE INDEX IF NOT EXISTS idx_metal_time ON metal_detection_log(detection_time);
CREATE INDEX IF NOT EXISTS idx_metal_detected ON metal_detection_log(metal_detected);
CREATE INDEX IF NOT EXISTS idx_metal_line ON metal_detection_log(line_cd);

-- 공정파라미터
CREATE INDEX IF NOT EXISTS idx_process_param_time ON process_param_log(recorded_at);
CREATE INDEX IF NOT EXISTS idx_process_param_equip ON process_param_log(equip_cd);
CREATE INDEX IF NOT EXISTS idx_process_param_batch ON process_param_log(batch_lot_no);
CREATE INDEX IF NOT EXISTS idx_process_param_cip ON process_param_log(cip_status);

-- 자재투입
CREATE INDEX IF NOT EXISTS idx_material_input_batch ON material_input_log(batch_lot_no);
CREATE INDEX IF NOT EXISTS idx_material_input_lot ON material_input_log(material_lot_no);
CREATE INDEX IF NOT EXISTS idx_material_input_time ON material_input_log(input_time);

-- 창고/재고이동
CREATE INDEX IF NOT EXISTS idx_inv_mov_date ON inventory_movement(movement_date);
CREATE INDEX IF NOT EXISTS idx_inv_mov_type ON inventory_movement(movement_type);
CREATE INDEX IF NOT EXISTS idx_inv_mov_item ON inventory_movement(item_cd);
