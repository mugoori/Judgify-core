#!/usr/bin/env python3
"""
MES/ERP 대량 데이터 생성 스크립트 (Ver2.0 - MES 샘플 기반 확장)
- MES 작업지시: 4개월치 (하루 평균 5-8개)
- ERP 구매/판매: 1년치
- 배치/충진/완제품 LOT: 추적성 연결
- 공정 결과: 살균, 균질, 발효, 충진, 냉각
- QC 검사: 원료입고, 공정중, 완제품
- 설비 이벤트: 다운타임, 알람
"""

import sqlite3
import random
from datetime import datetime, timedelta
import os
import json

# DB 경로
DB_PATH = os.path.expandvars(r"%APPDATA%\Judgify\judgify.db")

# ========== 제품 그룹별 분류 ==========
PRODUCT_GROUPS = {
    '백색시유': {
        'items': ['FG-001', 'FG-002', 'FG-003', 'FG-004', 'FG-005', 'FG-006', 'FG-007', 'FG-008', 'FG-009'],
        'boms': ['BOM-001', 'BOM-002', 'BOM-003', 'BOM-004', 'BOM-005', 'BOM-006', 'BOM-007', 'BOM-008', 'BOM-009'],
        'lines': ['LINE-MILK1', 'LINE-MILK2', 'LINE-LOWFAT'],
        'defect_rate': {'mean': 0.5, 'std': 0.2},  # 0.3~0.7%
        'processes': ['원유수입', '표준화', '균질', '살균', '냉각', '충진'],
    },
    '가공유': {
        'items': ['FG-010', 'FG-011', 'FG-012', 'FG-013', 'FG-014', 'FG-015', 'FG-016', 'FG-017'],
        'boms': ['BOM-010', 'BOM-011', 'BOM-012', 'BOM-013', 'BOM-014', 'BOM-015', 'BOM-016', 'BOM-017'],
        'lines': ['LINE-FLAVOR1', 'LINE-FLAVOR2'],
        'defect_rate': {'mean': 1.2, 'std': 0.4},  # 0.8~1.6%
        'processes': ['원료배합', '균질', '살균', '냉각', '충진'],
    },
    '발효유': {
        'items': ['FG-018', 'FG-019', 'FG-020', 'FG-021', 'FG-022', 'FG-023', 'FG-024', 'FG-025', 'FG-026', 'FG-027'],
        'boms': ['BOM-018', 'BOM-019', 'BOM-020', 'BOM-021', 'BOM-022', 'BOM-023', 'BOM-024', 'BOM-025', 'BOM-026', 'BOM-027'],
        'lines': ['LINE-YOGURT1', 'LINE-YOGURT2', 'LINE-YOGURT3'],
        'defect_rate': {'mean': 2.0, 'std': 0.6},  # 1.4~2.6%
        'processes': ['원료배합', '균질', '살균', '냉각', '접종', '발효', '충진'],
    },
    'UHT': {
        'items': ['FG-028', 'FG-029', 'FG-030', 'FG-031', 'FG-032'],
        'boms': ['BOM-028', 'BOM-029', 'BOM-030', 'BOM-031', 'BOM-032'],
        'lines': ['LINE-UHT1', 'LINE-UHT2'],
        'defect_rate': {'mean': 0.3, 'std': 0.1},  # 0.2~0.4%
        'processes': ['원료배합', '균질', 'UHT멸균', '무균충진'],
    },
    '두유': {
        'items': ['FG-033', 'FG-034', 'FG-035', 'FG-036', 'FG-037', 'FG-038', 'FG-039', 'FG-040'],
        'boms': ['BOM-033', 'BOM-034', 'BOM-035', 'BOM-036', 'BOM-037', 'BOM-038', 'BOM-039', 'BOM-040'],
        'lines': ['LINE-SOY', 'LINE-ALT'],
        'defect_rate': {'mean': 1.5, 'std': 0.5},  # 1.0~2.0%
        'processes': ['두유추출', '배합', '균질', '살균', '냉각', '충진'],
    },
}

# ========== 불량 유형 ==========
DEFECT_TYPES = {
    '부품불량': ['포장재 불량', '캡 불량', '라벨 인쇄 불량', '병 파손'],
    '완제품불량': ['충진량 미달', '이물 혼입', '맛/향 이상', '유통기한 오류', '점도 이상'],
}

# ========== 공정별 파라미터 기준 ==========
PROCESS_PARAMS = {
    '살균': {'target_temp': 72, 'temp_range': (70, 75), 'target_time': 15, 'time_range': (14, 17)},
    'UHT멸균': {'target_temp': 135, 'temp_range': (132, 138), 'target_time': 2, 'time_range': (1, 3)},
    '균질': {'target_temp': 65, 'temp_range': (62, 68), 'target_time': 30, 'time_range': (25, 35)},
    '발효': {'target_temp': 42, 'temp_range': (40, 44), 'target_time': 480, 'time_range': (420, 540)},
    '냉각': {'target_temp': 4, 'temp_range': (2, 6), 'target_time': 60, 'time_range': (50, 70)},
    '충진': {'target_temp': 4, 'temp_range': (2, 8), 'target_time': 120, 'time_range': (90, 150)},
    '원유수입': {'target_temp': 4, 'temp_range': (2, 6), 'target_time': 60, 'time_range': (45, 75)},
    '표준화': {'target_temp': 10, 'temp_range': (8, 12), 'target_time': 45, 'time_range': (35, 55)},
    '원료배합': {'target_temp': 25, 'temp_range': (20, 30), 'target_time': 30, 'time_range': (20, 40)},
    '접종': {'target_temp': 42, 'temp_range': (40, 44), 'target_time': 10, 'time_range': (8, 12)},
    '두유추출': {'target_temp': 90, 'temp_range': (85, 95), 'target_time': 60, 'time_range': (50, 70)},
    '배합': {'target_temp': 25, 'temp_range': (20, 30), 'target_time': 30, 'time_range': (20, 40)},
    '무균충진': {'target_temp': 25, 'temp_range': (20, 30), 'target_time': 90, 'time_range': (80, 100)},
}

# ========== 공정명 → DB 코드 매핑 ==========
# DB 스키마: process_type IN ('BATCHING', 'PASTEURIZATION', 'COOLING', 'HOLDING', 'TRANSFER')
PROCESS_TYPE_MAPPING = {
    # 배합/혼합 관련 → BATCHING
    '원유수입': 'BATCHING',
    '표준화': 'BATCHING',
    '원료배합': 'BATCHING',
    '배합': 'BATCHING',
    '두유추출': 'BATCHING',
    '접종': 'BATCHING',
    # 살균/멸균 관련 → PASTEURIZATION
    '살균': 'PASTEURIZATION',
    'UHT멸균': 'PASTEURIZATION',
    '균질': 'PASTEURIZATION',
    # 냉각 관련 → COOLING
    '냉각': 'COOLING',
    # 발효/유지 관련 → HOLDING
    '발효': 'HOLDING',
    # 충진/이송 관련 → TRANSFER
    '충진': 'TRANSFER',
    '무균충진': 'TRANSFER',
}

