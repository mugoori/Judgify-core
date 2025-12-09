#!/usr/bin/env python3
"""
샘플 데이터 생성 스크립트 (Ver2.0 Final)
==========================================
prompt_router.py의 12개 템플릿에 맞는 1년치 샘플 데이터 생성

실행 방법:
    python generate_sample_data.py [db_path]

    db_path: SQLite DB 경로 (기본값: %APPDATA%/Judgify/judgify.db)

생성 범위:
    - 기간: 2024-01-01 ~ 2024-12-31 (1년)
    - 일별 작업지시: 라인당 3~5건 → 하루 12~20건
    - 연간 총 데이터: 약 5,000~7,000건
"""

import sqlite3
import random
import os
import sys
from datetime import datetime, timedelta
from typing import List, Tuple, Optional
import math

# ============================================================
# 설정 상수
# ============================================================

# 데이터 생성 기간
START_DATE = datetime(2024, 1, 1)
END_DATE = datetime(2024, 12, 31)

# 라인 정보
LINES = ['L01', 'L02', 'L03', 'PILOT']
LINE_CAPACITIES = {
    'L01': 5000,   # 시간당 생산능력
    'L02': 5000,
    'L03': 8000,
    'PILOT': 1000
}

# 품목 정보
ITEMS = [f'ITEM{str(i).zfill(3)}' for i in range(1, 11)]

# 창고 정보
WAREHOUSES = ['WH001', 'WH002', 'WH003', 'WH004']

# 고객 정보
CUSTOMERS = [f'CUST{str(i).zfill(3)}' for i in range(1, 11)]

# 설비 정보
EQUIPMENTS = [f'EQ{str(i).zfill(3)}' for i in range(1, 13)]

# 공정 정보
OPERATIONS = ['OP001', 'OP002', 'OP003', 'OP004', 'OP005']

# CCP 정보
CCPS = ['CCP001', 'CCP002', 'CCP003', 'CCP004', 'CCP005']

# 사유코드 정보
REASON_CODES = [f'RC{str(i).zfill(3)}' for i in range(1, 13)]
DOWNTIME_REASONS = ['RC001', 'RC002', 'RC003', 'RC004', 'RC005', 'RC006']  # 비가동 사유
DEFECT_REASONS = ['RC007', 'RC008', 'RC009', 'RC010', 'RC011', 'RC012']    # 불량 사유

# 불량코드 정보
DEFECT_CODES = [f'DEF{str(i).zfill(3)}' for i in range(1, 7)]

# 파라미터 정보
PARAMS = ['TEMP', 'HUMIDITY', 'PRESSURE', 'PH', 'BRIX']

# ============================================================
# 유틸리티 함수
# ============================================================

def generate_id(prefix: str, date: datetime, seq: int) -> str:
    """ID 생성 (예: WO20240101001)"""
    return f"{prefix}{date.strftime('%Y%m%d')}{str(seq).zfill(3)}"

def random_datetime(date: datetime, start_hour: int = 6, end_hour: int = 22) -> str:
    """지정된 날짜 내에서 랜덤 시간 생성"""
    hour = random.randint(start_hour, end_hour)
    minute = random.randint(0, 59)
    second = random.randint(0, 59)
    return datetime(date.year, date.month, date.day, hour, minute, second).isoformat()

def get_season_factor(date: datetime) -> float:
    """계절성 반영 팩터 (여름 음료 수요 증가)"""
    month = date.month
    if month in [6, 7, 8]:  # 여름
        return 1.3
    elif month in [12, 1, 2]:  # 겨울
        return 0.8
    else:  # 봄, 가을
        return 1.0

def get_weekday_factor(date: datetime) -> float:
    """주중/주말 팩터"""
    if date.weekday() >= 5:  # 주말
        return 0.3
    return 1.0

# ============================================================
# 데이터 생성 함수
# ============================================================

