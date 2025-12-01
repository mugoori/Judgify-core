-- ========================================
-- 003_mes_schema.sql
-- 퓨어웰 음료㈜ MES 스키마
-- 15개 테이블: 마스터 + 작업실행 + 센서/CCP + 이벤트
-- ========================================

PRAGMA foreign_keys = ON;

-- ========================================
-- 1. MES 마스터 테이블
-- ========================================

-- 라인 마스터
CREATE TABLE IF NOT EXISTS line_mst (
    line_cd TEXT PRIMARY KEY,
    line_nm TEXT NOT NULL,
    line_type TEXT CHECK (line_type IN ('BATCHING', 'FILLING', 'PACKAGING')),
    capacity_per_hour REAL,
    is_active INTEGER DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 설비 마스터
CREATE TABLE IF NOT EXISTS equipment_mst (
    equip_cd TEXT PRIMARY KEY,
    equip_nm TEXT NOT NULL,
    line_cd TEXT,
    equip_type TEXT CHECK (equip_type IN ('MIXER', 'TANK', 'PASTEURIZER', 'COOLER', 'FILLER', 'CAPPER', 'LABELER', 'PACKER', 'DETECTOR', 'CONVEYOR', 'PUMP', 'SENSOR')),
    model TEXT,
    manufacturer TEXT,
    install_date TEXT,
    is_ccp INTEGER DEFAULT 0,
    ccp_type TEXT CHECK (ccp_type IN ('PASTEURIZATION', 'METAL_DETECTION', 'COOLING', NULL)),
    is_active INTEGER DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (line_cd) REFERENCES line_mst(line_cd)
);

-- 공정 마스터
CREATE TABLE IF NOT EXISTS operation_mst (
    oper_cd TEXT PRIMARY KEY,
    oper_nm TEXT NOT NULL,
    oper_seq INTEGER NOT NULL,
    line_cd TEXT,
    std_time_sec INTEGER,
    is_ccp INTEGER DEFAULT 0,
    ccp_params TEXT,              -- JSON: CCP 관리 기준
    is_active INTEGER DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (line_cd) REFERENCES line_mst(line_cd)
);

-- 교대 마스터
CREATE TABLE IF NOT EXISTS shift_mst (
    shift_cd TEXT PRIMARY KEY,
    shift_nm TEXT NOT NULL,
    start_time TEXT NOT NULL,
    end_time TEXT NOT NULL,
    is_active INTEGER DEFAULT 1
);

-- 작업자 마스터
CREATE TABLE IF NOT EXISTS operator_mst (
    operator_id TEXT PRIMARY KEY,
    operator_nm TEXT NOT NULL,
    dept TEXT,
    position TEXT,
    shift_cd TEXT,
    certifications TEXT,          -- JSON: 자격증 목록
    is_active INTEGER DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (shift_cd) REFERENCES shift_mst(shift_cd)
);

-- 사유코드 마스터
CREATE TABLE IF NOT EXISTS reason_code_mst (
    reason_cd TEXT PRIMARY KEY,
    reason_type TEXT NOT NULL CHECK (reason_type IN ('DOWNTIME', 'DEFECT', 'ALARM', 'DEVIATION')),
    reason_nm TEXT NOT NULL,
    category TEXT,
    is_active INTEGER DEFAULT 1
);

-- 파라미터 마스터
CREATE TABLE IF NOT EXISTS param_mst (
    param_cd TEXT PRIMARY KEY,
    param_nm TEXT NOT NULL,
    param_type TEXT CHECK (param_type IN ('TEMPERATURE', 'PRESSURE', 'FLOW', 'SPEED', 'TIME', 'WEIGHT', 'PH', 'BRIX', 'COUNT')),
    unit TEXT NOT NULL,
    equip_cd TEXT,
    min_val REAL,
    max_val REAL,
    target_val REAL,
    is_ccp INTEGER DEFAULT 0,
    ccp_critical_limit_min REAL,
    ccp_critical_limit_max REAL,
    alarm_enabled INTEGER DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (equip_cd) REFERENCES equipment_mst(equip_cd)
);

-- ========================================
-- 2. 작업 실행 테이블
-- ========================================

-- MES 작업지시
CREATE TABLE IF NOT EXISTS mes_work_order (
    wo_no TEXT PRIMARY KEY,
    prod_order_no TEXT NOT NULL,
    line_cd TEXT NOT NULL,
    shift_cd TEXT,
    plan_date TEXT NOT NULL,
    plan_start TEXT,
    plan_end TEXT,
    actual_start TEXT,
    actual_end TEXT,
    status TEXT DEFAULT 'SCHEDULED' CHECK (status IN ('SCHEDULED', 'READY', 'RUNNING', 'PAUSED', 'COMPLETED', 'CANCELLED')),
    plan_qty REAL NOT NULL,
    good_qty REAL DEFAULT 0,
    reject_qty REAL DEFAULT 0,
    operator_id TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (prod_order_no) REFERENCES production_order(prod_order_no),
    FOREIGN KEY (line_cd) REFERENCES line_mst(line_cd),
    FOREIGN KEY (shift_cd) REFERENCES shift_mst(shift_cd),
    FOREIGN KEY (operator_id) REFERENCES operator_mst(operator_id)
);

-- 공정 실행
CREATE TABLE IF NOT EXISTS operation_exec (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    wo_no TEXT NOT NULL,
    oper_cd TEXT NOT NULL,
    batch_lot_no TEXT,
    equip_cd TEXT,
    start_time TEXT NOT NULL,
    end_time TEXT,
    status TEXT DEFAULT 'RUNNING' CHECK (status IN ('RUNNING', 'COMPLETED', 'FAILED', 'PAUSED')),
    result TEXT CHECK (result IN ('OK', 'NG', 'DEVIATION')),
    operator_id TEXT,
    remark TEXT,
    FOREIGN KEY (wo_no) REFERENCES mes_work_order(wo_no) ON DELETE CASCADE,
    FOREIGN KEY (oper_cd) REFERENCES operation_mst(oper_cd),
    FOREIGN KEY (batch_lot_no) REFERENCES batch_lot(batch_lot_no),
    FOREIGN KEY (equip_cd) REFERENCES equipment_mst(equip_cd)
);

-- 공정 파라미터 목표값
CREATE TABLE IF NOT EXISTS operation_param_target (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    wo_no TEXT NOT NULL,
    oper_cd TEXT NOT NULL,
    param_cd TEXT NOT NULL,
    target_val REAL NOT NULL,
    tolerance_min REAL,
    tolerance_max REAL,
    FOREIGN KEY (wo_no) REFERENCES mes_work_order(wo_no) ON DELETE CASCADE,
    FOREIGN KEY (oper_cd) REFERENCES operation_mst(oper_cd),
    FOREIGN KEY (param_cd) REFERENCES param_mst(param_cd),
    UNIQUE(wo_no, oper_cd, param_cd)
);

-- 공정 파라미터 실적 로그
CREATE TABLE IF NOT EXISTS operation_param_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    operation_exec_id INTEGER NOT NULL,
    param_cd TEXT NOT NULL,
    recorded_at TEXT NOT NULL DEFAULT (datetime('now')),
    value REAL NOT NULL,
    is_within_spec INTEGER DEFAULT 1,
    FOREIGN KEY (operation_exec_id) REFERENCES operation_exec(id) ON DELETE CASCADE,
    FOREIGN KEY (param_cd) REFERENCES param_mst(param_cd)
);