# ========== 다운타임 사유 ==========
DOWNTIME_REASONS = {
    '계획정지': [
        ('PM', '정기점검'),
        ('CIP', 'CIP 세척'),
        ('CO', '품종교체'),
        ('CL', '라인 청소'),
        ('MT', '정기 유지보수'),
    ],
    '비계획정지': [
        ('BK', '설비고장'),
        ('QA', '품질이상'),
        ('MA', '원료부족'),
        ('OP', '작업자 부재'),
        ('UT', '유틸리티 이상'),
    ],
}

# ========== 알람 유형 ==========
# DB 스키마: alarm_type IN ('PARAM_HIGH', 'PARAM_LOW', 'CCP_DEVIATION', 'EQUIP_FAULT', 'QUALITY_ISSUE', 'SAFETY')
ALARM_TYPES = {
    'PARAM_HIGH': [
        ('TEMP_HIGH', '온도 상한 초과', 'TEMP'),
        ('PRESSURE_HIGH', '압력 상한 초과', 'PRES'),
        ('FLOW_HIGH', '유량 상한 초과', 'FLOW'),
    ],
    'PARAM_LOW': [
        ('TEMP_LOW', '온도 하한 미달', 'TEMP'),
        ('PRESSURE_LOW', '압력 하한 미달', 'PRES'),
        ('LEVEL_LOW', '레벨 하한 미달', 'LEVEL'),
    ],
    'CCP_DEVIATION': [
        ('CCP_TEMP', '살균온도 이탈', 'CCP-TEMP'),
        ('CCP_TIME', '살균시간 미달', 'CCP-TIME'),
        ('CCP_COOL', '냉각온도 이탈', 'CCP-COOL'),
    ],
    'EQUIP_FAULT': [
        ('MOTOR_OVERLOAD', '모터 과부하', 'MOTOR'),
        ('SENSOR_FAULT', '센서 이상', 'SENSOR'),
        ('VIBRATION', '진동 과다', 'VIBR'),
    ],
    'QUALITY_ISSUE': [
        ('METAL_DETECT', '금속 검출', 'METAL'),
        ('WEIGHT_DEV', '중량 이탈', 'WEIGHT'),
        ('VISCOSITY', '점도 이상', 'VISC'),
    ],
    'SAFETY': [
        ('DOOR_OPEN', '안전도어 개방', 'DOOR'),
        ('EMERGENCY_STOP', '비상정지 작동', 'ESTOP'),
        ('GUARD_OPEN', '안전가드 이탈', 'GUARD'),
    ],
}

# ========== QC 검사 항목 ==========
QC_TEST_ITEMS = {
    '원료입고': {
        'items': [
            ('수분함량', '%', 3.5, 4.5, 4.0),
            ('지방함량', '%', 3.2, 3.8, 3.5),
            ('세균수', 'CFU/ml', 0, 50000, 30000),
            ('산도', '°SH', 14, 18, 16),
            ('비중', '', 1.028, 1.034, 1.031),
        ],
    },
    '공정중': {
        'items': [
            ('온도', '°C', None, None, None),  # 공정별 다름
            ('pH', '', 6.4, 6.8, 6.6),
            ('점도', 'cP', 2.0, 4.0, 3.0),
            ('Brix', '°Bx', 9.0, 12.0, 10.5),
        ],
    },
    '완제품': {
        'items': [
            ('관능검사', '점', 3, 5, 4),
            ('대장균군', 'CFU/ml', 0, 10, 0),
            ('세균수', 'CFU/ml', 0, 20000, 5000),
            ('산도', '°SH', 14, 18, 16),
            ('유통기한확인', '', 1, 1, 1),
        ],
    },
}

# ========== 마스터 데이터 (기존) ==========
BOMS = ['BOM-001', 'BOM-002', 'BOM-003', 'BOM-004', 'BOM-005', 'BOM-006', 'BOM-007', 'BOM-008',
        'BOM-009', 'BOM-010', 'BOM-011', 'BOM-012', 'BOM-013', 'BOM-014', 'BOM-015', 'BOM-016',
        'BOM-017', 'BOM-018', 'BOM-019', 'BOM-020', 'BOM-021', 'BOM-022', 'BOM-023', 'BOM-024',
        'BOM-025', 'BOM-026', 'BOM-027', 'BOM-028', 'BOM-029', 'BOM-030', 'BOM-031', 'BOM-032',
        'BOM-033', 'BOM-034', 'BOM-035', 'BOM-036', 'BOM-037', 'BOM-038', 'BOM-039', 'BOM-040']
LINES = ['LINE-MILK1', 'LINE-MILK2', 'LINE-LOWFAT', 'LINE-YOGURT1', 'LINE-YOGURT2', 'LINE-YOGURT3',
         'LINE-FLAVOR1', 'LINE-FLAVOR2', 'LINE-UHT1', 'LINE-UHT2', 'LINE-SOY', 'LINE-ALT']
SHIFTS = ['SHIFT-A', 'SHIFT-B', 'SHIFT-C']
VENDORS = ['VD-001', 'VD-002', 'VD-003', 'VD-004', 'VD-005', 'VD-006', 'VD-007', 'VD-008']
CUSTOMERS = ['CT-001', 'CT-002', 'CT-003', 'CT-004', 'CT-005', 'CT-006', 'CT-007', 'CT-008']
OPERATORS = ['OP001', 'OP002', 'OP003', 'OP004', 'OP005', 'OP006']

# 설비 코드 (공정별)
EQUIPMENT_BY_PROCESS = {
    '원유수입': ['EQ-001', 'EQ-002', 'EQ-003'],
    '표준화': ['EQ-006'],
    '균질': ['EQ-007', 'EQ-008'],
    '살균': ['EQ-009', 'EQ-010'],
    'UHT멸균': ['EQ-011', 'EQ-012'],
    '발효': ['EQ-013', 'EQ-014', 'EQ-015'],
    '냉각': ['EQ-025', 'EQ-026', 'EQ-027'],
    '충진': ['EQ-028', 'EQ-029', 'EQ-030', 'EQ-031', 'EQ-032'],
    '원료배합': ['EQ-020', 'EQ-021', 'EQ-022'],
    '접종': ['EQ-019'],
    '두유추출': ['EQ-022'],
    '배합': ['EQ-020', 'EQ-021'],
    '무균충진': ['EQ-031', 'EQ-032'],
}