def generate_work_orders(conn: sqlite3.Connection, date: datetime) -> List[str]:
    """작업지시 생성"""
    cursor = conn.cursor()
    orders = []
    global_seq = 1  # 전역 시퀀스

    season_factor = get_season_factor(date)
    weekday_factor = get_weekday_factor(date)

    for line in LINES:
        if line == 'PILOT':
            # PILOT 라인은 주 2~3회만
            if random.random() > 0.4:
                continue
            num_orders = 1
        else:
            # 일반 라인: 3~5건
            base_orders = random.randint(3, 5)
            num_orders = max(1, int(base_orders * season_factor * weekday_factor))

        for seq in range(1, num_orders + 1):
            order_id = generate_id('WO', date, global_seq)
            global_seq += 1
            item = random.choice(ITEMS)
            capacity = LINE_CAPACITIES[line]

            # 계획 수량 (1~4시간 분량)
            hours = random.randint(1, 4)
            plan_qty = capacity * hours

            # 계획 시간
            start_hour = random.randint(6, 18)
            plan_start = datetime(date.year, date.month, date.day, start_hour, 0, 0)
            plan_end = plan_start + timedelta(hours=hours)

            # 상태 결정 (대부분 완료)
            if date < datetime.now() - timedelta(days=30):
                status = 'COMPLETED'
            elif date < datetime.now():
                status = random.choices(['COMPLETED', 'IN_PROGRESS', 'PLANNED'], [0.8, 0.15, 0.05])[0]
            else:
                status = 'PLANNED'

            lot_no = f"LOT{date.strftime('%Y%m%d')}{line}{str(seq).zfill(2)}"

            cursor.execute('''
                INSERT INTO mes_work_order
                (mes_order_id, line_id, item_id, lot_no, plan_qty, plan_start_dt, plan_end_dt, status)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ''', (order_id, line, item, lot_no, plan_qty, plan_start.isoformat(), plan_end.isoformat(), status))

            orders.append((order_id, line, item, plan_qty, status, lot_no))

    conn.commit()
    return orders

def generate_operation_exec(conn: sqlite3.Connection, orders: List, date: datetime):
    """공정 실행 데이터 생성"""
    cursor = conn.cursor()

    for order_id, line, item, plan_qty, status, lot_no in orders:
        if status != 'COMPLETED':
            continue

        for op_idx, op_id in enumerate(OPERATIONS):
            exec_id = f"OE{order_id[2:]}{str(op_idx + 1).zfill(2)}"

            # 실행 시간 계산
            start_dt = random_datetime(date, 6 + op_idx * 2, 8 + op_idx * 2)
            end_dt = random_datetime(date, 8 + op_idx * 2, 10 + op_idx * 2)

            # 투입/산출량 계산 (각 공정 단계별 감소)
            if op_idx == 0:
                qty_input = plan_qty
            else:
                qty_input = int(plan_qty * (0.99 ** op_idx))

            # 불량률: 0.5% ~ 3%
            defect_rate = random.uniform(0.005, 0.03)
            scrap_qty = int(qty_input * defect_rate)
            qty_output = qty_input - scrap_qty

            # 공정 유형
            op_types = ['MIXING', 'PASTEUR', 'FILL', 'FILL', 'PACK']
            op_type = op_types[op_idx]

            # 결과 플래그
            result_flag = 'PASS' if defect_rate < 0.02 else 'FAIL'

            cursor.execute('''
                INSERT INTO operation_exec
                (op_exec_id, mes_order_id, op_id, line_id, operation_type,
                 start_dt, end_dt, qty_input, qty_output, scrap_qty, result_flag)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ''', (exec_id, order_id, op_id, line, op_type,
                  start_dt, end_dt, qty_input, qty_output, scrap_qty, result_flag))

    conn.commit()

