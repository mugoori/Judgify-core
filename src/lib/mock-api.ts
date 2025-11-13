/**
 * Mock API for Web Browser Development
 *
 * 웹 브라우저 환경에서 Tauri 백엔드 없이도
 * UI 개발 및 테스트를 가능하게 하는 Mock API입니다.
 *
 * ⚠️ 주의: 이 파일은 개발 목적으로만 사용됩니다.
 * 프로덕션 환경에서는 실제 Tauri API를 사용합니다.
 */

import type {
  ExecuteJudgmentRequest,
  JudgmentResult,
  SaveFeedbackRequest,
  BiInsightResponse,
  ChatMessageRequest,
  ChatMessageResponse,
  CreateWorkflowRequest,
  WorkflowResponse,
  SystemStatus,
  SystemStats,
  TokenMetrics,
} from './tauri-api';

// ===========================
// Judgment API Mocks
// ===========================

export const executeJudgment = async (
  request: ExecuteJudgmentRequest
): Promise<JudgmentResult> => {
  console.warn('[Mock API] executeJudgment called with:', request);

  return {
    id: `mock-judgment-${Date.now()}`,
    workflow_id: request.workflow_id,
    result: true,
    confidence: 0.92,
    method_used: request.method || 'hybrid',
    explanation:
      'Mock 판단 결과입니다. 이것은 Tauri 백엔드 없이 웹 브라우저에서 표시되는 Mock 데이터입니다.',
    created_at: new Date().toISOString(),
  };
};

export const getJudgmentHistory = async (
  workflowId?: string,
  limit: number = 50
): Promise<JudgmentResult[]> => {
  console.warn('[Mock API] getJudgmentHistory called');

  // Mock 데이터 생성 (최대 limit 개수만큼)
  const mockHistory: JudgmentResult[] = [];
  const count = Math.min(limit, 20); // 최대 20개까지만 생성

  for (let i = 0; i < count; i++) {
    mockHistory.push({
      id: `mock-judgment-${i}`,
      workflow_id: workflowId || `mock-workflow-${i % 3}`,
      result: i % 2 === 0, // 교대로 true/false
      confidence: 0.7 + Math.random() * 0.3, // 0.7 ~ 1.0
      method_used: ['rule', 'llm', 'hybrid'][i % 3],
      explanation: `Mock 판단 ${i + 1}: 샘플 설명 텍스트`,
      created_at: new Date(Date.now() - i * 3600000).toISOString(), // 1시간 간격
    });
  }

  return mockHistory;
};

// ===========================
// Learning API Mocks
// ===========================

export const saveFeedback = async (request: SaveFeedbackRequest): Promise<void> => {
  console.warn('[Mock API] saveFeedback called with:', request);
  // Mock: 아무 작업도 하지 않음 (void 반환)
  await new Promise(resolve => setTimeout(resolve, 100));
};

export const getFewShotSamples = async (
  workflowId: string,
  limit: number
): Promise<any[]> => {
  console.warn('[Mock API] getFewShotSamples called');

  const mockSamples = [];
  const count = Math.min(limit, 10);

  for (let i = 0; i < count; i++) {
    mockSamples.push({
      id: `mock-sample-${i}`,
      workflow_id: workflowId,
      input_data: { temperature: 80 + i * 2, vibration: 40 + i * 3 },
      output: true,
      confidence: 0.8 + i * 0.02,
    });
  }

  return mockSamples;
};

export const extractRules = async (workflowId: string): Promise<string[]> => {
  console.warn('[Mock API] extractRules called for:', workflowId);

  return [
    'temperature > 85 AND vibration > 45 → result = true',
    'temperature < 70 → result = false',
    'vibration > 50 → result = true (confidence: 0.8)',
  ];
};

// ===========================
// BI API Mocks
// ===========================