# 아이템 코드
ITEMS_RM = ['RM-001', 'RM-002', 'RM-003', 'RM-004', 'RM-005', 'RM-006', 'RM-007', 'RM-008',
            'RM-009', 'RM-010', 'RM-011', 'RM-012', 'RM-013', 'RM-014', 'RM-015']
ITEMS_FG = ['FG-001', 'FG-002', 'FG-003', 'FG-004', 'FG-005', 'FG-006', 'FG-007', 'FG-008',
            'FG-009', 'FG-010', 'FG-011', 'FG-012', 'FG-013', 'FG-014', 'FG-015', 'FG-016',
            'FG-017', 'FG-018', 'FG-019', 'FG-020']
ITEMS = ITEMS_RM + ITEMS_FG

def get_date_str(d):
    return d.strftime('%Y-%m-%d')

def get_datetime_str(d):
    return d.strftime('%Y-%m-%d %H:%M:%S')

def generate_production_orders(conn, start_date, end_date):
    """생산 지시 생성 (4개월)"""
    cursor = conn.cursor()

    # 기존 데이터 삭제
    cursor.execute("DELETE FROM production_order WHERE prod_order_no LIKE 'PO-2024%' OR prod_order_no LIKE 'PO-2025%'")

    current = start_date
    prod_orders = []

    while current <= end_date:
        # 주말은 건너뛰기 (토,일)
        if current.weekday() < 5:
            # 하루에 2-4개 생산지시
            daily_count = random.randint(2, 4)
            for i in range(daily_count):
                prod_order_no = f"PO-{current.strftime('%Y%m%d')}-{i+1:03d}"
                bom = random.choice(BOMS)
                plan_qty = random.choice([500, 1000, 1500, 2000, 2500, 3000])
                priority = random.randint(1, 10)

                # 과거 데이터는 완료 상태
                if current < datetime.now() - timedelta(days=7):
                    status = random.choices(['COMPLETED', 'IN_PROGRESS', 'CANCELLED'], weights=[85, 10, 5])[0]
                    actual_qty = plan_qty * random.uniform(0.9, 1.02) if status == 'COMPLETED' else plan_qty * random.uniform(0.3, 0.7)
                elif current < datetime.now():
                    status = random.choices(['COMPLETED', 'IN_PROGRESS', 'RELEASED'], weights=[60, 30, 10])[0]
                    actual_qty = plan_qty * random.uniform(0.85, 1.0) if status == 'COMPLETED' else plan_qty * random.uniform(0.2, 0.5)
                else:
                    status = random.choices(['PLANNED', 'RELEASED'], weights=[70, 30])[0]
                    actual_qty = 0

                line = random.choice(LINES)

                prod_orders.append((
                    prod_order_no, bom, get_date_str(current), plan_qty, actual_qty,
                    status, priority, None, None, line, None, get_datetime_str(current)
                ))

        current += timedelta(days=1)

    cursor.executemany("""
        INSERT OR IGNORE INTO production_order
        (prod_order_no, bom_cd, plan_date, plan_qty, actual_qty, status, priority,
         start_time, end_time, line_cd, remark, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    """, prod_orders)

    print(f"생산지시 생성: {len(prod_orders)}건")
    return [p[0] for p in prod_orders]

def generate_work_orders(conn, prod_order_nos, start_date, end_date):
    """작업 지시 생성 (MES)"""
    cursor = conn.cursor()

    # 기존 데이터 삭제
    cursor.execute("DELETE FROM mes_work_order WHERE wo_no LIKE 'WO-2024%' OR wo_no LIKE 'WO-2025%'")

    work_orders = []
    wo_count = 0

    # prod_order_no별로 작업지시 생성
    cursor.execute("SELECT prod_order_no, plan_date, plan_qty, status, line_cd FROM production_order WHERE prod_order_no IN ({})".format(
        ','.join(['?']*len(prod_order_nos))), prod_order_nos)

    for prod_order_no, plan_date, plan_qty, prod_status, line_cd in cursor.fetchall():
        plan_dt = datetime.strptime(plan_date, '%Y-%m-%d')

        # 생산지시당 1-3개 작업지시
        wo_count_per_prod = random.randint(1, 3)
        remaining_qty = plan_qty

        for i in range(wo_count_per_prod):
            wo_no = f"WO-{plan_date.replace('-', '')}-{wo_count+1:04d}"
            wo_count += 1

            shift = random.choice(SHIFTS)
            operator = random.choice(OPERATORS)

            # 수량 분배
            if i == wo_count_per_prod - 1:
                wo_qty = remaining_qty
            else:
                wo_qty = remaining_qty * random.uniform(0.3, 0.5)
                remaining_qty -= wo_qty

            wo_qty = round(wo_qty)

            # 시간 설정
            if shift == 'SHIFT-A':
                plan_start = plan_dt.replace(hour=6, minute=0)
            elif shift == 'SHIFT-B':
                plan_start = plan_dt.replace(hour=14, minute=0)
            else:
                plan_start = plan_dt.replace(hour=22, minute=0)

            plan_end = plan_start + timedelta(hours=8)

            # 상태 결정
            if prod_status == 'COMPLETED':
                status = 'COMPLETED'
                good_qty = wo_qty * random.uniform(0.95, 0.99)
                reject_qty = wo_qty - good_qty
                actual_start = plan_start + timedelta(minutes=random.randint(-30, 30))
                actual_end = plan_end + timedelta(minutes=random.randint(-60, 60))
            elif prod_status == 'IN_PROGRESS':
                status = random.choice(['RUNNING', 'COMPLETED', 'PAUSED'])
                if status == 'COMPLETED':
                    good_qty = wo_qty * random.uniform(0.92, 0.98)
                    reject_qty = wo_qty - good_qty
                else:
                    good_qty = wo_qty * random.uniform(0.3, 0.7)
                    reject_qty = good_qty * random.uniform(0.01, 0.05)
                actual_start = plan_start + timedelta(minutes=random.randint(-20, 20))
                actual_end = None if status == 'RUNNING' else plan_end
            elif prod_status == 'CANCELLED':
                status = 'CANCELLED'
                good_qty = 0
                reject_qty = 0
                actual_start = None
                actual_end = None
            else:
                status = random.choice(['SCHEDULED', 'READY'])
                good_qty = 0
                reject_qty = 0
                actual_start = None
                actual_end = None

            work_orders.append((
                wo_no, prod_order_no, line_cd or random.choice(LINES), shift,
                plan_date, get_datetime_str(plan_start), get_datetime_str(plan_end),
                get_datetime_str(actual_start) if actual_start else None,
                get_datetime_str(actual_end) if actual_end else None,
                status, wo_qty, round(good_qty), round(reject_qty),
                operator, get_datetime_str(plan_dt)
            ))

    cursor.executemany("""
        INSERT OR IGNORE INTO mes_work_order
        (wo_no, prod_order_no, line_cd, shift_cd, plan_date, plan_start, plan_end,
         actual_start, actual_end, status, plan_qty, good_qty, reject_qty,
         operator_id, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    """, work_orders)

    print(f"작업지시 생성: {len(work_orders)}건")
    return work_orders

