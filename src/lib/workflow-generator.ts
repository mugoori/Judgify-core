import { Node, Edge } from 'reactflow';

/**
 * LLM 기반 워크플로우 자동 생성기
 *
 * 자연어 설명을 받아서 React Flow 노드 구조로 변환합니다.
 *
 * 예시 입력:
 * - "온도가 90도 이상이면 알림 보내기"
 * - "재고가 10개 미만이면 주문 생성"
 */

interface WorkflowGenerationResult {
  nodes: Node[];
  edges: Edge[];
  name: string;
  description: string;
}

// 패턴 기반 워크플로우 생성 (LLM 대체용 간단 구현)
export async function generateWorkflowFromDescription(
  description: string
): Promise<WorkflowGenerationResult> {
  // 패턴 매칭으로 조건 추출
  const patterns = [
    // "A가 B이면 C" 패턴
    /(.+?)가\s*(.+?)(이면|면)\s*(.+)/,
    // "만약 A 이면 B" 패턴
    /만약\s*(.+?)\s*(이면|면)\s*(.+)/,
    // "A > B 이면 C" 패턴
    /(.+?)\s*([><=!]+)\s*(.+?)\s*(이면|면)\s*(.+)/,
  ];

  let condition = '';
  let action = '';
  let workflowName = 'AI 생성 워크플로우';

  // 패턴 매칭
  for (const pattern of patterns) {
    const match = description.match(pattern);
    if (match) {
      if (pattern.source.includes('[><=!]')) {
        // 비교 연산자 포함 패턴
        const [, left, operator, right, , actionText] = match;
        condition = `${left.trim()} ${operator} ${right.trim()}`;
        action = actionText.trim();
        workflowName = `${left.trim()} 모니터링`;
      } else if (pattern.source.includes('만약')) {
        // "만약" 패턴
        const [, condText, , actionText] = match;
        condition = condText.trim();
        action = actionText.trim();
        workflowName = `${condText.trim()} 워크플로우`;
      } else {
        // "A가 B이면 C" 패턴
        const [, subject, predicate, , actionText] = match;
        condition = `${subject.trim()} ${predicate.trim()}`;
        action = actionText.trim();
        workflowName = `${subject.trim()} 모니터링`;
      }
      break;
    }
  }

  // 노드 생성
  const nodes: Node[] = [];
  const edges: Edge[] = [];

  // 1. Input Node
  const inputNode: Node = {
    id: 'input-1',
    type: 'custom',
    data: {
      label: '데이터 입력',
      type: 'input',
      description: '외부 센서 또는 시스템에서 데이터 수집',
    },
    position: { x: 250, y: 50 },
  };
  nodes.push(inputNode);

  // 2. Decision Node (조건이 있는 경우)
  if (condition) {
    const decisionNode: Node = {
      id: 'decision-1',
      type: 'custom',
      data: {
        label: '판단 로직',
        type: 'decision',
        description: '조건 평가 및 분기',
        rule: condition,
      },
      position: { x: 250, y: 180 },
    };
    nodes.push(decisionNode);

    // Input → Decision 연결
    edges.push({
      id: 'e-input-decision',
      source: 'input-1',
      target: 'decision-1',
      animated: true,
    });

    // 3. Action Node (참인 경우)
    if (action) {
      const actionNode: Node = {
        id: 'action-1',
        type: 'custom',
        data: {
          label: '외부 연동',
          type: 'action',
          description: action,
        },
        position: { x: 450, y: 310 },
      };
      nodes.push(actionNode);

      // Decision (true) → Action 연결
      edges.push({
        id: 'e-decision-action',
        source: 'decision-1',
        sourceHandle: 'true',
        target: 'action-1',
        animated: true,
        label: '참',
      });
    }

    // 4. Output Node (거짓인 경우)
    const falseOutputNode: Node = {
      id: 'output-1',
      type: 'custom',
      data: {
        label: '결과 출력',
        type: 'output',
        description: '조건 불만족 - 정상 종료',
      },
      position: { x: 50, y: 310 },
    };
    nodes.push(falseOutputNode);

    // Decision (false) → Output 연결
    edges.push({
      id: 'e-decision-output-false',
      source: 'decision-1',
      sourceHandle: 'false',
      target: 'output-1',
      animated: true,
      label: '거짓',
    });

    // 5. Final Output Node (참인 경우)
    if (action) {
      const trueOutputNode: Node = {
        id: 'output-2',
        type: 'custom',
        data: {
          label: '결과 출력',
          type: 'output',
          description: '작업 완료',
        },
        position: { x: 450, y: 440 },
      };
      nodes.push(trueOutputNode);

      // Action → Output 연결
      edges.push({
        id: 'e-action-output',
        source: 'action-1',
        target: 'output-2',
        animated: true,
      });
    }
  } else {
    // 조건 없는 단순 워크플로우
    const actionNode: Node = {
      id: 'action-1',
      type: 'custom',
      data: {
        label: '외부 연동',
        type: 'action',
        description: action || '작업 실행',
      },
      position: { x: 250, y: 180 },
    };
    nodes.push(actionNode);

    const outputNode: Node = {
      id: 'output-1',
      type: 'custom',
      data: {
        label: '결과 출력',
        type: 'output',
        description: '작업 완료',
      },
      position: { x: 250, y: 310 },
    };
    nodes.push(outputNode);

    edges.push({
      id: 'e-input-action',
      source: 'input-1',
      target: 'action-1',
      animated: true,
    });

    edges.push({
      id: 'e-action-output',
      source: 'action-1',
      target: 'output-1',
      animated: true,
    });
  }

  return {
    nodes,
    edges,
    name: workflowName,
    description,
  };
}

// 5개 테스트 시나리오
export const testScenarios = [
  '온도가 90도 이상이면 알림 보내기',
  '재고가 10개 미만이면 주문 생성',
  '고객 피드백이 부정적이면 매니저에게 전달',
  '결제 실패시 3번 재시도',
  '주말에는 자동 응답 활성화',
];

// 테스트 헬퍼 함수
export async function testWorkflowGeneration() {
  console.log('🧪 워크플로우 자동 생성 테스트 시작\n');

  for (const scenario of testScenarios) {
    const result = await generateWorkflowFromDescription(scenario);
    console.log(`✅ 시나리오: "${scenario}"`);
    console.log(`   생성된 워크플로우: ${result.name}`);
    console.log(`   노드 수: ${result.nodes.length}개`);
    console.log(`   엣지 수: ${result.edges.length}개\n`);
  }

  console.log('✅ 모든 테스트 완료!');
}
