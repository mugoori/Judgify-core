/**
 * Workflow Templates Library
 *
 * Phase 44에서 추가된 10개의 사전 정의된 워크플로우 템플릿입니다.
 * 사용자는 이 템플릿들을 빠르게 선택하여 커스터마이징할 수 있습니다.
 */

import type { Node, Edge } from 'reactflow';

export interface WorkflowTemplate {
  id: string;
  name: string;
  description: string;
  category: 'basic' | 'advanced' | 'integration' | 'automation';
  nodes: Node[];
  edges: Edge[];
  tags: string[];
}

// ========================================
// 1. 품질 검사 워크플로우
// ========================================
export const qualityCheckTemplate: WorkflowTemplate = {
  id: 'quality-check',
  name: '품질 검사 워크플로우',
  description: '센서 데이터 기반 품질 판정 및 알림',
  category: 'basic',
  tags: ['품질', '검사', '센서'],
  nodes: [
    {
      id: 'input-1',
      type: 'custom',
      data: { label: '데이터 입력', type: 'input', description: '센서 데이터 수집' },
      position: { x: 250, y: 50 },
    },
    {
      id: 'decision-1',
      type: 'custom',
      data: {
        label: '품질 판단',
        type: 'decision',
        description: '온도 및 진동 기준 검사',
        rule: 'temperature > 85 AND vibration > 45',
      },
      position: { x: 250, y: 180 },
    },
    {
      id: 'action-1',
      type: 'custom',
      data: { label: '불량 알림', type: 'action', description: 'Slack 알림 발송' },
      position: { x: 450, y: 310 },
    },
    {
      id: 'action-2',
      type: 'custom',
      data: { label: '정상 기록', type: 'action', description: 'DB에 정상 기록' },
      position: { x: 50, y: 310 },
    },
  ],
  edges: [
    { id: 'e1', source: 'input-1', target: 'decision-1', animated: true },
    { id: 'e2', source: 'decision-1', target: 'action-1', label: '불량' },
    { id: 'e3', source: 'decision-1', target: 'action-2', label: '정상' },
  ],
};

// ========================================
// 2. API 연동 워크플로우
// ========================================
export const apiIntegrationTemplate: WorkflowTemplate = {
  id: 'api-integration',
  name: 'API 연동 워크플로우',
  description: '외부 REST API 호출 및 응답 처리',
  category: 'integration',
  tags: ['API', 'REST', '연동'],
  nodes: [
    {
      id: 'input-1',
      type: 'custom',
      data: { label: '트리거', type: 'input', description: '웹훅 또는 스케줄' },
      position: { x: 250, y: 50 },
    },
    {
      id: 'action-1',
      type: 'custom',
      data: { label: 'API 호출', type: 'action', description: 'GET /api/data' },
      position: { x: 250, y: 180 },
    },
    {
      id: 'decision-1',
      type: 'custom',
      data: {
        label: '응답 검증',
        type: 'decision',
        description: 'status === 200',
        rule: 'response.status === 200',
      },
      position: { x: 250, y: 310 },
    },
    {
      id: 'action-2',
      type: 'custom',
      data: { label: '데이터 저장', type: 'action', description: 'PostgreSQL 저장' },
      position: { x: 450, y: 440 },
    },
    {
      id: 'action-3',
      type: 'custom',
      data: { label: '에러 로그', type: 'action', description: '에러 로깅' },
      position: { x: 50, y: 440 },
    },
  ],
  edges: [
    { id: 'e1', source: 'input-1', target: 'action-1', animated: true },
    { id: 'e2', source: 'action-1', target: 'decision-1', animated: true },
    { id: 'e3', source: 'decision-1', target: 'action-2', label: '성공' },
    { id: 'e4', source: 'decision-1', target: 'action-3', label: '실패' },
  ],
};

// ========================================
// 3. 데이터 변환 워크플로우
// ========================================
export const dataTransformTemplate: WorkflowTemplate = {
  id: 'data-transform',
  name: '데이터 변환 워크플로우',
  description: 'CSV/JSON 데이터 가공 및 포맷 변환',
  category: 'basic',
  tags: ['데이터', '변환', 'ETL'],
  nodes: [
    {
      id: 'input-1',
      type: 'custom',
      data: { label: 'CSV 입력', type: 'input', description: '원본 CSV 파일' },
      position: { x: 250, y: 50 },
    },
    {
      id: 'action-1',
      type: 'custom',
      data: { label: '파싱', type: 'action', description: 'CSV → JSON' },
      position: { x: 250, y: 180 },
    },
    {
      id: 'action-2',
      type: 'custom',
      data: { label: '필터링', type: 'action', description: '불필요 데이터 제거' },
      position: { x: 250, y: 310 },
    },
    {
      id: 'action-3',
      type: 'custom',
      data: { label: '출력', type: 'action', description: 'JSON 파일 저장' },
      position: { x: 250, y: 440 },
    },
  ],
  edges: [
    { id: 'e1', source: 'input-1', target: 'action-1', animated: true },
    { id: 'e2', source: 'action-1', target: 'action-2', animated: true },
    { id: 'e3', source: 'action-2', target: 'action-3', animated: true },
  ],
};

