#!/usr/bin/env python3
"""
마스터 데이터 업데이트 스크립트
- item_mst: 원료 및 완제품
- bom_mst: 유가공 제품 BOM
- vendor_mst: 공급업체
- customer_mst: 고객사
- equipment_mst: 설비
- line_mst: 생산라인
"""

import sqlite3
import os

DB_PATH = os.path.expandvars(r"%APPDATA%\Judgify\judgify.db")

def update_item_mst(conn):
    """품목 마스터 업데이트 - 유음료 원료 및 완제품 (음료만!)"""
    cursor = conn.cursor()

    # 기존 데이터 삭제
    cursor.execute("DELETE FROM item_mst")

    # item_mst 스키마:
    # item_cd, item_nm, item_type (RM/PKG/WIP/FG), unit, spec, shelf_life_days, storage_cond (RT/COLD/FROZEN), is_active
    item_data = [
        # 유제품 원료 (RM: Raw Material)
        ('RM-001', '원유 (1등급)', 'RM', 'kg', '1등급 원유', 3, 'COLD'),
        ('RM-002', '원유 (2등급)', 'RM', 'kg', '2등급 원유', 3, 'COLD'),
        ('RM-003', '탈지분유', 'RM', 'kg', '탈지분유 25kg', 365, 'RT'),
        ('RM-004', '전지분유', 'RM', 'kg', '전지분유 25kg', 365, 'RT'),
        ('RM-005', '농축유', 'RM', 'kg', '농축유', 7, 'COLD'),
        ('RM-006', '탈지농축유', 'RM', 'kg', '탈지농축유', 7, 'COLD'),
        ('RM-007', '유청분말 (WPC80)', 'RM', 'kg', 'WPC80', 365, 'RT'),
        ('RM-008', '유청분말 (WPI90)', 'RM', 'kg', 'WPI90', 365, 'RT'),

        # 발효/배양 (발효유용)
        ('RM-009', 'L.불가리쿠스 유산균', 'RM', 'unit', 'DVS 스타터', 365, 'FROZEN'),
        ('RM-010', 'S.써모필러스 유산균', 'RM', 'unit', 'DVS 스타터', 365, 'FROZEN'),
        ('RM-011', 'L.카제이 유산균', 'RM', 'unit', 'DVS 스타터', 365, 'FROZEN'),
        ('RM-012', '비피더스균 배양액', 'RM', 'L', 'BB-12', 30, 'FROZEN'),
        ('RM-013', 'R-1 유산균', 'RM', 'unit', 'R-1 스타터', 365, 'FROZEN'),
        ('RM-014', 'LGG 유산균', 'RM', 'unit', 'LGG 스타터', 365, 'FROZEN'),

        # 당류
        ('RM-015', '정제당', 'RM', 'kg', '백설탕 25kg', 730, 'RT'),
        ('RM-016', '과당', 'RM', 'kg', '결정과당', 365, 'RT'),
        ('RM-017', '올리고당', 'RM', 'kg', '프락토올리고당', 365, 'RT'),
        ('RM-018', '스테비아', 'RM', 'kg', '스테비올배당체', 730, 'RT'),
        ('RM-019', '알룰로스', 'RM', 'kg', 'D-알룰로스', 730, 'RT'),

        # 첨가물
        ('RM-020', '바닐라향', 'RM', 'kg', '천연바닐라향', 365, 'RT'),
        ('RM-021', '딸기향', 'RM', 'kg', '딸기향료', 365, 'RT'),
        ('RM-022', '초코향', 'RM', 'kg', '초콜릿향료', 365, 'RT'),
        ('RM-023', '바나나향', 'RM', 'kg', '바나나향료', 365, 'RT'),
        ('RM-024', '안정제 (펙틴)', 'RM', 'kg', '펙틴', 730, 'RT'),
        ('RM-025', '안정제 (카라기난)', 'RM', 'kg', '카라기난', 730, 'RT'),

        # 과일/기타 (음료 첨가용)
        ('RM-026', '딸기퓨레', 'RM', 'kg', '냉동딸기퓨레', 365, 'FROZEN'),
        ('RM-027', '블루베리퓨레', 'RM', 'kg', '냉동블루베리퓨레', 365, 'FROZEN'),
        ('RM-028', '코코아파우더', 'RM', 'kg', '네덜란드식 코코아', 730, 'RT'),
        ('RM-029', '커피원액', 'RM', 'L', '에스프레소 원액', 90, 'COLD'),

        # 두유/대체유 원료
        ('RM-030', '대두 (국산)', 'RM', 'kg', '국산 대두', 365, 'RT'),
        ('RM-031', '검은콩', 'RM', 'kg', '국산 검은콩', 365, 'RT'),
        ('RM-032', '오트', 'RM', 'kg', '귀리', 365, 'RT'),
        ('RM-033', '아몬드', 'RM', 'kg', '캘리포니아 아몬드', 365, 'RT'),
        ('RM-034', '코코넛밀크 원료', 'RM', 'L', '코코넛크림', 180, 'COLD'),

        # 완제품 (FG: Finished Goods) - 음료만!
        # 백색시유
        ('FG-001', '퓨어밀크 1L', 'FG', 'ea', '시유 1L 멸균팩', 14, 'COLD'),
        ('FG-002', '퓨어밀크 500ml', 'FG', 'ea', '시유 500ml 멸균팩', 14, 'COLD'),
        ('FG-003', '퓨어밀크 200ml', 'FG', 'ea', '시유 200ml', 14, 'COLD'),
        ('FG-004', '저지방 퓨어밀크 1L', 'FG', 'ea', '저지방 1L', 14, 'COLD'),
        ('FG-005', '저지방 퓨어밀크 500ml', 'FG', 'ea', '저지방 500ml', 14, 'COLD'),
        ('FG-006', '무지방 퓨어밀크 1L', 'FG', 'ea', '무지방 1L', 14, 'COLD'),
        ('FG-007', '유기농 목장우유 900ml', 'FG', 'ea', '유기농우유', 10, 'COLD'),
        ('FG-008', '프리미엄 목장우유 1L', 'FG', 'ea', '프리미엄 시유', 10, 'COLD'),
        ('FG-009', '고칼슘 우유 900ml', 'FG', 'ea', '칼슘강화', 14, 'COLD'),

        # 가공유 (맛우유)
        ('FG-010', '초코몽 200ml', 'FG', 'ea', '초코우유 200ml', 14, 'COLD'),
        ('FG-011', '초코몽 500ml', 'FG', 'ea', '초코우유 500ml', 14, 'COLD'),
        ('FG-012', '딸기몽 200ml', 'FG', 'ea', '딸기우유 200ml', 14, 'COLD'),
        ('FG-013', '바나나몽 200ml', 'FG', 'ea', '바나나우유 200ml', 14, 'COLD'),
        ('FG-014', '커피앤밀크 300ml', 'FG', 'ea', '커피우유 300ml', 14, 'COLD'),
        ('FG-015', '커피앤밀크 500ml', 'FG', 'ea', '커피우유 500ml', 14, 'COLD'),
        ('FG-016', '달콤바닐라 우유 200ml', 'FG', 'ea', '바닐라우유', 14, 'COLD'),
        ('FG-017', '흑임자 우유 200ml', 'FG', 'ea', '흑임자우유', 14, 'COLD'),

        # 발효유 (마시는 타입)
        ('FG-018', '프로바이오 드링크 150ml', 'FG', 'ea', '발효유 150ml', 21, 'COLD'),
        ('FG-019', '프로바이오 드링크 80ml', 'FG', 'ea', '발효유 80ml', 21, 'COLD'),
        ('FG-020', '장건강 발효유 65ml', 'FG', 'ea', '장건강 65ml', 21, 'COLD'),
        ('FG-021', '장건강 발효유 130ml', 'FG', 'ea', '장건강 130ml', 21, 'COLD'),
        ('FG-022', '쾌변 발효유 130ml', 'FG', 'ea', '쾌변 130ml', 21, 'COLD'),
        ('FG-023', '면역 플러스 발효유 100ml', 'FG', 'ea', '면역강화', 21, 'COLD'),
        ('FG-024', '마시는 요거트 딸기 200ml', 'FG', 'ea', '요거트 딸기', 21, 'COLD'),
        ('FG-025', '마시는 요거트 블루베리 200ml', 'FG', 'ea', '요거트 블루베리', 21, 'COLD'),
        ('FG-026', '마시는 요거트 플레인 200ml', 'FG', 'ea', '요거트 플레인', 21, 'COLD'),
        ('FG-027', '비피더스 음료 150ml', 'FG', 'ea', '비피더스', 21, 'COLD'),

        # UHT 멸균유
        ('FG-028', '멸균우유 1L', 'FG', 'ea', 'UHT 1L', 90, 'RT'),
        ('FG-029', '멸균우유 200ml', 'FG', 'ea', 'UHT 200ml', 90, 'RT'),
        ('FG-030', '멸균 저지방우유 1L', 'FG', 'ea', 'UHT 저지방', 90, 'RT'),
        ('FG-031', '멸균 초코우유 200ml', 'FG', 'ea', 'UHT 초코', 90, 'RT'),
        ('FG-032', '멸균 딸기우유 200ml', 'FG', 'ea', 'UHT 딸기', 90, 'RT'),

        # 두유/대체유
        ('FG-033', '검은콩 두유 190ml', 'FG', 'ea', '검은콩두유', 90, 'RT'),
        ('FG-034', '달콤한 두유 190ml', 'FG', 'ea', '가당두유', 90, 'RT'),
        ('FG-035', '무가당 두유 190ml', 'FG', 'ea', '무가당두유', 90, 'RT'),
        ('FG-036', '호두 아몬드 두유 190ml', 'FG', 'ea', '견과두유', 90, 'RT'),
        ('FG-037', '오트밀크 950ml', 'FG', 'ea', '귀리음료', 30, 'COLD'),
        ('FG-038', '아몬드밀크 950ml', 'FG', 'ea', '아몬드음료', 30, 'COLD'),
        ('FG-039', '귀리음료 200ml', 'FG', 'ea', '귀리음료 소용량', 30, 'COLD'),
        ('FG-040', '코코넛밀크 200ml', 'FG', 'ea', '코코넛음료', 30, 'COLD'),
    ]

    cursor.executemany("""
        INSERT INTO item_mst (item_cd, item_nm, item_type, unit, spec, shelf_life_days, storage_cond, is_active)
        VALUES (?, ?, ?, ?, ?, ?, ?, 1)
    """, item_data)

    print(f"품목 마스터 업데이트: {len(item_data)}건")