def generate_purchase_orders(conn, start_date, end_date):
    """구매 발주 생성 (1년)"""
    cursor = conn.cursor()

    # 기존 데이터 삭제 (새로 생성할 것만)
    cursor.execute("DELETE FROM purchase_order WHERE po_no LIKE 'PUR-2024%' OR po_no LIKE 'PUR-2025%'")
    cursor.execute("DELETE FROM purchase_order_dtl WHERE po_no LIKE 'PUR-2024%' OR po_no LIKE 'PUR-2025%'")

    current = start_date
    po_list = []
    dtl_list = []
    po_count = 0

    while current <= end_date:
        # 주 2-4회 구매발주
        if current.weekday() in [1, 3] or (current.weekday() == 4 and random.random() > 0.5):
            daily_count = random.randint(1, 3)
            for _ in range(daily_count):
                po_count += 1
                po_no = f"PUR-{current.strftime('%Y%m%d')}-{po_count:04d}"
                vendor = random.choice(VENDORS)
                expected_date = current + timedelta(days=random.randint(3, 14))

                # 과거는 대부분 완료
                if current < datetime.now() - timedelta(days=30):
                    status = random.choices(['COMPLETED', 'PARTIAL', 'CANCELLED'], weights=[85, 10, 5])[0]
                elif current < datetime.now():
                    status = random.choices(['COMPLETED', 'PARTIAL', 'CONFIRMED'], weights=[60, 25, 15])[0]
                else:
                    status = random.choices(['CONFIRMED', 'DRAFT'], weights=[70, 30])[0]

                # 상세 품목 (1-4개)
                dtl_count = random.randint(1, 4)
                total_amount = 0
                raw_items = [i for i in ITEMS if 'RM' in i]

                for seq in range(1, dtl_count + 1):
                    item = random.choice(raw_items)
                    qty = random.choice([100, 200, 300, 500, 1000])
                    unit_price = random.uniform(1000, 50000)
                    amount = qty * unit_price
                    total_amount += amount

                    recv_qty = qty if status == 'COMPLETED' else (qty * random.uniform(0.3, 0.8) if status == 'PARTIAL' else 0)

                    # purchase_order_dtl 스키마: po_no, seq, item_cd, qty, unit_price, amount, received_qty
                    dtl_list.append((po_no, seq, item, qty, unit_price, amount, recv_qty))

                po_list.append((
                    po_no, vendor, get_date_str(current), get_date_str(expected_date),
                    status, total_amount, None, 'SYSTEM', get_datetime_str(current)
                ))

        current += timedelta(days=1)

    cursor.executemany("""
        INSERT OR IGNORE INTO purchase_order
        (po_no, vendor_cd, order_date, expected_date, status, total_amount,
         remark, created_by, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
    """, po_list)

    cursor.executemany("""
        INSERT OR IGNORE INTO purchase_order_dtl
        (po_no, seq, item_cd, qty, unit_price, amount, received_qty)
        VALUES (?, ?, ?, ?, ?, ?, ?)
    """, dtl_list)

    print(f"구매발주 생성: {len(po_list)}건, 상세 {len(dtl_list)}건")

def generate_sales_orders(conn, start_date, end_date):
    """판매 주문 생성 (1년)"""
    cursor = conn.cursor()

    # 기존 데이터 삭제
    cursor.execute("DELETE FROM sales_order WHERE so_no LIKE 'SO-2024%' OR so_no LIKE 'SO-2025%'")
    cursor.execute("DELETE FROM sales_order_dtl WHERE so_no LIKE 'SO-2024%' OR so_no LIKE 'SO-2025%'")

    current = start_date
    so_list = []
    dtl_list = []
    so_count = 0

    while current <= end_date:
        # 주말 제외, 하루 3-8건
        if current.weekday() < 5:
            daily_count = random.randint(3, 8)
            for _ in range(daily_count):
                so_count += 1
                so_no = f"SO-{current.strftime('%Y%m%d')}-{so_count:05d}"
                customer = random.choice(CUSTOMERS)
                request_date = current + timedelta(days=random.randint(1, 7))

                # 상태 결정
                if current < datetime.now() - timedelta(days=14):
                    status = random.choices(['COMPLETED', 'SHIPPED', 'CANCELLED'], weights=[75, 15, 10])[0]
                elif current < datetime.now():
                    status = random.choices(['COMPLETED', 'SHIPPED', 'ALLOCATED', 'CONFIRMED'], weights=[50, 25, 15, 10])[0]
                else:
                    status = random.choices(['CONFIRMED', 'DRAFT', 'ALLOCATED'], weights=[50, 30, 20])[0]

                # 상세 품목 (1-5개)
                dtl_count = random.randint(1, 5)
                total_amount = 0
                fg_items = [i for i in ITEMS if 'FG' in i]

                for seq in range(1, dtl_count + 1):
                    item = random.choice(fg_items)
                    qty = random.choice([10, 20, 50, 100, 200, 500])
                    unit_price = random.uniform(5000, 100000)
                    amount = qty * unit_price
                    total_amount += amount

                    alloc_qty = qty if status in ['ALLOCATED', 'SHIPPED', 'COMPLETED'] else 0
                    ship_qty = qty if status in ['COMPLETED', 'SHIPPED'] else 0

                    # sales_order_dtl 스키마: so_no, seq, item_cd, qty, unit_price, amount, allocated_qty, shipped_qty
                    dtl_list.append((so_no, seq, item, qty, unit_price, amount, alloc_qty, ship_qty))

                addresses = ['서울시 강남구', '부산시 해운대구', '대구시 수성구', '인천시 연수구', '광주시 서구']

                so_list.append((
                    so_no, customer, get_date_str(current), get_date_str(request_date),
                    status, total_amount, random.choice(addresses), None, 'SYSTEM', get_datetime_str(current)
                ))

        current += timedelta(days=1)

    cursor.executemany("""
        INSERT OR IGNORE INTO sales_order
        (so_no, cust_cd, order_date, request_date, status, total_amount,
         ship_address, remark, created_by, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    """, so_list)

    cursor.executemany("""
        INSERT OR IGNORE INTO sales_order_dtl
        (so_no, seq, item_cd, qty, unit_price, amount, allocated_qty, shipped_qty)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
    """, dtl_list)

    print(f"판매주문 생성: {len(so_list)}건, 상세 {len(dtl_list)}건")


