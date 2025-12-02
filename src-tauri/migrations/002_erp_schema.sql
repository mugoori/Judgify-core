-- ========================================
-- 002_erp_schema.sql
-- 퓨어웰 음료㈜ ERP 스키마
-- 18개 테이블: 마스터 + 구매/입고 + 생산 + 품질 + 판매/출고
-- ========================================

PRAGMA foreign_keys = ON;

-- ========================================
-- 1. 마스터 테이블
-- ========================================

-- 품목 마스터
CREATE TABLE IF NOT EXISTS item_mst (
    item_cd TEXT PRIMARY KEY,
    item_nm TEXT NOT NULL,
    item_type TEXT NOT NULL CHECK (item_type IN ('RM', 'PKG', 'WIP', 'FG')),
    unit TEXT NOT NULL DEFAULT 'EA',
    spec TEXT,
    shelf_life_days INTEGER,
    storage_cond TEXT CHECK (storage_cond IN ('RT', 'COLD', 'FROZEN')),
    is_active INTEGER DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 거래처(공급업체) 마스터
CREATE TABLE IF NOT EXISTS vendor_mst (
    vendor_cd TEXT PRIMARY KEY,
    vendor_nm TEXT NOT NULL,
    vendor_type TEXT CHECK (vendor_type IN ('SUPPLIER', 'MANUFACTURER', 'BOTH')),
    contact_nm TEXT,
    phone TEXT,
    email TEXT,
    address TEXT,
    business_no TEXT,
    is_active INTEGER DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 고객 마스터
CREATE TABLE IF NOT EXISTS customer_mst (
    cust_cd TEXT PRIMARY KEY,
    cust_nm TEXT NOT NULL,
    cust_type TEXT CHECK (cust_type IN ('RETAIL', 'WHOLESALE', 'ONLINE', 'EXPORT')),
    contact_nm TEXT,
    phone TEXT,
    email TEXT,
    address TEXT,
    credit_limit REAL DEFAULT 0,
    is_active INTEGER DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- BOM 헤더
CREATE TABLE IF NOT EXISTS bom_mst (
    bom_cd TEXT PRIMARY KEY,
    fg_item_cd TEXT NOT NULL,
    bom_nm TEXT NOT NULL,
    batch_size REAL NOT NULL DEFAULT 1000,
    batch_unit TEXT NOT NULL DEFAULT 'L',
    version INTEGER DEFAULT 1,
    is_active INTEGER DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (fg_item_cd) REFERENCES item_mst(item_cd)
);

-- BOM 상세
CREATE TABLE IF NOT EXISTS bom_dtl (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    bom_cd TEXT NOT NULL,
    seq INTEGER NOT NULL,
    item_cd TEXT NOT NULL,
    qty REAL NOT NULL,
    unit TEXT NOT NULL DEFAULT 'KG',
    loss_rate REAL DEFAULT 0,
    remark TEXT,
    FOREIGN KEY (bom_cd) REFERENCES bom_mst(bom_cd) ON DELETE CASCADE,
    FOREIGN KEY (item_cd) REFERENCES item_mst(item_cd),
    UNIQUE(bom_cd, seq)
);

-- ========================================
-- 2. 구매/입고 테이블
-- ========================================

-- 발주서
CREATE TABLE IF NOT EXISTS purchase_order (
    po_no TEXT PRIMARY KEY,
    vendor_cd TEXT NOT NULL,
    order_date TEXT NOT NULL,
    expected_date TEXT,
    status TEXT NOT NULL DEFAULT 'DRAFT' CHECK (status IN ('DRAFT', 'CONFIRMED', 'PARTIAL', 'COMPLETED', 'CANCELLED')),
    total_amount REAL DEFAULT 0,
    remark TEXT,
    created_by TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (vendor_cd) REFERENCES vendor_mst(vendor_cd)
);

-- 발주 상세
CREATE TABLE IF NOT EXISTS purchase_order_dtl (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    po_no TEXT NOT NULL,
    seq INTEGER NOT NULL,
    item_cd TEXT NOT NULL,
    qty REAL NOT NULL,
    unit_price REAL DEFAULT 0,
    amount REAL DEFAULT 0,
    received_qty REAL DEFAULT 0,
    FOREIGN KEY (po_no) REFERENCES purchase_order(po_no) ON DELETE CASCADE,
    FOREIGN KEY (item_cd) REFERENCES item_mst(item_cd),
    UNIQUE(po_no, seq)
);

-- 입고
CREATE TABLE IF NOT EXISTS inbound (
    inbound_no TEXT PRIMARY KEY,
    po_no TEXT,
    vendor_cd TEXT NOT NULL,
    inbound_date TEXT NOT NULL,
    inbound_type TEXT DEFAULT 'NORMAL' CHECK (inbound_type IN ('NORMAL', 'RETURN', 'FREE')),
    status TEXT DEFAULT 'PENDING' CHECK (status IN ('PENDING', 'INSPECTING', 'PASSED', 'REJECTED', 'PARTIAL')),
    remark TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (po_no) REFERENCES purchase_order(po_no),
    FOREIGN KEY (vendor_cd) REFERENCES vendor_mst(vendor_cd)
);

-- 입고 상세 (LOT 생성)
CREATE TABLE IF NOT EXISTS inbound_dtl (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    inbound_no TEXT NOT NULL,
    seq INTEGER NOT NULL,
    item_cd TEXT NOT NULL,
    lot_no TEXT NOT NULL,
    qty REAL NOT NULL,
    unit_price REAL DEFAULT 0,
    mfg_date TEXT,
    exp_date TEXT,
    inspect_result TEXT CHECK (inspect_result IN ('PASS', 'FAIL', 'PENDING')),
    inspect_date TEXT,
    storage_loc TEXT,
    FOREIGN KEY (inbound_no) REFERENCES inbound(inbound_no) ON DELETE CASCADE,
    FOREIGN KEY (item_cd) REFERENCES item_mst(item_cd),
    UNIQUE(inbound_no, seq)
);

-- ========================================
-- 3. 생산 테이블
-- ========================================

-- 생산지시
CREATE TABLE IF NOT EXISTS production_order (
    prod_order_no TEXT PRIMARY KEY,
    bom_cd TEXT NOT NULL,
    plan_date TEXT NOT NULL,
    plan_qty REAL NOT NULL,
    actual_qty REAL DEFAULT 0,
    status TEXT DEFAULT 'PLANNED' CHECK (status IN ('PLANNED', 'RELEASED', 'IN_PROGRESS', 'COMPLETED', 'CANCELLED')),
    priority INTEGER DEFAULT 5,
    start_time TEXT,
    end_time TEXT,
    line_cd TEXT,
    remark TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (bom_cd) REFERENCES bom_mst(bom_cd)
);

-- 배합 LOT (Batch LOT)
CREATE TABLE IF NOT EXISTS batch_lot (
    batch_lot_no TEXT PRIMARY KEY,
    prod_order_no TEXT NOT NULL,
    bom_cd TEXT NOT NULL,
    batch_date TEXT NOT NULL,
    batch_size REAL NOT NULL,
    status TEXT DEFAULT 'BATCHING' CHECK (status IN ('BATCHING', 'PASTEURIZING', 'COOLING', 'HOLDING', 'FILLING', 'COMPLETED', 'REJECTED')),
    tank_no TEXT,
    start_time TEXT,
    end_time TEXT,
    operator_id TEXT,
    remark TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (prod_order_no) REFERENCES production_order(prod_order_no),
    FOREIGN KEY (bom_cd) REFERENCES bom_mst(bom_cd)
);

-- 원료 투입 (자재 출고)
CREATE TABLE IF NOT EXISTS material_issue (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    batch_lot_no TEXT NOT NULL,
    seq INTEGER NOT NULL,
    item_cd TEXT NOT NULL,
    lot_no TEXT NOT NULL,
    plan_qty REAL NOT NULL,
    actual_qty REAL,
    issue_time TEXT,
    operator_id TEXT,
    FOREIGN KEY (batch_lot_no) REFERENCES batch_lot(batch_lot_no) ON DELETE CASCADE,
    FOREIGN KEY (item_cd) REFERENCES item_mst(item_cd),
    UNIQUE(batch_lot_no, seq)
);

-- 공정 실적 (배합→살균→냉각)
CREATE TABLE IF NOT EXISTS process_result (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    batch_lot_no TEXT NOT NULL,
    process_type TEXT NOT NULL CHECK (process_type IN ('BATCHING', 'PASTEURIZATION', 'COOLING', 'HOLDING', 'TRANSFER')),
    start_time TEXT NOT NULL,
    end_time TEXT,
    target_temp REAL,
    actual_temp REAL,
    target_time_sec INTEGER,
    actual_time_sec INTEGER,
    equipment_cd TEXT,
    result TEXT CHECK (result IN ('OK', 'NG', 'PENDING')),
    remark TEXT,
    FOREIGN KEY (batch_lot_no) REFERENCES batch_lot(batch_lot_no) ON DELETE CASCADE
);

-- 충진 LOT
CREATE TABLE IF NOT EXISTS filling_lot (
    filling_lot_no TEXT PRIMARY KEY,
    batch_lot_no TEXT NOT NULL,
    filling_date TEXT NOT NULL,
    line_cd TEXT NOT NULL,
    pkg_item_cd TEXT NOT NULL,
    plan_qty INTEGER NOT NULL,
    good_qty INTEGER DEFAULT 0,
    reject_qty INTEGER DEFAULT 0,
    start_time TEXT,
    end_time TEXT,
    status TEXT DEFAULT 'FILLING' CHECK (status IN ('FILLING', 'COMPLETED', 'STOPPED')),
    FOREIGN KEY (batch_lot_no) REFERENCES batch_lot(batch_lot_no),
    FOREIGN KEY (pkg_item_cd) REFERENCES item_mst(item_cd)
);

-- 완제품 LOT
CREATE TABLE IF NOT EXISTS fg_lot (
    fg_lot_no TEXT PRIMARY KEY,
    filling_lot_no TEXT NOT NULL,
    fg_item_cd TEXT NOT NULL,
    qty INTEGER NOT NULL,
    mfg_date TEXT NOT NULL,
    exp_date TEXT NOT NULL,
    qc_status TEXT DEFAULT 'PENDING' CHECK (qc_status IN ('PENDING', 'TESTING', 'PASSED', 'FAILED', 'HOLD')),
    release_date TEXT,
    storage_loc TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (filling_lot_no) REFERENCES filling_lot(filling_lot_no),
    FOREIGN KEY (fg_item_cd) REFERENCES item_mst(item_cd)
);

-- ========================================
-- 4. 품질 테이블
-- ========================================

-- 품질검사
CREATE TABLE IF NOT EXISTS qc_test (
    qc_no TEXT PRIMARY KEY,
    test_type TEXT NOT NULL CHECK (test_type IN ('INCOMING', 'IN_PROCESS', 'FINAL', 'HOLD_RELEASE')),
    ref_type TEXT NOT NULL CHECK (ref_type IN ('INBOUND', 'BATCH', 'FILLING', 'FG')),
    ref_no TEXT NOT NULL,
    item_cd TEXT NOT NULL,
    lot_no TEXT NOT NULL,
    test_date TEXT NOT NULL,
    tester_id TEXT,
    result TEXT CHECK (result IN ('PASS', 'FAIL', 'CONDITIONAL')),
    test_items TEXT,              -- JSON: 검사항목 및 결과
    remark TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (item_cd) REFERENCES item_mst(item_cd)
);

-- ========================================
-- 5. 재고 테이블
-- ========================================

-- 재고
CREATE TABLE IF NOT EXISTS inventory (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    item_cd TEXT NOT NULL,
    lot_no TEXT NOT NULL,
    location TEXT NOT NULL DEFAULT 'WH01',
    qty REAL NOT NULL DEFAULT 0,
    reserved_qty REAL DEFAULT 0,
    unit TEXT NOT NULL DEFAULT 'EA',
    exp_date TEXT,
    last_move_date TEXT,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (item_cd) REFERENCES item_mst(item_cd),
    UNIQUE(item_cd, lot_no, location)
);

-- ========================================
-- 6. 판매/출고 테이블
-- ========================================

-- 수주
CREATE TABLE IF NOT EXISTS sales_order (
    so_no TEXT PRIMARY KEY,
    cust_cd TEXT NOT NULL,
    order_date TEXT NOT NULL,
    request_date TEXT,
    status TEXT DEFAULT 'DRAFT' CHECK (status IN ('DRAFT', 'CONFIRMED', 'ALLOCATED', 'SHIPPED', 'COMPLETED', 'CANCELLED')),
    total_amount REAL DEFAULT 0,
    ship_address TEXT,
    remark TEXT,
    created_by TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (cust_cd) REFERENCES customer_mst(cust_cd)
);

-- 수주 상세
CREATE TABLE IF NOT EXISTS sales_order_dtl (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    so_no TEXT NOT NULL,
    seq INTEGER NOT NULL,
    item_cd TEXT NOT NULL,
    qty INTEGER NOT NULL,
    unit_price REAL DEFAULT 0,
    amount REAL DEFAULT 0,
    allocated_qty INTEGER DEFAULT 0,
    shipped_qty INTEGER DEFAULT 0,
    FOREIGN KEY (so_no) REFERENCES sales_order(so_no) ON DELETE CASCADE,
    FOREIGN KEY (item_cd) REFERENCES item_mst(item_cd),
    UNIQUE(so_no, seq)
);

-- 출고
CREATE TABLE IF NOT EXISTS outbound (
    outbound_no TEXT PRIMARY KEY,
    so_no TEXT NOT NULL,
    cust_cd TEXT NOT NULL,
    ship_date TEXT NOT NULL,
    status TEXT DEFAULT 'PICKING' CHECK (status IN ('PICKING', 'PACKING', 'SHIPPED', 'DELIVERED', 'RETURNED')),
    carrier TEXT,
    tracking_no TEXT,
    remark TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (so_no) REFERENCES sales_order(so_no),
    FOREIGN KEY (cust_cd) REFERENCES customer_mst(cust_cd)
);

-- 출고 상세 (LOT 할당)
CREATE TABLE IF NOT EXISTS outbound_dtl (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    outbound_no TEXT NOT NULL,
    seq INTEGER NOT NULL,
    item_cd TEXT NOT NULL,
    fg_lot_no TEXT NOT NULL,
    qty INTEGER NOT NULL,
    pick_time TEXT,
    picker_id TEXT,
    FOREIGN KEY (outbound_no) REFERENCES outbound(outbound_no) ON DELETE CASCADE,
    FOREIGN KEY (item_cd) REFERENCES item_mst(item_cd),
    FOREIGN KEY (fg_lot_no) REFERENCES fg_lot(fg_lot_no),
    UNIQUE(outbound_no, seq)
);

-- ========================================
-- 인덱스
-- ========================================

-- 품목
CREATE INDEX IF NOT EXISTS idx_item_type ON item_mst(item_type);
CREATE INDEX IF NOT EXISTS idx_item_active ON item_mst(is_active);

-- 발주
CREATE INDEX IF NOT EXISTS idx_po_vendor ON purchase_order(vendor_cd);
CREATE INDEX IF NOT EXISTS idx_po_status ON purchase_order(status);
CREATE INDEX IF NOT EXISTS idx_po_date ON purchase_order(order_date);

-- 입고
CREATE INDEX IF NOT EXISTS idx_inbound_date ON inbound(inbound_date);
CREATE INDEX IF NOT EXISTS idx_inbound_status ON inbound(status);

-- 생산
CREATE INDEX IF NOT EXISTS idx_prod_order_date ON production_order(plan_date);
CREATE INDEX IF NOT EXISTS idx_prod_order_status ON production_order(status);
CREATE INDEX IF NOT EXISTS idx_batch_lot_date ON batch_lot(batch_date);
CREATE INDEX IF NOT EXISTS idx_batch_lot_status ON batch_lot(status);

-- 충진/완제품
CREATE INDEX IF NOT EXISTS idx_filling_lot_date ON filling_lot(filling_date);
CREATE INDEX IF NOT EXISTS idx_fg_lot_item ON fg_lot(fg_item_cd);
CREATE INDEX IF NOT EXISTS idx_fg_lot_status ON fg_lot(qc_status);

-- 품질
CREATE INDEX IF NOT EXISTS idx_qc_type ON qc_test(test_type);
CREATE INDEX IF NOT EXISTS idx_qc_result ON qc_test(result);
CREATE INDEX IF NOT EXISTS idx_qc_date ON qc_test(test_date);

-- 재고
CREATE INDEX IF NOT EXISTS idx_inv_item ON inventory(item_cd);
CREATE INDEX IF NOT EXISTS idx_inv_loc ON inventory(location);

-- 수주/출고
CREATE INDEX IF NOT EXISTS idx_so_cust ON sales_order(cust_cd);
CREATE INDEX IF NOT EXISTS idx_so_status ON sales_order(status);
CREATE INDEX IF NOT EXISTS idx_so_date ON sales_order(order_date);
CREATE INDEX IF NOT EXISTS idx_outbound_date ON outbound(ship_date);
CREATE INDEX IF NOT EXISTS idx_outbound_status ON outbound(status);