def update_bom_mst(conn):
    """BOM 마스터 업데이트 - 유음료 제품만!"""
    cursor = conn.cursor()

    # 기존 데이터 삭제
    cursor.execute("DELETE FROM bom_mst")

    # bom_mst 스키마:
    # bom_cd, fg_item_cd, bom_nm, batch_size, batch_unit, version, is_active
    bom_data = [
        # 백색시유
        ('BOM-001', 'FG-001', '퓨어밀크 1L BOM', 10000, 'L', 1),
        ('BOM-002', 'FG-002', '퓨어밀크 500ml BOM', 10000, 'L', 1),
        ('BOM-003', 'FG-003', '퓨어밀크 200ml BOM', 10000, 'L', 1),
        ('BOM-004', 'FG-004', '저지방 퓨어밀크 1L BOM', 10000, 'L', 1),
        ('BOM-005', 'FG-005', '저지방 퓨어밀크 500ml BOM', 10000, 'L', 1),
        ('BOM-006', 'FG-006', '무지방 퓨어밀크 1L BOM', 10000, 'L', 1),
        ('BOM-007', 'FG-007', '유기농 목장우유 900ml BOM', 9000, 'L', 1),
        ('BOM-008', 'FG-008', '프리미엄 목장우유 1L BOM', 10000, 'L', 1),
        ('BOM-009', 'FG-009', '고칼슘 우유 900ml BOM', 9000, 'L', 1),

        # 가공유 (맛우유)
        ('BOM-010', 'FG-010', '초코몽 200ml BOM', 5000, 'L', 1),
        ('BOM-011', 'FG-011', '초코몽 500ml BOM', 5000, 'L', 1),
        ('BOM-012', 'FG-012', '딸기몽 200ml BOM', 5000, 'L', 1),
        ('BOM-013', 'FG-013', '바나나몽 200ml BOM', 5000, 'L', 1),
        ('BOM-014', 'FG-014', '커피앤밀크 300ml BOM', 6000, 'L', 1),
        ('BOM-015', 'FG-015', '커피앤밀크 500ml BOM', 6000, 'L', 1),
        ('BOM-016', 'FG-016', '달콤바닐라 우유 200ml BOM', 5000, 'L', 1),
        ('BOM-017', 'FG-017', '흑임자 우유 200ml BOM', 5000, 'L', 1),

        # 발효유 (마시는 타입)
        ('BOM-018', 'FG-018', '프로바이오 드링크 150ml BOM', 3000, 'L', 1),
        ('BOM-019', 'FG-019', '프로바이오 드링크 80ml BOM', 3000, 'L', 1),
        ('BOM-020', 'FG-020', '장건강 발효유 65ml BOM', 2000, 'L', 1),
        ('BOM-021', 'FG-021', '장건강 발효유 130ml BOM', 2600, 'L', 1),
        ('BOM-022', 'FG-022', '쾌변 발효유 130ml BOM', 2600, 'L', 1),
        ('BOM-023', 'FG-023', '면역 플러스 발효유 100ml BOM', 2000, 'L', 1),
        ('BOM-024', 'FG-024', '마시는 요거트 딸기 200ml BOM', 4000, 'L', 1),
        ('BOM-025', 'FG-025', '마시는 요거트 블루베리 200ml BOM', 4000, 'L', 1),
        ('BOM-026', 'FG-026', '마시는 요거트 플레인 200ml BOM', 4000, 'L', 1),
        ('BOM-027', 'FG-027', '비피더스 음료 150ml BOM', 3000, 'L', 1),

        # UHT 멸균유
        ('BOM-028', 'FG-028', '멸균우유 1L BOM', 10000, 'L', 1),
        ('BOM-029', 'FG-029', '멸균우유 200ml BOM', 10000, 'L', 1),
        ('BOM-030', 'FG-030', '멸균 저지방우유 1L BOM', 10000, 'L', 1),
        ('BOM-031', 'FG-031', '멸균 초코우유 200ml BOM', 5000, 'L', 1),
        ('BOM-032', 'FG-032', '멸균 딸기우유 200ml BOM', 5000, 'L', 1),

        # 두유/대체유
        ('BOM-033', 'FG-033', '검은콩 두유 190ml BOM', 3800, 'L', 1),
        ('BOM-034', 'FG-034', '달콤한 두유 190ml BOM', 3800, 'L', 1),
        ('BOM-035', 'FG-035', '무가당 두유 190ml BOM', 3800, 'L', 1),
        ('BOM-036', 'FG-036', '호두 아몬드 두유 190ml BOM', 3800, 'L', 1),
        ('BOM-037', 'FG-037', '오트밀크 950ml BOM', 5700, 'L', 1),
        ('BOM-038', 'FG-038', '아몬드밀크 950ml BOM', 5700, 'L', 1),
        ('BOM-039', 'FG-039', '귀리음료 200ml BOM', 4000, 'L', 1),
        ('BOM-040', 'FG-040', '코코넛밀크 200ml BOM', 4000, 'L', 1),
    ]

    cursor.executemany("""
        INSERT INTO bom_mst (bom_cd, fg_item_cd, bom_nm, batch_size, batch_unit, version, is_active)
        VALUES (?, ?, ?, ?, ?, ?, 1)
    """, bom_data)

    print(f"BOM 마스터 업데이트: {len(bom_data)}건")

