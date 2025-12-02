/**
 * 예방정비 템플릿 Mock 데이터 생성기
 */
import { StepMockData } from '../../types';
import {
  randomFrom,
  randomInt,
  randomBetween,
  generateTimestamp,
  generateApprover,
  EQUIPMENT,
  KOREAN_NAMES,
} from '../common';

// 정비 항목
const MAINTENANCE_ITEMS = [
  { item: '베어링 점검', cycle: 30 },
  { item: '윤활유 교체', cycle: 90 },
  { item: '벨트 마모 점검', cycle: 60 },
  { item: '필터 교체', cycle: 30 },
  { item: '센서 캘리브레이션', cycle: 180 },
  { item: '누유 점검', cycle: 7 },
];

export function generatePreventiveMaintenanceMock(): StepMockData[] {
  const equipment = randomFrom(EQUIPMENT);
  const approver = generateApprover('MAINTENANCE');
  const technician = randomFrom(KOREAN_NAMES);

  // 정비 항목 선택 (2-4개)
  const selectedItems = MAINTENANCE_ITEMS
    .sort(() => Math.random() - 0.5)
    .slice(0, randomInt(2, 4));

  const totalHours = randomBetween(1.5, 4.0, 1);
  const runningHours = randomInt(1000, 5000);

  return [
    {
      trigger: {
        type: 'schedule',
        eventName: '예방정비 일정',
        timestamp: generateTimestamp(60),
        target: equipment,
        data: {
          담당자: technician,
          가동시간: `${runningHours}시간`,
        },
      },
    },
    {
      query: {
        tableName: '정비 점검 항목',
        results: selectedItems.map(m => ({
          item: m.item,
          standard: `주기: ${m.cycle}일`,
          actual: randomFrom(['양호', '양호', '양호', '교체필요', '점검필요']),
          pass: Math.random() > 0.2,
        })),
        totalCount: selectedItems.length,
      },
    },
    {
      calc: {
        formula: '다음 정비 예정일 = 오늘 + 최소 정비주기',
        inputs: {
          현재가동시간: runningHours,
          정비항목수: selectedItems.length,
        },
        result: Math.min(...selectedItems.map(m => m.cycle)),
        unit: '일 후',
        description: `다음 정비 예정: ${Math.min(...selectedItems.map(m => m.cycle))}일 후`,
      },
    },
    {
      judgment: {
        result: true,
        method: 'Rule Engine',
        confidence: 0.96,
        explanation: `${equipment} 예방정비 완료. ${selectedItems.length}개 항목 점검, 소요시간 ${totalHours}시간.`,
      },
    },
    {
      approval: {
        approver: approver.name,
        role: approver.role,
        status: 'pending',
        approvedAt: null,
        comment: '정비 완료 확인',
      },
    },
    {
      alert: {
        sent: true,
        recipients: ['설비팀', '생산팀'],
        channel: 'system',
        sentAt: generateTimestamp(0),
        message: `${equipment} 예방정비 완료 - ${technician} (${totalHours}시간)`,
      },
    },
  ];
}
