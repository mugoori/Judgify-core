-- ========================================
-- 007_seed_mes.sql
-- 퓨어웰 음료㈜ MES 마스터 + 실행 데이터
-- 라인/설비/공정/작업자 + 센서/CCP/알람 데이터
-- ========================================

-- ========================================
-- 1. MES 마스터 데이터
-- ========================================

-- 라인 마스터
INSERT INTO line_mst (line_cd, line_nm, line_type, capacity_per_hour) VALUES
('LINE-A', '배합/충진 A라인', 'FILLING', 5000),
('LINE-B', '배합/충진 B라인', 'FILLING', 5000),
('LINE-C', '포장 라인', 'PACKAGING', 8000);

-- 설비 마스터
INSERT INTO equipment_mst (equip_cd, equip_nm, line_cd, equip_type, model, manufacturer, install_date, is_ccp, ccp_type) VALUES
-- 배합/살균 설비
('EQ-MIX-01', '배합조 1호', 'LINE-A', 'MIXER', 'MX-2000', '한국기계', '2020-03-15', 0, NULL),
('EQ-MIX-02', '배합조 2호', 'LINE-A', 'MIXER', 'MX-2000', '한국기계', '2020-03-15', 0, NULL),
('EQ-MIX-03', '배합조 3호', 'LINE-B', 'MIXER', 'MX-2000', '한국기계', '2021-06-20', 0, NULL),
('EQ-TANK-01', '저장탱크 1호', 'LINE-A', 'TANK', 'TK-5000', '한국탱크', '2020-03-15', 0, NULL),
('EQ-TANK-02', '저장탱크 2호', 'LINE-A', 'TANK', 'TK-5000', '한국탱크', '2020-03-15', 0, NULL),
('EQ-TANK-03', '저장탱크 3호', 'LINE-B', 'TANK', 'TK-5000', '한국탱크', '2021-06-20', 0, NULL),
('EQ-PAST-01', '살균기 1호', 'LINE-A', 'PASTEURIZER', 'PS-3000', 'Tetra Pak', '2020-03-15', 1, 'PASTEURIZATION'),
('EQ-PAST-02', '살균기 2호', 'LINE-B', 'PASTEURIZER', 'PS-3000', 'Tetra Pak', '2021-06-20', 1, 'PASTEURIZATION'),
('EQ-COOL-01', '냉각기 1호', 'LINE-A', 'COOLER', 'CL-2000', 'Alfa Laval', '2020-03-15', 1, 'COOLING'),
('EQ-COOL-02', '냉각기 2호', 'LINE-B', 'COOLER', 'CL-2000', 'Alfa Laval', '2021-06-20', 1, 'COOLING'),
-- 충진/포장 설비
('EQ-FILL-01', '충진기 A-1', 'LINE-A', 'FILLER', 'FL-5000', 'Krones', '2020-03-15', 0, NULL),
('EQ-FILL-02', '충진기 B-1', 'LINE-B', 'FILLER', 'FL-5000', 'Krones', '2021-06-20', 0, NULL),
('EQ-CAP-01', '캡핑기 A-1', 'LINE-A', 'CAPPER', 'CP-3000', 'Krones', '2020-03-15', 0, NULL),
('EQ-CAP-02', '캡핑기 B-1', 'LINE-B', 'CAPPER', 'CP-3000', 'Krones', '2021-06-20', 0, NULL),
('EQ-LABEL-01', '라벨링기 A-1', 'LINE-A', 'LABELER', 'LB-2000', 'P.E. Labellers', '2020-03-15', 0, NULL),
('EQ-LABEL-02', '라벨링기 B-1', 'LINE-B', 'LABELER', 'LB-2000', 'P.E. Labellers', '2021-06-20', 0, NULL),
('EQ-MD-01', '금속검출기 A-1', 'LINE-A', 'DETECTOR', 'MD-500', 'Mettler Toledo', '2020-03-15', 1, 'METAL_DETECTION'),
('EQ-MD-02', '금속검출기 B-1', 'LINE-B', 'DETECTOR', 'MD-500', 'Mettler Toledo', '2021-06-20', 1, 'METAL_DETECTION'),
-- 포장 라인
('EQ-PACK-01', '박스 케이서', 'LINE-C', 'PACKER', 'PK-1000', '삼성포장기계', '2020-03-15', 0, NULL),
('EQ-PALLET-01', '팔레타이저', 'LINE-C', 'CONVEYOR', 'PL-500', '삼성포장기계', '2020-03-15', 0, NULL);

