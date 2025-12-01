-- ========================================
-- 005_seed_erp_master.sql
-- 퓨어웰 음료㈜ ERP 마스터 데이터
-- 품목, 거래처, 고객, BOM
-- ========================================

-- ========================================
-- 1. 품목 마스터 (item_mst)
-- ========================================

-- 원료 (RM: Raw Material)
INSERT INTO item_mst (item_cd, item_nm, item_type, unit, spec, shelf_life_days, storage_cond) VALUES
('RM-001', '프로바이오틱스 분말', 'RM', 'KG', '유산균 100억CFU/g', 365, 'COLD'),
('RM-002', '완두콩 단백질', 'RM', 'KG', '단백질 80% 이상', 730, 'RT'),
('RM-003', '귀리 분말', 'RM', 'KG', '베타글루칸 4% 이상', 365, 'RT'),
('RM-004', '비타민 프리믹스', 'RM', 'KG', 'Vit.B,C,D 복합', 365, 'RT'),
('RM-005', '저분자 콜라겐', 'RM', 'KG', '분자량 3000 Da', 730, 'RT'),
('RM-006', '아연', 'RM', 'KG', '아연 15%', 730, 'RT'),
('RM-007', '정제수', 'RM', 'L', '음료용 정제수', 1, 'RT'),
('RM-008', '과당', 'RM', 'KG', 'Brix 75', 730, 'RT'),
('RM-009', '구연산', 'RM', 'KG', '무수구연산', 1095, 'RT'),
('RM-010', '딸기향', 'RM', 'KG', '천연딸기향', 365, 'COLD'),
('RM-011', '포도향', 'RM', 'KG', '천연포도향', 365, 'COLD'),
('RM-012', '사과향', 'RM', 'KG', '천연사과향', 365, 'COLD'),
('RM-013', '유청단백', 'RM', 'KG', 'WPC 80%', 730, 'COLD'),
('RM-014', '초유분말', 'RM', 'KG', 'IgG 20%', 365, 'COLD'),
('RM-015', '아스코르브산', 'RM', 'KG', 'Vit.C 99%', 730, 'RT');

-- 포장재 (PKG: Packaging Material)
INSERT INTO item_mst (item_cd, item_nm, item_type, unit, spec, shelf_life_days, storage_cond) VALUES
('PKG-001', 'PET병 500ml', 'PKG', 'EA', '투명 PET, 28mm 구경', NULL, 'RT'),
('PKG-002', 'PET병 350ml', 'PKG', 'EA', '투명 PET, 28mm 구경', NULL, 'RT'),
('PKG-003', 'PP캡 28mm', 'PKG', 'EA', '화이트, 안전밴드', NULL, 'RT'),
('PKG-004', '슈링크라벨 500ml', 'PKG', 'EA', 'OPS 필름', NULL, 'RT'),
('PKG-005', '슈링크라벨 350ml', 'PKG', 'EA', 'OPS 필름', NULL, 'RT'),
('PKG-006', '박스 24입', 'PKG', 'EA', '골판지 박스', NULL, 'RT'),
('PKG-007', '박스 12입', 'PKG', 'EA', '골판지 박스', NULL, 'RT');

-- 완제품 (FG: Finished Goods)
INSERT INTO item_mst (item_cd, item_nm, item_type, unit, spec, shelf_life_days, storage_cond) VALUES
('FG-001', '프로바이오 장건강 500ml', 'FG', 'EA', '유산균 10억CFU/병', 120, 'COLD'),
('FG-002', '프로바이오 장건강 350ml', 'FG', 'EA', '유산균 7억CFU/병', 120, 'COLD'),
('FG-003', '식물성 프로틴쉐이크 딸기 500ml', 'FG', 'EA', '단백질 20g/병', 180, 'RT'),
('FG-004', '식물성 프로틴쉐이크 초코 500ml', 'FG', 'EA', '단백질 20g/병', 180, 'RT'),
('FG-005', '비타민워터 레몬 500ml', 'FG', 'EA', 'Vit.C 100mg/병', 365, 'RT'),
('FG-006', '비타민워터 오렌지 500ml', 'FG', 'EA', 'Vit.C 100mg/병', 365, 'RT'),
('FG-007', '콜라겐 뷰티드링크 350ml', 'FG', 'EA', '콜라겐 5000mg/병', 180, 'RT'),
('FG-008', '키즈 면역음료 200ml', 'FG', 'EA', '아연+초유', 180, 'COLD');

