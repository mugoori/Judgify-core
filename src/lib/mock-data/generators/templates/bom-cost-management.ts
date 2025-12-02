/**
 * BOM 원가관리 템플릿 Mock 데이터 생성기
 */
import { StepMockData } from '../../types';
import {
  randomFrom,
  randomInt,
  randomBetween,
  generateTimestamp,
  generateApprover,
  MATERIALS,
  PRODUCTS,
} from '../common';

export function generateBomCostManagementMock(): StepMockData[] {
  const product = randomFrom(PRODUCTS);
  const approver = generateApprover('MANAGER');

  // 원가 구성 (자재비, 노무비, 경비)
  const materialCosts = MATERIALS.slice(0, randomInt(2, 4)).map(m => ({
    name: m,
    unitPrice: randomInt(500, 5000),
    quantity: randomBetween(0.1, 2.0, 2),
    cost: 0,
  }));

  // 비용 계산
  materialCosts.forEach(m => {
    m.cost = Math.round(m.unitPrice * m.quantity);
  });

  const totalMaterialCost = materialCosts.reduce((sum, m) => sum + m.cost, 0);
  const laborCost = randomInt(200, 500);
  const overheadCost = randomInt(100, 300);
  const totalCost = totalMaterialCost + laborCost + overheadCost;
  const targetCost = randomInt(totalCost - 500, totalCost + 500);
  const variance = ((totalCost - targetCost) / targetCost * 100).toFixed(1);

  return [
    {
      trigger: {
        type: 'event',
        eventName: '원가 분석 요청',
        timestamp: generateTimestamp(20),
        target: product,
      },
    },
    {
      query: {
        tableName: 'BOM 원가 구성',
        results: [
          ...materialCosts.map(m => ({
            item: m.name,
            standard: `단가: ${m.unitPrice}원`,
            actual: `${m.cost}원 (${m.quantity}kg)`,
            pass: true,
          })),
          { item: '자재비 소계', standard: '-', actual: `${totalMaterialCost.toLocaleString()}원`, pass: true },
          { item: '노무비', standard: '-', actual: `${laborCost.toLocaleString()}원`, pass: true },
          { item: '제조경비', standard: '-', actual: `${overheadCost.toLocaleString()}원`, pass: true },
        ],
        totalCount: materialCosts.length + 3,
      },
    },
    {
      calc: {
        formula: '총원가 = 자재비 + 노무비 + 제조경비',
        inputs: {
          자재비: totalMaterialCost,
          노무비: laborCost,
          제조경비: overheadCost,
        },
        result: totalCost,
        unit: '원',
        description: `${product} 개당 원가: ${totalCost.toLocaleString()}원`,
      },
    },
    {
      judgment: {
        result: totalCost <= targetCost * 1.05,
        method: 'Rule Engine',
        confidence: 0.94,
        explanation: totalCost <= targetCost
          ? `목표원가 ${targetCost.toLocaleString()}원 대비 ${Math.abs(Number(variance))}% 절감`
          : `목표원가 ${targetCost.toLocaleString()}원 대비 ${variance}% 초과 - 원가절감 필요`,
      },
    },
    {
      approval: {
        approver: approver.name,
        role: approver.role,
        status: 'pending',
        approvedAt: null,
      },
    },
    {
      alert: {
        sent: true,
        recipients: ['경영지원팀', '생산팀', '구매팀'],
        channel: 'system',
        sentAt: generateTimestamp(0),
        message: `${product} 원가분석 완료 - ${totalCost.toLocaleString()}원 (${variance}%)`,
      },
    },
  ];
}