-- 공정 마스터
INSERT INTO operation_mst (oper_cd, oper_nm, oper_seq, line_cd, std_time_sec, is_ccp, ccp_params) VALUES
('OP-BATCH', '배합', 10, 'LINE-A', 3600, 0, NULL),
('OP-PAST', '살균', 20, 'LINE-A', 900, 1, '{"target_temp": 85, "min_time_sec": 15, "critical_limit_temp": 83}'),
('OP-COOL', '냉각', 30, 'LINE-A', 600, 1, '{"target_temp": 25, "max_time_sec": 1800, "critical_limit_temp": 30}'),
('OP-HOLD', '저장', 40, 'LINE-A', 7200, 0, NULL),
('OP-FILL', '충진', 50, 'LINE-A', 1800, 0, NULL),
('OP-CAP', '캡핑', 60, 'LINE-A', 300, 0, NULL),
('OP-LABEL', '라벨링', 70, 'LINE-A', 300, 0, NULL),
('OP-MD', '금속검출', 80, 'LINE-A', 60, 1, '{"fe_sensitivity": 1.5, "sus_sensitivity": 2.0}'),
('OP-PACK', '포장', 90, 'LINE-C', 600, 0, NULL);

-- 작업자 마스터
INSERT INTO operator_mst (operator_id, operator_nm, dept, position, shift_cd, certifications) VALUES
('OP001', '김생산', '생산1팀', '반장', 'SHIFT-A', '["HACCP관리자", "위생관리자"]'),
('OP002', '이공정', '생산1팀', '조장', 'SHIFT-A', '["품질관리사"]'),
('OP003', '박배합', '생산2팀', '반장', 'SHIFT-B', '["HACCP관리자"]'),
('OP004', '최충진', '생산2팀', '조장', 'SHIFT-B', '["식품기사"]'),
('OP005', '정품질', '품질관리팀', '과장', 'SHIFT-A', '["HACCP팀장", "식품기술사"]'),
('OP006', '한검사', '품질관리팀', '대리', 'SHIFT-A', '["품질관리사"]');

-- 사유코드 마스터
INSERT INTO reason_code_mst (reason_cd, reason_type, reason_nm, category) VALUES
('DT-001', 'DOWNTIME', '설비 고장', 'EQUIPMENT'),
('DT-002', 'DOWNTIME', '원자재 대기', 'MATERIAL'),
('DT-003', 'DOWNTIME', '품질 이상', 'QUALITY'),
('DT-004', 'DOWNTIME', '계획 정지', 'PLANNED'),
('DT-005', 'DOWNTIME', 'CIP/SIP 세척', 'MAINTENANCE'),
('DT-006', 'DOWNTIME', '교대 인수인계', 'SHIFT'),
('DF-001', 'DEFECT', '용량 미달', 'VOLUME'),
('DF-002', 'DEFECT', '라벨 불량', 'LABEL'),
('DF-003', 'DEFECT', '캡 불량', 'CAP'),
('DF-004', 'DEFECT', '이물 검출', 'CONTAMINATION'),
('AL-001', 'ALARM', '온도 이상', 'TEMPERATURE'),
('AL-002', 'ALARM', '압력 이상', 'PRESSURE'),
('AL-003', 'ALARM', 'CCP 이탈', 'CCP'),
('AL-004', 'ALARM', '설비 이상', 'EQUIPMENT');

