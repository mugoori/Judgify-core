/**
 * Workflow Type Definitions (Week 5)
 *
 * 노드 타입을 4가지에서 7가지로 확장하되, 기존 v1 워크플로우와의 하위 호환성 유지
 */

import { Node, Edge } from 'reactflow';

/**
 * 노드 타입 Enum (v2)
 *
 * 기존 타입 (v1 호환):
 * - INPUT: 데이터 입력 노드
 * - DECISION: 조건 분기 노드
 * - ACTION: 작업 실행 노드
 * - OUTPUT: 결과 출력 노드
 *
 * 신규 타입 (Week 5):
 * - DATA_INPUT: 외부 데이터 소스 연동 (DB, API 등)
 * - RULE_JUDGMENT: Rule Engine 기반 판단 (AST 파싱)
 * - LLM_JUDGMENT: LLM 기반 판단 (Claude API)
 * - ACTION_EXECUTION: 외부 시스템 작업 실행 (Slack, Email 등)
 * - NOTIFICATION: 알림 전송 (Slack, Teams, Email)
 * - DATA_AGGREGATION: 데이터 집계 및 변환
 */
export enum NodeType {
  // 기존 타입 (v1 호환)
  INPUT = 'input',
  DECISION = 'decision',
  ACTION = 'action',
  OUTPUT = 'output',

  // 신규 타입 (Week 5)
  DATA_INPUT = 'data_input',
  RULE_JUDGMENT = 'rule_judgment',
  LLM_JUDGMENT = 'llm_judgment',
  ACTION_EXECUTION = 'action_execution',
  NOTIFICATION = 'notification',
  DATA_AGGREGATION = 'data_aggregation',
}

/**
 * 레거시 노드 타입 (v1)
 * @deprecated Use NodeType enum instead
 */
export type LegacyNodeType = 'input' | 'decision' | 'action' | 'output';

/**
 * 하위 호환성 타입 가드
 * v1 워크플로우의 노드 타입 여부 확인
 */
export const isLegacyNodeType = (type: string): type is LegacyNodeType => {
  return ['input', 'decision', 'action', 'output'].includes(type);
};

/**
 * v2 확장 노드 타입 여부 확인
 */
export const isExtendedNodeType = (type: string): boolean => {
  return [
    'data_input',
    'rule_judgment',
    'llm_judgment',
    'action_execution',
    'notification',
    'data_aggregation',
  ].includes(type);
};

/**
 * 노드 타입 검증 (v1 + v2 모두 지원)
 */
export const isValidNodeType = (type: string): type is NodeType => {
  return Object.values(NodeType).includes(type as NodeType);
};

/**
 * 노드 데이터 공통 인터페이스
 */
export interface BaseNodeData {
  label: string;
  description?: string;
  icon?: string;
  color?: string;
}

/**
 * INPUT 노드 데이터
 */
export interface InputNodeData extends BaseNodeData {
  type: NodeType.INPUT;
  variableName: string; // 입력 변수명 (예: "temperature", "stock_level")
  dataType: 'number' | 'string' | 'boolean' | 'object';
  defaultValue?: any;
}

/**
 * DATA_INPUT 노드 데이터 (신규)
 */
export interface DataInputNodeData extends BaseNodeData {
  type: NodeType.DATA_INPUT;
  source: 'database' | 'api' | 'file' | 'webhook';
  connection: {
    url?: string;
    query?: string;
    method?: 'GET' | 'POST' | 'PUT' | 'DELETE';
    headers?: Record<string, string>;
  };
  mapping: Record<string, string>; // 데이터 필드 매핑
}

/**
 * DECISION 노드 데이터 (v1)
 */
export interface DecisionNodeData extends BaseNodeData {
  type: NodeType.DECISION;
  condition: string; // 조건 표현식 (예: "temperature > 90")
  operator?: '>' | '<' | '>=' | '<=' | '==' | '!=';
  value?: any;
}

/**
 * RULE_JUDGMENT 노드 데이터 (신규)
 */
export interface RuleJudgmentNodeData extends BaseNodeData {
  type: NodeType.RULE_JUDGMENT;
  ruleExpression: string; // Rhai AST 기반 규칙 표현식
  confidence?: number; // 신뢰도 임계값 (0-1)
  useAST?: boolean; // AST 검증 활성화 여부
}

/**
 * LLM_JUDGMENT 노드 데이터 (신규)
 */
export interface LLMJudgmentNodeData extends BaseNodeData {
  type: NodeType.LLM_JUDGMENT;
  prompt: string; // LLM 프롬프트 템플릿
  model?: string; // 모델명 (기본값: claude-3-5-sonnet-20241022)
  temperature?: number; // 생성 온도 (0-2)
  maxTokens?: number; // 최대 토큰 수
  context?: Record<string, any>; // 추가 컨텍스트
}

/**
 * ACTION 노드 데이터 (v1)
 */
export interface ActionNodeData extends BaseNodeData {
  type: NodeType.ACTION;
  actionType: string; // 작업 타입 (예: "send_notification", "update_database")
  parameters?: Record<string, any>;
}

/**
 * ACTION_EXECUTION 노드 데이터 (신규)
 */
