/**
 * 원료입고검사 템플릿 Mock 데이터 생성기
 */
import { StepMockData } from '../../types';
import {
  randomFrom,
  randomBetween,
  generateTimestamp,
  generateApprover,
  generateLotNumber,
  generateQuantity,
  MATERIALS,
  SUPPLIERS,
} from '../common';

export function generateMaterialInspectionMock(): StepMockData[] {
  const material = randomFrom(MATERIALS);
  const supplier = randomFrom(SUPPLIERS);
  const lotNo = generateLotNumber();
  const quantity = generateQuantity('kg', 100, 1000);
  const approver = generateApprover('QC');

  // 검사 항목별 결과 생성
  const moisture = randomBetween(2.5, 4.5, 1);
  const fat = randomBetween(0.3, 0.8, 1);
  const protein = randomBetween(34, 38, 1);
  const acidity = randomBetween(0.1, 0.15, 2);

  return [
    {
      trigger: {
        type: 'event',
        eventName: '원료 입고 등록',
        timestamp: generateTimestamp(10),
        target: material,
        data: {
          공급업체: supplier,
          LOT번호: lotNo,
          수량: quantity,
        },
      },
    },
    {
      query: {
        tableName: '입고검사 항목',
        results: [
          { item: '수분함량', standard: '< 5%', actual: `${moisture}%`, pass: moisture < 5 },
          { item: '지방함량', standard: '< 1%', actual: `${fat}%`, pass: fat < 1 },
          { item: '단백질', standard: '> 34%', actual: `${protein}%`, pass: protein > 34 },
          { item: '산도', standard: '< 0.15%', actual: `${acidity}%`, pass: acidity < 0.15 },
          { item: '이물질', standard: '없음', actual: '없음', pass: true },
          { item: '외관', standard: '양호', actual: '양호', pass: true },
        ],
        totalCount: 6,
      },
    },
    {
      judgment: {
        result: true,
        method: 'Rule Engine',
        confidence: 0.98,
        explanation: `모든 검사 항목이 기준치 이내입니다. ${material} ${quantity} 입고 적합 판정.`,
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
        recipients: ['구매팀', '생산팀', '창고팀'],
        channel: 'system',
        sentAt: generateTimestamp(0),
        message: `${material} 입고검사 완료 - ${lotNo} (${supplier})`,
      },
    },
  ];
}