# ========== 신규 MES 데이터 생성 함수들 ==========

def get_product_group_for_bom(bom_cd):
    """BOM 코드로 제품 그룹 찾기"""
    for group_name, group_data in PRODUCT_GROUPS.items():
        if bom_cd in group_data['boms']:
            return group_name, group_data
    # 기본값
    return '백색시유', PRODUCT_GROUPS['백색시유']


def generate_batch_lots(conn, start_date, end_date):
    """배치 LOT 생성 (production_order 기반)"""
    cursor = conn.cursor()

    # 기존 데이터 삭제
    cursor.execute("DELETE FROM batch_lot WHERE batch_lot_no LIKE 'BAT-%'")

    # 완료된 생산지시 조회
    cursor.execute("""
        SELECT prod_order_no, bom_cd, plan_date, actual_qty, status, line_cd
        FROM production_order
        WHERE status IN ('COMPLETED', 'IN_PROGRESS')
        ORDER BY plan_date
    """)
    prod_orders = cursor.fetchall()

    batch_lots = []
    batch_count = 0

    for prod_order_no, bom_cd, plan_date, actual_qty, status, line_cd in prod_orders:
        plan_dt = datetime.strptime(plan_date, '%Y-%m-%d')

        # 생산지시당 1-3개 배치
        batch_per_order = random.randint(1, 3) if status == 'COMPLETED' else 1
        remaining_qty = actual_qty or 0

        for i in range(batch_per_order):
            if remaining_qty <= 0:
                break

            batch_count += 1
            batch_lot_no = f"BAT-{plan_date.replace('-', '')}-{batch_count:04d}"

            # 배치 수량 분배
            if i == batch_per_order - 1:
                batch_size = remaining_qty
            else:
                batch_size = remaining_qty * random.uniform(0.3, 0.5)
                remaining_qty -= batch_size

            batch_size = round(batch_size, 1)

            # 탱크 할당
            tank_no = f"T-{random.randint(101, 110):03d}"

            # 시간 설정
            start_time = plan_dt.replace(hour=random.randint(6, 14), minute=random.randint(0, 59))
            duration_hours = random.randint(4, 8)
            end_time = start_time + timedelta(hours=duration_hours)

            batch_status = 'COMPLETED' if status == 'COMPLETED' else random.choice(['IN_PROGRESS', 'COMPLETED'])

            batch_lots.append((
                batch_lot_no, prod_order_no, bom_cd, plan_date, batch_size,
                batch_status, tank_no,
                get_datetime_str(start_time),
                get_datetime_str(end_time) if batch_status == 'COMPLETED' else None,
                random.choice(OPERATORS), None, get_datetime_str(plan_dt)
            ))

    cursor.executemany("""
        INSERT OR IGNORE INTO batch_lot
        (batch_lot_no, prod_order_no, bom_cd, batch_date, batch_size, status, tank_no,
         start_time, end_time, operator_id, remark, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    """, batch_lots)

    print(f"배치LOT 생성: {len(batch_lots)}건")
    return [b[0] for b in batch_lots]


def generate_filling_lots(conn, batch_lot_nos):
    """충진 LOT 생성 (batch_lot 기반)"""
    cursor = conn.cursor()

    # 기존 데이터 삭제
    cursor.execute("DELETE FROM filling_lot WHERE filling_lot_no LIKE 'FIL-%'")

    # 완료된 배치 조회
    cursor.execute("""
        SELECT batch_lot_no, bom_cd, batch_date, batch_size, status
        FROM batch_lot
        WHERE batch_lot_no IN ({})
        AND status = 'COMPLETED'
    """.format(','.join(['?']*len(batch_lot_nos))), batch_lot_nos)
    batches = cursor.fetchall()

    filling_lots = []
    fill_count = 0

    for batch_lot_no, bom_cd, batch_date, batch_size, status in batches:
        batch_dt = datetime.strptime(batch_date, '%Y-%m-%d')
        group_name, group_data = get_product_group_for_bom(bom_cd)

        # 배치당 1-4개 충진 LOT
        fill_per_batch = random.randint(1, 4)
        remaining_qty = int(batch_size * 10)  # 배치크기를 개수로 변환 (가정)

        # 제품군에 맞는 불량률 적용
        defect_rate = max(0, random.gauss(
            group_data['defect_rate']['mean'],
            group_data['defect_rate']['std']
        )) / 100

        for i in range(fill_per_batch):
            if remaining_qty <= 0:
                break

            fill_count += 1
            filling_lot_no = f"FIL-{batch_date.replace('-', '')}-{fill_count:05d}"

            # 수량 분배
            if i == fill_per_batch - 1:
                plan_qty = remaining_qty
            else:
                plan_qty = int(remaining_qty * random.uniform(0.2, 0.4))
                remaining_qty -= plan_qty

            # 불량 계산 (부품불량 + 완제품불량)
            total_reject = int(plan_qty * defect_rate)
            good_qty = plan_qty - total_reject

            # 라인 할당
            line = random.choice(group_data['lines']) if group_data['lines'] else random.choice(LINES)

            # 시간 설정
            start_time = batch_dt.replace(hour=random.randint(8, 18), minute=random.randint(0, 59))
            duration_mins = random.randint(60, 180)
            end_time = start_time + timedelta(minutes=duration_mins)

            # 포장재 아이템 (임의)
            pkg_item = random.choice(ITEMS_FG) if ITEMS_FG else 'PKG-001'

            filling_lots.append((
                filling_lot_no, batch_lot_no, batch_date, line, pkg_item,
                plan_qty, good_qty, total_reject,
                get_datetime_str(start_time), get_datetime_str(end_time),
                'COMPLETED'
            ))

    cursor.executemany("""
        INSERT OR IGNORE INTO filling_lot
        (filling_lot_no, batch_lot_no, filling_date, line_cd, pkg_item_cd,
         plan_qty, good_qty, reject_qty, start_time, end_time, status)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    """, filling_lots)

    print(f"충진LOT 생성: {len(filling_lots)}건")
    return [f[0] for f in filling_lots]