export interface ActionExecutionNodeData extends BaseNodeData {
  type: NodeType.ACTION_EXECUTION;
  service: 'slack' | 'email' | 'webhook' | 'database' | 'custom';
  action: string; // 실행할 작업 (예: "send_message", "create_record")
  config: Record<string, any>; // 서비스별 설정
  retryPolicy?: {
    maxRetries: number;
    backoffMs: number;
  };
}

/**
 * NOTIFICATION 노드 데이터 (신규)
 */
export interface NotificationNodeData extends BaseNodeData {
  type: NodeType.NOTIFICATION;
  channel: 'slack' | 'teams' | 'email' | 'sms' | 'webhook';
  recipients: string[]; // 수신자 목록
  template: string; // 메시지 템플릿
  priority?: 'low' | 'medium' | 'high' | 'urgent';
}

/**
 * DATA_AGGREGATION 노드 데이터 (신규)
 */
export interface DataAggregationNodeData extends BaseNodeData {
  type: NodeType.DATA_AGGREGATION;
  aggregationType: 'sum' | 'avg' | 'min' | 'max' | 'count' | 'group_by';
  sourceField: string; // 집계 대상 필드
  groupByField?: string; // 그룹화 필드 (GROUP BY용)
  filter?: string; // 필터 조건
}

/**
 * OUTPUT 노드 데이터 (v1)
 */
export interface OutputNodeData extends BaseNodeData {
  type: NodeType.OUTPUT;
  outputFormat: 'json' | 'text' | 'html' | 'markdown';
  template?: string; // 출력 템플릿
}

/**
 * 노드 데이터 Union Type
 */
export type WorkflowNodeData =
  | InputNodeData
  | DataInputNodeData
  | DecisionNodeData
  | RuleJudgmentNodeData
  | LLMJudgmentNodeData
  | ActionNodeData
  | ActionExecutionNodeData
  | NotificationNodeData
  | DataAggregationNodeData
  | OutputNodeData;

/**
 * 워크플로우 노드 (React Flow Node + 커스텀 데이터)
 */
export type WorkflowNode = Node<WorkflowNodeData>;

/**
 * 워크플로우 정의
 */
export interface WorkflowDefinition {
  id: string;
  name: string;
  description?: string;
  nodes: WorkflowNode[];
  edges: Edge[];
  version: number; // 버전 (v1 = 1, v2 = 2)
  createdAt: string;
  updatedAt: string;
  isActive: boolean;
  metadata?: {
    author?: string;
    tags?: string[];
    useASTValidation?: boolean; // AST 검증 활성화 여부 (v2)
  };
}

/**
 * 워크플로우 실행 결과
 */
export interface WorkflowExecutionResult {
  workflowId: string;
  executionId: string;
  startTime: string;
  endTime: string;
  status: 'success' | 'failure' | 'partial';
  steps: {
    nodeId: string;
    nodeType: NodeType;
    status: 'success' | 'failure' | 'skipped';
    input?: any;
    output?: any;
    error?: string;
    executionTimeMs: number;
  }[];
  finalResult: any;
  totalExecutionTimeMs: number;
}

/**
 * v1 → v2 마이그레이션 헬퍼
 * 기존 워크플로우를 새 타입 시스템으로 변환
 */
export const migrateWorkflowV1ToV2 = (
  v1Workflow: any
): WorkflowDefinition => {
  return {
    ...v1Workflow,
    version: 2,
    nodes: v1Workflow.nodes.map((node: any) => ({
      ...node,
      data: {
        ...node.data,
        type: node.data.type as NodeType, // String → Enum 변환
      },
    })),
    metadata: {
      ...v1Workflow.metadata,
      useASTValidation: false, // 기존 워크플로우는 AST 검증 비활성화
    },
  };
};

/**
 * 노드 타입별 기본 설정
 */
export const NodeTypeDefaults: Record<NodeType, Partial<BaseNodeData>> = {
  [NodeType.INPUT]: {
    label: '입력',
    icon: 'Input',
    color: '#3b82f6', // blue-500
  },
  [NodeType.DATA_INPUT]: {
    label: '데이터 소스',
    icon: 'Database',
    color: '#8b5cf6', // violet-500
  },
  [NodeType.DECISION]: {
    label: '조건 분기',
    icon: 'GitBranch',
    color: '#f59e0b', // amber-500
  },
  [NodeType.RULE_JUDGMENT]: {
    label: 'Rule 판단',
    icon: 'FileCode',
    color: '#10b981', // emerald-500
  },
  [NodeType.LLM_JUDGMENT]: {
    label: 'AI 판단',
    icon: 'Brain',
    color: '#06b6d4', // cyan-500
  },
  [NodeType.ACTION]: {
    label: '작업',
    icon: 'Play',
    color: '#ef4444', // red-500
  },
  [NodeType.ACTION_EXECUTION]: {
    label: '작업 실행',
    icon: 'Zap',
    color: '#f97316', // orange-500
  },
  [NodeType.NOTIFICATION]: {
    label: '알림',
    icon: 'Bell',
    color: '#ec4899', // pink-500
  },
  [NodeType.DATA_AGGREGATION]: {
    label: '데이터 집계',
    icon: 'BarChart',
    color: '#14b8a6', // teal-500
  },
  [NodeType.OUTPUT]: {
    label: '출력',
    icon: 'FileOutput',
    color: '#6366f1', // indigo-500
  },
};
