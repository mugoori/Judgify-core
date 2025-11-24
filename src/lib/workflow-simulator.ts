/**
 * Workflow Simulator Types (Phase 9 - types only)
 * 실제 시뮬레이션 로직은 WorkflowBuilderV2에서 처리됩니다.
 */

export type SimulationState = 'idle' | 'running' | 'paused' | 'completed' | 'error';

export interface SimulationStep {
  nodeId: string;
  nodeName: string;
  nodeType: string;
  timestamp: number;
  duration_ms: number;
  status: 'success' | 'error' | 'skipped';
  input: Record<string, any>;
  output: Record<string, any>;
  error?: string;
}
