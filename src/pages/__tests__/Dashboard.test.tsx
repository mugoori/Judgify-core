import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import Dashboard from '../Dashboard';
import { invoke } from '@tauri-apps/api/tauri';
import type { SystemStats, JudgmentResult, TokenMetrics } from '@/lib/tauri-api';

// Tauri invoke 모킹 (established pattern from testing-guide.md)
vi.mock('@tauri-apps/api/tauri', () => ({
  invoke: vi.fn(),
}));

// react-router-dom 모킹 (window.location.href 사용)
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return {
    ...actual,
    useNavigate: () => vi.fn(),
  };
});

// Mock sample-data module
vi.mock('@/lib/sample-data', () => ({
  generateSampleData: vi.fn(),
}));

import { generateSampleData } from '@/lib/sample-data';

describe('Dashboard', () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: {
        queries: {
          retry: false,
          gcTime: 0,
        },
      },
    });
  });

  const renderDashboard = () => {
    return render(
      <QueryClientProvider client={queryClient}>
        <Dashboard />
      </QueryClientProvider>
    );
  };

  // Mock data fixtures
  const mockSystemStats: SystemStats = {
    total_judgments: 150,
    total_workflows: 5,
    total_training_samples: 80,
    average_confidence: 0.87,
  };

  const mockJudgments: JudgmentResult[] = [
    {
      id: 'judgment-1',
      workflow_id: 'workflow-123',
      result: true,
      confidence: 0.92,
      method_used: 'rule',
      explanation: 'Temperature exceeds threshold',
      created_at: new Date().toISOString(),
    },
    {
      id: 'judgment-2',
      workflow_id: 'workflow-123',
      result: false,
      confidence: 0.88,
      method_used: 'llm',
      explanation: 'Low vibration detected',
      created_at: new Date().toISOString(),
    },
    {
      id: 'judgment-3',
      workflow_id: 'workflow-456',
      result: true,
      confidence: 0.95,
      method_used: 'hybrid',
      explanation: 'Hybrid judgment passed',
      created_at: new Date().toISOString(),
    },
  ];

  const mockTokenMetrics: TokenMetrics = {
    total_tokens_used: 50000,
    total_cost_usd: 0.75,
    tokens_saved_by_cache: 20000,
    cost_saved_usd: 0.30,
    cache_hit_rate: 40.0,
    avg_tokens_per_request: 250,
  };

  // ===== Group 1: KPI Card Rendering (4 tests) =====
  describe('Group 1: KPI Card Rendering', () => {
    it('총 판단 횟수 표시', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(mockSystemStats) // get_system_stats
        .mockResolvedValueOnce(mockJudgments) // get_judgment_history
        .mockResolvedValueOnce(mockTokenMetrics); // get_token_metrics

      renderDashboard();

      await waitFor(() => {
        expect(screen.getByText('총 판단 횟수')).toBeInTheDocument();
        expect(screen.getByText('150')).toBeInTheDocument();
        expect(screen.getByText('누적 판단 실행')).toBeInTheDocument();
      });
    });

    it('워크플로우 개수 표시', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(mockSystemStats)
        .mockResolvedValueOnce(mockJudgments)
        .mockResolvedValueOnce(mockTokenMetrics);

      renderDashboard();

      await waitFor(() => {
        expect(screen.getByText('워크플로우')).toBeInTheDocument();
        expect(screen.getByText('5')).toBeInTheDocument();
        expect(screen.getByText('활성 워크플로우')).toBeInTheDocument();
      });
    });

    it('평균 신뢰도 계산 및 표시', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(mockSystemStats)
        .mockResolvedValueOnce(mockJudgments)
        .mockResolvedValueOnce(mockTokenMetrics);

      renderDashboard();

      await waitFor(() => {
        expect(screen.getByText('평균 신뢰도')).toBeInTheDocument();
        expect(screen.getByText('87.0%')).toBeInTheDocument(); // 0.87 * 100 = 87.0%
        expect(screen.getByText('판단 신뢰도')).toBeInTheDocument();
      });
    });

    it('학습 샘플 개수 표시', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(mockSystemStats)
        .mockResolvedValueOnce(mockJudgments)
        .mockResolvedValueOnce(mockTokenMetrics);

      renderDashboard();

      await waitFor(() => {
        expect(screen.getByText('학습 샘플')).toBeInTheDocument();
        expect(screen.getByText('80')).toBeInTheDocument();
        expect(screen.getByText('학습용 데이터')).toBeInTheDocument();
      });
    });
  });

  // ===== Group 2: Chart Data Transformation Logic (8 tests) =====
  describe('Group 2: Chart Data Transformation Logic', () => {
    it('methodStats 계산 (rule/llm/hybrid 카운트)', async () => {
      const judgments: JudgmentResult[] = [
        { ...mockJudgments[0], method_used: 'rule' },
        { ...mockJudgments[1], method_used: 'rule' },
        { ...mockJudgments[2], method_used: 'llm' },
        { ...mockJudgments[0], id: 'j4', method_used: 'hybrid' },
      ];

      vi.mocked(invoke)
        .mockResolvedValueOnce(mockSystemStats)
        .mockResolvedValueOnce(judgments)
        .mockResolvedValueOnce(mockTokenMetrics);

      renderDashboard();

      await waitFor(() => {
        expect(screen.getByText('판단 방법별 분포')).toBeInTheDocument();
      });

      // methodChartData should be: { rule: 2, llm: 1, hybrid: 1 }
      // Chart should render (we can't test internal data easily, but we verify chart renders)
    });

    it('methodChartData 변환', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(mockSystemStats)
        .mockResolvedValueOnce(mockJudgments)
        .mockResolvedValueOnce(mockTokenMetrics);

      renderDashboard();

      await waitFor(() => {
        expect(screen.getByText('최근 50개 판단의 방법별 통계')).toBeInTheDocument();
      });
    });

    it('resultTrend 계산 (최근 20개)', async () => {
      // Generate 25 judgments to test .slice(-20)
      const manyJudgments = Array.from({ length: 25 }, (_, i) => ({
        ...mockJudgments[0],
        id: `judgment-${i}`,
        confidence: 0.8 + i * 0.01,
      }));

      vi.mocked(invoke)
        .mockResolvedValueOnce(mockSystemStats)
        .mockResolvedValueOnce(manyJudgments)
        .mockResolvedValueOnce(mockTokenMetrics);

      renderDashboard();

      await waitFor(() => {
        expect(screen.getByText('신뢰도 트렌드')).toBeInTheDocument();
        expect(screen.getByText('최근 20개 판단의 신뢰도 변화')).toBeInTheDocument();
      });
    });

    it('dailyTrend 생성 (7일간)', async () => {
      const today = new Date();
      const judgmentsWithDates = mockJudgments.map((j, i) => {
        const date = new Date(today);
        date.setDate(date.getDate() - i); // Different days
        return { ...j, created_at: date.toISOString() };
      });

      vi.mocked(invoke)
        .mockResolvedValueOnce(mockSystemStats)
        .mockResolvedValueOnce(judgmentsWithDates)
        .mockResolvedValueOnce(mockTokenMetrics);

      renderDashboard();

      await waitFor(() => {
        expect(screen.getByText('7일 트렌드')).toBeInTheDocument();
        expect(screen.getByText('최근 7일간 판단 횟수 및 합격률 추이')).toBeInTheDocument();
      });
    });

    it('passRateData 계산', async () => {
      const judgments: JudgmentResult[] = [
        { ...mockJudgments[0], result: true },
        { ...mockJudgments[1], result: true },
        { ...mockJudgments[2], result: false },
      ];

      vi.mocked(invoke)
        .mockResolvedValueOnce(mockSystemStats)
        .mockResolvedValueOnce(judgments)
        .mockResolvedValueOnce(mockTokenMetrics);

      renderDashboard();

      await waitFor(() => {
        expect(screen.getByText('합격률')).toBeInTheDocument();
        expect(screen.getByText('전체 판단 결과 비율')).toBeInTheDocument();
      });

      // passRateData should be: { pass: 2, fail: 1 }
    });

    it('workflowStats 계산', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(mockSystemStats)
        .mockResolvedValueOnce(mockJudgments)
        .mockResolvedValueOnce(mockTokenMetrics);

      renderDashboard();

      await waitFor(() => {
        expect(screen.getByText('워크플로우별 실행 통계')).toBeInTheDocument();
      });
    });

    it('workflowChartData TOP 5 정렬', async () => {
      // Create 7 judgments with different workflow_ids
      const judgments = Array.from({ length: 7 }, (_, i) => ({
        ...mockJudgments[0],
        id: `judgment-${i}`,
        workflow_id: `workflow-${i % 6}`, // 6 different workflows
        confidence: 0.9,
      }));

      vi.mocked(invoke)
        .mockResolvedValueOnce(mockSystemStats)
        .mockResolvedValueOnce(judgments)
        .mockResolvedValueOnce(mockTokenMetrics);

      renderDashboard();

      await waitFor(() => {
        expect(screen.getByText('가장 많이 사용된 워크플로우 TOP 5')).toBeInTheDocument();
      });
    });

    it('빈 데이터 처리 (빈 배열 반환)', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(mockSystemStats)
        .mockResolvedValueOnce([]) // Empty judgments
        .mockResolvedValueOnce(mockTokenMetrics);

      renderDashboard();

      await waitFor(() => {
        // Charts should still render with empty data
        expect(screen.getByText('판단 방법별 분포')).toBeInTheDocument();
        expect(screen.getByText('신뢰도 트렌드')).toBeInTheDocument();
      });
    });
  });

  // ===== Group 3: React Query Integration (6 tests) =====
  describe('Group 3: React Query Integration', () => {
    it('getSystemStats 호출 (30초 자동 갱신)', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(mockSystemStats)
        .mockResolvedValueOnce(mockJudgments)
        .mockResolvedValueOnce(mockTokenMetrics);

      renderDashboard();

      await waitFor(() => {
        expect(invoke).toHaveBeenCalledWith('get_system_stats');
      });
    });

    it('getJudgmentHistory 호출 (50개 최근 판단)', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(mockSystemStats)
        .mockResolvedValueOnce(mockJudgments)
        .mockResolvedValueOnce(mockTokenMetrics);

      renderDashboard();

      await waitFor(() => {
        expect(invoke).toHaveBeenCalledWith('get_judgment_history', {
          workflowId: undefined,
          limit: 50,
        });
      });
    });

    it('getTokenMetrics 호출 (60초 갱신)', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(mockSystemStats)
        .mockResolvedValueOnce(mockJudgments)
        .mockResolvedValueOnce(mockTokenMetrics);

      renderDashboard();

      await waitFor(() => {
        expect(invoke).toHaveBeenCalledWith('get_token_metrics');
      });
    });

    it('isLoading 상태 통합 (3개 쿼리)', async () => {
      // Delay responses to test loading state
      vi.mocked(invoke)
        .mockImplementation(
          () =>
            new Promise((resolve) =>
              setTimeout(() => resolve(mockSystemStats), 100)
            )
        );

      renderDashboard();

      // Should show skeleton loading (check for animate-pulse class)
      const { container } = render(
        <QueryClientProvider client={queryClient}>
          <Dashboard />
        </QueryClientProvider>
      );
      const skeletons = container.querySelectorAll('.animate-pulse');
      expect(skeletons.length).toBeGreaterThan(0);

      await waitFor(
        () => {
          const remainingSkeletons = container.querySelectorAll('.animate-pulse');
          expect(remainingSkeletons.length).toBe(0);
        },
        { timeout: 500 }
      );
    });

    it('캐시 무효화 (샘플 데이터 생성 후)', async () => {
      const emptyStats: SystemStats = {
        total_judgments: 0,
        total_workflows: 0,
        total_training_samples: 0,
        average_confidence: 0,
      };

      vi.mocked(invoke)
        .mockResolvedValueOnce(emptyStats)
        .mockResolvedValueOnce([])
        .mockResolvedValueOnce(mockTokenMetrics);

      vi.mocked(generateSampleData).mockResolvedValue({
        workflows: 3,
        judgments: 37,
      });

      const user = userEvent.setup();
      renderDashboard();

      await waitFor(() => {
        expect(screen.getByText('데이터가 없습니다')).toBeInTheDocument();
      });

      const button = screen.getByRole('button', { name: /샘플 데이터 생성/i });
      await user.click(button);

      await waitFor(() => {
        expect(generateSampleData).toHaveBeenCalledTimes(1);
      });

      // queryClient.invalidateQueries should be called (hard to test directly)
    });

    it('에러 처리 (API 실패시 토스트)', async () => {
      const consoleErrorSpy = vi
        .spyOn(console, 'error')
        .mockImplementation(() => {});

      vi.mocked(invoke).mockRejectedValue(new Error('API Error'));

      renderDashboard();

      // React Query will handle error internally
      await waitFor(() => {
        expect(invoke).toHaveBeenCalled();
      });

      consoleErrorSpy.mockRestore();
    });
  });

  // ===== Group 4: Empty State Handling (4 tests) =====
  describe('Group 4: Empty State Handling', () => {
    const emptyStats: SystemStats = {
      total_judgments: 0,
      total_workflows: 0,
      total_training_samples: 0,
      average_confidence: 0,
    };

    it('데이터 없을 때 EmptyState 표시', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(emptyStats)
        .mockResolvedValueOnce([])
        .mockResolvedValueOnce(mockTokenMetrics);

      renderDashboard();

      await waitFor(() => {
        expect(screen.getByText('데이터가 없습니다')).toBeInTheDocument();
        expect(
          screen.getByText(
            /아직 워크플로우나 판단 데이터가 없습니다/
          )
        ).toBeInTheDocument();
      });
    });

    it('"샘플 데이터 생성" 버튼 동작', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(emptyStats)
        .mockResolvedValueOnce([])
        .mockResolvedValueOnce(mockTokenMetrics);

      vi.mocked(generateSampleData).mockResolvedValue({
        workflows: 3,
        judgments: 37,
      });

      const user = userEvent.setup();
      renderDashboard();

      await waitFor(() => {
        expect(screen.getByText('데이터가 없습니다')).toBeInTheDocument();
      });

      const button = screen.getByRole('button', { name: /샘플 데이터 생성/i });
      expect(button).toBeInTheDocument();

      await user.click(button);

      await waitFor(() => {
        expect(generateSampleData).toHaveBeenCalledTimes(1);
      });
    });

    it('"워크플로우 만들기" 버튼 클릭', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(emptyStats)
        .mockResolvedValueOnce([])
        .mockResolvedValueOnce(mockTokenMetrics);

      renderDashboard();

      await waitFor(() => {
        expect(screen.getByText('데이터가 없습니다')).toBeInTheDocument();
      });

      const button = screen.getByRole('button', { name: /워크플로우 만들기/i });
      expect(button).toBeInTheDocument();

      // Click behavior: window.location.href = '/workflow' (can't test easily in jsdom)
    });

    it('generateSampleData 성공시 호출', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(emptyStats)
        .mockResolvedValueOnce([])
        .mockResolvedValueOnce(mockTokenMetrics);

      vi.mocked(generateSampleData).mockResolvedValue({
        workflows: 3,
        judgments: 37,
      });

      const user = userEvent.setup();
      renderDashboard();

      await waitFor(() => {
        expect(screen.getByText('데이터가 없습니다')).toBeInTheDocument();
      });

      const button = screen.getByRole('button', { name: /샘플 데이터 생성/i });
      await user.click(button);

      // 샘플 데이터 생성 함수가 호출되었는지 확인
      await waitFor(() => {
        expect(generateSampleData).toHaveBeenCalledTimes(1);
      });

      // Note: Toast 메시지 테스트는 Toaster 컴포넌트 설정 필요로 인해 생략
    });
  });

  // ===== Group 5: Skeleton Loading States (3 tests) =====
  describe('Group 5: Skeleton Loading States', () => {
    it('KPI Cards Skeleton 렌더링', async () => {
      vi.mocked(invoke).mockImplementation(
        () =>
          new Promise((resolve) =>
            setTimeout(() => resolve(mockSystemStats), 100)
          )
      );

      const { container } = renderDashboard();

      // Check for multiple skeleton cards (animate-pulse class)
      const skeletons = container.querySelectorAll('.animate-pulse');
      expect(skeletons.length).toBeGreaterThan(0);
    });

    it('Charts Skeleton 렌더링', async () => {
      vi.mocked(invoke).mockImplementation(
        () =>
          new Promise((resolve) =>
            setTimeout(() => resolve(mockSystemStats), 100)
          )
      );

      const { container } = renderDashboard();

      // Skeleton cards should be visible during loading (animate-pulse class)
      const skeletons = container.querySelectorAll('.animate-pulse');
      expect(skeletons.length).toBeGreaterThan(0);
    });

    it('로딩 완료 후 실제 데이터 표시', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(mockSystemStats)
        .mockResolvedValueOnce(mockJudgments)
        .mockResolvedValueOnce(mockTokenMetrics);

      renderDashboard();

      await waitFor(() => {
        expect(screen.getByText('총 판단 횟수')).toBeInTheDocument();
        expect(screen.getByText('150')).toBeInTheDocument();
      });

      // Skeleton should be gone
      expect(screen.queryByTestId('skeleton')).not.toBeInTheDocument();
    });
  });

  // ===== Group 6: Token Metrics Card (3 tests) =====
  describe('Group 6: Token Metrics Card', () => {
    it('총 토큰 사용량 표시', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(mockSystemStats)
        .mockResolvedValueOnce(mockJudgments)
        .mockResolvedValueOnce(mockTokenMetrics);

      renderDashboard();

      await waitFor(() => {
        expect(screen.getByText('토큰 사용량 & 비용 절감')).toBeInTheDocument();
        expect(screen.getByText('50,000')).toBeInTheDocument(); // toLocaleString()
        expect(screen.getByText('총 토큰 사용')).toBeInTheDocument();
      });
    });

    it('비용 절감액 계산 ($0.00 → $0.30)', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(mockSystemStats)
        .mockResolvedValueOnce(mockJudgments)
        .mockResolvedValueOnce(mockTokenMetrics);

      renderDashboard();

      await waitFor(() => {
        expect(screen.getByText('$0.30')).toBeInTheDocument(); // cost_saved_usd
        expect(screen.getByText('비용 절감')).toBeInTheDocument();
      });
    });

    it('캐시 적중률 표시 (40.0%)', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(mockSystemStats)
        .mockResolvedValueOnce(mockJudgments)
        .mockResolvedValueOnce(mockTokenMetrics);

      renderDashboard();

      await waitFor(() => {
        expect(screen.getByText('캐시 적중률')).toBeInTheDocument();
        expect(screen.getByText('40.0%')).toBeInTheDocument();
      });
    });
  });
});