def generate_production_orders(conn: sqlite3.Connection, date: datetime) -> List[str]:
    """생산오더 생성"""
    cursor = conn.cursor()
    orders = []
    global_seq = 1

    season_factor = get_season_factor(date)

    for line in LINES[:3]:  # PILOT 제외
        num_orders = random.randint(2, 4)

        for seq in range(1, num_orders + 1):
            order_id = generate_id('PO', date, global_seq)
            global_seq += 1
            item = random.choice(ITEMS)
            capacity = LINE_CAPACITIES[line]

            plan_qty = int(capacity * random.randint(2, 6) * season_factor)
            # 달성률: 95% ~ 102%
            achievement = random.uniform(0.95, 1.02)
            actual_qty = int(plan_qty * achievement)

            status = 'COMPLETED' if date < datetime.now() - timedelta(days=7) else 'PLANNED'

            cursor.execute('''
                INSERT INTO production_order
                (prod_order_id, line_id, item_id, plan_qty, actual_qty, status)
                VALUES (?, ?, ?, ?, ?, ?)
            ''', (order_id, line, item, plan_qty, actual_qty, status))

            orders.append((order_id, item, actual_qty))

    conn.commit()
    return orders

def generate_fg_lots(conn: sqlite3.Connection, prod_orders: List, date: datetime):
    """완제품 LOT 생성"""
    cursor = conn.cursor()

    for idx, (order_id, item, qty) in enumerate(prod_orders):
        lot_id = generate_id('LOT', date, idx + 1)

        # 유통기한: 생산일 + 6개월 ~ 1년
        expiry_days = random.randint(180, 365)
        expiry_dt = date + timedelta(days=expiry_days)

        # 상태
        if date < datetime.now() - timedelta(days=90):
            status = random.choices(['SHIPPED', 'AVAILABLE'], [0.7, 0.3])[0]
        else:
            status = 'AVAILABLE'

        cursor.execute('''
            INSERT INTO fg_lot
            (lot_id, prod_order_id, item_id, production_dt, qty, status, expiry_dt)
            VALUES (?, ?, ?, ?, ?, ?, ?)
        ''', (lot_id, order_id, item, date.isoformat(), qty, status, expiry_dt.isoformat()))

    conn.commit()

def generate_sales_orders(conn: sqlite3.Connection, date: datetime):
    """판매오더 생성 (월별로 그룹화)"""
    cursor = conn.cursor()

    # 일별 2~5건의 판매오더
    num_orders = random.randint(2, 5)
    season_factor = get_season_factor(date)

    for seq in range(1, num_orders + 1):
        so_no = generate_id('SO', date, seq)
        customer = random.choice(CUSTOMERS)

        # 배송일: 주문일 + 2~7일
        delivery_days = random.randint(2, 7)
        delivery_date = date + timedelta(days=delivery_days)

        # 상태
        if date < datetime.now() - timedelta(days=30):
            status = random.choices(['CLOSED', 'SHIPPED'], [0.6, 0.4])[0]
        elif date < datetime.now():
            status = random.choices(['SHIPPED', 'CONFIRMED', 'OPEN'], [0.5, 0.3, 0.2])[0]
        else:
            status = 'OPEN'

        # 총액 (상세에서 계산)
        total_amount = 0

        cursor.execute('''
            INSERT INTO sales_order
            (so_no, customer_id, so_date, delivery_date, status, total_amount)
            VALUES (?, ?, ?, ?, ?, ?)
        ''', (so_no, customer, date.isoformat(), delivery_date.isoformat(), status, total_amount))

        # 상세 생성 (1~4개 품목)
        num_details = random.randint(1, 4)
        order_total = 0

        for dtl_seq in range(1, num_details + 1):
            dtl_id = f"SOD{so_no[2:]}{str(dtl_seq).zfill(2)}"
            item = random.choice(ITEMS)

            # 주문 수량 (100~5000)
            order_qty = random.randint(100, 5000) * season_factor
            # 단가 (500~2000원)
            unit_price = random.randint(500, 2000)

            order_total += order_qty * unit_price

            cursor.execute('''
                INSERT INTO sales_order_dtl
                (so_dtl_id, so_no, item_id, order_qty, unit_price)
                VALUES (?, ?, ?, ?, ?)
            ''', (dtl_id, so_no, item, int(order_qty), unit_price))

        # 총액 업데이트
        cursor.execute('UPDATE sales_order SET total_amount = ? WHERE so_no = ?',
                      (order_total, so_no))

    conn.commit()

