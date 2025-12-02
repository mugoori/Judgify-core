/**
 * 워크플로우 시뮬레이션 Mock 데이터 타입 정의
 */

// 트리거 데이터
export interface TriggerData {
  type: 'event' | 'schedule' | 'manual';
  eventName: string;
  timestamp: string;
  target: string;
  data?: Record<string, string | number>;
}

// 조회 결과 항목
export interface QueryResultItem {
  item: string;
  standard: string;
  actual: string | number;
  pass: boolean;
}

// 조회 데이터
export interface QueryData {
  tableName: string;
  results: QueryResultItem[];
  totalCount: number;
}

// 계산 데이터
export interface CalcData {
  formula: string;
  inputs: Record<string, number>;
  result: number;
  unit: string;
  description: string;
}

// 판정 데이터
export interface JudgmentData {
  result: boolean;
  method: 'Rule Engine' | 'LLM' | 'Hybrid';
  confidence: number;
  explanation: string;
}

// 승인 데이터
export interface ApprovalData {
  approver: string;
  role: string;
  status: 'pending' | 'approved' | 'rejected';
  approvedAt: string | null;
  comment?: string;
}

// 알림 데이터
export interface AlertData {
  sent: boolean;
  recipients: string[];
  channel: 'email' | 'slack' | 'sms' | 'system';
  sentAt: string;
  message: string;
}

// 단계별 Mock 데이터 통합
export interface StepMockData {
  trigger?: TriggerData;
  query?: QueryData;
  calc?: CalcData;
  judgment?: JudgmentData;
  approval?: ApprovalData;
  alert?: AlertData;
}

// 템플릿별 전체 Mock 데이터
export interface TemplateMockData {
  templateId: string;
  templateName: string;
  steps: StepMockData[];
}

// 지원 템플릿 ID
export type TemplateId =
  | 'pasteurization-ccp'
  | 'metal-detection-ccp'
  | 'material-inspection'
  | 'release-approval'
  | 'shelf-life-monitoring'
  | 'inventory-forecast'
  | 'preventive-maintenance'
  | 'mrp-calculation'
  | 'bom-cost-management'
  | 'production-forecast';