// ========================================
// 4. 이메일 발송 워크플로우
// ========================================
export const emailSendTemplate: WorkflowTemplate = {
  id: 'email-send',
  name: '이메일 발송 워크플로우',
  description: '조건별 자동 이메일 발송',
  category: 'automation',
  tags: ['이메일', '알림', '자동화'],
  nodes: [
    {
      id: 'input-1',
      type: 'custom',
      data: { label: '이벤트 감지', type: 'input', description: '시스템 이벤트' },
      position: { x: 250, y: 50 },
    },
    {
      id: 'decision-1',
      type: 'custom',
      data: {
        label: '우선순위 판단',
        type: 'decision',
        description: 'priority > 5',
        rule: 'event.priority > 5',
      },
      position: { x: 250, y: 180 },
    },
    {
      id: 'action-1',
      type: 'custom',
      data: { label: '긴급 이메일', type: 'action', description: '관리자 즉시 통보' },
      position: { x: 450, y: 310 },
    },
    {
      id: 'action-2',
      type: 'custom',
      data: { label: '일반 이메일', type: 'action', description: '일일 리포트에 포함' },
      position: { x: 50, y: 310 },
    },
  ],
  edges: [
    { id: 'e1', source: 'input-1', target: 'decision-1', animated: true },
    { id: 'e2', source: 'decision-1', target: 'action-1', label: '긴급' },
    { id: 'e3', source: 'decision-1', target: 'action-2', label: '일반' },
  ],
};

// ========================================
// 5. 파일 업로드 처리 워크플로우
// ========================================
export const fileUploadTemplate: WorkflowTemplate = {
  id: 'file-upload',
  name: '파일 업로드 처리 워크플로우',
  description: '파일 업로드, 검증 및 저장',
  category: 'basic',
  tags: ['파일', '업로드', '저장'],
  nodes: [
    {
      id: 'input-1',
      type: 'custom',
      data: { label: '파일 수신', type: 'input', description: '사용자 업로드' },
      position: { x: 250, y: 50 },
    },
    {
      id: 'decision-1',
      type: 'custom',
      data: {
        label: '파일 타입 검증',
        type: 'decision',
        description: '허용 타입 확인',
        rule: 'file.type IN [pdf, png, jpg]',
      },
      position: { x: 250, y: 180 },
    },
    {
      id: 'action-1',
      type: 'custom',
      data: { label: 'S3 저장', type: 'action', description: 'AWS S3 업로드' },
      position: { x: 450, y: 310 },
    },
    {
      id: 'action-2',
      type: 'custom',
      data: { label: '에러 반환', type: 'action', description: '400 Bad Request' },
      position: { x: 50, y: 310 },
    },
  ],
  edges: [
    { id: 'e1', source: 'input-1', target: 'decision-1', animated: true },
    { id: 'e2', source: 'decision-1', target: 'action-1', label: '유효' },
    { id: 'e3', source: 'decision-1', target: 'action-2', label: '무효' },
  ],
};

// ========================================
// 6. 스케줄링 워크플로우
// ========================================
export const schedulingTemplate: WorkflowTemplate = {
  id: 'scheduling',
  name: '스케줄링 워크플로우',
  description: '매일 자정 데이터 백업',
  category: 'automation',
  tags: ['스케줄', '백업', 'Cron'],
  nodes: [
    {
      id: 'input-1',
      type: 'custom',
      data: { label: 'Cron 트리거', type: 'input', description: '매일 0시 (0 0 * * *)' },
      position: { x: 250, y: 50 },
    },
    {
      id: 'action-1',
      type: 'custom',
      data: { label: 'DB 백업', type: 'action', description: 'PostgreSQL dump' },
      position: { x: 250, y: 180 },
    },
    {
      id: 'action-2',
      type: 'custom',
      data: { label: 'S3 업로드', type: 'action', description: '백업 파일 저장' },
      position: { x: 250, y: 310 },
    },
    {
      id: 'action-3',
      type: 'custom',
      data: { label: '완료 알림', type: 'action', description: 'Slack 알림' },
      position: { x: 250, y: 440 },
    },
  ],
  edges: [
    { id: 'e1', source: 'input-1', target: 'action-1', animated: true },
    { id: 'e2', source: 'action-1', target: 'action-2', animated: true },
    { id: 'e3', source: 'action-2', target: 'action-3', animated: true },
  ],
};

