-- ============================================================
-- Judgify (퓨어웰 음료) 데이터베이스 스키마
-- Version: 2.0 (prompt_router.py 템플릿 호환)
-- 생성일: 2024-12-09
-- 총 테이블: 22개 (마스터 10개 + 트랜잭션 12개)
-- ============================================================

-- 기존 테이블 삭제 (역순으로 - 외래키 의존성 고려)
DROP TABLE IF EXISTS alarm_event;
DROP TABLE IF EXISTS outbound;
DROP TABLE IF EXISTS inventory;
DROP TABLE IF EXISTS qc_test;
DROP TABLE IF EXISTS ccp_check_log;
DROP TABLE IF EXISTS sensor_log;
DROP TABLE IF EXISTS downtime_event;
DROP TABLE IF EXISTS sales_order_dtl;
DROP TABLE IF EXISTS sales_order;
DROP TABLE IF EXISTS fg_lot;
DROP TABLE IF EXISTS production_order;
DROP TABLE IF EXISTS operation_exec;
DROP TABLE IF EXISTS mes_work_order;
DROP TABLE IF EXISTS defect_mst;
DROP TABLE IF EXISTS ccp_master;
DROP TABLE IF EXISTS operation_mst;
DROP TABLE IF EXISTS param_mst;
DROP TABLE IF EXISTS reason_code_mst;
DROP TABLE IF EXISTS customer_mst;
DROP TABLE IF EXISTS warehouse_mst;
DROP TABLE IF EXISTS item_mst;
DROP TABLE IF EXISTS equipment_mst;
DROP TABLE IF EXISTS line_mst;

-- ============================================================
-- 1. 마스터 테이블 (10개)
-- ============================================================

-- 1.1 라인 마스터 (OEE 템플릿용 max_capacity 포함)
CREATE TABLE line_mst (
    line_id TEXT PRIMARY KEY,        -- L01, L02, L03, PILOT
    line_name TEXT NOT NULL,         -- 1라인, 2라인, 3라인, PILOT라인
    line_type TEXT CHECK (line_type IN ('BATCHING', 'FILLING', 'PACKAGING', 'PILOT')),
    capacity_per_hour REAL,          -- 시간당 생산능력
    max_capacity REAL,               -- OEE 성능률 계산용 (일일 최대 생산량)
    is_active INTEGER DEFAULT 1,
    created_at TEXT DEFAULT (datetime('now'))
);

-- 1.2 설비 마스터
CREATE TABLE equipment_mst (
    equip_id TEXT PRIMARY KEY,       -- EQ001, EQ002, ...
    equip_name TEXT NOT NULL,
    equip_type TEXT,                 -- MIXER, FILLER, CAPPER, PACKER 등
    line_id TEXT REFERENCES line_mst(line_id),
    is_active INTEGER DEFAULT 1,
    created_at TEXT DEFAULT (datetime('now'))
);

-- 1.3 품목 마스터 (재고/매출 템플릿용 확장)
CREATE TABLE item_mst (
    item_id TEXT PRIMARY KEY,        -- ITEM001, ITEM002, ...
    item_name TEXT NOT NULL,
    category TEXT,                   -- 생수, 주스, 차음료 등
    item_type TEXT,                  -- 완제품, 원자재, 반제품
    unit TEXT DEFAULT 'EA',
    unit_cost REAL,                  -- 단가 (재고 금액 계산용)
    safety_stock REAL,               -- 안전재고
    reorder_point REAL,              -- 재주문점
    is_active INTEGER DEFAULT 1,
    created_at TEXT DEFAULT (datetime('now'))
);

-- 1.4 창고 마스터 (재고 템플릿용 확장)
CREATE TABLE warehouse_mst (
    warehouse_id TEXT PRIMARY KEY,   -- WH001, WH002, ...
    warehouse_name TEXT NOT NULL,
    warehouse_type TEXT,             -- 원자재, 완제품, 반제품
    storage_type TEXT,               -- 상온, 냉장, 냉동
    capacity_pallet REAL,            -- 창고 용량 (팔렛 기준)
    location TEXT,
    is_active INTEGER DEFAULT 1,
    created_at TEXT DEFAULT (datetime('now'))
);

-- 1.5 고객 마스터
CREATE TABLE customer_mst (
    customer_id TEXT PRIMARY KEY,    -- CUST001, CUST002, ...
    customer_name TEXT NOT NULL,
    biz_type TEXT,                   -- 도매, 소매, 온라인
    region TEXT,
    is_active INTEGER DEFAULT 1,
    created_at TEXT DEFAULT (datetime('now'))
);