_downtime_seq = 0  # 전역 비가동 시퀀스

def generate_downtime_events(conn: sqlite3.Connection, date: datetime):
    """비가동 이벤트 생성"""
    global _downtime_seq
    cursor = conn.cursor()

    # 주당 평균 2~4시간 비가동 → 일별 확률로 변환
    if random.random() > 0.3:  # 30% 확률로 비가동 발생
        return

    for line in LINES[:3]:  # PILOT 제외
        if random.random() > 0.5:  # 라인별 50% 확률
            continue

        _downtime_seq += 1
        event_id = generate_id('DT', date, _downtime_seq)
        equip = random.choice([e for e in EQUIPMENTS if 'L' + e[2:4] == line or True])

        # 비가동 시간: 15분 ~ 120분
        duration = random.randint(15, 120)

        start_hour = random.randint(8, 18)
        start_dt = datetime(date.year, date.month, date.day, start_hour,
                           random.randint(0, 59), 0)
        end_dt = start_dt + timedelta(minutes=duration)

        reason = random.choice(DOWNTIME_REASONS)

        cursor.execute('''
            INSERT INTO downtime_event
            (downtime_id, line_id, equip_id, start_dt, end_dt, duration_min, reason_code, description)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        ''', (event_id, line, equip, start_dt.isoformat(), end_dt.isoformat(),
              duration, reason, f"비가동 이벤트 - {reason}"))

    conn.commit()

def generate_sensor_logs(conn: sqlite3.Connection, date: datetime):
    """센서 로그 생성 (시간별)"""
    cursor = conn.cursor()

    # 매 시간마다 각 설비에서 센서 데이터 생성
    for hour in range(6, 22):  # 6시 ~ 22시
        for equip in EQUIPMENTS[:6]:  # 주요 설비만
            for param in PARAMS:
                log_id = f"SL{date.strftime('%Y%m%d')}{str(hour).zfill(2)}{equip}{param}"
                sensor_id = f"SENSOR_{equip}_{param}"

                log_dt = datetime(date.year, date.month, date.day, hour,
                                 random.randint(0, 59), random.randint(0, 59))

                # 파라미터별 정상 범위 내 값 생성
                if param == 'TEMP':
                    value = random.uniform(60, 90)  # 온도
                elif param == 'HUMIDITY':
                    value = random.uniform(40, 70)  # 습도
                elif param == 'PRESSURE':
                    value = random.uniform(2, 4)    # 압력
                elif param == 'PH':
                    value = random.uniform(4, 6)    # pH
                else:  # BRIX
                    value = random.uniform(10, 14)  # 당도

                cursor.execute('''
                    INSERT INTO sensor_log
                    (log_id, sensor_id, equip_id, param_id, log_dt, param_value)
                    VALUES (?, ?, ?, ?, ?, ?)
                ''', (log_id, sensor_id, equip, param, log_dt.isoformat(), round(value, 2)))

    conn.commit()

