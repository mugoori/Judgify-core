/**
 * Workflow Step-by-Step Simulator
 *
 * 워크플로우를 단계별로 실행하며 디버깅/테스트할 수 있는 시뮬레이터
 */

import { Node, Edge } from 'reactflow';

export type NodeStatus = 'pending' | 'running' | 'success' | 'error' | 'skipped';

export interface SimulationStep {
  nodeId: string;
  nodeName: string;
  nodeType: string;
  status: NodeStatus;
  input: Record<string, any>;
  output?: Record<string, any>;
  error?: string;
  executionTimeMs?: number;
  timestamp: string;
}

export interface SimulationState {
  steps: SimulationStep[];
  currentStepIndex: number;
  isRunning: boolean;
  isPaused: boolean;
  globalData: Record<string, any>;
}

export class WorkflowSimulator {
  private nodes: Node[];
  private edges: Edge[];
  private state: SimulationState;
  private stepDelay: number;

  constructor(nodes: Node[], edges: Edge[], initialData: Record<string, any> = {}) {
    this.nodes = nodes;
    this.edges = edges;
    this.stepDelay = 1000; // 1초 딜레이 (디버깅용)
    this.state = {
      steps: [],
      currentStepIndex: -1,
      isRunning: false,
      isPaused: false,
      globalData: { ...initialData },
    };
  }

  /**
   * 시뮬레이션 시작
   */
  async start(): Promise<SimulationState> {
    this.state.isRunning = true;
    this.state.isPaused = false;
    this.state.steps = [];
    this.state.currentStepIndex = -1;

    // Input 노드 찾기 (워크플로우 시작점)
    const inputNodes = this.nodes.filter((n) => n.data.type === 'input');
    if (inputNodes.length === 0) {
      throw new Error('Input 노드를 찾을 수 없습니다.');
    }

    // 첫 번째 Input 노드부터 시작
    await this.executeNode(inputNodes[0]);

    return this.state;
  }

  /**
   * 다음 단계 실행
   */
  async stepForward(): Promise<SimulationState> {
    if (this.state.currentStepIndex >= this.state.steps.length - 1) {
      // 마지막 단계인 경우, 다음 노드 실행
      const lastStep = this.state.steps[this.state.steps.length - 1];
      const nextNodes = this.getNextNodes(lastStep.nodeId);

      if (nextNodes.length > 0) {
        await this.executeNode(nextNodes[0]);
      } else {
        // 워크플로우 종료
        this.state.isRunning = false;
      }
    } else {
      // 이미 실행된 단계로 이동
      this.state.currentStepIndex++;
    }

    return this.state;
  }

  /**
   * 이전 단계로 이동
   */
  stepBackward(): SimulationState {
    if (this.state.currentStepIndex > 0) {
      this.state.currentStepIndex--;
    }
    return this.state;
  }

  /**
   * 시뮬레이션 일시정지
   */
  pause(): SimulationState {
    this.state.isPaused = true;
    return this.state;
  }

  /**
   * 시뮬레이션 재개
   */
  resume(): SimulationState {
    this.state.isPaused = false;
    return this.state;
  }

  /**
   * 시뮬레이션 초기화
   */
  reset(): SimulationState {
    this.state = {
      steps: [],
      currentStepIndex: -1,
      isRunning: false,
      isPaused: false,
      globalData: {},
    };
    return this.state;
  }

  /**
   * 현재 상태 가져오기
   */
  getState(): SimulationState {
    return this.state;
  }