-- ========================================
-- 3. 센서/CCP 테이블
-- ========================================

-- 센서 로그 (실시간 수집)
CREATE TABLE IF NOT EXISTS sensor_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    equip_cd TEXT NOT NULL,
    param_cd TEXT NOT NULL,
    batch_lot_no TEXT,
    recorded_at TEXT NOT NULL DEFAULT (datetime('now')),
    value REAL NOT NULL,
    is_alarm INTEGER DEFAULT 0,
    alarm_type TEXT CHECK (alarm_type IN ('LOW', 'HIGH', 'CRITICAL_LOW', 'CRITICAL_HIGH', NULL)),
    FOREIGN KEY (equip_cd) REFERENCES equipment_mst(equip_cd),
    FOREIGN KEY (param_cd) REFERENCES param_mst(param_cd)
);

-- CCP 체크 로그
CREATE TABLE IF NOT EXISTS ccp_check_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    batch_lot_no TEXT NOT NULL,
    ccp_type TEXT NOT NULL CHECK (ccp_type IN ('PASTEURIZATION', 'METAL_DETECTION', 'COOLING')),
    check_time TEXT NOT NULL DEFAULT (datetime('now')),
    equip_cd TEXT NOT NULL,
    operator_id TEXT,

    -- 살균 CCP
    target_temp REAL,
    actual_temp REAL,
    target_time_sec INTEGER,
    actual_time_sec INTEGER,

    -- 금속검출 CCP
    sensitivity_fe REAL,
    sensitivity_sus REAL,
    test_piece_detected INTEGER,
    reject_confirmed INTEGER,

    -- 냉각 CCP
    target_cool_temp REAL,
    actual_cool_temp REAL,
    cool_time_sec INTEGER,

    result TEXT NOT NULL CHECK (result IN ('PASS', 'FAIL', 'DEVIATION')),
    corrective_action TEXT,
    verified_by TEXT,
    verified_at TEXT,
    remark TEXT,

    FOREIGN KEY (batch_lot_no) REFERENCES batch_lot(batch_lot_no),
    FOREIGN KEY (equip_cd) REFERENCES equipment_mst(equip_cd),
    FOREIGN KEY (operator_id) REFERENCES operator_mst(operator_id)
);

-- 체크리스트 결과
CREATE TABLE IF NOT EXISTS checklist_result (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    wo_no TEXT NOT NULL,
    checklist_type TEXT NOT NULL CHECK (checklist_type IN ('PRE_START', 'HOURLY', 'SHIFT_END', 'CIP', 'QUALITY')),
    check_time TEXT NOT NULL DEFAULT (datetime('now')),
    operator_id TEXT NOT NULL,
    items TEXT NOT NULL,          -- JSON: 체크 항목 및 결과
    overall_result TEXT CHECK (overall_result IN ('OK', 'NG', 'NA')),
    remark TEXT,
    FOREIGN KEY (wo_no) REFERENCES mes_work_order(wo_no) ON DELETE CASCADE,
    FOREIGN KEY (operator_id) REFERENCES operator_mst(operator_id)
);

