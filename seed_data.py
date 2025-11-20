import sqlite3
import random
from datetime import datetime, timedelta
import uuid

# ---------------------------------------------------------
# 설정: 데이터 생성 규모 (2년치)
# ---------------------------------------------------------
DB_NAME = "judgify_large.db"
START_DATE = datetime(2024, 1, 1)
DAYS_TO_GENERATE = 730  # 2년
PROD_PER_DAY_MIN = 2    # 하루 최소 생산 배치 수
PROD_PER_DAY_MAX = 5    # 하루 최대 생산 배치 수

def get_conn():
    conn = sqlite3.connect(DB_NAME)
    conn.execute("PRAGMA foreign_keys = ON;")
    return conn

def generate_id(prefix):
    return f"{prefix}-{str(uuid.uuid4())[:8]}"

# ---------------------------------------------------------
# 1. 스키마 정의 (ERP + MES 통합)
# ---------------------------------------------------------
# (이전 스크립트와 동일한 스키마지만, Sales Order 등 누락된 부분 보강)
SCHEMA_SQL = """
-- MASTER
CREATE TABLE IF NOT EXISTS item_mst (item_id TEXT PRIMARY KEY, item_name TEXT, item_type TEXT, unit TEXT, base_unit_weight REAL);
CREATE TABLE IF NOT EXISTS customer_mst (customer_id TEXT PRIMARY KEY, customer_name TEXT, biz_type TEXT);
CREATE TABLE IF NOT EXISTS vendor_mst (vendor_id TEXT PRIMARY KEY, vendor_name TEXT, vendor_type TEXT);

-- ERP TRANSACTION
CREATE TABLE IF NOT EXISTS purchase_order (po_no TEXT PRIMARY KEY, vendor_id TEXT, po_date TEXT, status TEXT);
CREATE TABLE IF NOT EXISTS purchase_order_dtl (po_no TEXT, seq INT, item_id TEXT, order_qty REAL, PRIMARY KEY(po_no, seq));
CREATE TABLE IF NOT EXISTS inbound (inbound_no TEXT PRIMARY KEY, po_no TEXT, inbound_date TEXT);
CREATE TABLE IF NOT EXISTS inbound_dtl (inbound_no TEXT, seq INT, item_id TEXT, lot_no TEXT, qty REAL, qc_status TEXT, PRIMARY KEY(inbound_no, seq));

CREATE TABLE IF NOT EXISTS sales_order (so_no TEXT PRIMARY KEY, customer_id TEXT, so_date TEXT, due_date TEXT, status TEXT);
CREATE TABLE IF NOT EXISTS sales_order_dtl (so_no TEXT, seq INT, item_id TEXT, order_qty REAL, PRIMARY KEY(so_no, seq));

CREATE TABLE IF NOT EXISTS production_order (prod_order_no TEXT PRIMARY KEY, fg_item_id TEXT, order_qty REAL, planned_start_dt TEXT, status TEXT, line_id TEXT);
CREATE TABLE IF NOT EXISTS batch_lot (batch_lot_no TEXT PRIMARY KEY, prod_order_no TEXT, batch_start TEXT, batch_end TEXT, operator TEXT);
CREATE TABLE IF NOT EXISTS filling_lot (filling_lot_no TEXT PRIMARY KEY, batch_lot_no TEXT, qty REAL, fill_dt TEXT, line_id TEXT);
CREATE TABLE IF NOT EXISTS fg_lot (fg_lot_no TEXT PRIMARY KEY, filling_lot_no TEXT, fg_item_id TEXT, mfg_date TEXT, qc_status TEXT, qty REAL);
CREATE TABLE IF NOT EXISTS qc_test (qc_test_no TEXT PRIMARY KEY, lot_no TEXT, test_type TEXT, ph REAL, brix REAL, final_status TEXT, test_dt TEXT);

-- MES MASTER & EXEC
CREATE TABLE IF NOT EXISTS line_mst (line_id TEXT PRIMARY KEY, line_name TEXT);
CREATE TABLE IF NOT EXISTS operation_mst (operation_id TEXT PRIMARY KEY, operation_name TEXT, is_ccp TEXT);
CREATE TABLE IF NOT EXISTS mes_work_order (mes_order_id TEXT PRIMARY KEY, prod_order_no TEXT, line_id TEXT, status TEXT);
CREATE TABLE IF NOT EXISTS operation_exec (op_exec_id TEXT PRIMARY KEY, mes_order_id TEXT, operation_id TEXT, batch_lot_no TEXT, start_dt TEXT, result_flag TEXT);
CREATE TABLE IF NOT EXISTS operation_param_log (op_exec_id TEXT, param_code TEXT, value_avg REAL, within_spec TEXT, PRIMARY KEY(op_exec_id, param_code));
CREATE TABLE IF NOT EXISTS ccp_check_log (ccp_check_id TEXT PRIMARY KEY, check_dt TEXT, operation_id TEXT, measured_value REAL, result_flag TEXT);
"""