-- 1.6 사유코드 마스터 (OEE 6대 손실 분류용)
CREATE TABLE reason_code_mst (
    reason_code TEXT PRIMARY KEY,    -- RC001, RC002, ...
    reason_type TEXT,                -- 비가동, 불량 등
    reason_category TEXT,            -- OEE 6대 손실: BREAKDOWN, SETUP, MINOR_STOP, SPEED, DEFECT, STARTUP
    description TEXT NOT NULL,
    is_active INTEGER DEFAULT 1,
    created_at TEXT DEFAULT (datetime('now'))
);

-- 1.7 파라미터 마스터 (센서용)
CREATE TABLE param_mst (
    param_id TEXT PRIMARY KEY,       -- TEMP, HUMIDITY, PRESSURE, ...
    param_name TEXT NOT NULL,
    param_type TEXT,                 -- 온도, 습도, 압력 등
    unit TEXT,
    lower_limit REAL,
    upper_limit REAL,
    is_active INTEGER DEFAULT 1,
    created_at TEXT DEFAULT (datetime('now'))
);

-- 1.8 공정 마스터 (CCP 템플릿용 is_ccp 플래그 포함)
CREATE TABLE operation_mst (
    op_id TEXT PRIMARY KEY,          -- OP001, OP002, ...
    op_name TEXT NOT NULL,
    op_type TEXT,                    -- 배합, 충진, 포장 등
    operation_type TEXT,             -- MIXING, FERMENT, PASTEUR, FILL, PACK
    sequence INTEGER,
    is_ccp TEXT DEFAULT 'N',         -- CCP 공정 여부 ('Y'/'N')
    is_active INTEGER DEFAULT 1,
    created_at TEXT DEFAULT (datetime('now'))
);

-- 1.9 CCP 마스터 (HACCP 템플릿용)
CREATE TABLE ccp_master (
    ccp_id TEXT PRIMARY KEY,         -- CCP001, CCP002, ...
    ccp_name TEXT NOT NULL,
    ccp_type TEXT,                   -- 온도, 금속검출 등
    hazard_type TEXT,                -- 생물학적(B), 화학적(C), 물리적(P)
    control_measure TEXT,            -- 관리수단 (살균, 금속검출 등)
    op_id TEXT REFERENCES operation_mst(op_id),
    critical_limit_min REAL,         -- CCP 임계 하한
    critical_limit_max REAL,         -- CCP 임계 상한
    unit TEXT,                       -- 단위 (℃, ppm, mm 등)
    lower_limit REAL,                -- 일반 하한 (경고용)
    upper_limit REAL,                -- 일반 상한 (경고용)
    is_active INTEGER DEFAULT 1,
    created_at TEXT DEFAULT (datetime('now'))
);

-- 1.10 불량 마스터 (불량률 템플릿용)
CREATE TABLE defect_mst (
    defect_code TEXT PRIMARY KEY,    -- DEF001, DEF002, ...
    defect_name TEXT NOT NULL,
    defect_type TEXT,                -- 외관, 내용물, 포장 등
    defect_category TEXT,            -- APPEARANCE, CONTENT, PACKAGING, LABEL, WEIGHT
    severity TEXT,                   -- MINOR, MAJOR, CRITICAL
    is_active INTEGER DEFAULT 1,
    created_at TEXT DEFAULT (datetime('now'))
);

-- ============================================================
-- 2. 트랜잭션 테이블 (12개)
-- ============================================================

-- 2.1 작업지시
CREATE TABLE mes_work_order (
    mes_order_id TEXT PRIMARY KEY,   -- WO20240101001, ...
    line_id TEXT REFERENCES line_mst(line_id),
    item_id TEXT REFERENCES item_mst(item_id),
    lot_no TEXT,
    plan_qty REAL,
    plan_start_dt TEXT,
    plan_end_dt TEXT,
    status TEXT DEFAULT 'PLANNED',   -- PLANNED, IN_PROGRESS, COMPLETED, CANCELLED
    created_at TEXT DEFAULT (datetime('now'))
);

