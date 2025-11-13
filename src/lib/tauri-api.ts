import { invoke } from '@tauri-apps/api/tauri';

// Judgment API
export interface ExecuteJudgmentRequest {
  workflow_id: string;
  input_data: Record<string, any>;
  method?: 'rule' | 'llm' | 'hybrid';
}

export interface JudgmentResult {
  id: string;
  workflow_id: string;
  result: boolean;
  confidence: number;
  method_used: string;
  explanation: string;
  created_at: string;
}

export const executeJudgment = (request: ExecuteJudgmentRequest): Promise<JudgmentResult> =>
  invoke('execute_judgment', { request });

export const getJudgmentHistory = (
  workflowId?: string,
  limit?: number
): Promise<JudgmentResult[]> =>
  invoke('get_judgment_history', { workflowId, limit });

// Learning API
export interface SaveFeedbackRequest {
  judgment_id: string;
  feedback_type: string;
  value: number;
  comment?: string;
}

export const saveFeedback = (request: SaveFeedbackRequest): Promise<void> =>
  invoke('save_feedback', { request });

export const getFewShotSamples = (
  workflowId: string,
  limit: number
): Promise<any[]> =>
  invoke('get_few_shot_samples', { request: { workflow_id: workflowId, limit } });

export const extractRules = (workflowId: string): Promise<string[]> =>
  invoke('extract_rules', { workflowId });

// BI API
export interface BiInsightResponse {
  title: string;
  insights: string[];
  component_code: string;
  recommendations: string[];
}

export const generateBiInsight = (userRequest: string): Promise<BiInsightResponse> =>
  invoke('generate_bi_insight', { request: { user_request: userRequest } });

// Chat API
export interface ChatMessageRequest {
  message: string;
  session_id?: string;
}

export interface ChatMessageResponse {
  response: string;
  session_id: string;
  intent: string;
  action_result?: any;
}

export const sendChatMessage = (request: ChatMessageRequest): Promise<ChatMessageResponse> =>
  invoke('send_chat_message', { request });

export const getChatHistory = (sessionId: string): Promise<any[]> =>
  invoke('get_chat_history', { sessionId });

// Workflow API
export interface CreateWorkflowRequest {
  name: string;
  definition: any;
  rule_expression?: string;
}

export interface WorkflowResponse {
  id: string;
  name: string;
  definition: any;
  rule_expression?: string;
  version: number;
  is_active: boolean;
  created_at: string;
}

export const createWorkflow = (request: CreateWorkflowRequest): Promise<WorkflowResponse> =>
  invoke('create_workflow', { request });

export const getWorkflow = (id: string): Promise<WorkflowResponse> =>
  invoke('get_workflow', { id });

export const getAllWorkflows = (): Promise<WorkflowResponse[]> =>
  invoke('get_all_workflows');

export const updateWorkflow = (request: any): Promise<WorkflowResponse> =>
  invoke('update_workflow', { request });

export const deleteWorkflow = (id: string): Promise<void> =>
  invoke('delete_workflow', { id });

export const validateWorkflow = (definition: any): Promise<boolean> =>
  invoke('validate_workflow', { definition });

// System API
export interface SystemStatus {
  database_connected: boolean;
  database_path: string;
  claude_configured: boolean;
  version: string;
  uptime_seconds: number;
}

export interface SystemStats {
  total_judgments: number;
  total_workflows: number;
  total_training_samples: number;
  average_confidence: number;
}

export const getSystemStatus = (): Promise<SystemStatus> =>
  invoke('get_system_status');

export const getSystemStats = (): Promise<SystemStats> =>
  invoke('get_system_stats');

export const getDataDirectory = (): Promise<string> =>
  invoke('get_data_directory');

export const exportDatabase = (exportPath: string): Promise<void> =>
  invoke('export_database', { exportPath });

// Token Metrics API
export interface TokenMetrics {
  total_tokens_used: number;
  total_cost_usd: number;
  tokens_saved_by_cache: number;
  cost_saved_usd: number;
  cache_hit_rate: number;
  avg_tokens_per_request: number;
}

export const getTokenMetrics = (): Promise<TokenMetrics> =>
  invoke('get_token_metrics');