# ---------------------------------------------------------
# 2. 기초 데이터 (대폭 확장)
# ---------------------------------------------------------
def insert_master_data(conn):
    c = conn.cursor()
    
    # 2-1. 제품 라인업 확장 (다양한 맛/용량)
    items = [
        ('FG-PB-100', '퓨어웰 프로바이오틱스 100ml', 'FG', 'EA', 100),
        ('FG-PB-150', '퓨어웰 프로바이오틱스 플러스 150ml', 'FG', 'EA', 150),
        ('FG-PT-250', '식물성단백질 오리지널 250ml', 'FG', 'EA', 250),
        ('FG-PT-CHO', '식물성단백질 초코 250ml', 'FG', 'EA', 250),
        ('FG-DT-500', '데일리 디톡스 콤부차 500ml', 'FG', 'EA', 500),
        ('RM-WATER', '정제수', 'RM', 'L', 1),
        ('RM-SUGAR', '액상과당', 'RM', 'kg', 1),
        ('RM-BASE-A', '유산균배양액-A', 'RM', 'kg', 1),
        ('RM-PROT', '완두단백분말', 'RM', 'kg', 1),
        ('RM-COCOA', '코코아파우더', 'RM', 'kg', 1),
    ]
    c.executemany("INSERT OR IGNORE INTO item_mst VALUES (?,?,?,?,?)", items)

    # 2-2. 고객사 대폭 확장 (30개 이상)
    customers = [
        # 유통/마트
        ('CUST-001', '쿠팡 (Rocket)', 'ONLINE'), ('CUST-002', '마켓컬리', 'ONLINE'),
        ('CUST-003', '이마트', 'OFFLINE'), ('CUST-004', '홈플러스', 'OFFLINE'),
        ('CUST-005', '롯데마트', 'OFFLINE'), ('CUST-006', '코스트코 코리아', 'OFFLINE'),
        # 편의점
        ('CUST-007', 'CU 리테일', 'CVS'), ('CUST-008', 'GS25', 'CVS'),
        ('CUST-009', '세븐일레븐', 'CVS'), ('CUST-010', '이마트24', 'CVS'),
        # 카페/프랜차이즈 OEM
        ('CUST-011', '스타벅스 RTD', 'OEM'), ('CUST-012', '이디야 커피', 'OEM'),
        ('CUST-013', '빽다방', 'OEM'), ('CUST-014', '메가커피', 'OEM'),
        ('CUST-015', '투썸플레이스', 'OEM'),
        # 건강기능식품 브랜드
        ('CUST-016', '종근당건강', 'PARTNER'), ('CUST-017', '뉴트리원', 'PARTNER'),
        ('CUST-018', 'GC녹십자', 'PARTNER'), ('CUST-019', '대웅제약', 'PARTNER'),
        ('CUST-020', '프롬바이오', 'PARTNER'),
        # 해외 수출
        ('CUST-EX-01', 'Walmart Asia', 'EXPORT'), ('CUST-EX-02', 'Aeon Mall Japan', 'EXPORT'),
        ('CUST-EX-03', 'Shopee Thai', 'EXPORT'), ('CUST-EX-04', 'Amazon Global', 'EXPORT')
    ]
    c.executemany("INSERT OR IGNORE INTO customer_mst VALUES (?,?,?)", customers)

    # 2-3. MES 기초 데이터
    c.execute("INSERT OR IGNORE INTO line_mst VALUES ('L1', 'L1-유산균')")
    c.execute("INSERT OR IGNORE INTO line_mst VALUES ('L2', 'L2-단백질')")
    
    ops = [('OP-BATCH', '배합', 'N'), ('OP-PAST', '살균', 'Y'), ('OP-FILL', '충진', 'N')]
    c.executemany("INSERT OR IGNORE INTO operation_mst VALUES (?,?,?)", ops)

    conn.commit()

