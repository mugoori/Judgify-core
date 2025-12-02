/**
 * 출하승인 템플릿 Mock 데이터 생성기
 */
import { StepMockData } from '../../types';
import {
  randomFrom,
  randomInt,
  generateTimestamp,
  generateApprover,
  generateBatchNumber,
  PRODUCTS,
} from '../common';

// 배송처 풀
const DESTINATIONS = [
  '이마트 물류센터', '롯데마트 물류센터', 'GS리테일 물류센터',
  'CU 중앙물류', '세븐일레븐 물류센터', '홈플러스 물류센터',
];

export function generateReleaseApprovalMock(): StepMockData[] {
  const product = randomFrom(PRODUCTS);
  const destination = randomFrom(DESTINATIONS);
  const batchNo = generateBatchNumber();
  const quantity = randomInt(100, 500);
  const approver = generateApprover('QC');

  // 품질 검사 결과
  const taste = randomFrom(['양호', '양호', '양호', '양호', '우수']);
  const appearance = randomFrom(['양호', '양호', '우수']);

  return [
    {
      trigger: {
        type: 'event',
        eventName: '출하 요청',
        timestamp: generateTimestamp(15),
        target: product,
        data: {
          배송처: destination,
          수량: `${quantity}박스`,
          배치번호: batchNo,
        },
      },
    },
    {
      query: {
        tableName: '출하 전 품질검사',
        results: [
          { item: '관능검사 (맛)', standard: '양호 이상', actual: taste, pass: true },
          { item: '관능검사 (외관)', standard: '양호 이상', actual: appearance, pass: true },
          { item: '유통기한', standard: 'D+7 이상', actual: 'D+14', pass: true },
          { item: '포장상태', standard: '이상없음', actual: '정상', pass: true },
          { item: '표시사항', standard: '규정준수', actual: '적합', pass: true },
        ],
        totalCount: 5,
      },
    },
    {
      judgment: {
        result: true,
        method: 'Rule Engine',
        confidence: 0.97,
        explanation: `${product} ${quantity}박스 출하 적합. 모든 품질검사 통과.`,
      },
    },
    {
      approval: {
        approver: approver.name,
        role: approver.role,
        status: 'pending',
        approvedAt: null,
        comment: '품질 이상 없음, 출하 승인',
      },
    },
    {
      alert: {
        sent: true,
        recipients: ['물류팀', '영업팀'],
        channel: 'system',
        sentAt: generateTimestamp(0),
        message: `${product} 출하승인 완료 → ${destination} (${quantity}박스)`,
      },
    },
  ];
}
