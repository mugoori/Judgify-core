/**
 * 금속검출 CCP 템플릿 Mock 데이터 생성기
 */
import { StepMockData } from '../../types';
import {
  randomFrom,
  randomBetween,
  generateTimestamp,
  generateApprover,
  generateBatchNumber,
  EQUIPMENT,
  PRODUCTS,
  PRODUCTION_LINES,
} from '../common';

export function generateMetalDetectionCcpMock(): StepMockData[] {
  const detector = randomFrom(EQUIPMENT.filter(e => e.includes('금속검출기') || e.includes('X-Ray')));
  const product = randomFrom(PRODUCTS);
  const line = randomFrom(PRODUCTION_LINES);
  const batchNo = generateBatchNumber();
  const feSensitivity = randomBetween(1.0, 1.5, 1);
  const susSensitivity = randomBetween(1.5, 2.0, 1);
  const approver = generateApprover('QC');

  return [
    {
      trigger: {
        type: 'event',
        eventName: '포장라인 금속검출 테스트',
        timestamp: generateTimestamp(3),
        target: `${detector} (${line})`,
        data: {
          제품: product,
          배치번호: batchNo,
        },
      },
    },
    {
      query: {
        tableName: '금속검출기 테스트 결과',
        results: [
          { item: 'Fe 테스트 (철)', standard: '≤1.5mm', actual: `${feSensitivity}mm`, pass: feSensitivity <= 1.5 },
          { item: 'SUS 테스트 (스테인리스)', standard: '≤2.0mm', actual: `${susSensitivity}mm`, pass: susSensitivity <= 2.0 },
          { item: 'Non-Fe 테스트 (비철)', standard: '≤2.0mm', actual: `${randomBetween(1.5, 2.0, 1)}mm`, pass: true },
          { item: '리젝터 작동', standard: '정상', actual: '정상', pass: true },
        ],
        totalCount: 4,
      },
    },
    {
      judgment: {
        result: true,
        method: 'Rule Engine',
        confidence: 0.99,
        explanation: `금속검출기 민감도 테스트 통과 (Fe: ${feSensitivity}mm, SUS: ${susSensitivity}mm)`,
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
        recipients: ['생산팀', '품질팀'],
        channel: 'system',
        sentAt: generateTimestamp(0),
        message: `${detector} CCP 테스트 완료 - ${product} (${batchNo})`,
      },
    },
  ];
}