def update_vendor_mst(conn):
    """공급업체 마스터 업데이트"""
    cursor = conn.cursor()

    cursor.execute("DELETE FROM vendor_mst")

    # vendor_mst 스키마:
    # vendor_cd, vendor_nm, vendor_type (SUPPLIER/MANUFACTURER/BOTH), contact_nm, phone, email, address, business_no, is_active
    vendor_data = [
        ('VD-001', '한국낙농', 'SUPPLIER', '김대리', '031-123-4567', 'dairy@hankook.co.kr', '경기도 화성시', '123-45-67890'),
        ('VD-002', '제일유업', 'MANUFACTURER', '이과장', '02-234-5678', 'supply@jeil.co.kr', '서울시 강남구', '234-56-78901'),
        ('VD-003', '그린푸드', 'SUPPLIER', '박부장', '031-345-6789', 'fruit@greenfood.kr', '경기도 이천시', '345-67-89012'),
        ('VD-004', '코스모향료', 'SUPPLIER', '최대리', '02-456-7890', 'flavor@cosmo.co.kr', '서울시 금천구', '456-78-90123'),
        ('VD-005', '대한설탕', 'MANUFACTURER', '정차장', '051-567-8901', 'sugar@daehan.kr', '부산시 사하구', '567-89-01234'),
        ('VD-006', '바이오랩', 'MANUFACTURER', '한연구원', '031-678-9012', 'culture@biolab.co.kr', '경기도 성남시', '678-90-12345'),
        ('VD-007', '명산목장', 'SUPPLIER', '조목장', '033-789-0123', 'milk@myungsan.kr', '강원도 평창군', '789-01-23456'),
        ('VD-008', '글로벌데어리', 'BOTH', '윤팀장', '02-890-1234', 'import@globaldairy.kr', '인천시 연수구', '890-12-34567'),
    ]

    cursor.executemany("""
        INSERT INTO vendor_mst (vendor_cd, vendor_nm, vendor_type, contact_nm, phone, email, address, business_no, is_active)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, 1)
    """, vendor_data)

    print(f"공급업체 마스터 업데이트: {len(vendor_data)}건")

