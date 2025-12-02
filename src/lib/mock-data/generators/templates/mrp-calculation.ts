/**
 * MRP 계산 템플릿 Mock 데이터 생성기
 */
import { StepMockData } from '../../types';
import {
  randomFrom,
  randomInt,
  generateTimestamp,
  generateApprover,
  MATERIALS,
  PRODUCTS,
} from '../common';

export function generateMrpCalculationMock(): StepMockData[] {
  const product = randomFrom(PRODUCTS);
  const approver = generateApprover('PRODUCTION');

  // BOM 구성 자재 (2-4개)
  const bomMaterials = MATERIALS.slice(0, randomInt(2, 4)).map(m => ({
    name: m,
    requirement: randomInt(50, 200),
    stock: randomInt(100, 500),
    onOrder: randomInt(0, 100),
  }));

  const productionQty = randomInt(500, 2000);

  return [
    {
      trigger: {
        type: 'event',
        eventName: '생산계획 등록',
        timestamp: generateTimestamp(30),
        target: product,
        data: {
          생산수량: `${productionQty}개`,
          생산일: generateTimestamp(0).split(' ')[0],
        },
      },
    },
    {
      query: {
        tableName: 'BOM 소요량 계산',
        results: bomMaterials.map(m => {
          const available = m.stock + m.onOrder;
          const shortage = Math.max(0, m.requirement - available);
          return {
            item: m.name,
            standard: `소요: ${m.requirement}kg`,
            actual: `재고: ${m.stock}kg + 입고예정: ${m.onOrder}kg`,
            pass: shortage === 0,
          };
        }),
        totalCount: bomMaterials.length,
      },
    },
    {
      calc: {
        formula: '부족량 = 소요량 - (현재고 + 입고예정)',
        inputs: bomMaterials.reduce((acc, m) => {
          acc[m.name] = m.requirement - (m.stock + m.onOrder);
          return acc;
        }, {} as Record<string, number>),
        result: bomMaterials.reduce((sum, m) => sum + Math.max(0, m.requirement - m.stock - m.onOrder), 0),
        unit: 'kg',
        description: 'MRP 소요량 계산 완료',
      },
    },
    {
      judgment: {
        result: bomMaterials.every(m => m.stock + m.onOrder >= m.requirement),
        method: 'Rule Engine',
        confidence: 1.0,
        explanation: bomMaterials.every(m => m.stock + m.onOrder >= m.requirement)
          ? `${product} ${productionQty}개 생산 가능 - 모든 자재 확보`
          : `일부 자재 부족 - 추가 발주 필요`,
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
        recipients: ['생산팀', '구매팀', '창고팀'],
        channel: 'system',
        sentAt: generateTimestamp(0),
        message: `MRP 계산 완료 - ${product} ${productionQty}개`,
      },
    },
  ];
}