-- 파라미터 마스터
INSERT INTO param_mst (param_cd, param_nm, param_type, unit, equip_cd, min_val, max_val, target_val, is_ccp, ccp_critical_limit_min, ccp_critical_limit_max, alarm_enabled) VALUES
-- 살균 파라미터 (CCP)
('PARAM-PAST-TEMP', '살균 온도', 'TEMPERATURE', '°C', 'EQ-PAST-01', 80, 95, 85, 1, 83, 95, 1),
('PARAM-PAST-TIME', '살균 시간', 'TIME', 'sec', 'EQ-PAST-01', 15, 30, 20, 1, 15, NULL, 1),
('PARAM-PAST-FLOW', '살균 유량', 'FLOW', 'L/min', 'EQ-PAST-01', 80, 120, 100, 0, NULL, NULL, 1),
-- 냉각 파라미터 (CCP)
('PARAM-COOL-TEMP', '냉각 온도', 'TEMPERATURE', '°C', 'EQ-COOL-01', 5, 30, 25, 1, NULL, 30, 1),
('PARAM-COOL-INLET', '냉각수 입구온도', 'TEMPERATURE', '°C', 'EQ-COOL-01', 3, 15, 8, 0, NULL, NULL, 1),
-- 배합 파라미터
('PARAM-MIX-TEMP', '배합 온도', 'TEMPERATURE', '°C', 'EQ-MIX-01', 20, 55, 45, 0, NULL, NULL, 1),
('PARAM-MIX-RPM', '교반 속도', 'SPEED', 'RPM', 'EQ-MIX-01', 50, 200, 120, 0, NULL, NULL, 1),
('PARAM-MIX-PH', '배합액 pH', 'PH', '', 'EQ-MIX-01', 3.0, 5.0, 4.0, 0, NULL, NULL, 1),
('PARAM-MIX-BRIX', '배합액 Brix', 'BRIX', '°Bx', 'EQ-MIX-01', 8, 15, 12, 0, NULL, NULL, 1),
-- 충진 파라미터
('PARAM-FILL-VOL', '충진량', 'WEIGHT', 'ml', 'EQ-FILL-01', 490, 510, 500, 0, NULL, NULL, 1),
('PARAM-FILL-SPEED', '충진 속도', 'SPEED', 'BPM', 'EQ-FILL-01', 50, 100, 80, 0, NULL, NULL, 1),
-- 금속검출 파라미터 (CCP)
('PARAM-MD-FE', 'Fe 감도', 'COUNT', 'mm', 'EQ-MD-01', 1.0, 2.0, 1.5, 1, NULL, 1.5, 1),
('PARAM-MD-SUS', 'SUS 감도', 'COUNT', 'mm', 'EQ-MD-01', 1.5, 2.5, 2.0, 1, NULL, 2.0, 1);

-- ========================================
-- 2. MES 작업지시 (mes_work_order)
-- ========================================

INSERT INTO mes_work_order (wo_no, prod_order_no, line_cd, shift_cd, plan_date, plan_start, plan_end, actual_start, actual_end, status, plan_qty, good_qty, reject_qty, operator_id) VALUES
('WO-240910-A01', 'PROD-2024-09-001', 'LINE-A', 'SHIFT-A', '2024-09-10', '2024-09-10 06:00:00', '2024-09-10 14:00:00', '2024-09-10 06:05:00', '2024-09-10 14:10:00', 'COMPLETED', 2000, 1980, 20, 'OP001'),
('WO-240912-A01', 'PROD-2024-09-002', 'LINE-A', 'SHIFT-A', '2024-09-12', '2024-09-12 06:00:00', '2024-09-12 14:00:00', '2024-09-12 06:10:00', '2024-09-12 14:05:00', 'COMPLETED', 1500, 1485, 15, 'OP002'),
('WO-240915-B01', 'PROD-2024-09-003', 'LINE-B', 'SHIFT-A', '2024-09-15', '2024-09-15 06:00:00', '2024-09-15 18:00:00', '2024-09-15 06:15:00', '2024-09-15 18:20:00', 'COMPLETED', 3000, 2970, 30, 'OP003'),
('WO-241008-A01', 'PROD-2024-10-001', 'LINE-A', 'SHIFT-A', '2024-10-08', '2024-10-08 06:00:00', '2024-10-08 16:00:00', '2024-10-08 06:08:00', '2024-10-08 16:15:00', 'COMPLETED', 2500, 2475, 25, 'OP001'),
('WO-241010-B01', 'PROD-2024-10-002', 'LINE-B', 'SHIFT-A', '2024-10-10', '2024-10-10 06:00:00', '2024-10-10 14:00:00', '2024-10-10 06:05:00', '2024-10-10 14:08:00', 'COMPLETED', 2000, 1960, 40, 'OP003'),
('WO-241015-A01', 'PROD-2024-10-003', 'LINE-A', 'SHIFT-A', '2024-10-15', '2024-10-15 06:00:00', '2024-10-15 13:00:00', '2024-10-15 06:12:00', '2024-10-15 13:10:00', 'COMPLETED', 1800, 1782, 18, 'OP001'),
('WO-241105-A01', 'PROD-2024-11-001', 'LINE-A', 'SHIFT-A', '2024-11-05', '2024-11-05 06:00:00', '2024-11-05 15:00:00', '2024-11-05 06:10:00', '2024-11-05 15:05:00', 'COMPLETED', 2000, 1980, 20, 'OP002'),
('WO-241108-B01', 'PROD-2024-11-002', 'LINE-B', 'SHIFT-A', '2024-11-08', '2024-11-08 06:00:00', '2024-11-08 16:00:00', '2024-11-08 06:05:00', '2024-11-08 16:12:00', 'COMPLETED', 2500, 2450, 50, 'OP003'),
('WO-241120-A01', 'PROD-2024-11-003', 'LINE-A', 'SHIFT-A', '2024-11-20', '2024-11-20 06:00:00', '2024-11-20 18:00:00', '2024-11-20 06:08:00', '2024-11-20 18:25:00', 'COMPLETED', 3000, 2850, 150, 'OP001');