# ---------------------------------------------------------
# 3. 대규모 트랜잭션 생성 (2년치)
# ---------------------------------------------------------
def generate_large_scale_data(conn):
    c = conn.cursor()
    
    # 날짜 루프
    current_date = START_DATE
    end_date = START_DATE + timedelta(days=DAYS_TO_GENERATE)
    
    print(f"Generating data from {START_DATE.date()} to {end_date.date()}...")
    
    po_seq = 1
    so_seq = 1
    prod_seq = 1
    
    # 고객 목록 로딩
    c.execute("SELECT customer_id FROM customer_mst")
    cust_ids = [row[0] for row in c.fetchall()]
    
    # 완제품 목록 로딩
    c.execute("SELECT item_id FROM item_mst WHERE item_type='FG'")
    fg_items = [row[0] for row in c.fetchall()]

    while current_date <= end_date:
        date_str = current_date.strftime("%Y-%m-%d")
        
        if current_date.day == 1:
            print(f"  -> Processing {date_str} ...")

        # -----------------------------------------------------
        # A. Sales Order (수주) - 매일 3~8건 랜덤 발생
        # -----------------------------------------------------
        daily_orders = random.randint(3, 8)
        for _ in range(daily_orders):
            cust = random.choice(cust_ids)
            fg = random.choice(fg_items)
            qty = random.choice([1000, 3000, 5000, 10000, 20000]) # 대량 주문 포함
            
            so_no = f"SO-{date_str.replace('-','')}-{so_seq:04d}"
            due_date = (current_date + timedelta(days=random.randint(7, 14))).strftime("%Y-%m-%d")
            
            c.execute("INSERT INTO sales_order VALUES (?,?,?,?, 'CLOSED')", (so_no, cust, date_str, due_date))
            c.execute("INSERT INTO sales_order_dtl VALUES (?, 1, ?, ?)", (so_no, fg, qty))
            so_seq += 1

        # -----------------------------------------------------
        # B. Purchase Order (발주) - 재고 보충 (매주 월/목)
        # -----------------------------------------------------
        if current_date.weekday() in [0, 3]: 
            po_no = f"PO-{date_str.replace('-','')}-{po_seq:04d}"
            c.execute("INSERT INTO purchase_order VALUES (?, 'VD-001', ?, 'CLOSED')", (po_no, date_str))
            c.execute("INSERT INTO purchase_order_dtl VALUES (?, 1, 'RM-BASE-A', 5000)", (po_no,))
            
            # 입고 (2일 뒤)
            in_date = (current_date + timedelta(days=2)).strftime("%Y-%m-%d")
            in_no = f"IN-{in_date.replace('-','')}-{po_seq:04d}"
            c.execute("INSERT INTO inbound VALUES (?, ?, ?)", (in_no, po_no, in_date))
            c.execute("INSERT INTO inbound_dtl VALUES (?, 1, 'RM-BASE-A', ?, 5000, 'PASS')", 
                      (in_no, f"LOT-RM-{in_date.replace('-','')}-{po_seq:03d}"))
            po_seq += 1

        # -----------------------------------------------------
        # C. Production & MES (생산) - 매일 2~5건 배치
        # -----------------------------------------------------
        daily_prods = random.randint(PROD_PER_DAY_MIN, PROD_PER_DAY_MAX)
        for _ in range(daily_prods):
            # 생산할 제품과 라인 선정
            target_fg = random.choice(fg_items)
            target_line = 'L1' if 'PB' in target_fg else 'L2'
            prod_qty = random.choice([2000, 3000, 5000])
            
            prod_no = f"WO-{date_str.replace('-','')}-{prod_seq:04d}"
            
            # 1. ERP 생산지시
            c.execute("INSERT INTO production_order VALUES (?,?,?,?,'DONE',?)", 
                      (prod_no, target_fg, prod_qty, f"{date_str} 08:00:00", target_line))
            
            # 2. 배합 (Batch)
            batch_lot = f"B-{date_str.replace('-','')}-{prod_seq:03d}"
            c.execute("INSERT INTO batch_lot VALUES (?,?,?,?, '김작업')",
                      (batch_lot, prod_no, f"{date_str} 09:00:00", f"{date_str} 10:30:00"))
            
            # 3. MES 실행 (살균 - CCP)
            mes_id = f"MO-{prod_no}"
            c.execute("INSERT INTO mes_work_order VALUES (?,?,?, 'DONE')", (mes_id, prod_no, target_line))
            
            op_exec_id = f"OPEX-{date_str}-{prod_seq:03d}-PAST"
            
            # 불량 시나리오 (2년치 데이터에서 약 3% 확률로 온도 이탈 발생)
            is_fail = random.random() < 0.03
            temp_val = random.uniform(84.5, 85.5) if not is_fail else random.uniform(81.0, 83.0) # 85도 기준
            res_flag = 'FAIL' if is_fail else 'PASS'

            c.execute("INSERT INTO operation_exec VALUES (?,?,?,?,?,?)", 
                      (op_exec_id, mes_id, 'OP-PAST', batch_lot, f"{date_str} 09:30:00", res_flag))
            
            # 파라미터 로그
            c.execute("INSERT INTO operation_param_log VALUES (?,?,?,?)", 
                      (op_exec_id, 'TEMP', temp_val, 'N' if is_fail else 'Y'))
            
            # CCP 점검 로그
            c.execute("INSERT INTO ccp_check_log VALUES (?,?,?,?,?)", 
                      (f"CCP-{date_str}-{prod_seq:03d}", f"{date_str} 09:45:00", 'OP-PAST', temp_val, res_flag))

            # 4. 충진 및 완제품
            fill_lot = f"F-{date_str.replace('-','')}-{prod_seq:03d}"
            c.execute("INSERT INTO filling_lot VALUES (?,?,?,?,?)", 
                      (fill_lot, batch_lot, prod_qty, f"{date_str} 11:00:00", target_line))
            
            fg_lot = f"FG-{date_str.replace('-','')}-{prod_seq:03d}"
            c.execute("INSERT INTO fg_lot VALUES (?,?,?,?, 'PASS', ?)", 
                      (fg_lot, fill_lot, target_fg, date_str, prod_qty))
            
            # 5. QC 테스트
            # pH (목표 3.8), Brix (목표 12.0) 노이즈 추가
            ph = random.normalvariate(3.8, 0.1)
            brix = random.normalvariate(12.0, 0.5)
            c.execute("INSERT INTO qc_test VALUES (?,?,'FG',?,?, 'PASS', ?)", 
                      (f"QC-{fg_lot}", fg_lot, ph, brix, f"{date_str} 14:00:00"))

            prod_seq += 1
            
        current_date += timedelta(days=1)
        
    conn.commit()

if __name__ == "__main__":
    try:
        conn = get_conn()
        conn.executescript(SCHEMA_SQL)
        
        print("Step 1. Inserting Master Data (30+ Customers)...")
        insert_master_data(conn)
        
        print(f"Step 2. Generating {DAYS_TO_GENERATE} days of transactions...")
        generate_large_scale_data(conn)
        
        print(f"\nDone! 'judgify_large.db' has been created successfully.")
        
        # 결과 확인용 카운트 출력
        print("\n--- Data Statistics ---")
        tables = ['sales_order', 'production_order', 'fg_lot', 'ccp_check_log']
        for t in tables:
            cnt = conn.execute(f"SELECT count(*) FROM {t}").fetchone()[0]
            print(f"Table [{t}]: {cnt:,} rows")
            
        conn.close()
        
    except Exception as e:
        print(f"Error: {e}")