def generate_ccp_check_logs(conn: sqlite3.Connection, orders: List, date: datetime):
    """CCP 점검 로그 생성"""
    cursor = conn.cursor()

    for order_id, line, item, plan_qty, status, lot_no in orders:
        if status != 'COMPLETED':
            continue

        for ccp_idx, ccp_id in enumerate(CCPS):
            check_id = f"CCP{order_id[2:]}{str(ccp_idx + 1).zfill(2)}"

            # 관련 공정
            op_id = OPERATIONS[min(ccp_idx, len(OPERATIONS) - 1)]
            equip = EQUIPMENTS[ccp_idx % len(EQUIPMENTS)]

            check_dt = random_datetime(date, 7 + ccp_idx, 9 + ccp_idx)

            # CCP별 측정값 생성
            if ccp_id == 'CCP001':  # 배합 온도
                measured = random.uniform(62, 68)
                lower, upper = 60.0, 70.0
            elif ccp_id == 'CCP002':  # 살균 온도
                measured = random.uniform(87, 93)
                lower, upper = 85.0, 95.0
            elif ccp_id == 'CCP003':  # 충진 온도
                measured = random.uniform(5, 8)
                lower, upper = 4.0, 10.0
            elif ccp_id == 'CCP004':  # 금속 검출
                measured = random.uniform(0, 1.5)
                lower, upper = 0.0, 2.0
            else:  # 중량 검사
                measured = random.uniform(497, 503)
                lower, upper = 495.0, 505.0

            # 결과 판정 (2% 확률로 이탈)
            if random.random() < 0.02:
                # 이탈 케이스
                if random.random() < 0.5:
                    measured = lower - random.uniform(1, 5)
                else:
                    measured = upper + random.uniform(1, 5)
                result_flag = 'FAIL'
                corrective_action = '즉시 조정 및 재검사 실시'
                disposition = random.choice(['재가공', '폐기', '합격(재검사)'])
            else:
                result_flag = 'PASS'
                corrective_action = None
                disposition = '합격'

            checker_id = f"CHK{random.randint(1, 5):03d}"

            cursor.execute('''
                INSERT INTO ccp_check_log
                (check_id, ccp_id, op_id, equip_id, lot_id, lot_no, check_dt,
                 measured_value, check_value, lower_limit, upper_limit,
                 result_flag, checker_id, corrective_action, disposition, action_taken)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ''', (check_id, ccp_id, op_id, equip, None, lot_no, check_dt,
                  round(measured, 2), round(measured, 2), lower, upper,
                  result_flag, checker_id, corrective_action, disposition, corrective_action))

    conn.commit()

def generate_qc_tests(conn: sqlite3.Connection, orders: List, date: datetime):
    """품질검사 데이터 생성"""
    cursor = conn.cursor()

    test_items = ['당도', 'pH', '색상', '탁도', '이물']

    for order_id, line, item, plan_qty, status, lot_no in orders:
        if status != 'COMPLETED':
            continue

        for test_idx, test_item in enumerate(test_items):
            test_id = f"QC{order_id[2:]}{str(test_idx + 1).zfill(2)}"

            test_dt = random_datetime(date, 10 + test_idx, 12 + test_idx)

            # 검사 유형
            test_type = random.choice(['공정검사', '출하검사'])

            # 검사값 생성
            if test_item == '당도':
                test_value = random.uniform(10, 14)
                lower, upper = 9.0, 15.0
            elif test_item == 'pH':
                test_value = random.uniform(4, 6)
                lower, upper = 3.0, 7.0
            elif test_item == '색상':
                test_value = random.uniform(80, 100)  # 색상 지수
                lower, upper = 70.0, 110.0
            elif test_item == '탁도':
                test_value = random.uniform(0.5, 2.0)
                lower, upper = 0.0, 3.0
            else:  # 이물
                test_value = random.uniform(0, 0.5)
                lower, upper = 0.0, 1.0

            # 샘플 수량 및 합격/불합격
            sample_qty = random.randint(10, 50)

            # 불합격률: 0.5% ~ 3%
            fail_rate = random.uniform(0.005, 0.03)
            fail_qty = int(sample_qty * fail_rate)
            pass_qty = sample_qty - fail_qty

            # 개별 검사 결과
            result_flag = 'PASS' if test_value >= lower and test_value <= upper else 'FAIL'
            final_status = 'PASS' if fail_qty == 0 else 'FAIL'

            # 불량코드 (불합격시)
            defect_code = random.choice(DEFECT_CODES) if final_status == 'FAIL' else None

            cursor.execute('''
                INSERT INTO qc_test
                (test_id, lot_id, op_exec_id, test_dt, test_type, test_item,
                 test_value, lower_limit, upper_limit, sample_qty, pass_qty,
                 fail_qty, result_flag, final_status, defect_code)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ''', (test_id, None, None, test_dt, test_type, test_item,
                  round(test_value, 2), lower, upper, sample_qty, pass_qty,
                  fail_qty, result_flag, final_status, defect_code))

    conn.commit()