-- ========================================
-- 3. 공정 실행 (operation_exec)
-- ========================================

INSERT INTO operation_exec (wo_no, oper_cd, batch_lot_no, equip_cd, start_time, end_time, status, result, operator_id) VALUES
-- 2024-09-10 작업
('WO-240910-A01', 'OP-BATCH', 'BATCH-240910-001', 'EQ-MIX-01', '2024-09-10 06:30:00', '2024-09-10 09:00:00', 'COMPLETED', 'OK', 'OP001'),
('WO-240910-A01', 'OP-PAST', 'BATCH-240910-001', 'EQ-PAST-01', '2024-09-10 09:05:00', '2024-09-10 09:25:00', 'COMPLETED', 'OK', 'OP001'),
('WO-240910-A01', 'OP-COOL', 'BATCH-240910-001', 'EQ-COOL-01', '2024-09-10 09:25:00', '2024-09-10 09:40:00', 'COMPLETED', 'OK', 'OP001'),
('WO-240910-A01', 'OP-FILL', 'BATCH-240910-001', 'EQ-FILL-01', '2024-09-10 10:00:00', '2024-09-10 12:00:00', 'COMPLETED', 'OK', 'OP002'),
('WO-240910-A01', 'OP-MD', 'BATCH-240910-001', 'EQ-MD-01', '2024-09-10 12:00:00', '2024-09-10 12:10:00', 'COMPLETED', 'OK', 'OP002'),
-- 2024-10-08 작업
('WO-241008-A01', 'OP-BATCH', 'BATCH-241008-001', 'EQ-MIX-01', '2024-10-08 06:30:00', '2024-10-08 09:00:00', 'COMPLETED', 'OK', 'OP001'),
('WO-241008-A01', 'OP-PAST', 'BATCH-241008-001', 'EQ-PAST-01', '2024-10-08 09:05:00', '2024-10-08 09:25:00', 'COMPLETED', 'OK', 'OP001'),
('WO-241008-A01', 'OP-COOL', 'BATCH-241008-001', 'EQ-COOL-01', '2024-10-08 09:25:00', '2024-10-08 09:40:00', 'COMPLETED', 'OK', 'OP001'),
('WO-241008-A01', 'OP-FILL', 'BATCH-241008-001', 'EQ-FILL-01', '2024-10-08 10:00:00', '2024-10-08 12:00:00', 'COMPLETED', 'OK', 'OP002'),
('WO-241008-A01', 'OP-MD', 'BATCH-241008-001', 'EQ-MD-01', '2024-10-08 12:00:00', '2024-10-08 12:10:00', 'COMPLETED', 'OK', 'OP002'),
-- 2024-11-20 작업 (일부 이상 발생)
('WO-241120-A01', 'OP-BATCH', 'BATCH-241120-001', 'EQ-MIX-01', '2024-11-20 06:30:00', '2024-11-20 09:00:00', 'COMPLETED', 'OK', 'OP001'),
('WO-241120-A01', 'OP-PAST', 'BATCH-241120-001', 'EQ-PAST-01', '2024-11-20 09:05:00', '2024-11-20 09:25:00', 'COMPLETED', 'DEVIATION', 'OP001'),
('WO-241120-A01', 'OP-COOL', 'BATCH-241120-001', 'EQ-COOL-01', '2024-11-20 09:30:00', '2024-11-20 09:50:00', 'COMPLETED', 'OK', 'OP001'),
('WO-241120-A01', 'OP-FILL', 'BATCH-241120-001', 'EQ-FILL-01', '2024-11-20 10:00:00', '2024-11-20 12:00:00', 'COMPLETED', 'OK', 'OP002'),
('WO-241120-A01', 'OP-MD', 'BATCH-241120-001', 'EQ-MD-01', '2024-11-20 12:00:00', '2024-11-20 12:15:00', 'COMPLETED', 'OK', 'OP002');