export const generateBiInsight = async (
  userRequest: string
): Promise<BiInsightResponse> => {
  console.warn('[Mock API] generateBiInsight called with:', userRequest);

  return {
    title: 'Mock BI 인사이트: 품질 데이터 분석',
    insights: [
      '최근 7일간 불량률이 12% → 8%로 감소했습니다.',
      '온도 센서 이상 패턴이 3건 감지되었습니다.',
      '워크플로우 A의 정확도가 95%로 가장 높습니다.',
    ],
    component_code: `
      <div className="grid grid-cols-2 gap-4">
        <div className="p-4 border rounded">
          <h3>불량률 트렌드</h3>
          <p className="text-2xl font-bold">8%</p>
        </div>
        <div className="p-4 border rounded">
          <h3>검사 건수</h3>
          <p className="text-2xl font-bold">1,247</p>
        </div>
      </div>
    `,
    recommendations: [
      '온도 센서 교정을 권장합니다.',
      '워크플로우 B의 판단 로직을 워크플로우 A 기준으로 개선하세요.',
    ],
  };
};

// ===========================
// Chat API Mocks
// ===========================

let mockSessionCounter = 0;

export const sendChatMessage = async (
  request: ChatMessageRequest
): Promise<ChatMessageResponse> => {
  console.warn('[Mock API] sendChatMessage called with:', request.message);

  const sessionId = request.session_id || `mock-session-${++mockSessionCounter}`;

  // 간단한 의도 분류
  let intent = 'general_query';
  if (request.message.includes('워크플로우')) {
    intent = 'workflow_execution';
  } else if (request.message.includes('데이터') || request.message.includes('분석')) {
    intent = 'data_visualization';
  }

  return {
    response: `Mock AI 응답: "${request.message}"에 대한 답변입니다. 이것은 웹 브라우저용 Mock 데이터입니다.`,
    session_id: sessionId,
    intent,
    action_result: {
      status: 'success',
      data: 'Mock 작업 결과',
    },
  };
};

export const getChatHistory = async (sessionId: string): Promise<any[]> => {
  console.warn('[Mock API] getChatHistory called for:', sessionId);

  return [
    {
      role: 'user',
      content: '지난 주 불량률 트렌드 보여줘',
      timestamp: new Date(Date.now() - 3600000).toISOString(),
    },
    {
      role: 'assistant',
      content: 'Mock 응답: 지난 주 불량률은 8%입니다.',
      timestamp: new Date(Date.now() - 3500000).toISOString(),
    },
  ];
};

// ===========================
// Workflow API Mocks
// ===========================

const mockWorkflows: WorkflowResponse[] = [
  {
    id: 'mock-workflow-1',
    name: 'Mock 품질 검사 워크플로우',
    definition: {
      nodes: [
        { id: 'start', type: 'START', label: '시작' },
        { id: 'judge', type: 'RULE_JUDGMENT', label: '판단' },
        { id: 'end', type: 'END', label: '종료' },
      ],
      edges: [
        { source: 'start', target: 'judge' },
        { source: 'judge', target: 'end' },
      ],
    },
    rule_expression: 'temperature > 85 AND vibration > 45',
    version: 1,
    is_active: true,
    created_at: new Date(Date.now() - 86400000).toISOString(),
  },
  {
    id: 'mock-workflow-2',
    name: 'Mock 온도 모니터링',
    definition: {
      nodes: [
        { id: 'start', type: 'START', label: '시작' },
        { id: 'decision', type: 'DECISION', label: '분기' },
      ],
      edges: [{ source: 'start', target: 'decision' }],
    },
    version: 2,
    is_active: false,
    created_at: new Date(Date.now() - 172800000).toISOString(),
  },
  {
    id: 'mock-workflow-3',
    name: 'Mock 진동 감지',
    definition: {
      nodes: [
        { id: 'start', type: 'START', label: '시작' },
        { id: 'llm', type: 'LLM_JUDGMENT', label: 'AI 판단' },
        { id: 'end', type: 'END', label: '종료' },
      ],
      edges: [
        { source: 'start', target: 'llm' },
        { source: 'llm', target: 'end' },
      ],
    },
    version: 1,
    is_active: true,
    created_at: new Date(Date.now() - 259200000).toISOString(),
  },
];