// ========================================
// 7. 반복 처리 워크플로우
// ========================================
export const loopProcessingTemplate: WorkflowTemplate = {
  id: 'loop-processing',
  name: '반복 처리 워크플로우',
  description: '배열 데이터 순회 및 처리',
  category: 'advanced',
  tags: ['반복', '루프', 'forEach'],
  nodes: [
    {
      id: 'input-1',
      type: 'custom',
      data: { label: '배열 입력', type: 'input', description: '처리할 아이템 목록' },
      position: { x: 250, y: 50 },
    },
    {
      id: 'action-1',
      type: 'custom',
      data: { label: '반복 시작', type: 'action', description: 'forEach(item)' },
      position: { x: 250, y: 180 },
    },
    {
      id: 'action-2',
      type: 'custom',
      data: { label: '아이템 처리', type: 'action', description: '개별 로직 실행' },
      position: { x: 250, y: 310 },
    },
    {
      id: 'action-3',
      type: 'custom',
      data: { label: '결과 수집', type: 'action', description: '처리 결과 배열' },
      position: { x: 250, y: 440 },
    },
  ],
  edges: [
    { id: 'e1', source: 'input-1', target: 'action-1', animated: true },
    { id: 'e2', source: 'action-1', target: 'action-2', animated: true },
    { id: 'e3', source: 'action-2', target: 'action-3', animated: true },
  ],
};

// ========================================
// 8. 조건 분기 워크플로우
// ========================================
export const conditionalBranchingTemplate: WorkflowTemplate = {
  id: 'conditional-branching',
  name: '조건 분기 워크플로우',
  description: '다중 조건 평가 및 분기',
  category: 'advanced',
  tags: ['조건', '분기', 'if-else'],
  nodes: [
    {
      id: 'input-1',
      type: 'custom',
      data: { label: '데이터 입력', type: 'input', description: '평가할 데이터' },
      position: { x: 250, y: 50 },
    },
    {
      id: 'decision-1',
      type: 'custom',
      data: {
        label: '1차 조건',
        type: 'decision',
        description: 'value > 100',
        rule: 'data.value > 100',
      },
      position: { x: 250, y: 180 },
    },
    {
      id: 'decision-2',
      type: 'custom',
      data: {
        label: '2차 조건',
        type: 'decision',
        description: 'value > 50',
        rule: 'data.value > 50',
      },
      position: { x: 50, y: 310 },
    },
    {
      id: 'action-1',
      type: 'custom',
      data: { label: '높음 처리', type: 'action', description: '우선 순위 높음' },
      position: { x: 450, y: 310 },
    },
    {
      id: 'action-2',
      type: 'custom',
      data: { label: '중간 처리', type: 'action', description: '우선 순위 중간' },
      position: { x: 250, y: 440 },
    },
    {
      id: 'action-3',
      type: 'custom',
      data: { label: '낮음 처리', type: 'action', description: '우선 순위 낮음' },
      position: { x: 50, y: 560 },
    },
  ],
  edges: [
    { id: 'e1', source: 'input-1', target: 'decision-1', animated: true },
    { id: 'e2', source: 'decision-1', target: 'action-1', label: '> 100' },
    { id: 'e3', source: 'decision-1', target: 'decision-2', label: '≤ 100' },
    { id: 'e4', source: 'decision-2', target: 'action-2', label: '> 50' },
    { id: 'e5', source: 'decision-2', target: 'action-3', label: '≤ 50' },
  ],
};