-- ========================================
-- 2. 거래처 마스터 (vendor_mst)
-- ========================================

INSERT INTO vendor_mst (vendor_cd, vendor_nm, vendor_type, contact_nm, phone, email, address, business_no) VALUES
('VD-001', '㈜바이오팜', 'SUPPLIER', '김영수', '031-123-4567', 'kim@biopharm.co.kr', '경기도 화성시 동탄산단로 123', '123-45-67890'),
('VD-002', '대한식품원료㈜', 'SUPPLIER', '이정민', '032-234-5678', 'lee@daehan.co.kr', '인천시 서구 청라대로 456', '234-56-78901'),
('VD-003', '그린팩㈜', 'SUPPLIER', '박미영', '031-345-6789', 'park@greenpack.co.kr', '경기도 평택시 산단로 789', '345-67-89012'),
('VD-004', '프레시케미칼', 'SUPPLIER', '최준혁', '02-456-7890', 'choi@freshchem.co.kr', '서울시 금천구 가산디지털로 101', '456-78-90123'),
('VD-005', '㈜향기나라', 'SUPPLIER', '정소연', '031-567-8901', 'jung@hyang.co.kr', '경기도 안산시 시화산단로 202', '567-89-01234'),
('VD-006', '코리아패키지㈜', 'SUPPLIER', '한동우', '031-678-9012', 'han@koreapack.co.kr', '경기도 이천시 대월면 산단로 303', '678-90-12345');

-- ========================================
-- 3. 고객 마스터 (customer_mst)
-- ========================================

INSERT INTO customer_mst (cust_cd, cust_nm, cust_type, contact_nm, phone, email, address, credit_limit) VALUES
('CT-001', '㈜이마트', 'RETAIL', '김상무', '02-111-2222', 'emart@emart.co.kr', '서울시 성동구 성수동', 500000000),
('CT-002', '㈜롯데마트', 'RETAIL', '박차장', '02-222-3333', 'lotte@lotte.co.kr', '서울시 중구 명동', 400000000),
('CT-003', '㈜CU편의점', 'RETAIL', '이대리', '02-333-4444', 'cu@bgfretail.co.kr', '서울시 강남구 삼성동', 200000000),
('CT-004', '한국건강식품㈜', 'WHOLESALE', '최부장', '031-444-5555', 'korea@health.co.kr', '경기도 성남시 분당구', 300000000),
('CT-005', '㈜헬스24', 'ONLINE', '정팀장', '02-555-6666', 'health24@health24.co.kr', '서울시 송파구 잠실동', 150000000),
('CT-006', 'Healthy Japan Co.', 'EXPORT', 'Tanaka', '+81-3-1234-5678', 'tanaka@healthyjapan.jp', 'Tokyo, Japan', 100000000),
('CT-007', '㈜GS25', 'RETAIL', '윤과장', '02-666-7777', 'gs25@gsretail.co.kr', '서울시 강남구 역삼동', 250000000),
('CT-008', '쿠팡㈜', 'ONLINE', '강매니저', '02-777-8888', 'coupang@coupang.com', '서울시 송파구 신천동', 350000000);

-- ========================================
-- 4. BOM 마스터 (bom_mst + bom_dtl)
-- ========================================

-- BOM 헤더
INSERT INTO bom_mst (bom_cd, fg_item_cd, bom_nm, batch_size, batch_unit, version) VALUES
('BOM-001', 'FG-001', '프로바이오 장건강 500ml BOM', 1000, 'L', 1),
('BOM-002', 'FG-002', '프로바이오 장건강 350ml BOM', 1000, 'L', 1),
('BOM-003', 'FG-003', '식물성 프로틴쉐이크 딸기 500ml BOM', 1000, 'L', 1),
('BOM-004', 'FG-004', '식물성 프로틴쉐이크 초코 500ml BOM', 1000, 'L', 1),
('BOM-005', 'FG-005', '비타민워터 레몬 500ml BOM', 1000, 'L', 1),
('BOM-006', 'FG-006', '비타민워터 오렌지 500ml BOM', 1000, 'L', 1),
('BOM-007', 'FG-007', '콜라겐 뷰티드링크 350ml BOM', 1000, 'L', 1),
('BOM-008', 'FG-008', '키즈 면역음료 200ml BOM', 1000, 'L', 1);