-- 2.2 공정 실행 (불량률/OEE 템플릿용 확장)
CREATE TABLE operation_exec (
    op_exec_id TEXT PRIMARY KEY,     -- OE20240101001, ...
    mes_order_id TEXT REFERENCES mes_work_order(mes_order_id),
    op_id TEXT REFERENCES operation_mst(op_id),
    line_id TEXT REFERENCES line_mst(line_id),  -- 라인 직접 참조 (OEE 계산용)
    operation_type TEXT,             -- MIXING, FERMENT, PASTEUR, FILL, PACK
    start_dt TEXT,
    end_dt TEXT,
    qty_input REAL,                  -- 투입량
    qty_output REAL,                 -- 생산량
    scrap_qty REAL,                  -- 불량량
    result_flag TEXT DEFAULT 'PASS', -- PASS, FAIL, PARTIAL
    created_at TEXT DEFAULT (datetime('now'))
);

-- 2.3 생산오더
CREATE TABLE production_order (
    prod_order_id TEXT PRIMARY KEY,  -- PO20240101001, ...
    line_id TEXT REFERENCES line_mst(line_id),
    item_id TEXT REFERENCES item_mst(item_id),
    plan_qty REAL,
    actual_qty REAL,
    status TEXT DEFAULT 'PLANNED',
    created_at TEXT DEFAULT (datetime('now'))
);

-- 2.4 완제품 LOT
CREATE TABLE fg_lot (
    lot_id TEXT PRIMARY KEY,         -- LOT20240101001, ...
    prod_order_id TEXT REFERENCES production_order(prod_order_id),
    item_id TEXT REFERENCES item_mst(item_id),
    production_dt TEXT,
    qty REAL,
    status TEXT DEFAULT 'AVAILABLE', -- AVAILABLE, HOLD, SHIPPED
    expiry_dt TEXT,
    created_at TEXT DEFAULT (datetime('now'))
);

-- 2.5 판매오더
CREATE TABLE sales_order (
    so_no TEXT PRIMARY KEY,          -- SO20240101001, ...
    customer_id TEXT REFERENCES customer_mst(customer_id),
    so_date TEXT,
    delivery_date TEXT,
    status TEXT DEFAULT 'OPEN',      -- OPEN, CONFIRMED, SHIPPED, CLOSED
    total_amount REAL,
    created_at TEXT DEFAULT (datetime('now'))
);

-- 2.6 판매오더 상세
CREATE TABLE sales_order_dtl (
    so_dtl_id TEXT PRIMARY KEY,      -- SOD20240101001, ...
    so_no TEXT REFERENCES sales_order(so_no),
    item_id TEXT REFERENCES item_mst(item_id),
    order_qty REAL,
    unit_price REAL,
    created_at TEXT DEFAULT (datetime('now'))
);

-- 2.7 비가동 이벤트
CREATE TABLE downtime_event (
    downtime_id TEXT PRIMARY KEY,    -- DT20240101001, ...
    line_id TEXT REFERENCES line_mst(line_id),
    equip_id TEXT REFERENCES equipment_mst(equip_id),
    start_dt TEXT,
    end_dt TEXT,
    duration_min REAL,
    reason_code TEXT REFERENCES reason_code_mst(reason_code),
    description TEXT,
    created_at TEXT DEFAULT (datetime('now'))
);

-- 2.8 센서 로그
CREATE TABLE sensor_log (
    log_id TEXT PRIMARY KEY,         -- SL20240101001, ...
    sensor_id TEXT,
    equip_id TEXT REFERENCES equipment_mst(equip_id),
    param_id TEXT REFERENCES param_mst(param_id),
    log_dt TEXT,
    param_value REAL,
    created_at TEXT DEFAULT (datetime('now'))
);

-- 2.9 CCP 점검 로그 (HACCP 템플릿용)
CREATE TABLE ccp_check_log (
    check_id TEXT PRIMARY KEY,       -- CCP20240101001, ...
    ccp_id TEXT REFERENCES ccp_master(ccp_id),
    op_id TEXT REFERENCES operation_mst(op_id),
    equip_id TEXT REFERENCES equipment_mst(equip_id),
    lot_id TEXT,
    lot_no TEXT,
    check_dt TEXT,
    measured_value REAL,             -- 측정값
    check_value REAL,                -- 레거시 호환
    lower_limit REAL,
    upper_limit REAL,
    result_flag TEXT DEFAULT 'PASS', -- PASS, FAIL
    checker_id TEXT,                 -- 점검자 ID
    corrective_action TEXT,          -- 개선조치 (이상 발생시)
    disposition TEXT,                -- 처리결과 (폐기, 재가공, 합격 등)
    action_taken TEXT,
    created_at TEXT DEFAULT (datetime('now'))
);