def generate_fg_lots(conn, filling_lot_nos):
    """완제품 LOT 생성 (filling_lot 기반)"""
    cursor = conn.cursor()

    # 기존 데이터 삭제
    cursor.execute("DELETE FROM fg_lot WHERE fg_lot_no LIKE 'FG-LOT-%'")

    # 완료된 충진 조회
    cursor.execute("""
        SELECT fl.filling_lot_no, fl.batch_lot_no, fl.filling_date, fl.good_qty, bl.bom_cd
        FROM filling_lot fl
        JOIN batch_lot bl ON fl.batch_lot_no = bl.batch_lot_no
        WHERE fl.filling_lot_no IN ({})
        AND fl.status = 'COMPLETED'
    """.format(','.join(['?']*len(filling_lot_nos))), filling_lot_nos)
    fillings = cursor.fetchall()

    fg_lots = []
    fg_count = 0

    for filling_lot_no, batch_lot_no, filling_date, good_qty, bom_cd in fillings:
        fill_dt = datetime.strptime(filling_date, '%Y-%m-%d')

        fg_count += 1
        fg_lot_no = f"FG-LOT-{filling_date.replace('-', '')}-{fg_count:05d}"

        # BOM에서 완제품 코드 찾기
        fg_item_cd = f"FG-{bom_cd.replace('BOM-', '')}" if bom_cd else random.choice(ITEMS_FG)

        # 제조일, 유통기한
        mfg_date = filling_date
        exp_date = (fill_dt + timedelta(days=random.randint(7, 30))).strftime('%Y-%m-%d')

        # QC 상태 (대부분 합격) - DB: PENDING, TESTING, PASSED, FAILED, HOLD
        qc_status = random.choices(['PASSED', 'HOLD', 'FAILED'], weights=[95, 4, 1])[0]

        # 출하일
        release_date = (fill_dt + timedelta(days=random.randint(1, 3))).strftime('%Y-%m-%d') if qc_status == 'PASSED' else None

        # 저장 위치
        storage_loc = f"W{random.randint(1, 5):02d}-R{random.randint(1, 20):02d}"

        fg_lots.append((
            fg_lot_no, filling_lot_no, fg_item_cd, good_qty,
            mfg_date, exp_date, qc_status, release_date, storage_loc,
            get_datetime_str(fill_dt)
        ))

    cursor.executemany("""
        INSERT OR IGNORE INTO fg_lot
        (fg_lot_no, filling_lot_no, fg_item_cd, qty, mfg_date, exp_date,
         qc_status, release_date, storage_loc, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    """, fg_lots)

    print(f"완제품LOT 생성: {len(fg_lots)}건")
    return fg_lots


def generate_process_results(conn, batch_lot_nos):
    """공정 결과 생성 (batch_lot 기반)"""
    cursor = conn.cursor()

    # 기존 데이터 삭제
    cursor.execute("DELETE FROM process_result")

    # 배치 조회
    cursor.execute("""
        SELECT batch_lot_no, bom_cd, batch_date, start_time, end_time
        FROM batch_lot
        WHERE batch_lot_no IN ({})
        AND status = 'COMPLETED'
    """.format(','.join(['?']*len(batch_lot_nos))), batch_lot_nos)
    batches = cursor.fetchall()

    process_results = []

    for batch_lot_no, bom_cd, batch_date, start_time_str, end_time_str in batches:
        group_name, group_data = get_product_group_for_bom(bom_cd)
        processes = group_data['processes']

        if not start_time_str:
            batch_dt = datetime.strptime(batch_date, '%Y-%m-%d')
            current_time = batch_dt.replace(hour=6, minute=0)
        else:
            current_time = datetime.strptime(start_time_str, '%Y-%m-%d %H:%M:%S')

        for process_name in processes:
            params = PROCESS_PARAMS.get(process_name, PROCESS_PARAMS['살균'])
            # 한글 공정명 → DB 코드 변환
            db_process_type = PROCESS_TYPE_MAPPING.get(process_name, 'BATCHING')

            # 목표값
            target_temp = params['target_temp']
            target_time = params['target_time']

            # 실제값 (대부분 정상 범위, 가끔 이탈)
            if random.random() < 0.95:  # 95% 정상
                actual_temp = random.uniform(*params['temp_range'])
                actual_time = random.randint(*params['time_range'])
                result = 'OK'
            else:  # 5% 이탈
                if random.random() < 0.5:
                    actual_temp = params['temp_range'][0] - random.uniform(1, 5)
                else:
                    actual_temp = params['temp_range'][1] + random.uniform(1, 5)
                actual_time = random.randint(params['time_range'][0] - 5, params['time_range'][1] + 5)
                result = 'NG'

            # 설비 선택
            equip_list = EQUIPMENT_BY_PROCESS.get(process_name, ['EQ-001'])
            equip_cd = random.choice(equip_list)

            # 시간 계산
            start_time = current_time
            end_time = start_time + timedelta(seconds=actual_time * 60)
            current_time = end_time + timedelta(minutes=random.randint(5, 15))  # 공정 간 간격

            process_results.append((
                batch_lot_no, db_process_type,  # DB 코드 사용
                get_datetime_str(start_time), get_datetime_str(end_time),
                target_temp, round(actual_temp, 1),
                target_time * 60, actual_time * 60,  # 초 단위
                equip_cd, result, f"{process_name}"  # 한글 공정명은 remark에 저장
            ))

    cursor.executemany("""
        INSERT INTO process_result
        (batch_lot_no, process_type, start_time, end_time,
         target_temp, actual_temp, target_time_sec, actual_time_sec,
         equipment_cd, result, remark)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    """, process_results)

    print(f"공정결과 생성: {len(process_results)}건")