-- BOM 상세 - 프로바이오 장건강 500ml
INSERT INTO bom_dtl (bom_cd, seq, item_cd, qty, unit, loss_rate, remark) VALUES
('BOM-001', 1, 'RM-007', 950, 'L', 2.0, '정제수'),
('BOM-001', 2, 'RM-001', 0.5, 'KG', 1.0, '프로바이오틱스'),
('BOM-001', 3, 'RM-008', 30, 'KG', 0.5, '과당'),
('BOM-001', 4, 'RM-009', 2, 'KG', 0.5, '구연산'),
('BOM-001', 5, 'RM-010', 0.3, 'KG', 1.0, '딸기향'),
('BOM-001', 6, 'PKG-001', 2000, 'EA', 3.0, 'PET병 500ml'),
('BOM-001', 7, 'PKG-003', 2000, 'EA', 2.0, 'PP캡'),
('BOM-001', 8, 'PKG-004', 2000, 'EA', 3.0, '슈링크라벨');

-- BOM 상세 - 프로바이오 장건강 350ml
INSERT INTO bom_dtl (bom_cd, seq, item_cd, qty, unit, loss_rate, remark) VALUES
('BOM-002', 1, 'RM-007', 950, 'L', 2.0, '정제수'),
('BOM-002', 2, 'RM-001', 0.5, 'KG', 1.0, '프로바이오틱스'),
('BOM-002', 3, 'RM-008', 30, 'KG', 0.5, '과당'),
('BOM-002', 4, 'RM-009', 2, 'KG', 0.5, '구연산'),
('BOM-002', 5, 'RM-011', 0.3, 'KG', 1.0, '포도향'),
('BOM-002', 6, 'PKG-002', 2857, 'EA', 3.0, 'PET병 350ml'),
('BOM-002', 7, 'PKG-003', 2857, 'EA', 2.0, 'PP캡'),
('BOM-002', 8, 'PKG-005', 2857, 'EA', 3.0, '슈링크라벨');

-- BOM 상세 - 식물성 프로틴쉐이크 딸기
INSERT INTO bom_dtl (bom_cd, seq, item_cd, qty, unit, loss_rate, remark) VALUES
('BOM-003', 1, 'RM-007', 900, 'L', 2.0, '정제수'),
('BOM-003', 2, 'RM-002', 40, 'KG', 1.0, '완두콩 단백질'),
('BOM-003', 3, 'RM-003', 20, 'KG', 1.0, '귀리 분말'),
('BOM-003', 4, 'RM-008', 25, 'KG', 0.5, '과당'),
('BOM-003', 5, 'RM-010', 0.5, 'KG', 1.0, '딸기향'),
('BOM-003', 6, 'PKG-001', 2000, 'EA', 3.0, 'PET병 500ml'),
('BOM-003', 7, 'PKG-003', 2000, 'EA', 2.0, 'PP캡'),
('BOM-003', 8, 'PKG-004', 2000, 'EA', 3.0, '슈링크라벨');

-- BOM 상세 - 식물성 프로틴쉐이크 초코
INSERT INTO bom_dtl (bom_cd, seq, item_cd, qty, unit, loss_rate, remark) VALUES
('BOM-004', 1, 'RM-007', 900, 'L', 2.0, '정제수'),
('BOM-004', 2, 'RM-002', 40, 'KG', 1.0, '완두콩 단백질'),
('BOM-004', 3, 'RM-003', 20, 'KG', 1.0, '귀리 분말'),
('BOM-004', 4, 'RM-008', 25, 'KG', 0.5, '과당'),
('BOM-004', 5, 'RM-012', 0.5, 'KG', 1.0, '사과향'),
('BOM-004', 6, 'PKG-001', 2000, 'EA', 3.0, 'PET병 500ml'),
('BOM-004', 7, 'PKG-003', 2000, 'EA', 2.0, 'PP캡'),
('BOM-004', 8, 'PKG-004', 2000, 'EA', 3.0, '슈링크라벨');