-- 2.10 품질검사 (불량률 템플릿용)
CREATE TABLE qc_test (
    test_id TEXT PRIMARY KEY,        -- QC20240101001, ...
    lot_id TEXT,
    op_exec_id TEXT,                 -- 공정 실행 참조 (불량률 계산용)
    test_dt TEXT,
    test_type TEXT,                  -- 입고검사, 공정검사, 출하검사
    test_item TEXT,                  -- 당도, pH, 색상 등
    test_value REAL,
    lower_limit REAL,
    upper_limit REAL,
    sample_qty REAL,                 -- 검사 샘플 수량
    pass_qty REAL,                   -- 합격 수량
    fail_qty REAL,                   -- 불합격 수량
    result_flag TEXT DEFAULT 'PASS', -- PASS, FAIL (개별 검사 결과)
    final_status TEXT DEFAULT 'PASS',
    defect_code TEXT REFERENCES defect_mst(defect_code),
    created_at TEXT DEFAULT (datetime('now'))
);

-- 2.11 재고
CREATE TABLE inventory (
    inv_id TEXT PRIMARY KEY,         -- INV001, INV002, ...
    warehouse_id TEXT REFERENCES warehouse_mst(warehouse_id),
    item_id TEXT REFERENCES item_mst(item_id),
    lot_id TEXT,
    qty_on_hand REAL,
    safety_stock REAL,
    last_update TEXT,
    created_at TEXT DEFAULT (datetime('now'))
);

-- 2.12 출고
CREATE TABLE outbound (
    outbound_id TEXT PRIMARY KEY,    -- OB20240101001, ...
    warehouse_id TEXT REFERENCES warehouse_mst(warehouse_id),
    item_id TEXT REFERENCES item_mst(item_id),
    lot_id TEXT,
    qty REAL,
    outbound_dt TEXT,
    so_no TEXT REFERENCES sales_order(so_no),
    created_at TEXT DEFAULT (datetime('now'))
);

-- 2.13 알람 이벤트
CREATE TABLE alarm_event (
    alarm_id TEXT PRIMARY KEY,       -- AL20240101001, ...
    equip_id TEXT REFERENCES equipment_mst(equip_id),
    line_id TEXT REFERENCES line_mst(line_id),
    alarm_dt TEXT,
    alarm_type TEXT,                 -- WARNING, CRITICAL
    alarm_code TEXT,
    description TEXT,
    resolved_dt TEXT,
    created_at TEXT DEFAULT (datetime('now'))
);

-- ============================================================
-- 3. 마스터 데이터 삽입
-- ============================================================

-- 3.1 라인 마스터 (4개)
INSERT INTO line_mst (line_id, line_name, line_type, capacity_per_hour, max_capacity, is_active) VALUES
('L01', '1라인', 'FILLING', 5000, 40000, 1),
('L02', '2라인', 'FILLING', 5000, 40000, 1),
('L03', '3라인', 'PACKAGING', 8000, 64000, 1),
('PILOT', 'PILOT라인', 'PILOT', 1000, 8000, 1);

-- 3.2 설비 마스터 (12개)
INSERT INTO equipment_mst (equip_id, equip_name, equip_type, line_id, is_active) VALUES
('EQ001', '배합기 A', 'MIXER', 'L01', 1),
('EQ002', '충진기 A', 'FILLER', 'L01', 1),
('EQ003', '캐퍼 A', 'CAPPER', 'L01', 1),
('EQ004', '배합기 B', 'MIXER', 'L02', 1),
('EQ005', '충진기 B', 'FILLER', 'L02', 1),
('EQ006', '캐퍼 B', 'CAPPER', 'L02', 1),
('EQ007', '포장기 A', 'PACKER', 'L03', 1),
('EQ008', '박스 포장기', 'BOX_PACKER', 'L03', 1),
('EQ009', '팔레타이저', 'PALLETIZER', 'L03', 1),
('EQ010', 'PILOT 배합기', 'MIXER', 'PILOT', 1),
('EQ011', 'PILOT 충진기', 'FILLER', 'PILOT', 1),
('EQ012', 'PILOT 포장기', 'PACKER', 'PILOT', 1);