def update_customer_mst(conn):
    """고객사 마스터 업데이트"""
    cursor = conn.cursor()

    cursor.execute("DELETE FROM customer_mst")

    # customer_mst 스키마:
    # cust_cd, cust_nm, cust_type (RETAIL/WHOLESALE/ONLINE/EXPORT), contact_nm, phone, email, address, credit_limit, is_active
    customer_data = [
        ('CT-001', '이마트', 'RETAIL', '김MD', '02-111-2222', 'emart@emart.co.kr', '서울시 성동구', 500000000),
        ('CT-002', '롯데마트', 'RETAIL', '이바이어', '02-222-3333', 'lottemart@lotte.kr', '서울시 송파구', 500000000),
        ('CT-003', 'GS25 본사', 'WHOLESALE', '박담당', '02-333-4444', 'gs25@gsretail.kr', '서울시 강남구', 300000000),
        ('CT-004', 'CU 본사', 'WHOLESALE', '최담당', '02-444-5555', 'cu@bgfretail.kr', '서울시 강남구', 300000000),
        ('CT-005', '세븐일레븐', 'WHOLESALE', '정담당', '02-555-6666', 'seven@7eleven.kr', '서울시 중구', 200000000),
        ('CT-006', '스타벅스코리아', 'WHOLESALE', '한바이어', '02-666-7777', 'starbucks@starbucks.kr', '서울시 강남구', 100000000),
        ('CT-007', '배달의민족', 'ONLINE', '조MD', '02-777-8888', 'bmart@woowahan.kr', '서울시 송파구', 200000000),
        ('CT-008', '쿠팡', 'ONLINE', '윤MD', '02-888-9999', 'food@coupang.kr', '서울시 송파구', 400000000),
        ('CT-009', '코스트코코리아', 'RETAIL', '배바이어', '02-999-0000', 'costco@costco.kr', '서울시 양천구', 300000000),
        ('CT-010', '홈플러스', 'RETAIL', '신MD', '02-100-1111', 'homeplus@homeplus.kr', '서울시 강서구', 400000000),
    ]

    cursor.executemany("""
        INSERT INTO customer_mst (cust_cd, cust_nm, cust_type, contact_nm, phone, email, address, credit_limit, is_active)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, 1)
    """, customer_data)

    print(f"고객사 마스터 업데이트: {len(customer_data)}건")