// ========================================
// 9. Webhook 수신 워크플로우
// ========================================
export const webhookReceiverTemplate: WorkflowTemplate = {
  id: 'webhook-receiver',
  name: 'Webhook 수신 워크플로우',
  description: '외부 시스템으로부터 Webhook 수신 및 처리',
  category: 'integration',
  tags: ['Webhook', '수신', '연동'],
  nodes: [
    {
      id: 'input-1',
      type: 'custom',
      data: { label: 'Webhook 수신', type: 'input', description: 'POST /webhook' },
      position: { x: 250, y: 50 },
    },
    {
      id: 'action-1',
      type: 'custom',
      data: { label: '페이로드 파싱', type: 'action', description: 'JSON 파싱' },
      position: { x: 250, y: 180 },
    },
    {
      id: 'decision-1',
      type: 'custom',
      data: {
        label: '시그니처 검증',
        type: 'decision',
        description: 'HMAC 검증',
        rule: 'verifySignature(payload)',
      },
      position: { x: 250, y: 310 },
    },
    {
      id: 'action-2',
      type: 'custom',
      data: { label: '비즈니스 로직', type: 'action', description: '이벤트 처리' },
      position: { x: 450, y: 440 },
    },
    {
      id: 'action-3',
      type: 'custom',
      data: { label: '에러 응답', type: 'action', description: '401 Unauthorized' },
      position: { x: 50, y: 440 },
    },
  ],
  edges: [
    { id: 'e1', source: 'input-1', target: 'action-1', animated: true },
    { id: 'e2', source: 'action-1', target: 'decision-1', animated: true },
    { id: 'e3', source: 'decision-1', target: 'action-2', label: '유효' },
    { id: 'e4', source: 'decision-1', target: 'action-3', label: '무효' },
  ],
};

// ========================================
// 10. 멀티스텝 승인 워크플로우
// ========================================
export const approvalWorkflowTemplate: WorkflowTemplate = {
  id: 'approval-workflow',
  name: '멀티스텝 승인 워크플로우',
  description: '단계별 승인 및 에스컬레이션',
  category: 'advanced',
  tags: ['승인', '에스컬레이션', '워크플로우'],
  nodes: [
    {
      id: 'input-1',
      type: 'custom',
      data: { label: '승인 요청', type: 'input', description: '사용자 요청' },
      position: { x: 250, y: 50 },
    },
    {
      id: 'decision-1',
      type: 'custom',
      data: {
        label: '1차 승인',
        type: 'decision',
        description: '팀장 승인',
        rule: 'manager.approve === true',
      },
      position: { x: 250, y: 180 },
    },
    {
      id: 'decision-2',
      type: 'custom',
      data: {
        label: '2차 승인',
        type: 'decision',
        description: '부서장 승인',
        rule: 'director.approve === true',
      },
      position: { x: 450, y: 310 },
    },
    {
      id: 'action-1',
      type: 'custom',
      data: { label: '최종 승인', type: 'action', description: '요청 처리 실행' },
      position: { x: 650, y: 440 },
    },
    {
      id: 'action-2',
      type: 'custom',
      data: { label: '반려', type: 'action', description: '요청 거부 및 통보' },
      position: { x: 50, y: 310 },
    },
  ],
  edges: [
    { id: 'e1', source: 'input-1', target: 'decision-1', animated: true },
    { id: 'e2', source: 'decision-1', target: 'decision-2', label: '승인' },
    { id: 'e3', source: 'decision-1', target: 'action-2', label: '반려' },
    { id: 'e4', source: 'decision-2', target: 'action-1', label: '승인' },
    { id: 'e5', source: 'decision-2', target: 'action-2', label: '반려' },
  ],
};

// ========================================
// All Templates Collection
// ========================================
export const ALL_TEMPLATES: WorkflowTemplate[] = [
  qualityCheckTemplate,
  apiIntegrationTemplate,
  dataTransformTemplate,
  emailSendTemplate,
  fileUploadTemplate,
  schedulingTemplate,
  loopProcessingTemplate,
  conditionalBranchingTemplate,
  webhookReceiverTemplate,
  approvalWorkflowTemplate,
];

// ========================================
// Helper Functions
// ========================================

/**
 * 카테고리별 템플릿 필터링
 */
export function getTemplatesByCategory(
  category: WorkflowTemplate['category']
): WorkflowTemplate[] {
  return ALL_TEMPLATES.filter((t) => t.category === category);
}

/**
 * 태그별 템플릿 검색
 */
export function searchTemplatesByTag(tag: string): WorkflowTemplate[] {
  return ALL_TEMPLATES.filter((t) =>
    t.tags.some((tTag) => tTag.toLowerCase().includes(tag.toLowerCase()))
  );
}

/**
 * ID로 템플릿 찾기
 */
export function getTemplateById(id: string): WorkflowTemplate | undefined {
  return ALL_TEMPLATES.find((t) => t.id === id);
}

/**
 * 템플릿을 React Flow 포맷으로 변환
 */
export function templateToReactFlow(template: WorkflowTemplate): {
  nodes: Node[];
  edges: Edge[];
} {
  return {
    nodes: template.nodes,
    edges: template.edges,
  };
}
