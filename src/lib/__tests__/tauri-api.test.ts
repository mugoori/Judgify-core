import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/tauri';
import {
  // Judgment API
  executeJudgment,
  getJudgmentHistory,
  type ExecuteJudgmentRequest,
  type JudgmentResult,
  // Learning API
  saveFeedback,
  getFewShotSamples,
  extractRules,
  type SaveFeedbackRequest,
  // BI API
  generateBiInsight,
  type BiInsightResponse,
  // Chat API
  sendChatMessage,
  getChatHistory,
  type ChatMessageRequest,
  type ChatMessageResponse,
  // Workflow API
  createWorkflow,
  getWorkflow,
  getAllWorkflows,
  // updateWorkflow, // Used for type reference only
  deleteWorkflow,
  validateWorkflow,
  type CreateWorkflowRequest,
  type WorkflowResponse,
  // System API
  getSystemStatus,
  getSystemStats,
  getDataDirectory,
  exportDatabase,
  type SystemStatus,
  type SystemStats,
  // Token Metrics API
  getTokenMetrics,
  type TokenMetrics,
} from '../tauri-api';

// Tauri invoke 모킹 (useRuleValidation에서 확립한 패턴)
vi.mock('@tauri-apps/api/tauri', () => ({
  invoke: vi.fn(),
}));

