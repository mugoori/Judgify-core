/**
 * WorkflowGenerator 통합 테스트
 * Week 5 Task 8: 15개 통합 테스트 작성
 *
 * 테스트 구성:
 * - 패턴 기반 생성 테스트 5개
 * - LLM 기반 생성 테스트 5개 (Mock 사용)
 * - 통합 및 에러 처리 테스트 5개
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { WorkflowGenerator } from '../workflow-generator';
import type {
  LLMProvider,
  LLMProviderConfig,
  WorkflowGenerationRequest,
  WorkflowGenerationResponse,
} from '../llm-provider';

// ========================================
// Mock LLM Provider
// ========================================

class MockLLMProvider implements LLMProvider {
  readonly name = 'MockProvider';
  readonly defaultModel = 'mock-model';

  async generateWorkflow(
    request: WorkflowGenerationRequest,
    config: LLMProviderConfig
  ): Promise<WorkflowGenerationResponse> {
    // 모의 워크플로우 생성 (description 기반)
    const description = request.description.toLowerCase();

    // 품질 검사 시나리오
    if (description.includes('품질') || description.includes('검사')) {
      return {
        nodes: [
          {
            id: 'input-1',
            type: 'custom',
            label: '데이터 입력',
            config: { type: 'input' },
            position: { x: 250, y: 50 },
          },
          {
            id: 'decision-1',
            type: 'custom',
            label: '품질 검사',
            config: { type: 'decision', rule: '품질 >= 90' },
            position: { x: 250, y: 180 },
          },
          {
            id: 'action-1',
            type: 'custom',
            label: '합격 처리',
            config: { type: 'action' },
            position: { x: 450, y: 310 },
          },
          {
            id: 'output-1',
            type: 'custom',
            label: '불합격 처리',
            config: { type: 'output' },
            position: { x: 50, y: 310 },
          },
        ],
        edges: [
          { id: 'e1', source: 'input-1', target: 'decision-1' },
          { id: 'e2', source: 'decision-1', target: 'action-1' },
          { id: 'e3', source: 'decision-1', target: 'output-1' },
        ],
        metadata: {
          provider: 'MockProvider',
          model: 'mock-model',
          confidence: 0.95,
          generationTime: 100,
        },
      };
    }

    // 재고 관리 시나리오
    if (description.includes('재고')) {
      return {
        nodes: [
          {
            id: 'input-1',
            type: 'custom',
            label: '재고 데이터 수집',
            config: { type: 'input' },
            position: { x: 250, y: 50 },
          },
          {
            id: 'decision-1',
            type: 'custom',
            label: '임계값 확인',
            config: { type: 'decision', rule: '재고 < 10' },
            position: { x: 250, y: 180 },
          },
          {
            id: 'notification-1',
            type: 'custom',
            label: '알림 발송',
            config: { type: 'notification' },
            position: { x: 450, y: 310 },
          },
        ],
        edges: [
          { id: 'e1', source: 'input-1', target: 'decision-1' },
          { id: 'e2', source: 'decision-1', target: 'notification-1' },
        ],
        metadata: {
          provider: 'MockProvider',
          model: 'mock-model',
          confidence: 0.92,
          generationTime: 120,
        },
      };
    }

    // 기본 시나리오 (결함 패턴, 승인 프로세스 등)
    return {
      nodes: [
        {
          id: 'input-1',
          type: 'custom',
          label: '데이터 입력',
          config: { type: 'input' },
          position: { x: 250, y: 50 },
        },
        {
          id: 'action-1',
          type: 'custom',
          label: '작업 실행',
          config: { type: 'action' },
          position: { x: 250, y: 180 },
        },
        {
          id: 'output-1',
          type: 'custom',
          label: '결과 출력',
          config: { type: 'output' },
          position: { x: 250, y: 310 },
        },
      ],
      edges: [
        { id: 'e1', source: 'input-1', target: 'action-1' },
        { id: 'e2', source: 'action-1', target: 'output-1' },
      ],
      metadata: {
        provider: 'MockProvider',
        model: 'mock-model',
        confidence: 0.85,
        generationTime: 90,
      },
    };
  }

  validateApiKey(apiKey: string): boolean {
    return apiKey.length > 0 && apiKey.startsWith('mock-');
  }

  getErrorMessage(error: unknown): string {
    if (error instanceof Error) {
      return error.message;
    }
    return 'Unknown error';
  }
}

// ========================================
// 테스트 Suite
// ========================================

describe('WorkflowGenerator - 패턴 기반 생성 테스트 (5개)', () => {
  let generator: WorkflowGenerator;

  beforeEach(() => {
    generator = new WorkflowGenerator();
  });

  it('1. Linear Pattern: "온도가 90도 이상이면 알림 보내기"', async () => {
    const result = await generator.generate('온도가 90도 이상이면 알림 보내기', {
      mode: 'pattern',
    });

    // 기본 검증
    expect(result.nodes).toBeDefined();
    expect(result.edges).toBeDefined();
    expect(result.name).toBe('온도 모니터링');
    expect(result.description).toBe('온도가 90도 이상이면 알림 보내기');

    // 메타데이터 검증
    expect(result.metadata?.generationMode).toBe('pattern');
    expect(result.metadata?.usedLLM).toBe(false);
    expect(result.metadata?.patternMatched).toBe(true);

    // 노드 구조 검증 (조건 분기)
    expect(result.nodes.length).toBeGreaterThanOrEqual(3); // input + decision + output
    expect(result.nodes.some((n) => n.data.type === 'input')).toBe(true);
    expect(result.nodes.some((n) => n.data.type === 'decision')).toBe(true);

    // 엣지 검증
    expect(result.edges.length).toBeGreaterThan(0);
    expect(result.edges[0].animated).toBe(true); // 애니메이션 활성화 확인

    // 성능 검증
    expect(result.metadata?.generationTime).toBeLessThan(5000); // 5초 이내
  });

  it('2. Branching Pattern: "만약 재고가 10개 미만이면 주문 생성"', async () => {
    const result = await generator.generate('만약 재고가 10개 미만이면 주문 생성', {
      mode: 'pattern',
    });

    expect(result.nodes.length).toBeGreaterThanOrEqual(3);
    expect(result.metadata?.generationMode).toBe('pattern');
    expect(result.metadata?.patternMatched).toBe(true);
    expect(result.metadata?.generationTime).toBeLessThan(5000);

    // Decision 노드 존재 확인
    const decisionNode = result.nodes.find((n) => n.data.type === 'decision');
    expect(decisionNode).toBeDefined();
    expect(decisionNode?.data.rule).toContain('재고');
  });

  it('3. API 호출 Pattern: "외부 API 호출하여 데이터 수집"', async () => {
    const result = await generator.generate('외부 API 호출하여 데이터 수집', {
      mode: 'pattern',
    });

    expect(result.nodes.length).toBeGreaterThanOrEqual(2);
    expect(result.metadata?.generationMode).toBe('pattern');
    expect(result.name).toBe('API 연동 워크플로우');
  });

  it('4. Email Pattern: "결제 실패시 고객에게 이메일 발송"', async () => {
    const result = await generator.generate('결제 실패시 고객에게 이메일 발송', {
      mode: 'pattern',
    });

    expect(result.nodes.length).toBeGreaterThanOrEqual(2);
    expect(result.metadata?.generationMode).toBe('pattern');
    expect(result.name).toBe('이메일 발송 워크플로우');
  });

  it('5. Scheduled Pattern: "매일 아침 9시 리포트 생성"', async () => {
    const result = await generator.generate('매일 아침 9시 리포트 생성', {
      mode: 'pattern',
    });

    expect(result.nodes.length).toBeGreaterThanOrEqual(2);
    expect(result.metadata?.generationMode).toBe('pattern');
    expect(result.name).toBe('스케줄링 워크플로우');
  });
});

describe('WorkflowGenerator - LLM 기반 생성 테스트 (5개, Mock 사용)', () => {
  let generator: WorkflowGenerator;
  let mockProvider: MockLLMProvider;

  beforeEach(() => {
    mockProvider = new MockLLMProvider();
    generator = new WorkflowGenerator(mockProvider);
  });

  it('1. 품질 검사 자동화 워크플로우 생성', async () => {
    const result = await generator.generate('품질 검사 자동화', {
      mode: 'llm',
      llmConfig: {
        apiKey: 'mock-api-key',
        model: 'mock-model',
      },
    });

    expect(result.nodes.length).toBeGreaterThanOrEqual(3);
    expect(result.metadata?.generationMode).toBe('llm');
    expect(result.metadata?.usedLLM).toBe(true);
    expect(result.metadata?.provider).toBe('MockProvider');
    expect(result.metadata?.confidence).toBeGreaterThan(0.9);

    // 품질 검사 노드 확인
    const qualityCheck = result.nodes.find((n) => n.label.includes('품질'));
    expect(qualityCheck).toBeDefined();
  });

  it('2. 재고 임계값 알림 워크플로우 생성', async () => {
    const result = await generator.generate('재고 임계값 알림', {
      mode: 'llm',
      llmConfig: {
        apiKey: 'mock-api-key',
      },
    });

    expect(result.nodes.length).toBeGreaterThanOrEqual(3);
    expect(result.metadata?.usedLLM).toBe(true);

    // 알림 노드 확인
    const notificationNode = result.nodes.find((n) => n.config.type === 'notification');
    expect(notificationNode).toBeDefined();
  });

  it('3. 결함 패턴 분석 워크플로우 생성', async () => {
    const result = await generator.generate('결함 패턴 분석', {
      mode: 'llm',
      llmConfig: {
        apiKey: 'mock-api-key',
      },
    });

    expect(result.nodes.length).toBeGreaterThanOrEqual(2);
    expect(result.metadata?.usedLLM).toBe(true);
    expect(result.metadata?.generationTime).toBeLessThan(5000);
  });

  it('4. 다단계 승인 프로세스 워크플로우 생성', async () => {
    const result = await generator.generate('다단계 승인 프로세스', {
      mode: 'llm',
      llmConfig: {
        apiKey: 'mock-api-key',
      },
    });

    expect(result.nodes.length).toBeGreaterThanOrEqual(2);
    expect(result.metadata?.usedLLM).toBe(true);
  });

  it('5. 조건부 분기 판단 워크플로우 생성', async () => {
    const result = await generator.generate('조건부 분기 판단', {
      mode: 'llm',
      llmConfig: {
        apiKey: 'mock-api-key',
      },
    });

    expect(result.nodes.length).toBeGreaterThanOrEqual(2);
    expect(result.metadata?.usedLLM).toBe(true);
  });
});

describe('WorkflowGenerator - 통합 및 에러 처리 테스트 (5개)', () => {
  let generator: WorkflowGenerator;
  let mockProvider: MockLLMProvider;

  beforeEach(() => {
    mockProvider = new MockLLMProvider();
    generator = new WorkflowGenerator(mockProvider);
  });

  it('1. Hybrid Mode: Pattern 충분한 경우 LLM 미사용', async () => {
    const result = await generator.generate('온도가 90도 이상이면 알림', {
      mode: 'hybrid',
      llmConfig: {
        apiKey: 'mock-api-key',
      },
    });

    // Pattern이 충분하므로 LLM 미사용
    expect(result.metadata?.generationMode).toBe('hybrid');
    expect(result.metadata?.usedLLM).toBe(false); // Pattern만 사용
    expect(result.metadata?.patternMatched).toBe(true);
    expect(result.nodes.length).toBeGreaterThanOrEqual(3);
  });

  it('2. Hybrid Mode: Pattern 불충분한 경우 LLM Fallback', async () => {
    const result = await generator.generate('복잡한 비즈니스 로직 처리', {
      mode: 'hybrid',
      llmConfig: {
        apiKey: 'mock-api-key',
      },
    });

    // Pattern 실패 → LLM Fallback
    expect(result.metadata?.generationMode).toBe('hybrid');
    // 이 경우 Pattern 매칭 실패하면 LLM 사용
    expect(result.nodes.length).toBeGreaterThanOrEqual(2);
  });

  it('3. 에러 처리: LLM Mode에서 Provider 없는 경우', async () => {
    const generatorWithoutLLM = new WorkflowGenerator();

    await expect(
      generatorWithoutLLM.generate('테스트', {
        mode: 'llm',
        llmConfig: {
          apiKey: 'test-key',
        },
      })
    ).rejects.toThrow('LLM provider is required for "llm" mode');
  });

  it('4. 에러 처리: LLM Mode에서 API Key 없는 경우', async () => {
    await expect(
      generator.generate('테스트', {
        mode: 'llm',
        llmConfig: {
          apiKey: '', // 빈 API 키
        },
      })
    ).rejects.toThrow('LLM configuration (apiKey) is required for "llm" mode');
  });

  it('5. 성능 검증: 모든 모드에서 5초 이내 생성', async () => {
    const testCases = [
      { description: '온도 모니터링', mode: 'pattern' as const },
      { description: '품질 검사', mode: 'llm' as const },
      { description: '재고 관리', mode: 'hybrid' as const },
    ];

    for (const testCase of testCases) {
      const result = await generator.generate(testCase.description, {
        mode: testCase.mode,
        llmConfig:
          testCase.mode !== 'pattern'
            ? {
                apiKey: 'mock-api-key',
              }
            : undefined,
      });

      expect(result.metadata?.generationTime).toBeLessThan(5000);
      expect(result.nodes.length).toBeGreaterThan(0);
      expect(result.edges.length).toBeGreaterThan(0);
    }
  });
});

describe('WorkflowGenerator - 노드/엣지 생성 검증', () => {
  let generator: WorkflowGenerator;

  beforeEach(() => {
    generator = new WorkflowGenerator();
  });

  it('모든 노드는 필수 필드를 가져야 함 (id, type, data)', async () => {
    const result = await generator.generate('온도가 90도 이상이면 알림', {
      mode: 'pattern',
    });

    for (const node of result.nodes) {
      expect(node.id).toBeDefined();
      expect(node.type).toBeDefined();
      expect(node.data).toBeDefined();
      expect(node.data.label).toBeDefined();
      expect(node.data.type).toBeDefined();
      expect(node.position).toBeDefined();
      expect(node.position.x).toBeGreaterThanOrEqual(0);
      expect(node.position.y).toBeGreaterThanOrEqual(0);
    }
  });

  it('모든 엣지는 올바른 source/target 핸들을 가져야 함', async () => {
    const result = await generator.generate('재고가 10개 미만이면 주문', {
      mode: 'pattern',
    });

    for (const edge of result.edges) {
      expect(edge.id).toBeDefined();
      expect(edge.source).toBeDefined();
      expect(edge.target).toBeDefined();
      expect(edge.sourceHandle).toBeDefined();
      expect(edge.targetHandle).toBeDefined();

      // source/target 노드가 존재해야 함
      const sourceNode = result.nodes.find((n) => n.id === edge.source);
      const targetNode = result.nodes.find((n) => n.id === edge.target);
      expect(sourceNode).toBeDefined();
      expect(targetNode).toBeDefined();
    }
  });
});