-- ========================================
-- 4. CCP 체크 로그 (ccp_check_log)
-- ========================================

-- 정상 CCP 기록
INSERT INTO ccp_check_log (batch_lot_no, ccp_type, check_time, equip_cd, operator_id, target_temp, actual_temp, target_time_sec, actual_time_sec, result, verified_by, verified_at) VALUES
-- 살균 CCP - 정상
('BATCH-240910-001', 'PASTEURIZATION', '2024-09-10 09:20:00', 'EQ-PAST-01', 'OP001', 85.0, 86.2, 20, 22, 'PASS', 'OP005', '2024-09-10 10:00:00'),
('BATCH-240910-002', 'PASTEURIZATION', '2024-09-10 13:50:00', 'EQ-PAST-01', 'OP001', 85.0, 85.8, 20, 21, 'PASS', 'OP005', '2024-09-10 14:30:00'),
('BATCH-240912-001', 'PASTEURIZATION', '2024-09-12 10:15:00', 'EQ-PAST-01', 'OP002', 85.0, 86.0, 20, 20, 'PASS', 'OP005', '2024-09-12 11:00:00'),
('BATCH-241008-001', 'PASTEURIZATION', '2024-10-08 09:20:00', 'EQ-PAST-01', 'OP001', 85.0, 85.5, 20, 21, 'PASS', 'OP005', '2024-10-08 10:00:00'),
('BATCH-241008-002', 'PASTEURIZATION', '2024-10-08 13:50:00', 'EQ-PAST-01', 'OP002', 85.0, 86.1, 20, 20, 'PASS', 'OP005', '2024-10-08 14:30:00'),
-- 살균 CCP - 이탈 (11월 20일)
('BATCH-241120-001', 'PASTEURIZATION', '2024-11-20 09:20:00', 'EQ-PAST-01', 'OP001', 85.0, 82.5, 20, 25, 'DEVIATION', 'OP005', '2024-11-20 10:00:00'),
-- 냉각 CCP
('BATCH-240910-001', 'COOLING', '2024-09-10 09:40:00', 'EQ-COOL-01', 'OP001', 25.0, 23.5, NULL, NULL, 'PASS', 'OP005', '2024-09-10 10:00:00'),
('BATCH-241008-001', 'COOLING', '2024-10-08 09:40:00', 'EQ-COOL-01', 'OP001', 25.0, 24.2, NULL, NULL, 'PASS', 'OP005', '2024-10-08 10:00:00'),
('BATCH-241120-001', 'COOLING', '2024-11-20 09:50:00', 'EQ-COOL-01', 'OP001', 25.0, 24.8, NULL, NULL, 'PASS', 'OP005', '2024-11-20 10:30:00');

-- 금속검출 CCP
INSERT INTO ccp_check_log (batch_lot_no, ccp_type, check_time, equip_cd, operator_id, sensitivity_fe, sensitivity_sus, test_piece_detected, reject_confirmed, result, verified_by, verified_at) VALUES
('BATCH-240910-001', 'METAL_DETECTION', '2024-09-10 10:00:00', 'EQ-MD-01', 'OP002', 1.5, 2.0, 1, 0, 'PASS', 'OP006', '2024-09-10 10:30:00'),
('BATCH-240910-001', 'METAL_DETECTION', '2024-09-10 12:00:00', 'EQ-MD-01', 'OP002', 1.5, 2.0, 1, 0, 'PASS', 'OP006', '2024-09-10 12:30:00'),
('BATCH-241008-001', 'METAL_DETECTION', '2024-10-08 10:00:00', 'EQ-MD-01', 'OP002', 1.5, 2.0, 1, 0, 'PASS', 'OP006', '2024-10-08 10:30:00'),
('BATCH-241008-001', 'METAL_DETECTION', '2024-10-08 12:00:00', 'EQ-MD-01', 'OP002', 1.5, 2.0, 1, 0, 'PASS', 'OP006', '2024-10-08 12:30:00'),
('BATCH-241120-001', 'METAL_DETECTION', '2024-11-20 10:00:00', 'EQ-MD-01', 'OP002', 1.5, 2.0, 1, 0, 'PASS', 'OP006', '2024-11-20 10:30:00'),
('BATCH-241120-001', 'METAL_DETECTION', '2024-11-20 12:00:00', 'EQ-MD-01', 'OP002', 1.5, 2.0, 1, 3, 'FAIL', 'OP006', '2024-11-20 12:30:00');