def generate_inventory(conn: sqlite3.Connection, date: datetime):
    """재고 데이터 생성 (월초 기준)"""
    if date.day != 1:  # 월초에만 재고 스냅샷
        return

    cursor = conn.cursor()

    for wh in WAREHOUSES:
        for item in ITEMS:
            inv_id = f"INV{wh[2:]}{item[4:]}{date.strftime('%Y%m')}"

            # 재고 수량 (창고 유형별)
            if wh == 'WH001':  # 원자재
                qty = random.randint(5000, 20000)
            elif wh == 'WH002':  # 반제품
                qty = random.randint(1000, 5000)
            elif wh == 'WH003':  # 완제품
                qty = random.randint(3000, 15000)
            else:  # 출하대기
                qty = random.randint(500, 3000)

            safety_stock = qty * 0.2  # 안전재고: 현재고의 20%

            # 기존 레코드 업데이트 또는 삽입
            cursor.execute('''
                INSERT OR REPLACE INTO inventory
                (inv_id, warehouse_id, item_id, lot_id, qty_on_hand, safety_stock, last_update)
                VALUES (?, ?, ?, ?, ?, ?, ?)
            ''', (inv_id, wh, item, None, qty, int(safety_stock), date.isoformat()))

    conn.commit()

def generate_outbound(conn: sqlite3.Connection, date: datetime):
    """출고 데이터 생성"""
    cursor = conn.cursor()

    # 일별 3~8건 출고
    num_outbounds = random.randint(3, 8)

    for seq in range(1, num_outbounds + 1):
        outbound_id = generate_id('OB', date, seq)
        wh = random.choice(['WH003', 'WH004'])  # 완제품/출하대기 창고만
        item = random.choice(ITEMS)

        qty = random.randint(100, 2000)
        outbound_dt = random_datetime(date, 8, 18)

        # 판매오더 연결 (있을 수도 없을 수도)
        so_no = None  # 간소화

        cursor.execute('''
            INSERT INTO outbound
            (outbound_id, warehouse_id, item_id, lot_id, qty, outbound_dt, so_no)
            VALUES (?, ?, ?, ?, ?, ?, ?)
        ''', (outbound_id, wh, item, None, qty, outbound_dt, so_no))

    conn.commit()

_alarm_seq = 0  # 전역 알람 시퀀스

def generate_alarm_events(conn: sqlite3.Connection, date: datetime):
    """알람 이벤트 생성"""
    global _alarm_seq
    cursor = conn.cursor()

    # 10% 확률로 알람 발생
    if random.random() > 0.1:
        return

    for line in LINES[:3]:
        if random.random() > 0.3:
            continue

        _alarm_seq += 1
        alarm_id = generate_id('AL', date, _alarm_seq)
        equip = random.choice([e for e in EQUIPMENTS])

        alarm_dt = random_datetime(date, 6, 22)
        alarm_type = random.choice(['WARNING', 'CRITICAL'])
        alarm_code = f"ALM{random.randint(1, 50):03d}"

        descriptions = [
            '온도 상한 초과',
            '압력 이상 감지',
            '진동 수준 경고',
            '모터 과부하',
            '센서 통신 오류'
        ]
        description = random.choice(descriptions)

        # 해결 시간 (10분 ~ 2시간 후)
        resolved_dt = (datetime.fromisoformat(alarm_dt.replace('T', ' ').split('.')[0]) +
                      timedelta(minutes=random.randint(10, 120))).isoformat()

        cursor.execute('''
            INSERT INTO alarm_event
            (alarm_id, equip_id, line_id, alarm_dt, alarm_type, alarm_code, description, resolved_dt)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        ''', (alarm_id, equip, line, alarm_dt, alarm_type, alarm_code, description, resolved_dt))

    conn.commit()