-- BOM 상세 - 비타민워터 레몬
INSERT INTO bom_dtl (bom_cd, seq, item_cd, qty, unit, loss_rate, remark) VALUES
('BOM-005', 1, 'RM-007', 980, 'L', 2.0, '정제수'),
('BOM-005', 2, 'RM-004', 0.2, 'KG', 1.0, '비타민 프리믹스'),
('BOM-005', 3, 'RM-015', 0.1, 'KG', 1.0, '아스코르브산'),
('BOM-005', 4, 'RM-008', 15, 'KG', 0.5, '과당'),
('BOM-005', 5, 'RM-009', 1, 'KG', 0.5, '구연산'),
('BOM-005', 6, 'PKG-001', 2000, 'EA', 3.0, 'PET병 500ml'),
('BOM-005', 7, 'PKG-003', 2000, 'EA', 2.0, 'PP캡'),
('BOM-005', 8, 'PKG-004', 2000, 'EA', 3.0, '슈링크라벨');

-- BOM 상세 - 비타민워터 오렌지
INSERT INTO bom_dtl (bom_cd, seq, item_cd, qty, unit, loss_rate, remark) VALUES
('BOM-006', 1, 'RM-007', 980, 'L', 2.0, '정제수'),
('BOM-006', 2, 'RM-004', 0.2, 'KG', 1.0, '비타민 프리믹스'),
('BOM-006', 3, 'RM-015', 0.1, 'KG', 1.0, '아스코르브산'),
('BOM-006', 4, 'RM-008', 15, 'KG', 0.5, '과당'),
('BOM-006', 5, 'RM-009', 1, 'KG', 0.5, '구연산'),
('BOM-006', 6, 'PKG-001', 2000, 'EA', 3.0, 'PET병 500ml'),
('BOM-006', 7, 'PKG-003', 2000, 'EA', 2.0, 'PP캡'),
('BOM-006', 8, 'PKG-004', 2000, 'EA', 3.0, '슈링크라벨');

-- BOM 상세 - 콜라겐 뷰티드링크
INSERT INTO bom_dtl (bom_cd, seq, item_cd, qty, unit, loss_rate, remark) VALUES
('BOM-007', 1, 'RM-007', 960, 'L', 2.0, '정제수'),
('BOM-007', 2, 'RM-005', 15, 'KG', 1.0, '저분자 콜라겐'),
('BOM-007', 3, 'RM-015', 0.2, 'KG', 1.0, '아스코르브산'),
('BOM-007', 4, 'RM-008', 20, 'KG', 0.5, '과당'),
('BOM-007', 5, 'RM-011', 0.3, 'KG', 1.0, '포도향'),
('BOM-007', 6, 'PKG-002', 2857, 'EA', 3.0, 'PET병 350ml'),
('BOM-007', 7, 'PKG-003', 2857, 'EA', 2.0, 'PP캡'),
('BOM-007', 8, 'PKG-005', 2857, 'EA', 3.0, '슈링크라벨');

-- BOM 상세 - 키즈 면역음료
INSERT INTO bom_dtl (bom_cd, seq, item_cd, qty, unit, loss_rate, remark) VALUES
('BOM-008', 1, 'RM-007', 940, 'L', 2.0, '정제수'),
('BOM-008', 2, 'RM-006', 0.1, 'KG', 1.0, '아연'),
('BOM-008', 3, 'RM-014', 5, 'KG', 1.0, '초유분말'),
('BOM-008', 4, 'RM-004', 0.3, 'KG', 1.0, '비타민 프리믹스'),
('BOM-008', 5, 'RM-008', 40, 'KG', 0.5, '과당'),
('BOM-008', 6, 'RM-012', 0.2, 'KG', 1.0, '사과향');

-- ========================================
-- 5. 교대 마스터 (shift_mst)
-- ========================================

INSERT INTO shift_mst (shift_cd, shift_nm, start_time, end_time) VALUES
('SHIFT-A', '주간 A조', '06:00', '14:00'),
('SHIFT-B', '주간 B조', '14:00', '22:00'),
('SHIFT-C', '야간 C조', '22:00', '06:00');