-- 3.3 품목 마스터 (10개)
INSERT INTO item_mst (item_id, item_name, category, item_type, unit, unit_cost, safety_stock, reorder_point, is_active) VALUES
('ITEM001', '퓨어웰 생수 500ml', '생수', '완제품', 'EA', 500, 10000, 5000, 1),
('ITEM002', '퓨어웰 생수 1L', '생수', '완제품', 'EA', 800, 8000, 4000, 1),
('ITEM003', '퓨어웰 생수 2L', '생수', '완제품', 'EA', 1200, 5000, 2500, 1),
('ITEM004', '오렌지 주스 500ml', '주스', '완제품', 'EA', 1500, 6000, 3000, 1),
('ITEM005', '사과 주스 500ml', '주스', '완제품', 'EA', 1500, 6000, 3000, 1),
('ITEM006', '녹차 음료 350ml', '차음료', '완제품', 'EA', 1200, 5000, 2500, 1),
('ITEM007', '보리차 500ml', '차음료', '완제품', 'EA', 1000, 7000, 3500, 1),
('ITEM008', '이온음료 500ml', '스포츠음료', '완제품', 'EA', 1300, 8000, 4000, 1),
('ITEM009', '에너지드링크 250ml', '에너지음료', '완제품', 'EA', 2000, 4000, 2000, 1),
('ITEM010', '탄산수 350ml', '탄산음료', '완제품', 'EA', 900, 6000, 3000, 1);

-- 3.4 창고 마스터 (4개)
INSERT INTO warehouse_mst (warehouse_id, warehouse_name, warehouse_type, storage_type, capacity_pallet, location, is_active) VALUES
('WH001', '원자재 창고', '원자재', '상온', 500, 'A동 1층', 1),
('WH002', '반제품 창고', '반제품', '냉장', 300, 'A동 2층', 1),
('WH003', '완제품 창고', '완제품', '상온', 800, 'B동 1층', 1),
('WH004', '출하대기 창고', '완제품', '상온', 200, 'B동 출하장', 1);

-- 3.5 고객 마스터 (10개)
INSERT INTO customer_mst (customer_id, customer_name, biz_type, region, is_active) VALUES
('CUST001', '대형마트 A', '도매', '서울', 1),
('CUST002', '대형마트 B', '도매', '경기', 1),
('CUST003', '편의점 본사 C', '도매', '서울', 1),
('CUST004', '식자재 유통 D', '도매', '인천', 1),
('CUST005', '프랜차이즈 E', '도매', '부산', 1),
('CUST006', '슈퍼마켓 F', '소매', '대구', 1),
('CUST007', '슈퍼마켓 G', '소매', '광주', 1),
('CUST008', '미니마트 H', '소매', '대전', 1),
('CUST009', '온라인몰 I', '온라인', '전국', 1),
('CUST010', '소셜커머스 J', '온라인', '전국', 1);

-- 3.6 사유코드 마스터 (12개 - OEE 6대 손실)
INSERT INTO reason_code_mst (reason_code, reason_type, reason_category, description, is_active) VALUES
-- 비가동 사유 (OEE 6대 손실)
('RC001', '비가동', 'BREAKDOWN', '설비 고장 (고장 손실)', 1),
('RC002', '비가동', 'SETUP', '품목 교체/셋업 (준비 교체 손실)', 1),
('RC003', '비가동', 'MINOR_STOP', '일시 정지/청소 (순간 정지 손실)', 1),
('RC004', '비가동', 'SPEED', '속도 저하 (속도 저하 손실)', 1),
('RC005', '비가동', 'DEFECT', '불량 재작업 (불량 손실)', 1),
('RC006', '비가동', 'STARTUP', '초기 가동 손실 (수율 손실)', 1),
-- 불량 사유
('RC007', '불량', 'DEFECT', '외관 불량', 1),
('RC008', '불량', 'DEFECT', '내용물 이상', 1),
('RC009', '불량', 'DEFECT', '포장 불량', 1),
('RC010', '불량', 'DEFECT', '라벨 불량', 1),
('RC011', '불량', 'DEFECT', '중량 미달', 1),
('RC012', '불량', 'DEFECT', '이물 혼입', 1);

-- 3.7 파라미터 마스터 (5개)
INSERT INTO param_mst (param_id, param_name, param_type, unit, lower_limit, upper_limit, is_active) VALUES
('TEMP', '온도', '온도', '℃', 4.0, 95.0, 1),
('HUMIDITY', '습도', '습도', '%', 30.0, 80.0, 1),
('PRESSURE', '압력', '압력', 'bar', 1.0, 5.0, 1),
('PH', 'pH', 'pH', 'pH', 3.0, 7.0, 1),
('BRIX', '당도', '당도', '°Bx', 8.0, 15.0, 1);

