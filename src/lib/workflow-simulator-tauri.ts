import { invoke } from '@tauri-apps/api/tauri';
import type { Node, Edge } from 'reactflow';
import type { SimulationState, SimulationStep } from './workflow-simulator';

export interface SimulationStepRequest {
  workflow_id: string;
  nodes: Node[];
  edges: Edge[];
  current_node_id: string;
  global_data: Record<string, any>;
}

export interface SimulationStepResponse {
  node_id: string;
  node_name: string;
  node_type: string;
  status: 'success' | 'error' | 'running';
  input: Record<string, any>;
  output: Record<string, any> | null;
  error: string | null;
  execution_time_ms: number;
  next_node_id: string | null;
}

export class TauriWorkflowSimulator {
  private workflowId: string;
  private nodes: Node[];
  private edges: Edge[];
  private initialData: Record<string, any>;
  private state: SimulationState;
  private nextNodeId: string | null = null;

  constructor(
    nodes: Node[],
    edges: Edge[],
    initialData: Record<string, any>,
    workflowId?: string
  ) {
    const timestamp = new Date().getTime();
    this.workflowId = workflowId || `wf-${timestamp}`;
    this.nodes = nodes;
    this.edges = edges;
    this.initialData = { ...initialData };

    this.state = {
      currentStepIndex: -1,
      steps: [],
      globalData: { ...initialData },
      isRunning: false,
      isPaused: false,
    };
  }

  async start(): Promise<SimulationState> {
    const firstNode = this.nodes.find(n =>
      n.type === 'data-input' || n.type === 'input'
    );

    if (!firstNode) {
      console.error('[TauriSimulator] No input node found');
      return this.state;
    }

    this.state.isRunning = true;
    this.state.isPaused = false;
    this.state.steps = [];
    this.state.currentStepIndex = -1;
    this.state.globalData = { ...this.initialData };
    this.nextNodeId = null;

    return await this.stepForward();
  }

  async stepForward(): Promise<SimulationState> {
    if (!this.state.isRunning) {
      return this.state;
    }

    let currentNodeId: string;

    if (this.state.currentStepIndex === -1) {
      const firstNode = this.nodes.find(n =>
        n.type === 'data-input' || n.type === 'input'
      );
      if (!firstNode) return this.state;
      currentNodeId = firstNode.id;
    } else {
      if (!this.nextNodeId) {
        this.state.isRunning = false;
        return this.state;
      }
      currentNodeId = this.nextNodeId;
    }

    try {
      const request: SimulationStepRequest = {
        workflow_id: this.workflowId,
        nodes: this.nodes as any,
        edges: this.edges as any,
        current_node_id: currentNodeId,
        global_data: this.state.globalData,
      };

      const response = await invoke<SimulationStepResponse>(
        'simulate_workflow_step',
        { request }
      );

      if (response.output) {
        this.state.globalData = {
          ...this.state.globalData,
          ...response.output,
        };
      }

      const step: SimulationStep = {
        nodeId: response.node_id,
        nodeName: response.node_name,
        nodeType: response.node_type,
        status: response.status,
        input: response.input,
        output: response.output || {},
        error: response.error || undefined,
        executionTimeMs: response.execution_time_ms,
        timestamp: new Date().toISOString(),
      };

      this.state.steps.push(step);
      this.state.currentStepIndex = this.state.steps.length - 1;
      this.nextNodeId = response.next_node_id;

      if (!response.next_node_id ||
          response.node_type === 'data-output' ||
          response.node_type === 'output') {
        this.state.isRunning = false;
      }

      return { ...this.state };
    } catch (error) {
      console.error('[TauriSimulator] Failed:', error);

      const errorStep: SimulationStep = {
        nodeId: currentNodeId,
        nodeName: 'Error',
        nodeType: 'error',
        status: 'error',
        input: this.state.globalData,
        output: {},
        error: error instanceof Error ? error.message : String(error),
        executionTimeMs: 0,
        timestamp: new Date().toISOString(),
      };

      this.state.steps.push(errorStep);
      this.state.currentStepIndex = this.state.steps.length - 1;
      this.state.isRunning = false;

      return { ...this.state };
    }
  }

  stepBackward(): SimulationState {
    if (this.state.currentStepIndex > 0) {
      this.state.currentStepIndex--;
    }
    return { ...this.state };
  }

  pause(): SimulationState {
    this.state.isPaused = true;
    return { ...this.state };
  }

  resume(): SimulationState {
    this.state.isPaused = false;
    return { ...this.state };
  }

  reset(): SimulationState {
    this.state = {
      currentStepIndex: -1,
      steps: [],
      globalData: { ...this.initialData },
      isRunning: false,
      isPaused: false,
    };
    this.nextNodeId = null;
    return { ...this.state };
  }

  getState(): SimulationState {
    return { ...this.state };
  }
}