export const createWorkflow = async (
  request: CreateWorkflowRequest
): Promise<WorkflowResponse> => {
  console.warn('[Mock API] createWorkflow called with:', request.name);

  const newWorkflow: WorkflowResponse = {
    id: `mock-workflow-${Date.now()}`,
    name: request.name,
    definition: request.definition,
    rule_expression: request.rule_expression,
    version: 1,
    is_active: true,
    created_at: new Date().toISOString(),
  };

  mockWorkflows.push(newWorkflow);
  return newWorkflow;
};

export const getWorkflow = async (id: string): Promise<WorkflowResponse> => {
  console.warn('[Mock API] getWorkflow called for:', id);

  const workflow = mockWorkflows.find(w => w.id === id);
  if (workflow) {
    return workflow;
  }

  // 없으면 첫 번째 Mock 워크플로우 반환
  return mockWorkflows[0];
};

export const getAllWorkflows = async (): Promise<WorkflowResponse[]> => {
  console.warn('[Mock API] getAllWorkflows called');
  return mockWorkflows;
};

export const updateWorkflow = async (request: any): Promise<WorkflowResponse> => {
  console.warn('[Mock API] updateWorkflow called');

  const index = mockWorkflows.findIndex(w => w.id === request.id);
  if (index !== -1) {
    mockWorkflows[index] = {
      ...mockWorkflows[index],
      ...request,
      version: mockWorkflows[index].version + 1,
    };
    return mockWorkflows[index];
  }

  return mockWorkflows[0];
};

export const deleteWorkflow = async (id: string): Promise<void> => {
  console.warn('[Mock API] deleteWorkflow called for:', id);

  const index = mockWorkflows.findIndex(w => w.id === id);
  if (index !== -1) {
    mockWorkflows.splice(index, 1);
  }
};

export const validateWorkflow = async (definition: any): Promise<boolean> => {
  console.warn('[Mock API] validateWorkflow called');
  // Mock: 항상 유효하다고 가정
  return true;
};

// ===========================
// System API Mocks
// ===========================

export const getSystemStatus = async (): Promise<SystemStatus> => {
  console.warn('[Mock API] getSystemStatus called');

  return {
    database_connected: false, // Mock 환경에서는 DB 미연결
    database_path: 'Mock: 웹 브라우저 환경 (DB 없음)',
    claude_configured: false,
    version: '2.0.0-mock',
    uptime_seconds: Math.floor(performance.now() / 1000),
  };
};

export const getSystemStats = async (): Promise<SystemStats> => {
  console.warn('[Mock API] getSystemStats called');

  return {
    total_judgments: 37,
    total_workflows: 3,
    total_training_samples: 15,
    average_confidence: 0.87,
  };
};

export const getDataDirectory = async (): Promise<string> => {
  console.warn('[Mock API] getDataDirectory called');
  return 'Mock: 웹 브라우저 환경 (데이터 디렉토리 없음)';
};

export const exportDatabase = async (exportPath: string): Promise<void> => {
  console.warn('[Mock API] exportDatabase called for:', exportPath);
  // Mock: 아무 작업도 하지 않음
  await new Promise(resolve => setTimeout(resolve, 100));
};

// ===========================
// Token Metrics API Mocks
// ===========================

export const getTokenMetrics = async (): Promise<TokenMetrics> => {
  console.warn('[Mock API] getTokenMetrics called');

  return {
    total_tokens_used: 125430,
    total_cost_usd: 0.87,
    tokens_saved_by_cache: 45200,
    cost_saved_usd: 0.31,
    cache_hit_rate: 0.36,
    avg_tokens_per_request: 1520,
  };
};
