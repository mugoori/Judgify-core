/**
 * 샘플 데이터 생성 유틸리티
 * 데이터베이스가 비어있을 때 자동으로 Demo 데이터 생성
 */

import { invoke } from '@tauri-apps/api/tauri';

interface SampleWorkflow {
  name: string;
  definition: any;
  rule_expression: string;
}

interface SampleJudgment {
  workflow_id: string;
  input_data: any;
  method?: 'rule' | 'llm' | 'hybrid';
}

/**
 * 샘플 워크플로우 데이터
 */
const SAMPLE_WORKFLOWS: SampleWorkflow[] = [
  {
    name: '품질 검사 워크플로우',
    definition: {
      nodes: [
        { id: '1', type: 'start', label: '시작' },
        { id: '2', type: 'condition', label: '온도 체크' },
        { id: '3', type: 'condition', label: '진동 체크' },
        { id: '4', type: 'end', label: '합격' },
        { id: '5', type: 'end', label: '불합격' },
      ],
      edges: [
        { source: '1', target: '2' },
        { source: '2', target: '3', condition: 'pass' },
        { source: '2', target: '5', condition: 'fail' },
        { source: '3', target: '4', condition: 'pass' },
        { source: '3', target: '5', condition: 'fail' },
      ],
    },
    rule_expression: 'temperature < 100 && vibration < 50',
  },
  {
    name: '긴급도 분류 워크플로우',
    definition: {
      nodes: [
        { id: '1', type: 'start', label: '요청 접수' },
        { id: '2', type: 'condition', label: '긴급도 판단' },
        { id: '3', type: 'action', label: '즉시 처리' },
        { id: '4', type: 'action', label: '대기열 추가' },
      ],
      edges: [
        { source: '1', target: '2' },
        { source: '2', target: '3', condition: 'urgent' },
        { source: '2', target: '4', condition: 'normal' },
      ],
    },
    rule_expression: 'priority === "high" || impact > 8',
  },
  {
    name: '재고 보충 워크플로우',
    definition: {
      nodes: [
        { id: '1', type: 'start', label: '재고 확인' },
        { id: '2', type: 'condition', label: '최소 수량 체크' },
        { id: '3', type: 'action', label: '발주 요청' },
        { id: '4', type: 'end', label: '완료' },
      ],
      edges: [
        { source: '1', target: '2' },
        { source: '2', target: '3', condition: 'low' },
        { source: '2', target: '4', condition: 'sufficient' },
        { source: '3', target: '4' },
      ],
    },
    rule_expression: 'stock_quantity < min_threshold',
  },
];

/**
 * 샘플 판단 실행 데이터 생성 함수
 */
function generateSampleJudgments(workflowIds: string[]): SampleJudgment[] {
  const judgments: SampleJudgment[] = [];

  // 품질 검사 워크플로우 판단 (15개)
  if (workflowIds[0]) {
    for (let i = 0; i < 15; i++) {
      judgments.push({
        workflow_id: workflowIds[0],
        input_data: {
          temperature: 80 + Math.random() * 30, // 80-110
          vibration: 35 + Math.random() * 25, // 35-60
          batch_id: `BATCH-${1000 + i}`,
        },
        method: Math.random() > 0.5 ? 'rule' : 'hybrid',
      });
    }
  }

  // 긴급도 분류 워크플로우 판단 (10개)
  if (workflowIds[1]) {
    const priorities = ['low', 'medium', 'high', 'critical'];
    for (let i = 0; i < 10; i++) {
      judgments.push({
        workflow_id: workflowIds[1],
        input_data: {
          priority: priorities[Math.floor(Math.random() * priorities.length)],
          impact: Math.floor(Math.random() * 10) + 1, // 1-10
          request_id: `REQ-${2000 + i}`,
        },
        method: 'llm',
      });
    }
  }

  // 재고 보충 워크플로우 판단 (12개)
  if (workflowIds[2]) {
    for (let i = 0; i < 12; i++) {
      judgments.push({
        workflow_id: workflowIds[2],
        input_data: {
          stock_quantity: Math.floor(Math.random() * 150), // 0-150
          min_threshold: 50,
          product_id: `PROD-${3000 + i}`,
        },
        method: 'rule',
      });
    }
  }

  return judgments;
}

/**
 * 샘플 데이터 생성 메인 함수
 */
export async function generateSampleData() {
  try {
    console.log('[SampleData] Starting sample data generation...');

    // 1. 샘플 워크플로우 생성
    const workflowIds: string[] = [];
    for (const workflow of SAMPLE_WORKFLOWS) {
      try {
        const result = await invoke('create_workflow', {
          request: workflow,
        });
        workflowIds.push((result as any).id);
        console.log(`[SampleData] Created workflow: ${workflow.name}`);
      } catch (error) {
        console.error(`[SampleData] Failed to create workflow: ${workflow.name}`, error);
      }
    }

    // 2. 샘플 판단 실행 생성 (비동기, 시간 간격)
    const judgments = generateSampleJudgments(workflowIds);
    let successCount = 0;

    for (const judgment of judgments) {
      try {
        await invoke('execute_judgment', {
          request: judgment,
        });
        successCount++;
        // 100ms 간격으로 생성 (realistic timestamp)
        await new Promise((resolve) => setTimeout(resolve, 100));
      } catch (error) {
        console.error('[SampleData] Failed to execute judgment', error);
      }
    }

    console.log(
      `[SampleData] Generated ${workflowIds.length} workflows and ${successCount} judgments`
    );

    return {
      workflows: workflowIds.length,
      judgments: successCount,
    };
  } catch (error) {
    console.error('[SampleData] Sample data generation failed:', error);
    throw error;
  }
}

/**
 * 데이터베이스가 비어있는지 확인
 */
export async function isDatabaseEmpty(): Promise<boolean> {
  try {
    const stats = await invoke('get_system_stats');
    const { total_judgments, total_workflows } = stats as any;
    return total_judgments === 0 && total_workflows === 0;
  } catch (error) {
    console.error('[SampleData] Failed to check database:', error);
    return false;
  }
}