describe('tauri-api', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Judgment API', () => {
    it('executeJudgment - 판단 실행 성공', async () => {
      const mockRequest: ExecuteJudgmentRequest = {
        workflow_id: 'workflow-123',
        input_data: { temperature: 90 },
        method: 'hybrid',
      };

      const mockResult: JudgmentResult = {
        id: 'judgment-456',
        workflow_id: 'workflow-123',
        result: true,
        confidence: 0.95,
        method_used: 'rule',
        explanation: 'Temperature exceeds threshold',
        created_at: '2025-11-06T10:00:00Z',
      };

      vi.mocked(invoke).mockResolvedValue(mockResult);

      const result = await executeJudgment(mockRequest);

      expect(invoke).toHaveBeenCalledWith('execute_judgment', { request: mockRequest });
      expect(result).toEqual(mockResult);
      expect(result.confidence).toBeGreaterThanOrEqual(0.9);
    });

    it('getJudgmentHistory - 히스토리 조회 성공', async () => {
      const mockHistory: JudgmentResult[] = [
        {
          id: 'judgment-1',
          workflow_id: 'workflow-123',
          result: true,
          confidence: 0.92,
          method_used: 'hybrid',
          explanation: 'Test 1',
          created_at: '2025-11-06T09:00:00Z',
        },
        {
          id: 'judgment-2',
          workflow_id: 'workflow-123',
          result: false,
          confidence: 0.88,
          method_used: 'rule',
          explanation: 'Test 2',
          created_at: '2025-11-06T10:00:00Z',
        },
      ];

      vi.mocked(invoke).mockResolvedValue(mockHistory);

      const result = await getJudgmentHistory('workflow-123', 10);

      expect(invoke).toHaveBeenCalledWith('get_judgment_history', {
        workflowId: 'workflow-123',
        limit: 10,
      });
      expect(result).toHaveLength(2);
      expect(result[0].workflow_id).toBe('workflow-123');
    });

    it('executeJudgment - 네트워크 에러 처리', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('Network error'));

      await expect(
        executeJudgment({
          workflow_id: 'workflow-123',
          input_data: { temperature: 90 },
        })
      ).rejects.toThrow('Network error');
    });
  });

  describe('Learning API', () => {
    it('saveFeedback - 피드백 저장 성공', async () => {
      const mockRequest: SaveFeedbackRequest = {
        judgment_id: 'judgment-123',
        feedback_type: 'thumbs_up',
        value: 1,
        comment: 'Good judgment!',
      };

      vi.mocked(invoke).mockResolvedValue(undefined);

      await saveFeedback(mockRequest);

      expect(invoke).toHaveBeenCalledWith('save_feedback', { request: mockRequest });
    });

    it('getFewShotSamples - Few-shot 샘플 조회', async () => {
      const mockSamples = [
        { input: { temp: 80 }, output: true },
        { input: { temp: 90 }, output: false },
      ];

      vi.mocked(invoke).mockResolvedValue(mockSamples);

      const result = await getFewShotSamples('workflow-123', 5);

      expect(invoke).toHaveBeenCalledWith('get_few_shot_samples', {
        request: { workflow_id: 'workflow-123', limit: 5 },
      });
      expect(result).toHaveLength(2);
    });

    it('extractRules - Rule 추출 성공', async () => {
      const mockRules = [
        'if temperature > 80 then alert',
        'if vibration > 50 then warning',
      ];

      vi.mocked(invoke).mockResolvedValue(mockRules);

      const result = await extractRules('workflow-123');

      expect(invoke).toHaveBeenCalledWith('extract_rules', { workflowId: 'workflow-123' });
      expect(result).toHaveLength(2);
      expect(result[0]).toContain('temperature');
    });
  });

  describe('BI API', () => {
    it('generateBiInsight - BI 인사이트 생성', async () => {
      const mockInsight: BiInsightResponse = {
        title: 'Inventory Analysis',
        insights: ['Stock level is low', 'Reorder point reached'],
        component_code: '<div>Chart Component</div>',
        recommendations: ['Order more items', 'Check supplier'],
      };

      vi.mocked(invoke).mockResolvedValue(mockInsight);

      const result = await generateBiInsight('Show me inventory trends');

      expect(invoke).toHaveBeenCalledWith('generate_bi_insight', {
        request: { user_request: 'Show me inventory trends' },
      });
      expect(result.title).toBe('Inventory Analysis');
      expect(result.insights).toHaveLength(2);
      expect(result.recommendations).toHaveLength(2);
    });
  });

  describe('Chat API', () => {
    it('sendChatMessage - 채팅 메시지 전송', async () => {
      const mockRequest: ChatMessageRequest = {
        message: 'Hello, Claude!',
        session_id: 'session-123',
      };

      const mockResponse: ChatMessageResponse = {
        response: 'Hello! How can I help?',
        session_id: 'session-123',
        intent: 'greeting',
        action_result: null,
      };

      vi.mocked(invoke).mockResolvedValue(mockResponse);

      const result = await sendChatMessage(mockRequest);

      expect(invoke).toHaveBeenCalledWith('send_chat_message', { request: mockRequest });
      expect(result.response).toContain('Hello');
      expect(result.session_id).toBe('session-123');
    });

    it('getChatHistory - 채팅 히스토리 조회', async () => {
      const mockHistory = [
        { role: 'user', content: 'Hello' },
        { role: 'assistant', content: 'Hi there!' },
      ];

      vi.mocked(invoke).mockResolvedValue(mockHistory);

      const result = await getChatHistory('session-123');

      expect(invoke).toHaveBeenCalledWith('get_chat_history', { sessionId: 'session-123' });
      expect(result).toHaveLength(2);
    });
  });

  describe('Workflow API', () => {
    it('createWorkflow - 워크플로우 생성', async () => {
      const mockRequest: CreateWorkflowRequest = {
        name: 'Test Workflow',
        definition: { nodes: [], edges: [] },
        rule_expression: 'temperature > 80',
      };

      const mockResponse: WorkflowResponse = {
        id: 'workflow-123',
        name: 'Test Workflow',
        definition: { nodes: [], edges: [] },
        rule_expression: 'temperature > 80',
        version: 1,
        is_active: true,
        created_at: '2025-11-06T10:00:00Z',
      };

      vi.mocked(invoke).mockResolvedValue(mockResponse);

      const result = await createWorkflow(mockRequest);

      expect(invoke).toHaveBeenCalledWith('create_workflow', { request: mockRequest });
      expect(result.id).toBe('workflow-123');
      expect(result.version).toBe(1);
    });

    it('getWorkflow - 워크플로우 조회', async () => {
      const mockWorkflow: WorkflowResponse = {
        id: 'workflow-123',
        name: 'Test Workflow',
        definition: { nodes: [], edges: [] },
        version: 1,
        is_active: true,
        created_at: '2025-11-06T10:00:00Z',
      };

      vi.mocked(invoke).mockResolvedValue(mockWorkflow);

      const result = await getWorkflow('workflow-123');

      expect(invoke).toHaveBeenCalledWith('get_workflow', { id: 'workflow-123' });
      expect(result.id).toBe('workflow-123');
    });

    it('getAllWorkflows - 모든 워크플로우 조회', async () => {
      const mockWorkflows: WorkflowResponse[] = [
        {
          id: 'workflow-1',
          name: 'Workflow 1',
          definition: {},
          version: 1,
          is_active: true,
          created_at: '2025-11-06T09:00:00Z',
        },
        {
          id: 'workflow-2',
          name: 'Workflow 2',
          definition: {},
          version: 1,
          is_active: false,
          created_at: '2025-11-06T10:00:00Z',
        },
      ];

      vi.mocked(invoke).mockResolvedValue(mockWorkflows);

      const result = await getAllWorkflows();

      expect(invoke).toHaveBeenCalledWith('get_all_workflows');
      expect(result).toHaveLength(2);
    });

    it('validateWorkflow - 워크플로우 검증', async () => {
      vi.mocked(invoke).mockResolvedValue(true);

      const result = await validateWorkflow({ nodes: [], edges: [] });

      expect(invoke).toHaveBeenCalledWith('validate_workflow', {
        definition: { nodes: [], edges: [] },
      });
      expect(result).toBe(true);
    });

    it('deleteWorkflow - 워크플로우 삭제', async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);

      await deleteWorkflow('workflow-123');

      expect(invoke).toHaveBeenCalledWith('delete_workflow', { id: 'workflow-123' });
    });
  });

  describe('System API', () => {
    it('getSystemStatus - 시스템 상태 조회', async () => {
      const mockStatus: SystemStatus = {
        database_connected: true,
        database_path: '/path/to/db.sqlite',
        claude_configured: true,
        version: '0.1.0',
        uptime_seconds: 3600,
      };

      vi.mocked(invoke).mockResolvedValue(mockStatus);

      const result = await getSystemStatus();

      expect(invoke).toHaveBeenCalledWith('get_system_status');
      expect(result.database_connected).toBe(true);
      expect(result.claude_configured).toBe(true);
      expect(result.uptime_seconds).toBeGreaterThan(0);
    });

    it('getSystemStats - 시스템 통계 조회', async () => {
      const mockStats: SystemStats = {
        total_judgments: 150,
        total_workflows: 5,
        total_training_samples: 80,
        average_confidence: 0.87,
      };

      vi.mocked(invoke).mockResolvedValue(mockStats);

      const result = await getSystemStats();

      expect(invoke).toHaveBeenCalledWith('get_system_stats');
      expect(result.total_judgments).toBeGreaterThan(0);
      expect(result.average_confidence).toBeGreaterThanOrEqual(0);
      expect(result.average_confidence).toBeLessThanOrEqual(1);
    });

    it('getDataDirectory - 데이터 디렉토리 경로', async () => {
      vi.mocked(invoke).mockResolvedValue('/home/user/data');

      const result = await getDataDirectory();

      expect(invoke).toHaveBeenCalledWith('get_data_directory');
      expect(result).toContain('data');
    });

    it('exportDatabase - 데이터베이스 내보내기', async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);

      await exportDatabase('/path/to/export.db');

      expect(invoke).toHaveBeenCalledWith('export_database', {
        exportPath: '/path/to/export.db',
      });
    });
  });

  describe('Token Metrics API', () => {
    it('getTokenMetrics - 토큰 메트릭 조회', async () => {
      const mockMetrics: TokenMetrics = {
        total_tokens_used: 50000,
        total_cost_usd: 0.75,
        tokens_saved_by_cache: 20000,
        cost_saved_usd: 0.30,
        cache_hit_rate: 0.40,
        avg_tokens_per_request: 250,
      };

      vi.mocked(invoke).mockResolvedValue(mockMetrics);

      const result = await getTokenMetrics();

      expect(invoke).toHaveBeenCalledWith('get_token_metrics');
      expect(result.total_tokens_used).toBeGreaterThan(0);
      expect(result.cache_hit_rate).toBeGreaterThanOrEqual(0);
      expect(result.cache_hit_rate).toBeLessThanOrEqual(1);
      expect(result.cost_saved_usd).toBeGreaterThan(0);
    });
  });

  describe('Error Handling', () => {
    it('네트워크 타임아웃 처리', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('Timeout'));

      await expect(getSystemStatus()).rejects.toThrow('Timeout');
    });

    it('잘못된 응답 형식 처리', async () => {
      vi.mocked(invoke).mockResolvedValue(null);

      const result = await getSystemStatus();

      expect(result).toBeNull();
    });
  });
});