# ============================================================
# 메인 실행
# ============================================================

def main(db_path: Optional[str] = None):
    """메인 실행 함수"""

    # DB 경로 설정
    if db_path is None:
        appdata = os.environ.get('APPDATA', os.path.expanduser('~'))
        db_path = os.path.join(appdata, 'Judgify', 'judgify.db')

    print(f"=" * 60)
    print(f"샘플 데이터 생성 시작")
    print(f"=" * 60)
    print(f"DB 경로: {db_path}")
    print(f"기간: {START_DATE.date()} ~ {END_DATE.date()}")
    print()

    # DB 연결
    if not os.path.exists(db_path):
        print(f"[ERROR] DB 파일이 없습니다. init_schema.sql을 먼저 실행하세요.")
        sys.exit(1)

    conn = sqlite3.connect(db_path)

    try:
        # 기존 트랜잭션 데이터 삭제 (마스터 데이터 유지)
        print("기존 트랜잭션 데이터 정리 중...")
        transaction_tables = [
            'alarm_event', 'outbound', 'inventory', 'qc_test',
            'ccp_check_log', 'sensor_log', 'downtime_event',
            'sales_order_dtl', 'sales_order', 'fg_lot',
            'production_order', 'operation_exec', 'mes_work_order'
        ]

        for table in transaction_tables:
            conn.execute(f'DELETE FROM {table}')
        conn.commit()
        print("[OK] 기존 데이터 정리 완료")
        print()

        # 날짜별 데이터 생성
        current_date = START_DATE
        total_days = (END_DATE - START_DATE).days + 1
        day_count = 0

        while current_date <= END_DATE:
            day_count += 1

            # 진행률 표시 (10일마다)
            if day_count % 10 == 0 or day_count == 1:
                progress = (day_count / total_days) * 100
                print(f"[PROGRESS] {progress:.1f}% ({current_date.date()})")

            # 작업지시 생성
            orders = generate_work_orders(conn, current_date)

            # 공정 실행 생성
            generate_operation_exec(conn, orders, current_date)

            # 생산오더 생성
            prod_orders = generate_production_orders(conn, current_date)

            # 완제품 LOT 생성
            generate_fg_lots(conn, prod_orders, current_date)

            # 판매오더 생성
            generate_sales_orders(conn, current_date)

            # 비가동 이벤트 생성
            generate_downtime_events(conn, current_date)

            # 센서 로그 생성
            generate_sensor_logs(conn, current_date)

            # CCP 점검 로그 생성
            generate_ccp_check_logs(conn, orders, current_date)

            # 품질검사 생성
            generate_qc_tests(conn, orders, current_date)

            # 재고 생성 (월초)
            generate_inventory(conn, current_date)

            # 출고 생성
            generate_outbound(conn, current_date)

            # 알람 이벤트 생성
            generate_alarm_events(conn, current_date)

            current_date += timedelta(days=1)

        print()
        print("=" * 60)
        print("[COMPLETE] 샘플 데이터 생성 완료!")
        print("=" * 60)

        # 생성된 데이터 통계
        print("\n[STATS] 생성된 데이터 통계:")
        tables = [
            'mes_work_order', 'operation_exec', 'production_order', 'fg_lot',
            'sales_order', 'sales_order_dtl', 'downtime_event', 'sensor_log',
            'ccp_check_log', 'qc_test', 'inventory', 'outbound', 'alarm_event'
        ]

        for table in tables:
            count = conn.execute(f'SELECT COUNT(*) FROM {table}').fetchone()[0]
            print(f"  - {table}: {count:,}건")

    except Exception as e:
        print(f"[ERROR] 오류 발생: {e}")
        conn.rollback()
        raise
    finally:
        conn.close()

if __name__ == '__main__':
    db_path = sys.argv[1] if len(sys.argv) > 1 else None
    main(db_path)