-- 3.8 공정 마스터 (5개 - CCP 플래그 포함)
INSERT INTO operation_mst (op_id, op_name, op_type, operation_type, sequence, is_ccp, is_active) VALUES
('OP001', '배합 공정', '배합', 'MIXING', 1, 'Y', 1),
('OP002', '살균 공정', '살균', 'PASTEUR', 2, 'Y', 1),
('OP003', '충진 공정', '충진', 'FILL', 3, 'Y', 1),
('OP004', '캡핑 공정', '캡핑', 'FILL', 4, 'N', 1),
('OP005', '포장 공정', '포장', 'PACK', 5, 'N', 1);

-- 3.9 CCP 마스터 (5개 - HACCP)
INSERT INTO ccp_master (ccp_id, ccp_name, ccp_type, hazard_type, control_measure, op_id, critical_limit_min, critical_limit_max, unit, lower_limit, upper_limit, is_active) VALUES
('CCP001', '배합 온도', '온도', 'B', '온도 관리', 'OP001', 60.0, 70.0, '℃', 62.0, 68.0, 1),
('CCP002', '살균 온도', '온도', 'B', '살균 처리', 'OP002', 85.0, 95.0, '℃', 87.0, 93.0, 1),
('CCP003', '충진 온도', '온도', 'B', '온도 관리', 'OP003', 4.0, 10.0, '℃', 5.0, 8.0, 1),
('CCP004', '금속 검출', '금속검출', 'P', '금속 검출기', 'OP003', 0.0, 2.0, 'mm', 0.0, 1.5, 1),
('CCP005', '중량 검사', '중량', 'C', '중량 검사기', 'OP005', 495.0, 505.0, 'g', 497.0, 503.0, 1);

-- 3.10 불량 마스터 (6개)
INSERT INTO defect_mst (defect_code, defect_name, defect_type, defect_category, severity, is_active) VALUES
('DEF001', '외관 불량', '외관', 'APPEARANCE', 'MINOR', 1),
('DEF002', '내용물 불량', '내용물', 'CONTENT', 'MAJOR', 1),
('DEF003', '포장 불량', '포장', 'PACKAGING', 'MINOR', 1),
('DEF004', '라벨 불량', '라벨', 'LABEL', 'MINOR', 1),
('DEF005', '중량 불량', '중량', 'WEIGHT', 'MAJOR', 1),
('DEF006', '이물 혼입', '이물', 'FOREIGN', 'CRITICAL', 1);

-- ============================================================
-- 4. 인덱스 생성 (쿼리 성능 최적화)
-- ============================================================

-- 자주 사용되는 조인 및 필터 컬럼에 인덱스 생성
CREATE INDEX idx_operation_exec_mes_order ON operation_exec(mes_order_id);
CREATE INDEX idx_operation_exec_line ON operation_exec(line_id);
CREATE INDEX idx_operation_exec_end_dt ON operation_exec(end_dt);
CREATE INDEX idx_mes_work_order_line ON mes_work_order(line_id);
CREATE INDEX idx_mes_work_order_status ON mes_work_order(status);
CREATE INDEX idx_downtime_event_line ON downtime_event(line_id);
CREATE INDEX idx_downtime_event_start_dt ON downtime_event(start_dt);
CREATE INDEX idx_sensor_log_equip ON sensor_log(equip_id);
CREATE INDEX idx_sensor_log_dt ON sensor_log(log_dt);
CREATE INDEX idx_ccp_check_log_ccp ON ccp_check_log(ccp_id);
CREATE INDEX idx_ccp_check_log_dt ON ccp_check_log(check_dt);
CREATE INDEX idx_sales_order_customer ON sales_order(customer_id);
CREATE INDEX idx_sales_order_date ON sales_order(so_date);
CREATE INDEX idx_inventory_warehouse ON inventory(warehouse_id);
CREATE INDEX idx_inventory_item ON inventory(item_id);
CREATE INDEX idx_qc_test_dt ON qc_test(test_dt);
CREATE INDEX idx_fg_lot_production_dt ON fg_lot(production_dt);
CREATE INDEX idx_outbound_dt ON outbound(outbound_dt);

-- ============================================================
-- 스키마 생성 완료!
-- 다음 단계: generate_sample_data.py 실행하여 1년치 샘플 데이터 생성
-- ============================================================
