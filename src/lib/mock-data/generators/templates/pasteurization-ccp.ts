/**
 * 살균온도 CCP 템플릿 Mock 데이터 생성기
 */
import { StepMockData } from '../../types';
import {
  randomFrom,
  randomBetween,
  generateTimestamp,
  generateApprover,
  generateBatchNumber,
  EQUIPMENT,
  PRODUCTION_LINES,
} from '../common';

export function generatePasteurizationCcpMock(): StepMockData[] {
  const equipment = randomFrom(EQUIPMENT.filter(e => e.includes('살균기')));
  const line = randomFrom(PRODUCTION_LINES);
  const batchNo = generateBatchNumber();
  const temp = randomBetween(83, 87, 1); // 살균 온도 범위
  const holdTime = randomBetween(15, 18, 0); // 살균 시간 (초)
  const approver = generateApprover('QC');

  return [
    {
      trigger: {
        type: 'schedule',
        eventName: '살균 공정 완료',
        timestamp: generateTimestamp(5),
        target: `${equipment} (${line})`,
        data: {
          배치번호: batchNo,
          설비: equipment,
        },
      },
    },
    {
      query: {
        tableName: 'CCP 모니터링 데이터',
        results: [
          { item: '살균 온도', standard: '85±2°C', actual: `${temp}°C`, pass: temp >= 83 && temp <= 87 },
          { item: '유지 시간', standard: '≥15초', actual: `${holdTime}초`, pass: holdTime >= 15 },
          { item: '냉각 온도', standard: '≤4°C', actual: `${randomBetween(2, 4, 1)}°C`, pass: true },
        ],
        totalCount: 3,
      },
    },
    {
      judgment: {
        result: true,
        method: 'Rule Engine',
        confidence: 0.98,
        explanation: `살균 온도 ${temp}°C, 유지 시간 ${holdTime}초로 CCP 기준을 충족합니다.`,
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
        message: `${equipment} 살균 CCP 검증 완료 - ${batchNo}`,
      },
    },
  ];
}