def generate_qc_tests(conn, batch_lot_nos, filling_lot_nos, fg_lots):
    """QC 검사 데이터 생성"""
    cursor = conn.cursor()

    # 기존 데이터 삭제
    cursor.execute("DELETE FROM qc_test WHERE qc_no LIKE 'QC-%'")

    qc_tests = []
    qc_count = 0

    # 1. 원료입고 검사 (inbound 기반) - 샘플링
    cursor.execute("SELECT DISTINCT batch_lot_no, batch_date FROM batch_lot LIMIT 100")
    for batch_lot_no, batch_date in cursor.fetchall():
        if random.random() < 0.3:  # 30% 샘플링
            qc_count += 1
            qc_no = f"QC-RM-{batch_date.replace('-', '')}-{qc_count:04d}"

            test_items = []
            for item_name, unit, min_val, max_val, target in QC_TEST_ITEMS['원료입고']['items']:
                actual = random.gauss(target, (max_val - min_val) / 6)
                actual = max(min_val * 0.9, min(max_val * 1.1, actual))
                result = 'PASS' if min_val <= actual <= max_val else 'FAIL'
                test_items.append({
                    'item': item_name, 'unit': unit,
                    'min': min_val, 'max': max_val, 'target': target,
                    'actual': round(actual, 2), 'result': result
                })

            overall_result = 'PASS' if all(t['result'] == 'PASS' for t in test_items) else 'FAIL'

            qc_tests.append((
                qc_no, 'INCOMING', 'BATCH', batch_lot_no,
                random.choice(ITEMS_RM), batch_lot_no,
                batch_date, random.choice(OPERATORS),
                overall_result, json.dumps(test_items, ensure_ascii=False),
                None, get_datetime_str(datetime.strptime(batch_date, '%Y-%m-%d'))
            ))

    # 2. 공정중 검사 (batch_lot 기반)
    for batch_lot_no in batch_lot_nos[:200]:  # 최대 200건
        if random.random() < 0.4:  # 40% 샘플링
            cursor.execute("SELECT batch_date FROM batch_lot WHERE batch_lot_no = ?", (batch_lot_no,))
            row = cursor.fetchone()
            if not row:
                continue
            batch_date = row[0]

            qc_count += 1
            qc_no = f"QC-IP-{batch_date.replace('-', '')}-{qc_count:04d}"

            test_items = []
            for item_name, unit, min_val, max_val, target in QC_TEST_ITEMS['공정중']['items']:
                if min_val is None:  # 온도는 공정별로 다름
                    actual = random.uniform(4, 75)
                    result = 'PASS'
                else:
                    actual = random.gauss(target, (max_val - min_val) / 6)
                    actual = max(min_val * 0.9, min(max_val * 1.1, actual))
                    result = 'PASS' if min_val <= actual <= max_val else 'FAIL'
                test_items.append({
                    'item': item_name, 'unit': unit,
                    'min': min_val, 'max': max_val, 'target': target,
                    'actual': round(actual, 2), 'result': result
                })

            overall_result = 'PASS' if all(t['result'] == 'PASS' for t in test_items) else 'FAIL'

            # item_cd는 NOT NULL이므로 연관 제품 선택
            # batch_lot → production_order → bom_mst → fg_item_cd
            cursor.execute("""
                SELECT b.fg_item_cd FROM batch_lot bl
                JOIN production_order po ON bl.prod_order_no = po.prod_order_no
                JOIN bom_mst b ON po.bom_cd = b.bom_cd
                WHERE bl.batch_lot_no = ?
                LIMIT 1
            """, (batch_lot_no,))
            item_row = cursor.fetchone()
            item_cd = item_row[0] if item_row else random.choice(ITEMS_FG)

            qc_tests.append((
                qc_no, 'IN_PROCESS', 'BATCH', batch_lot_no,
                item_cd, batch_lot_no,
                batch_date, random.choice(OPERATORS),
                overall_result, json.dumps(test_items, ensure_ascii=False),
                None, get_datetime_str(datetime.strptime(batch_date, '%Y-%m-%d'))
            ))

    # 3. 완제품 검사 (fg_lot 기반)
    for fg_lot_no, filling_lot_no, fg_item_cd, qty, mfg_date, exp_date, qc_status, _, _ , _ in fg_lots[:300]:
        qc_count += 1
        qc_no = f"QC-FG-{mfg_date.replace('-', '')}-{qc_count:04d}"

        test_items = []
        for item_name, unit, min_val, max_val, target in QC_TEST_ITEMS['완제품']['items']:
            actual = random.gauss(target, (max_val - min_val) / 6) if max_val != min_val else target
            actual = max(min_val * 0.9, min(max_val * 1.1, actual))
            if item_name in ['관능검사', '유통기한확인']:
                actual = int(round(actual))
            result = 'PASS' if min_val <= actual <= max_val else 'FAIL'
            test_items.append({
                'item': item_name, 'unit': unit,
                'min': min_val, 'max': max_val, 'target': target,
                'actual': round(actual, 2), 'result': result
            })

        # DB result 형식으로 변환 (PASS/FAIL/CONDITIONAL)
        # qc_status: PASSED/FAILED/HOLD → qc_test result: PASS/FAIL/CONDITIONAL
        if qc_status == 'HOLD':
            overall_result = 'CONDITIONAL'
        elif qc_status == 'PASSED':
            overall_result = 'PASS'
        else:  # FAILED
            overall_result = 'FAIL'

        qc_tests.append((
            qc_no, 'FINAL', 'FG', fg_lot_no,
            fg_item_cd, fg_lot_no,
            mfg_date, random.choice(OPERATORS),
            overall_result, json.dumps(test_items, ensure_ascii=False),
            None, get_datetime_str(datetime.strptime(mfg_date, '%Y-%m-%d'))
        ))

    cursor.executemany("""
        INSERT OR IGNORE INTO qc_test
        (qc_no, test_type, ref_type, ref_no, item_cd, lot_no,
         test_date, tester_id, result, test_items, remark, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    """, qc_tests)

    print(f"QC검사 생성: {len(qc_tests)}건")


def generate_downtime_events(conn, start_date, end_date):
    """다운타임 이벤트 생성"""
    cursor = conn.cursor()

    # 기존 데이터 삭제
    cursor.execute("DELETE FROM downtime_event WHERE id > 0")

    downtime_events = []
    current = start_date

    all_equips = ['EQ-' + str(i).zfill(3) for i in range(1, 44)]

    while current <= end_date:
        # 주말은 이벤트 적음
        if current.weekday() < 5:
            daily_count = random.randint(2, 5)
        else:
            daily_count = random.randint(0, 2)

        for _ in range(daily_count):
            # 계획/비계획 선택
            is_planned = random.random() < 0.6  # 60% 계획정지

            if is_planned:
                reason_cd, reason_detail = random.choice(DOWNTIME_REASONS['계획정지'])
                duration = random.randint(30, 120)  # 30분~2시간
            else:
                reason_cd, reason_detail = random.choice(DOWNTIME_REASONS['비계획정지'])
                duration = random.randint(10, 90)  # 10분~1.5시간

            # 시간 설정
            start_hour = random.randint(6, 22)
            start_time = current.replace(hour=start_hour, minute=random.randint(0, 59))
            end_time = start_time + timedelta(minutes=duration)

            # 설비/라인 선택
            equip_cd = random.choice(all_equips)
            line_cd = random.choice(LINES)

            # 작업지시 연결 (있으면)
            cursor.execute("""
                SELECT wo_no FROM mes_work_order
                WHERE plan_date = ? AND line_cd = ?
                LIMIT 1
            """, (get_date_str(current), line_cd))
            wo_row = cursor.fetchone()
            wo_no = wo_row[0] if wo_row else None

            downtime_events.append((
                wo_no, equip_cd, line_cd,
                get_datetime_str(start_time), get_datetime_str(end_time),
                duration, reason_cd, reason_detail,
                1 if is_planned else 0,
                random.choice(OPERATORS),
                get_datetime_str(current)
            ))

        current += timedelta(days=1)

    cursor.executemany("""
        INSERT INTO downtime_event
        (wo_no, equip_cd, line_cd, start_time, end_time, duration_min,
         reason_cd, reason_detail, is_planned, reported_by, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    """, downtime_events)

    print(f"다운타임이벤트 생성: {len(downtime_events)}건")