-- ========================================
-- 5. 센서 로그 (sensor_log) - 샘플 데이터
-- ========================================

-- 살균기 온도 센서 로그 (정상)
INSERT INTO sensor_log (equip_cd, param_cd, batch_lot_no, recorded_at, value, is_alarm, alarm_type) VALUES
('EQ-PAST-01', 'PARAM-PAST-TEMP', 'BATCH-240910-001', '2024-09-10 09:10:00', 84.5, 0, NULL),
('EQ-PAST-01', 'PARAM-PAST-TEMP', 'BATCH-240910-001', '2024-09-10 09:15:00', 85.2, 0, NULL),
('EQ-PAST-01', 'PARAM-PAST-TEMP', 'BATCH-240910-001', '2024-09-10 09:20:00', 86.1, 0, NULL),
('EQ-PAST-01', 'PARAM-PAST-TEMP', 'BATCH-241008-001', '2024-10-08 09:10:00', 84.8, 0, NULL),
('EQ-PAST-01', 'PARAM-PAST-TEMP', 'BATCH-241008-001', '2024-10-08 09:15:00', 85.3, 0, NULL),
('EQ-PAST-01', 'PARAM-PAST-TEMP', 'BATCH-241008-001', '2024-10-08 09:20:00', 85.7, 0, NULL),
-- 살균기 온도 센서 로그 (이상 - 11월 20일)
('EQ-PAST-01', 'PARAM-PAST-TEMP', 'BATCH-241120-001', '2024-11-20 09:10:00', 84.2, 0, NULL),
('EQ-PAST-01', 'PARAM-PAST-TEMP', 'BATCH-241120-001', '2024-11-20 09:15:00', 83.5, 0, NULL),
('EQ-PAST-01', 'PARAM-PAST-TEMP', 'BATCH-241120-001', '2024-11-20 09:18:00', 82.8, 1, 'LOW'),
('EQ-PAST-01', 'PARAM-PAST-TEMP', 'BATCH-241120-001', '2024-11-20 09:20:00', 82.5, 1, 'CRITICAL_LOW'),
('EQ-PAST-01', 'PARAM-PAST-TEMP', 'BATCH-241120-001', '2024-11-20 09:22:00', 83.2, 1, 'LOW'),
('EQ-PAST-01', 'PARAM-PAST-TEMP', 'BATCH-241120-001', '2024-11-20 09:25:00', 84.5, 0, NULL),
-- 냉각기 온도 센서 로그
('EQ-COOL-01', 'PARAM-COOL-TEMP', 'BATCH-240910-001', '2024-09-10 09:30:00', 45.2, 0, NULL),
('EQ-COOL-01', 'PARAM-COOL-TEMP', 'BATCH-240910-001', '2024-09-10 09:35:00', 32.5, 0, NULL),
('EQ-COOL-01', 'PARAM-COOL-TEMP', 'BATCH-240910-001', '2024-09-10 09:40:00', 23.8, 0, NULL),
('EQ-COOL-01', 'PARAM-COOL-TEMP', 'BATCH-241008-001', '2024-10-08 09:30:00', 44.8, 0, NULL),
('EQ-COOL-01', 'PARAM-COOL-TEMP', 'BATCH-241008-001', '2024-10-08 09:35:00', 31.2, 0, NULL),
('EQ-COOL-01', 'PARAM-COOL-TEMP', 'BATCH-241008-001', '2024-10-08 09:40:00', 24.5, 0, NULL);

-- ========================================
-- 6. 알람 이벤트 (alarm_event)
-- ========================================