-- ========================================
-- 4. 이벤트 테이블
-- ========================================

-- 비가동 이벤트
CREATE TABLE IF NOT EXISTS downtime_event (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    wo_no TEXT,
    equip_cd TEXT NOT NULL,
    line_cd TEXT NOT NULL,
    start_time TEXT NOT NULL,
    end_time TEXT,
    duration_min INTEGER,
    reason_cd TEXT,
    reason_detail TEXT,
    is_planned INTEGER DEFAULT 0,
    reported_by TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (wo_no) REFERENCES mes_work_order(wo_no),
    FOREIGN KEY (equip_cd) REFERENCES equipment_mst(equip_cd),
    FOREIGN KEY (line_cd) REFERENCES line_mst(line_cd),
    FOREIGN KEY (reason_cd) REFERENCES reason_code_mst(reason_cd)
);

-- 알람 이벤트
CREATE TABLE IF NOT EXISTS alarm_event (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    equip_cd TEXT NOT NULL,
    param_cd TEXT,
    batch_lot_no TEXT,
    alarm_time TEXT NOT NULL DEFAULT (datetime('now')),
    alarm_level TEXT NOT NULL CHECK (alarm_level IN ('INFO', 'WARNING', 'CRITICAL')),
    alarm_type TEXT NOT NULL CHECK (alarm_type IN ('PARAM_HIGH', 'PARAM_LOW', 'CCP_DEVIATION', 'EQUIP_FAULT', 'QUALITY_ISSUE', 'SAFETY')),
    message TEXT NOT NULL,
    value REAL,
    threshold REAL,
    is_acknowledged INTEGER DEFAULT 0,
    acknowledged_by TEXT,
    acknowledged_at TEXT,
    is_resolved INTEGER DEFAULT 0,
    resolved_by TEXT,
    resolved_at TEXT,
    resolution TEXT,
    FOREIGN KEY (equip_cd) REFERENCES equipment_mst(equip_cd),
    FOREIGN KEY (param_cd) REFERENCES param_mst(param_cd)
);

-- ========================================
-- 인덱스
-- ========================================

-- 설비
CREATE INDEX IF NOT EXISTS idx_equip_line ON equipment_mst(line_cd);
CREATE INDEX IF NOT EXISTS idx_equip_type ON equipment_mst(equip_type);
CREATE INDEX IF NOT EXISTS idx_equip_ccp ON equipment_mst(is_ccp);

-- 파라미터
CREATE INDEX IF NOT EXISTS idx_param_equip ON param_mst(equip_cd);
CREATE INDEX IF NOT EXISTS idx_param_ccp ON param_mst(is_ccp);

-- 작업지시
CREATE INDEX IF NOT EXISTS idx_wo_date ON mes_work_order(plan_date);
CREATE INDEX IF NOT EXISTS idx_wo_status ON mes_work_order(status);
CREATE INDEX IF NOT EXISTS idx_wo_line ON mes_work_order(line_cd);

-- 공정실행
CREATE INDEX IF NOT EXISTS idx_oper_exec_wo ON operation_exec(wo_no);
CREATE INDEX IF NOT EXISTS idx_oper_exec_time ON operation_exec(start_time);

-- 센서로그
CREATE INDEX IF NOT EXISTS idx_sensor_equip ON sensor_log(equip_cd);
CREATE INDEX IF NOT EXISTS idx_sensor_time ON sensor_log(recorded_at);
CREATE INDEX IF NOT EXISTS idx_sensor_batch ON sensor_log(batch_lot_no);
CREATE INDEX IF NOT EXISTS idx_sensor_alarm ON sensor_log(is_alarm);

-- CCP 체크
CREATE INDEX IF NOT EXISTS idx_ccp_batch ON ccp_check_log(batch_lot_no);
CREATE INDEX IF NOT EXISTS idx_ccp_type ON ccp_check_log(ccp_type);
CREATE INDEX IF NOT EXISTS idx_ccp_time ON ccp_check_log(check_time);
CREATE INDEX IF NOT EXISTS idx_ccp_result ON ccp_check_log(result);

-- 비가동
CREATE INDEX IF NOT EXISTS idx_downtime_equip ON downtime_event(equip_cd);
CREATE INDEX IF NOT EXISTS idx_downtime_time ON downtime_event(start_time);

-- 알람
CREATE INDEX IF NOT EXISTS idx_alarm_equip ON alarm_event(equip_cd);
CREATE INDEX IF NOT EXISTS idx_alarm_time ON alarm_event(alarm_time);
CREATE INDEX IF NOT EXISTS idx_alarm_level ON alarm_event(alarm_level);
CREATE INDEX IF NOT EXISTS idx_alarm_resolved ON alarm_event(is_resolved);