def generate_alarm_events(conn, start_date, end_date):
    """알람 이벤트 생성"""
    cursor = conn.cursor()

    # 기존 데이터 삭제
    cursor.execute("DELETE FROM alarm_event WHERE id > 0")

    alarm_events = []
    current = start_date

    all_equips = ['EQ-' + str(i).zfill(3) for i in range(1, 44)]

    while current <= end_date:
        # 하루 1-3건 알람
        daily_count = random.randint(1, 3)

        for _ in range(daily_count):
            # 알람 유형 선택 (DB alarm_type 값을 직접 키로 사용)
            alarm_type = random.choice(list(ALARM_TYPES.keys()))
            alarm_code, alarm_message, param_cd = random.choice(ALARM_TYPES[alarm_type])

            # 알람 레벨
            alarm_level = random.choices(['INFO', 'WARNING', 'CRITICAL'], weights=[50, 35, 15])[0]

            # 설비 선택
            equip_cd = random.choice(all_equips)

            # 값/임계값
            threshold = random.uniform(50, 150)
            if alarm_level == 'CRITICAL':
                value = threshold * random.uniform(1.1, 1.3)
            elif alarm_level == 'WARNING':
                value = threshold * random.uniform(1.02, 1.1)
            else:
                value = threshold * random.uniform(0.95, 1.02)

            # 알람 시간
            alarm_time = current.replace(
                hour=random.randint(0, 23),
                minute=random.randint(0, 59)
            )

            # 배치 연결 (있으면)
            cursor.execute("""
                SELECT batch_lot_no FROM batch_lot
                WHERE batch_date = ?
                LIMIT 1
            """, (get_date_str(current),))
            batch_row = cursor.fetchone()
            batch_lot_no = batch_row[0] if batch_row else None

            # 확인/해결 여부
            is_acknowledged = random.random() < 0.9  # 90% 확인
            is_resolved = random.random() < 0.85 if is_acknowledged else False  # 85% 해결

            ack_time = alarm_time + timedelta(minutes=random.randint(5, 30)) if is_acknowledged else None
            resolve_time = ack_time + timedelta(minutes=random.randint(10, 60)) if is_resolved else None

            alarm_events.append((
                equip_cd, param_cd, batch_lot_no,
                get_datetime_str(alarm_time), alarm_level, alarm_type,
                alarm_message, round(value, 2), round(threshold, 2),
                1 if is_acknowledged else 0,
                random.choice(OPERATORS) if is_acknowledged else None,
                get_datetime_str(ack_time) if ack_time else None,
                1 if is_resolved else 0,
                random.choice(OPERATORS) if is_resolved else None,
                get_datetime_str(resolve_time) if resolve_time else None,
                f"{alarm_message} 조치 완료" if is_resolved else None
            ))

        current += timedelta(days=1)

    cursor.executemany("""
        INSERT INTO alarm_event
        (equip_cd, param_cd, batch_lot_no, alarm_time, alarm_level, alarm_type,
         message, value, threshold, is_acknowledged, acknowledged_by, acknowledged_at,
         is_resolved, resolved_by, resolved_at, resolution)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    """, alarm_events)

    print(f"알람이벤트 생성: {len(alarm_events)}건")


def main():
    print("=" * 60)
    print("MES/ERP 대량 데이터 생성 시작")
    print("=" * 60)

    conn = sqlite3.connect(DB_PATH)

    try:
        # 기준 날짜
        today = datetime.now()

        # MES 데이터: 4개월 전부터
        mes_start = today - timedelta(days=120)  # 약 4개월
        mes_end = today + timedelta(days=14)  # 2주 뒤까지

        # ERP 데이터: 1년 전부터
        erp_start = today - timedelta(days=365)  # 1년
        erp_end = today + timedelta(days=30)  # 1개월 뒤까지

        print(f"\nMES 기간: {mes_start.strftime('%Y-%m-%d')} ~ {mes_end.strftime('%Y-%m-%d')}")
        print(f"ERP 기간: {erp_start.strftime('%Y-%m-%d')} ~ {erp_end.strftime('%Y-%m-%d')}")
        print()

        # 1. 생산지시 생성 (production_order)
        prod_order_nos = generate_production_orders(conn, mes_start, mes_end)

        # 2. 작업지시 생성 (mes_work_order)
        generate_work_orders(conn, prod_order_nos, mes_start, mes_end)

        # 3. 구매발주 생성 (purchase_order)
        generate_purchase_orders(conn, erp_start, erp_end)

        # 4. 판매주문 생성 (sales_order)
        generate_sales_orders(conn, erp_start, erp_end)

        # 5. 배치 LOT 생성 (batch_lot) - 생산지시당 1-3개 배치
        batch_lot_nos = generate_batch_lots(conn, mes_start, mes_end)

        # 6. 충진 LOT 생성 (filling_lot) - 배치당 1-4개 충진
        filling_lot_nos = generate_filling_lots(conn, batch_lot_nos)

        # 7. 완제품 LOT 생성 (fg_lot) - 충진당 1개 완제품
        fg_lots = generate_fg_lots(conn, filling_lot_nos)

        # 8. 공정 결과 생성 (process_result) - 배치당 공정별 결과
        generate_process_results(conn, batch_lot_nos)

        # 9. QC 검사 생성 (qc_test) - LOT별 검사
        generate_qc_tests(conn, batch_lot_nos, filling_lot_nos, fg_lots)

        # 10. 다운타임 이벤트 생성 (downtime_event) - 설비 정지 기록
        generate_downtime_events(conn, mes_start, mes_end)

        # 11. 알람 이벤트 생성 (alarm_event) - 설비 알람 기록
        generate_alarm_events(conn, mes_start, mes_end)

        conn.commit()

        print("\n" + "=" * 60)
        print("데이터 생성 완료!")
        print("=" * 60)

        # 최종 카운트
        cursor = conn.cursor()
        print("\n[최종 데이터 현황]")
        for table in ['production_order', 'mes_work_order', 'purchase_order', 'purchase_order_dtl',
                      'sales_order', 'sales_order_dtl', 'batch_lot', 'filling_lot', 'fg_lot',
                      'process_result', 'qc_test', 'downtime_event', 'alarm_event']:
            cursor.execute(f"SELECT COUNT(*) FROM {table}")
            count = cursor.fetchone()[0]
            print(f"  {table}: {count}건")

    except Exception as e:
        print(f"오류 발생: {e}")
        conn.rollback()
        raise
    finally:
        conn.close()

if __name__ == "__main__":
    main()
