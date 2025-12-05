-- ========================================
-- 017: 창고-재고 매핑 수정
-- 목적: inventory.location과 warehouse_mst.warehouse_id 일치시키기
--       "창고별 재고 비율" 차트가 정상 표시되도록 수정
-- ========================================

-- 기존 warehouse_mst 데이터 유지하고 inventory.location에 사용된 창고 추가
-- inventory 테이블에서 사용되는 location 값들:
-- 원료창고: COLD-A01, COLD-A02, WH-A01, WH-A02, WH-A03, WH-A04, WH-A05
--          WH-B02, COLD-B01, COLD-B02, COLD-C01
-- 포장재: WH-D01, WH-D02, WH-D03, WH-D04
-- 완제품: WH-FG01, WH-FG02, FG-WH01, FG-WH02, FG-WH03, FG-WH04, FG-COLD01

INSERT OR REPLACE INTO warehouse_mst (warehouse_id, warehouse_nm, warehouse_type, location, temp_min, temp_max) VALUES
-- 원료 냉장창고 (COLD-*)
('COLD-A01', '냉장창고 A-1', 'COLD', '공장 A동 1층', 0, 5),
('COLD-A02', '냉장창고 A-2', 'COLD', '공장 A동 1층', 0, 5),
('COLD-B01', '냉장창고 B-1', 'COLD', '공장 B동 1층', 0, 5),
('COLD-B02', '냉장창고 B-2', 'COLD', '공장 B동 1층', 0, 5),
('COLD-C01', '냉장창고 C-1', 'COLD', '공장 C동 1층', 0, 5),

-- 원료 일반창고 (WH-A*, WH-B*)
('WH-A01', '원료창고 A-1', 'RAW', '공장 A동 2층', 15, 25),
('WH-A02', '원료창고 A-2', 'RAW', '공장 A동 2층', 15, 25),
('WH-A03', '원료창고 A-3', 'RAW', '공장 A동 2층', 15, 25),
('WH-A04', '원료창고 A-4', 'RAW', '공장 A동 3층', 15, 25),
('WH-A05', '원료창고 A-5', 'RAW', '공장 A동 3층', 15, 25),
('WH-B02', '원료창고 B-2', 'RAW', '공장 B동 2층', 15, 25),

-- 포장재 창고 (WH-D*)
('WH-D01', '포장재창고 D-1', 'RAW', '공장 D동 1층', 15, 25),
('WH-D02', '포장재창고 D-2', 'RAW', '공장 D동 1층', 15, 25),
('WH-D03', '포장재창고 D-3', 'RAW', '공장 D동 2층', 15, 25),
('WH-D04', '포장재창고 D-4', 'RAW', '공장 D동 2층', 15, 25),

-- 완제품 창고 (WH-FG*, FG-WH*, FG-COLD*)
('WH-FG01', '완제품창고 1동', 'FG', '물류센터 1층', 0, 5),
('WH-FG02', '완제품창고 2동 (냉동)', 'FROZEN', '물류센터 지하', -25, -18),
('FG-WH01', '완제품창고 A', 'FG', '물류센터 A구역', 0, 5),
('FG-WH02', '완제품창고 B', 'FG', '물류센터 B구역', 0, 5),
('FG-WH03', '완제품창고 C', 'FG', '물류센터 C구역', 0, 5),
('FG-WH04', '완제품창고 D', 'FG', '물류센터 D구역', 0, 5),
('FG-COLD01', '완제품 냉장창고', 'COLD', '물류센터 냉장구역', 0, 5);

-- ========================================
-- 확인용 뷰: 창고별 재고 현황
-- ========================================
DROP VIEW IF EXISTS v_warehouse_inventory_summary;

CREATE VIEW v_warehouse_inventory_summary AS
SELECT
    i.location as warehouse_id,
    COALESCE(w.warehouse_nm, i.location) as warehouse_nm,
    COALESCE(w.warehouse_type, 'UNKNOWN') as warehouse_type,
    SUM(i.qty) as total_qty,
    COUNT(DISTINCT i.item_cd) as item_count
FROM inventory i
LEFT JOIN warehouse_mst w ON i.location = w.warehouse_id
GROUP BY i.location
ORDER BY total_qty DESC;

