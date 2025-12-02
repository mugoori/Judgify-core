/**
 * 재고예측 템플릿 Mock 데이터 생성기
 */
import { StepMockData } from '../../types';
import {
  randomFrom,
  randomInt,
  generateTimestamp,
  MATERIALS,
} from '../common';

export function generateInventoryForecastMock(): StepMockData[] {
  const material = randomFrom(MATERIALS);

  const currentStock = randomInt(500, 2000);
  const dailyUsage = randomInt(50, 150);
  const leadTime = randomInt(3, 7);
  const safetyStock = dailyUsage * 3;
  const forecastDays = Math.floor(currentStock / dailyUsage);
  const reorderPoint = (dailyUsage * leadTime) + safetyStock;
  const needReorder = currentStock <= reorderPoint;

  return [
    {
      trigger: {
        type: 'schedule',
        eventName: '일일 재고 예측 분석',
        timestamp: generateTimestamp(5),
        target: material,
      },
    },
    {
      query: {
        tableName: '재고 현황',
        results: [
          { item: '현재고', standard: '-', actual: `${currentStock}kg`, pass: true },
          { item: '일평균 사용량', standard: '-', actual: `${dailyUsage}kg/일`, pass: true },
          { item: '안전재고', standard: `≥${safetyStock}kg`, actual: `${safetyStock}kg`, pass: true },
          { item: '리드타임', standard: '-', actual: `${leadTime}일`, pass: true },
        ],
        totalCount: 4,
      },
    },
    {
      calc: {
        formula: '예상 소진일 = 현재고 ÷ 일평균 사용량',
        inputs: {
          현재고: currentStock,
          일평균사용량: dailyUsage,
        },
        result: forecastDays,
        unit: '일',
        description: `${material} 예상 소진일: ${forecastDays}일 후`,
      },
    },
    {
      judgment: {
        result: !needReorder,
        method: 'Rule Engine',
        confidence: 0.95,
        explanation: needReorder
          ? `현재고 ${currentStock}kg이 발주점 ${reorderPoint}kg 이하. 발주 필요!`
          : `현재고 ${currentStock}kg, 발주점 ${reorderPoint}kg. 재고 적정.`,
      },
    },
    {
      alert: {
        sent: true,
        recipients: needReorder ? ['구매팀', '창고팀', '생산팀'] : ['창고팀'],
        channel: 'system',
        sentAt: generateTimestamp(0),
        message: needReorder
          ? `⚠️ ${material} 발주 필요 - 현재고 ${currentStock}kg (D-${forecastDays})`
          : `${material} 재고 정상 - ${currentStock}kg (D-${forecastDays})`,
      },
    },
  ];
}