def update_equipment_mst(conn):
    """설비 마스터 업데이트 - 유음료 공정만!"""
    cursor = conn.cursor()

    cursor.execute("DELETE FROM equipment_mst")

    # equipment_mst 스키마:
    # equip_cd, equip_nm, line_cd, equip_type (MIXER/TANK/PASTEURIZER/COOLER/FILLER/CAPPER/LABELER/PACKER/DETECTOR/CONVEYOR/PUMP/SENSOR),
    # model, manufacturer, install_date, is_ccp, ccp_type (PASTEURIZATION/METAL_DETECTION/COOLING), is_active
    equipment_data = [
        # 원유 수입/저장
        ('EQ-001', '원유저장탱크 T-101', 'LINE-MILK1', 'TANK', 'ST-50000', 'Alfa Laval', '2020-01-15', 0, None),
        ('EQ-002', '원유저장탱크 T-102', 'LINE-MILK1', 'TANK', 'ST-50000', 'Alfa Laval', '2020-01-15', 0, None),
        ('EQ-003', '원유저장탱크 T-103', 'LINE-MILK2', 'TANK', 'ST-50000', 'Alfa Laval', '2021-03-10', 0, None),
        ('EQ-004', '원유분리기 SP-01', 'LINE-MILK1', 'MIXER', 'SP-2000', 'Alfa Laval', '2020-01-18', 0, None),
        ('EQ-005', '청정기 CL-01', 'LINE-MILK1', 'MIXER', 'CL-1000', 'Alfa Laval', '2020-01-20', 0, None),
        ('EQ-006', '표준화탱크 ST-01', 'LINE-MILK1', 'TANK', 'ST-10000', 'Alfa Laval', '2020-01-22', 0, None),

        # 균질/살균 (CCP)
        ('EQ-007', '균질기 HG-01', 'LINE-MILK1', 'MIXER', 'HM-3000', 'GEA', '2020-03-20', 0, None),
        ('EQ-008', '균질기 HG-02', 'LINE-MILK2', 'MIXER', 'HM-3000', 'GEA', '2021-05-10', 0, None),
        ('EQ-009', 'HTST 살균기 PS-01', 'LINE-MILK1', 'PASTEURIZER', 'HTST-5000', 'Tetra Pak', '2020-01-25', 1, 'PASTEURIZATION'),
        ('EQ-010', 'HTST 살균기 PS-02', 'LINE-MILK2', 'PASTEURIZER', 'HTST-5000', 'Tetra Pak', '2021-03-15', 1, 'PASTEURIZATION'),
        ('EQ-011', 'UHT 멸균기 UHT-01', 'LINE-UHT1', 'PASTEURIZER', 'UHT-3000', 'Tetra Pak', '2020-06-01', 1, 'PASTEURIZATION'),
        ('EQ-012', 'UHT 멸균기 UHT-02', 'LINE-UHT2', 'PASTEURIZER', 'UHT-3000', 'Tetra Pak', '2022-01-10', 1, 'PASTEURIZATION'),

        # 발효 (발효유 전용)
        ('EQ-013', '발효탱크 FT-01', 'LINE-YOGURT1', 'TANK', 'FT-5000', 'Chr. Hansen', '2020-02-10', 0, None),
        ('EQ-014', '발효탱크 FT-02', 'LINE-YOGURT1', 'TANK', 'FT-5000', 'Chr. Hansen', '2020-02-10', 0, None),
        ('EQ-015', '발효탱크 FT-03', 'LINE-YOGURT2', 'TANK', 'FT-5000', 'Chr. Hansen', '2021-06-15', 0, None),
        ('EQ-016', '배양탱크 CT-01', 'LINE-YOGURT1', 'TANK', 'CT-1000', 'Chr. Hansen', '2020-02-15', 0, None),
        ('EQ-017', '접종장치 IN-01', 'LINE-YOGURT1', 'MIXER', 'IN-500', 'Chr. Hansen', '2020-02-18', 0, None),

        # 배합/혼합 (가공유/두유)
        ('EQ-018', '배합탱크 MX-01', 'LINE-FLAVOR1', 'TANK', 'MX-5000', 'GEA', '2020-04-01', 0, None),
        ('EQ-019', '배합탱크 MX-02', 'LINE-FLAVOR2', 'TANK', 'MX-5000', 'GEA', '2021-04-15', 0, None),
        ('EQ-020', '용해탱크 DS-01', 'LINE-SOY', 'TANK', 'DS-3000', 'GEA', '2020-05-01', 0, None),
        ('EQ-021', '교반기 AG-01', 'LINE-FLAVOR1', 'MIXER', 'AG-200', 'Silverson', '2020-04-05', 0, None),
        ('EQ-022', '인라인믹서 IM-01', 'LINE-FLAVOR1', 'MIXER', 'IM-100', 'Silverson', '2020-04-08', 0, None),

        # 냉각 (CCP)
        ('EQ-023', '냉각탱크 CL-01', 'LINE-MILK1', 'COOLER', 'CT-10000', 'Alfa Laval', '2020-01-28', 1, 'COOLING'),
        ('EQ-024', '냉각탱크 CL-02', 'LINE-MILK2', 'COOLER', 'CT-10000', 'Alfa Laval', '2021-04-01', 1, 'COOLING'),
        ('EQ-025', '판형냉각기 PH-01', 'LINE-YOGURT1', 'COOLER', 'PH-5000', 'Alfa Laval', '2020-02-20', 1, 'COOLING'),

        # 충진/포장
        ('EQ-026', '우유충진기 FL-01', 'LINE-MILK1', 'FILLER', 'TBA-19', 'Tetra Pak', '2020-02-01', 0, None),
        ('EQ-027', '우유충진기 FL-02', 'LINE-MILK2', 'FILLER', 'TBA-19', 'Tetra Pak', '2021-04-20', 0, None),
        ('EQ-028', '우유충진기 FL-03', 'LINE-LOWFAT', 'FILLER', 'TBA-19', 'Tetra Pak', '2021-06-01', 0, None),
        ('EQ-029', '페트충진기 PT-01', 'LINE-FLAVOR1', 'FILLER', 'PT-3000', 'Krones', '2020-05-01', 0, None),
        ('EQ-030', '페트충진기 PT-02', 'LINE-SOY', 'FILLER', 'PT-3000', 'Krones', '2020-06-01', 0, None),
        ('EQ-031', '파우치충진기 PF-01', 'LINE-YOGURT1', 'FILLER', 'PF-2000', 'Bosch', '2020-03-01', 0, None),
        ('EQ-032', '캡핑기 CP-01', 'LINE-MILK1', 'CAPPER', 'CP-200', 'Krones', '2020-02-05', 0, None),
        ('EQ-033', '캡핑기 CP-02', 'LINE-FLAVOR1', 'CAPPER', 'CP-200', 'Krones', '2020-05-05', 0, None),

        # 검사/품질 (CCP)
        ('EQ-034', '금속검출기 MD-01', 'LINE-MILK1', 'DETECTOR', 'MD-500', 'Mettler Toledo', '2020-02-10', 1, 'METAL_DETECTION'),
        ('EQ-035', '금속검출기 MD-02', 'LINE-YOGURT1', 'DETECTOR', 'MD-500', 'Mettler Toledo', '2020-03-05', 1, 'METAL_DETECTION'),
        ('EQ-036', 'X-Ray검사기 XR-01', 'LINE-PACK', 'DETECTOR', 'XR-3000', 'Eagle', '2021-01-15', 1, 'METAL_DETECTION'),
        ('EQ-037', '중량선별기 CW-01', 'LINE-PACK', 'DETECTOR', 'CW-100', 'Mettler Toledo', '2020-04-01', 0, None),
        ('EQ-038', '라벨검사기 LI-01', 'LINE-PACK', 'DETECTOR', 'LI-200', 'Cognex', '2020-04-05', 0, None),

        # 포장/물류
        ('EQ-039', '자동포장기 AP-01', 'LINE-PACK', 'PACKER', 'AP-500', 'Multivac', '2020-04-10', 0, None),
        ('EQ-040', '밴딩기 BD-01', 'LINE-PACK', 'PACKER', 'BD-100', 'Signode', '2020-04-12', 0, None),
        ('EQ-041', '파렛타이저 PL-01', 'LINE-PACK', 'PACKER', 'PL-1000', 'KUKA', '2020-04-15', 0, None),

        # 세정(CIP)
        ('EQ-042', 'CIP 유닛 CIP-01', 'LINE-MILK1', 'TANK', 'CIP-3000', 'Alfa Laval', '2020-01-30', 0, None),
        ('EQ-043', 'CIP 유닛 CIP-02', 'LINE-YOGURT1', 'TANK', 'CIP-3000', 'Alfa Laval', '2020-02-25', 0, None),
    ]

    cursor.executemany("""
        INSERT INTO equipment_mst (equip_cd, equip_nm, line_cd, equip_type, model, manufacturer, install_date, is_ccp, ccp_type, is_active)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 1)
    """, equipment_data)

    print(f"설비 마스터 업데이트: {len(equipment_data)}건")

