/**
 * 시뮬레이션 결과 컴포넌트 통합 모듈
 */

// 단계별 결과 컴포넌트
export { TriggerResult } from './steps/TriggerResult';
export { QueryResult } from './steps/QueryResult';
export { CalcResult } from './steps/CalcResult';
export { JudgmentResult } from './steps/JudgmentResult';
export { ApprovalResult } from './steps/ApprovalResult';
export { AlertResult } from './steps/AlertResult';

// 타입 재export
export type {
  TriggerData,
  QueryData,
  CalcData,
  JudgmentData,
  ApprovalData,
  AlertData,
  StepMockData,
} from '../../lib/mock-data/types';
