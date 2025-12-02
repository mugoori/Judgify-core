/**
 * 생산예측 템플릿 Mock 데이터 생성기
 */
import { StepMockData } from '../../types';
import {
  randomFrom,
  randomInt,
  randomBetween,
  generateTimestamp,
  generateApprover,
  PRODUCTS,
  PRODUCTION_LINES,
} from '../common';

export function generateProductionForecastMock(): StepMockData[] {
  const product = randomFrom(PRODUCTS);
  const line = randomFrom(PRODUCTION_LINES);
  const approver = generateApprover('PRODUCTION');

  // 생산 예측 데이터
  const demandForecast = randomInt(1000, 5000);
  const currentCapacity = randomInt(800, 4000);
  const efficiency = randomBetween(85, 98, 1);
  const effectiveCapacity = Math.round(currentCapacity * efficiency / 100);
  const gap = demandForecast - effectiveCapacity;

  // 지난 주 실적
  const lastWeekActual = randomInt(800, 4000);
  const lastWeekTarget = randomInt(900, 4500);
  const achievementRate = ((lastWeekActual / lastWeekTarget) * 100).toFixed(1);

  return [
    {
      trigger: {
        type: 'schedule',
        eventName: '주간 생산계획 수립',
        timestamp: generateTimestamp(60),
        target: `${product} (${line})`,
      },
    },
    {
      query: {
        tableName: '생산 현황 및 예측',
        results: [
          { item: '수요 예측', standard: '-', actual: `${demandForecast.toLocaleString()}개`, pass: true },
          { item: '현재 생산능력', standard: '-', actual: `${currentCapacity.toLocaleString()}개/주`, pass: true },
          { item: '설비 효율', standard: '≥85%', actual: `${efficiency}%`, pass: efficiency >= 85 },
          { item: '유효 생산능력', standard: '-', actual: `${effectiveCapacity.toLocaleString()}개/주`, pass: true },
          { item: '지난주 달성률', standard: '≥95%', actual: `${achievementRate}%`, pass: Number(achievementRate) >= 95 },
        ],
        totalCount: 5,
      },
    },
    {
      calc: {
        formula: '과부족 = 수요예측 - 유효생산능력',
        inputs: {
          수요예측: demandForecast,
          현재생산능력: currentCapacity,
          효율: efficiency,
        },
        result: gap,
        unit: '개',
        description: gap > 0
          ? `생산능력 ${gap.toLocaleString()}개 부족 - 증산 또는 외주 검토 필요`
          : `생산능력 여유 ${Math.abs(gap).toLocaleString()}개`,
      },
    },
    {
      judgment: {
        result: gap <= 0,
        method: 'Hybrid',
        confidence: 0.92,
        explanation: gap > 0
          ? `수요 ${demandForecast.toLocaleString()}개 대비 생산능력 부족. 잔업/외주 검토 필요.`
          : `수요 ${demandForecast.toLocaleString()}개 충족 가능. 정상 운영 가능.`,
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
        recipients: ['생산팀', '영업팀', '경영지원팀'],
        channel: 'system',
        sentAt: generateTimestamp(0),
        message: `${product} 주간 생산계획 수립 - 예측 ${demandForecast.toLocaleString()}개`,
      },
    },
  ];
}