  /**
   * 특정 노드 실행
   */
  private async executeNode(node: Node): Promise<void> {
    const startTime = Date.now();
    const step: SimulationStep = {
      nodeId: node.id,
      nodeName: node.data.label || node.id,
      nodeType: node.data.type || 'default',
      status: 'running',
      input: { ...this.state.globalData },
      timestamp: new Date().toISOString(),
    };

    // 단계 추가
    this.state.steps.push(step);
    this.state.currentStepIndex = this.state.steps.length - 1;

    try {
      // 노드 타입별 실행 로직
      switch (node.data.type) {
        case 'input':
          await this.executeInputNode(node, step);
          break;
        case 'decision':
          await this.executeDecisionNode(node, step);
          break;
        case 'action':
          await this.executeActionNode(node, step);
          break;
        case 'output':
          await this.executeOutputNode(node, step);
          break;
        default:
          throw new Error(`지원하지 않는 노드 타입: ${node.data.type}`);
      }

      // 성공
      step.status = 'success';
      step.executionTimeMs = Date.now() - startTime;
    } catch (error) {
      // 실패
      step.status = 'error';
      step.error = error instanceof Error ? error.message : String(error);
      step.executionTimeMs = Date.now() - startTime;
      this.state.isRunning = false;
    }

    // 딜레이 (디버깅용)
    await this.delay(this.stepDelay);
  }

  /**
   * Input 노드 실행
   */
  private async executeInputNode(node: Node, step: SimulationStep): Promise<void> {
    // Input 노드는 초기 데이터를 설정
    step.output = { ...this.state.globalData };
    // globalData는 이미 초기화되어 있음
  }

  /**
   * Decision 노드 실행 (Rule Engine)
   */
  private async executeDecisionNode(node: Node, step: SimulationStep): Promise<void> {
    const rule = node.data.rule;
    if (!rule) {
      throw new Error('Decision 노드에 Rule 표현식이 없습니다.');
    }

    // 간단한 Rule 평가 (실제로는 RuleEngine 사용)
    const result = this.evaluateSimpleRule(rule, this.state.globalData);

    step.output = {
      decision: result,
      rule,
      context: { ...this.state.globalData },
    };

    // 결과를 globalData에 저장
    this.state.globalData.lastDecision = result;
  }

  /**
   * Action 노드 실행
   */
  private async executeActionNode(node: Node, step: SimulationStep): Promise<void> {
    const action = node.data.action;
    if (!action) {
      throw new Error('Action 노드에 액션이 설정되지 않았습니다.');
    }

    // 액션 시뮬레이션 (실제로는 외부 시스템 호출)
    step.output = {
      action,
      executed: true,
      result: `액션 실행됨: ${action}`,
    };

    // 액션 결과를 globalData에 저장
    this.state.globalData.lastAction = action;
  }

  /**
   * Output 노드 실행
   */
  private async executeOutputNode(node: Node, step: SimulationStep): Promise<void> {
    // Output 노드는 최종 결과를 반환
    step.output = {
      finalResult: { ...this.state.globalData },
      workflowCompleted: true,
    };

    // 워크플로우 종료
    this.state.isRunning = false;
  }

  /**
   * 간단한 Rule 평가 (프로토타입용)
   */
  private evaluateSimpleRule(rule: string, data: Record<string, any>): boolean {
    try {
      // 변수명 치환
      let evalRule = rule;
      Object.keys(data).forEach((key) => {
        const value = data[key];
        const valueStr = typeof value === 'string' ? `"${value}"` : String(value);
        evalRule = evalRule.replace(new RegExp(key, 'g'), valueStr);
      });

      // eval 사용 (프로토타입용, 실제로는 안전한 파서 사용)
      // eslint-disable-next-line no-eval
      return eval(evalRule);
    } catch (error) {
      console.error('Rule 평가 실패:', error);
      return false;
    }
  }

  /**
   * 다음 노드 찾기
   */
  private getNextNodes(currentNodeId: string): Node[] {
    const outgoingEdges = this.edges.filter((e) => e.source === currentNodeId);
    return outgoingEdges.map((e) => this.nodes.find((n) => n.id === e.target)!).filter(Boolean);
  }

  /**
   * 딜레이 헬퍼
   */
  private delay(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }

  /**
   * 실행 속도 설정
   */
  setStepDelay(delayMs: number): void {
    this.stepDelay = delayMs;
  }
}
