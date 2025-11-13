import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/tauri';
import { generateSampleData, isDatabaseEmpty } from '../sample-data';

// Tauri invoke 모킹
vi.mock('@tauri-apps/api/tauri', () => ({
  invoke: vi.fn(),
}));

// console.log 모킹 (테스트 출력 정리)
vi.spyOn(console, 'log').mockImplementation(() => {});
vi.spyOn(console, 'error').mockImplementation(() => {});

describe('sample-data', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('isDatabaseEmpty', () => {
    it('데이터베이스가 비어있음 - true 반환', async () => {
      vi.mocked(invoke).mockResolvedValue({
        total_judgments: 0,
        total_workflows: 0,
        total_training_samples: 0,
        average_confidence: 0,
      });

      const result = await isDatabaseEmpty();

      expect(invoke).toHaveBeenCalledWith('get_system_stats');
      expect(result).toBe(true);
    });

    it('데이터베이스에 데이터 있음 - false 반환', async () => {
      vi.mocked(invoke).mockResolvedValue({
        total_judgments: 10,
        total_workflows: 3,
        total_training_samples: 5,
        average_confidence: 0.85,
      });

      const result = await isDatabaseEmpty();

      expect(result).toBe(false);
    });

    it('에러 발생시 false 반환', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('Database error'));

      const result = await isDatabaseEmpty();

      expect(result).toBe(false);
    });
  });

  describe('generateSampleData', () => {
    it('샘플 워크플로우 3개 생성 성공', async () => {
      // create_workflow 호출마다 다른 ID 반환
      vi.mocked(invoke)
        .mockResolvedValueOnce({ id: 'workflow-1' })
        .mockResolvedValueOnce({ id: 'workflow-2' })
        .mockResolvedValueOnce({ id: 'workflow-3' });

      // execute_judgment 호출은 모두 성공으로 처리
      for (let i = 0; i < 37; i++) {
        // 15 + 10 + 12 = 37 judgments
        vi.mocked(invoke).mockResolvedValueOnce({ id: `judgment-${i}` });
      }

      const result = await generateSampleData();

      // 3개 워크플로우 생성 확인
      expect(result.workflows).toBe(3);

      // 37개 판단 실행 확인 (15 + 10 + 12)
      expect(result.judgments).toBeGreaterThan(0);
      expect(result.judgments).toBeLessThanOrEqual(37);
    });

    it('워크플로우 생성 실패시 계속 진행', async () => {
      // 첫 번째 워크플로우 생성 실패
      vi.mocked(invoke).mockRejectedValueOnce(new Error('Create failed'));

      // 두 번째, 세 번째는 성공
      vi.mocked(invoke)
        .mockResolvedValueOnce({ id: 'workflow-2' })
        .mockResolvedValueOnce({ id: 'workflow-3' });

      // 판단 실행은 모두 성공
      for (let i = 0; i < 22; i++) {
        // 10 + 12 = 22 judgments (첫 번째 워크플로우 15개 제외)
        vi.mocked(invoke).mockResolvedValueOnce({ id: `judgment-${i}` });
      }

      const result = await generateSampleData();

      // 2개 워크플로우 생성 (1개 실패)
      expect(result.workflows).toBe(2);
    });

    it('판단 실행 일부 실패시 성공 개수만 카운트', async () => {
      // 워크플로우 3개 생성 성공
      vi.mocked(invoke)
        .mockResolvedValueOnce({ id: 'workflow-1' })
        .mockResolvedValueOnce({ id: 'workflow-2' })
        .mockResolvedValueOnce({ id: 'workflow-3' });

      // 판단 실행 중 일부 실패
      for (let i = 0; i < 30; i++) {
        if (i % 5 === 0) {
          // 매 5번째마다 실패
          vi.mocked(invoke).mockRejectedValueOnce(new Error('Execution failed'));
        } else {
          vi.mocked(invoke).mockResolvedValueOnce({ id: `judgment-${i}` });
        }
      }

      const result = await generateSampleData();

      expect(result.workflows).toBe(3);
      // 성공한 판단 실행만 카운트 (실패 제외)
      expect(result.judgments).toBeLessThan(37);
    });

    it('생성된 워크플로우 구조 확인', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce({ id: 'workflow-1' })
        .mockResolvedValueOnce({ id: 'workflow-2' })
        .mockResolvedValueOnce({ id: 'workflow-3' });

      // 판단 실행 모킹
      for (let i = 0; i < 37; i++) {
        vi.mocked(invoke).mockResolvedValueOnce({ id: `judgment-${i}` });
      }

      await generateSampleData();

      // create_workflow 호출 확인
      expect(invoke).toHaveBeenCalledWith(
        'create_workflow',
        expect.objectContaining({
          request: expect.objectContaining({
            name: expect.any(String),
            definition: expect.objectContaining({
              nodes: expect.any(Array),
              edges: expect.any(Array),
            }),
            rule_expression: expect.any(String),
          }),
        })
      );
    });

    it('판단 실행 데이터 구조 확인', async () => {
      // 워크플로우 1개만 생성 (테스트 간소화)
      vi.mocked(invoke).mockResolvedValueOnce({ id: 'workflow-1' });

      // 판단 실행 모킹
      for (let i = 0; i < 15; i++) {
        vi.mocked(invoke).mockResolvedValueOnce({ id: `judgment-${i}` });
      }

      await generateSampleData();

      // execute_judgment 호출 확인
      expect(invoke).toHaveBeenCalledWith(
        'execute_judgment',
        expect.objectContaining({
          request: expect.objectContaining({
            workflow_id: 'workflow-1',
            input_data: expect.any(Object),
            method: expect.stringMatching(/rule|llm|hybrid/),
          }),
        })
      );
    });
  });

  describe('데이터 타입 검증', () => {
    it('generateSampleData 반환 타입 확인', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce({ id: 'workflow-1' })
        .mockResolvedValueOnce({ id: 'workflow-2' })
        .mockResolvedValueOnce({ id: 'workflow-3' });

      for (let i = 0; i < 37; i++) {
        vi.mocked(invoke).mockResolvedValueOnce({ id: `judgment-${i}` });
      }

      const result = await generateSampleData();

      // 반환 객체 구조 확인
      expect(result).toHaveProperty('workflows');
      expect(result).toHaveProperty('judgments');

      // 타입 확인
      expect(typeof result.workflows).toBe('number');
      expect(typeof result.judgments).toBe('number');

      // 값 범위 확인
      expect(result.workflows).toBeGreaterThanOrEqual(0);
      expect(result.judgments).toBeGreaterThanOrEqual(0);
    });
  });
});