INSERT INTO alarm_event (equip_cd, param_cd, batch_lot_no, alarm_time, alarm_level, alarm_type, message, value, threshold, is_acknowledged, acknowledged_by, acknowledged_at, is_resolved, resolved_by, resolved_at, resolution) VALUES
-- 2024-11-20 살균 온도 이탈 알람
('EQ-PAST-01', 'PARAM-PAST-TEMP', 'BATCH-241120-001', '2024-11-20 09:18:00', 'WARNING', 'PARAM_LOW', '살균 온도 저하 경고: 82.8°C (기준: 83°C 이상)', 82.8, 83.0, 1, 'OP001', '2024-11-20 09:19:00', 1, 'OP001', '2024-11-20 09:30:00', '스팀 밸브 조절로 온도 회복'),
('EQ-PAST-01', 'PARAM-PAST-TEMP', 'BATCH-241120-001', '2024-11-20 09:20:00', 'CRITICAL', 'CCP_DEVIATION', '★CCP 이탈★ 살균 온도: 82.5°C (한계기준: 83°C 이상)', 82.5, 83.0, 1, 'OP005', '2024-11-20 09:21:00', 1, 'OP005', '2024-11-20 09:35:00', '재순환 처리 후 재살균 완료'),
-- 2024-11-20 금속검출 알람
('EQ-MD-01', NULL, 'BATCH-241120-001', '2024-11-20 12:05:00', 'CRITICAL', 'CCP_DEVIATION', '★CCP 이탈★ 금속이물 검출! 제품 리젝트 처리', NULL, NULL, 1, 'OP002', '2024-11-20 12:06:00', 1, 'OP006', '2024-11-20 12:30:00', '리젝트 제품 3EA 격리 및 폐기 처리');

-- ========================================
-- 7. 비가동 이벤트 (downtime_event)
-- ========================================

INSERT INTO downtime_event (wo_no, equip_cd, line_cd, start_time, end_time, duration_min, reason_cd, reason_detail, is_planned, reported_by) VALUES
-- 계획 정지 (CIP)
('WO-240910-A01', 'EQ-PAST-01', 'LINE-A', '2024-09-10 05:00:00', '2024-09-10 06:00:00', 60, 'DT-005', 'CIP 세척 (살균기)', 1, 'OP001'),
('WO-241008-A01', 'EQ-PAST-01', 'LINE-A', '2024-10-08 05:00:00', '2024-10-08 06:00:00', 60, 'DT-005', 'CIP 세척 (살균기)', 1, 'OP001'),
-- 비계획 정지
('WO-241120-A01', 'EQ-PAST-01', 'LINE-A', '2024-11-20 09:18:00', '2024-11-20 09:35:00', 17, 'DT-003', '살균 온도 이탈로 인한 재순환', 0, 'OP001'),
('WO-241120-A01', 'EQ-MD-01', 'LINE-A', '2024-11-20 12:05:00', '2024-11-20 12:25:00', 20, 'DT-003', '금속이물 검출로 인한 점검', 0, 'OP002');

-- ========================================
-- 8. 체크리스트 결과 (checklist_result)
-- ========================================

INSERT INTO checklist_result (wo_no, checklist_type, check_time, operator_id, items, overall_result, remark) VALUES
('WO-240910-A01', 'PRE_START', '2024-09-10 06:00:00', 'OP001', '{"items": [{"name": "CIP 완료 확인", "result": "OK"}, {"name": "밸브 상태 점검", "result": "OK"}, {"name": "계측기 영점 확인", "result": "OK"}]}', 'OK', NULL),
('WO-240910-A01', 'SHIFT_END', '2024-09-10 14:00:00', 'OP001', '{"items": [{"name": "생산량 기록", "result": "OK"}, {"name": "설비 청소", "result": "OK"}, {"name": "인수인계 완료", "result": "OK"}]}', 'OK', NULL),
('WO-241008-A01', 'PRE_START', '2024-10-08 06:00:00', 'OP001', '{"items": [{"name": "CIP 완료 확인", "result": "OK"}, {"name": "밸브 상태 점검", "result": "OK"}, {"name": "계측기 영점 확인", "result": "OK"}]}', 'OK', NULL),
('WO-241120-A01', 'PRE_START', '2024-11-20 06:00:00', 'OP001', '{"items": [{"name": "CIP 완료 확인", "result": "OK"}, {"name": "밸브 상태 점검", "result": "OK"}, {"name": "계측기 영점 확인", "result": "OK"}]}', 'OK', NULL),
('WO-241120-A01', 'SHIFT_END', '2024-11-20 18:00:00', 'OP001', '{"items": [{"name": "생산량 기록", "result": "OK"}, {"name": "설비 청소", "result": "OK"}, {"name": "CCP 이탈 보고서 작성", "result": "OK"}]}', 'OK', '살균 CCP 이탈 1건 발생, 시정조치 완료');
