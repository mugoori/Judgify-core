/**
 * 유통기한 모니터링 템플릿 Mock 데이터 생성기
 */
import { StepMockData } from '../../types';
import {
  randomFrom,
  randomInt,
  generateTimestamp,
  generateBatchNumber,
  PRODUCTS,
} from '../common';

export function generateShelfLifeMonitoringMock(): StepMockData[] {
  const product = randomFrom(PRODUCTS);
  const batchNo = generateBatchNumber();

  // 날짜 계산
  const today = new Date();
  const prodDate = new Date(today);
  prodDate.setDate(prodDate.getDate() - randomInt(5, 15));
  const expDate = new Date(prodDate);
  expDate.setDate(expDate.getDate() + 14); // 유통기한 14일

  const remainingDays = Math.ceil((expDate.getTime() - today.getTime()) / (1000 * 60 * 60 * 24));
  const currentStock = randomInt(50, 200);

  const formatDate = (d: Date) => {
    const y = d.getFullYear();
    const m = String(d.getMonth() + 1).padStart(2, '0');
    const day = String(d.getDate()).padStart(2, '0');
    return `${y}-${m}-${day}`;
  };

  return [
    {
      trigger: {
        type: 'schedule',
        eventName: '일일 유통기한 점검',
        timestamp: generateTimestamp(5),
        target: '전체 재고',
      },
    },
    {
      query: {
        tableName: '유통기한 임박 제품',
        results: [
          { item: '제품명', standard: '-', actual: product, pass: true },
          { item: '배치번호', standard: '-', actual: batchNo, pass: true },
          { item: '제조일자', standard: '-', actual: formatDate(prodDate), pass: true },
          { item: '유통기한', standard: '-', actual: formatDate(expDate), pass: true },
          { item: '잔여일수', standard: '> 3일', actual: `${remainingDays}일`, pass: remainingDays > 3 },
          { item: '현재고', standard: '-', actual: `${currentStock}박스`, pass: true },
        ],
        totalCount: 1,
      },
    },
    {
      calc: {
        formula: '잔여일수 = 유통기한 - 현재일',
        inputs: {
          유통기한: expDate.getTime(),
          현재일: today.getTime(),
        },
        result: remainingDays,
        unit: '일',
        description: `${product} 유통기한까지 ${remainingDays}일 남음`,
      },
    },
    {
      judgment: {
        result: remainingDays > 3,
        method: 'Rule Engine',
        confidence: 1.0,
        explanation: remainingDays > 7
          ? `유통기한 ${remainingDays}일 남음 - 정상`
          : remainingDays > 3
            ? `유통기한 ${remainingDays}일 남음 - 조기 출고 권장`
            : `유통기한 ${remainingDays}일 남음 - 즉시 조치 필요`,
      },
    },
    {
      alert: {
        sent: true,
        recipients: ['창고팀', '영업팀'],
        channel: 'system',
        sentAt: generateTimestamp(0),
        message: `유통기한 모니터링 완료 - ${product} D-${remainingDays}`,
      },
    },
  ];
}