def update_line_mst(conn):
    """생산라인 마스터 업데이트 - 유음료 공정만!"""
    cursor = conn.cursor()

    cursor.execute("DELETE FROM line_mst")

    # line_mst 스키마:
    # line_cd, line_nm, line_type (BATCHING/FILLING/PACKAGING), capacity_per_hour, is_active
    line_data = [
        # 백색시유 라인
        ('LINE-MILK1', '시유 1라인', 'FILLING', 10000),
        ('LINE-MILK2', '시유 2라인', 'FILLING', 10000),
        ('LINE-LOWFAT', '저지방 시유 라인', 'FILLING', 8000),

        # 발효유 라인
        ('LINE-YOGURT1', '발효유 A라인', 'BATCHING', 5000),
        ('LINE-YOGURT2', '발효유 B라인', 'BATCHING', 5000),
        ('LINE-YOGURT3', '발효유 C라인', 'BATCHING', 5000),

        # 가공유 라인
        ('LINE-FLAVOR1', '가공유 1라인', 'FILLING', 6000),
        ('LINE-FLAVOR2', '가공유 2라인', 'FILLING', 6000),

        # UHT 멸균유 라인
        ('LINE-UHT1', 'UHT 1라인', 'FILLING', 8000),
        ('LINE-UHT2', 'UHT 2라인', 'FILLING', 8000),

        # 두유/대체유 라인
        ('LINE-SOY', '두유 라인', 'BATCHING', 4000),
        ('LINE-ALT', '대체유 라인', 'BATCHING', 3000),

        # 포장라인
        ('LINE-PACK', '포장 라인', 'PACKAGING', 20000),
    ]

    cursor.executemany("""
        INSERT INTO line_mst (line_cd, line_nm, line_type, capacity_per_hour, is_active)
        VALUES (?, ?, ?, ?, 1)
    """, line_data)

    print(f"생산라인 마스터 업데이트: {len(line_data)}건")

def main():
    print("=" * 60)
    print("마스터 데이터 업데이트 시작")
    print("=" * 60)

    conn = sqlite3.connect(DB_PATH)

    try:
        # item_mst를 먼저 (bom_mst가 참조함)
        update_item_mst(conn)
        update_line_mst(conn)  # equipment_mst가 참조함
        update_bom_mst(conn)
        update_vendor_mst(conn)
        update_customer_mst(conn)
        update_equipment_mst(conn)

        conn.commit()

        print("\n" + "=" * 60)
        print("마스터 데이터 업데이트 완료!")
        print("=" * 60)

        # 최종 카운트
        cursor = conn.cursor()
        print("\n[최종 데이터 현황]")
        for table in ['item_mst', 'bom_mst', 'vendor_mst', 'customer_mst', 'line_mst', 'equipment_mst']:
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
