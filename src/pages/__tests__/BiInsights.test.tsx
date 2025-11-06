import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { userEvent } from '@testing-library/user-event';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import BiInsights from '../BiInsights';
import * as tauriApi from '@/lib/tauri-api';

// Mock Tauri API
vi.mock('@/lib/tauri-api', () => ({
  generateBiInsight: vi.fn(),
}));

// Test wrapper
function renderWithQueryClient(ui: React.ReactElement) {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });
  return render(
    <QueryClientProvider client={queryClient}>{ui}</QueryClientProvider>
  );
}

describe('BiInsights', () => {
  const user = userEvent.setup();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  // ========================================
  // Group 1: Initial Rendering
  // ========================================
  describe('Group 1: Initial Rendering', () => {
    it('초기 렌더링시 헤더 및 설명 표시', () => {
      renderWithQueryClient(<BiInsights />);

      expect(screen.getByText('BI 인사이트')).toBeInTheDocument();
      expect(
        screen.getByText(/AI가 데이터를 분석하고 자동으로 인사이트/)
      ).toBeInTheDocument();
    });

    it('요청 입력 카드 표시', () => {
      renderWithQueryClient(<BiInsights />);

      expect(screen.getByText('요청 입력')).toBeInTheDocument();
      expect(
        screen.getByText(/자연어로 분석 요청을 입력하면/)
      ).toBeInTheDocument();
    });

    it('4개 예시 요청 버튼 표시', () => {
      renderWithQueryClient(<BiInsights />);

      expect(screen.getByText('지난 주 불량률 트렌드를 보여줘')).toBeInTheDocument();
      expect(screen.getByText('워크플로우별 성공률을 분석해줘')).toBeInTheDocument();
      expect(
        screen.getByText('평균 신뢰도가 낮은 판단들을 찾아줘')
      ).toBeInTheDocument();
      expect(
        screen.getByText('시간대별 판단 실행 추이를 시각화해줘')
      ).toBeInTheDocument();
    });

    it('Empty State 표시 (초기 상태)', () => {
      renderWithQueryClient(<BiInsights />);

      expect(screen.getByText('AI 인사이트를 생성해보세요')).toBeInTheDocument();
      expect(
        screen.getByText(/위에 분석 요청을 입력하면 AI가 자동으로/)
      ).toBeInTheDocument();
    });

    it('Textarea placeholder 정상 표시', () => {
      renderWithQueryClient(<BiInsights />);

      const textarea = screen.getByPlaceholderText(
        /예: 지난 주 워크플로우별 성공률/
      );
      expect(textarea).toBeInTheDocument();
    });
  });

  // ========================================
  // Group 2: Example Buttons
  // ========================================
  describe('Group 2: Example Buttons', () => {
    it('예시 버튼 클릭시 Textarea에 텍스트 입력', async () => {
      renderWithQueryClient(<BiInsights />);

      const exampleButton = screen.getByText('지난 주 불량률 트렌드를 보여줘');
      await user.click(exampleButton);

      const textarea = screen.getByRole('textbox');
      expect(textarea).toHaveValue('지난 주 불량률 트렌드를 보여줘');
    });

    it('여러 예시 버튼 클릭시 덮어쓰기', async () => {
      renderWithQueryClient(<BiInsights />);

      // 첫 번째 예시 클릭
      await user.click(screen.getByText('워크플로우별 성공률을 분석해줘'));
      const textarea = screen.getByRole('textbox');
      expect(textarea).toHaveValue('워크플로우별 성공률을 분석해줘');

      // 두 번째 예시 클릭 (덮어쓰기)
      await user.click(screen.getByText('평균 신뢰도가 낮은 판단들을 찾아줘'));
      expect(textarea).toHaveValue('평균 신뢰도가 낮은 판단들을 찾아줘');
    });
  });

  // ========================================
  // Group 3: Request Input & Generation
  // ========================================
  describe('Group 3: Request Input & Generation', () => {
    it('사용자 입력 후 생성 버튼 클릭시 API 호출', async () => {
      const mockResponse = {
        title: '불량률 분석 결과',
        insights: ['인사이트 1', '인사이트 2'],
        recommendations: ['권장사항 1'],
        component_code: '<div>Chart</div>',
      };

      vi.mocked(tauriApi.generateBiInsight).mockResolvedValueOnce(mockResponse);

      renderWithQueryClient(<BiInsights />);

      const textarea = screen.getByRole('textbox');
      const generateButton = screen.getByText('AI 인사이트 생성');

      // 텍스트 입력
      await user.type(textarea, '불량률 분석해줘');
      expect(textarea).toHaveValue('불량률 분석해줘');

      // 생성 버튼 클릭
      await user.click(generateButton);

      // API 호출 확인
      expect(tauriApi.generateBiInsight).toHaveBeenCalledWith('불량률 분석해줘');
    });

    it('빈 요청시 생성 버튼 비활성화', () => {
      renderWithQueryClient(<BiInsights />);

      const generateButton = screen.getByText('AI 인사이트 생성');
      expect(generateButton).toBeDisabled();
    });

    it('공백만 있는 요청시 생성 버튼 비활성화', async () => {
      renderWithQueryClient(<BiInsights />);

      const textarea = screen.getByRole('textbox');
      await user.type(textarea, '   ');

      const generateButton = screen.getByText('AI 인사이트 생성');
      expect(generateButton).toBeDisabled();
    });

    it('생성 중 상태 표시 (생성 중...)', async () => {
      let resolvePromise: (value: any) => void;
      const promise = new Promise((resolve) => {
        resolvePromise = resolve;
      });

      vi.mocked(tauriApi.generateBiInsight).mockReturnValueOnce(promise as any);

      renderWithQueryClient(<BiInsights />);

      const textarea = screen.getByRole('textbox');
      await user.type(textarea, '분석 요청');
      await user.click(screen.getByText('AI 인사이트 생성'));

      // "생성 중..." 표시 확인
      await waitFor(() => {
        expect(screen.getByText('생성 중...')).toBeInTheDocument();
      });

      // 생성 버튼 비활성화
      const generateButton = screen.getByText('생성 중...');
      expect(generateButton.closest('button')).toBeDisabled();

      // 프로미스 해결
      resolvePromise!({
        title: '분석 완료',
        insights: ['결과'],
        recommendations: [],
        component_code: '<div>Chart</div>',
      });

      // 버튼 다시 "AI 인사이트 생성"으로 변경
      await waitFor(() => {
        expect(screen.getByText('AI 인사이트 생성')).toBeInTheDocument();
      });
    });
  });

  // ========================================
  // Group 4: Insight Display
  // ========================================
  describe('Group 4: Insight Display', () => {
    const mockInsight = {
      title: '불량률 트렌드 분석',
      insights: [
        '지난 주 불량률 3.2% 감소',
        '품질 개선 워크플로우 효과 확인',
        '신뢰도 0.9 이상 판단 85% 달성',
      ],
      recommendations: [
        '검사 주기 2시간으로 단축 권장',
        'A라인 센서 재보정 필요',
      ],
      component_code: '<div class="chart">Bar Chart</div>',
    };

    it('생성 성공시 인사이트 제목 표시', async () => {
      vi.mocked(tauriApi.generateBiInsight).mockResolvedValueOnce(mockInsight);

      renderWithQueryClient(<BiInsights />);

      const textarea = screen.getByRole('textbox');
      await user.type(textarea, '불량률 분석');
      await user.click(screen.getByText('AI 인사이트 생성'));

      // 제목 표시 확인
      await waitFor(() => {
        expect(screen.getByText('불량률 트렌드 분석')).toBeInTheDocument();
      });
    });

    it('주요 인사이트 목록 표시', async () => {
      vi.mocked(tauriApi.generateBiInsight).mockResolvedValueOnce(mockInsight);

      renderWithQueryClient(<BiInsights />);

      const textarea = screen.getByRole('textbox');
      await user.type(textarea, '분석');
      await user.click(screen.getByText('AI 인사이트 생성'));

      // 인사이트 목록 확인
      await waitFor(() => {
        expect(screen.getByText('주요 인사이트')).toBeInTheDocument();
        expect(screen.getByText('지난 주 불량률 3.2% 감소')).toBeInTheDocument();
        expect(
          screen.getByText('품질 개선 워크플로우 효과 확인')
        ).toBeInTheDocument();
        expect(
          screen.getByText('신뢰도 0.9 이상 판단 85% 달성')
        ).toBeInTheDocument();
      });
    });

    it('권장사항 카드 표시 (recommendations 있을 때)', async () => {
      vi.mocked(tauriApi.generateBiInsight).mockResolvedValueOnce(mockInsight);

      renderWithQueryClient(<BiInsights />);

      const textarea = screen.getByRole('textbox');
      await user.type(textarea, '분석');
      await user.click(screen.getByText('AI 인사이트 생성'));

      // 권장사항 확인
      await waitFor(() => {
        expect(screen.getByText('권장사항')).toBeInTheDocument();
        expect(screen.getByText('검사 주기 2시간으로 단축 권장')).toBeInTheDocument();
        expect(screen.getByText('A라인 센서 재보정 필요')).toBeInTheDocument();
      });
    });

    it('권장사항 없을 때 카드 미표시', async () => {
      const insightWithoutRecommendations = {
        ...mockInsight,
        recommendations: [],
      };

      vi.mocked(tauriApi.generateBiInsight).mockResolvedValueOnce(
        insightWithoutRecommendations
      );

      renderWithQueryClient(<BiInsights />);

      const textarea = screen.getByRole('textbox');
      await user.type(textarea, '분석');
      await user.click(screen.getByText('AI 인사이트 생성'));

      // 주요 인사이트는 표시됨
      await waitFor(() => {
        expect(screen.getByText('주요 인사이트')).toBeInTheDocument();
      });

      // 권장사항 카드는 표시되지 않음
      expect(screen.queryByText('권장사항')).not.toBeInTheDocument();
    });

    it('자동 생성된 대시보드 컴포넌트 표시', async () => {
      vi.mocked(tauriApi.generateBiInsight).mockResolvedValueOnce(mockInsight);

      renderWithQueryClient(<BiInsights />);

      const textarea = screen.getByRole('textbox');
      await user.type(textarea, '분석');
      await user.click(screen.getByText('AI 인사이트 생성'));

      // 대시보드 컴포넌트 확인
      await waitFor(() => {
        expect(screen.getByText('자동 생성된 대시보드')).toBeInTheDocument();
        expect(screen.getByText('생성된 코드 보기')).toBeInTheDocument();
      });
    });

    it('생성된 코드 보기 details 토글 동작', async () => {
      vi.mocked(tauriApi.generateBiInsight).mockResolvedValueOnce(mockInsight);

      renderWithQueryClient(<BiInsights />);

      const textarea = screen.getByRole('textbox');
      await user.type(textarea, '분석');
      await user.click(screen.getByText('AI 인사이트 생성'));

      // details 요소 확인
      await waitFor(() => {
        const details = screen.getByText('생성된 코드 보기').closest('details');
        expect(details).toBeInTheDocument();
      });
    });

    it('Empty State 숨김 (인사이트 생성 후)', async () => {
      vi.mocked(tauriApi.generateBiInsight).mockResolvedValueOnce(mockInsight);

      renderWithQueryClient(<BiInsights />);

      // 초기 Empty State 확인
      expect(screen.getByText('AI 인사이트를 생성해보세요')).toBeInTheDocument();

      // 인사이트 생성
      const textarea = screen.getByRole('textbox');
      await user.type(textarea, '분석');
      await user.click(screen.getByText('AI 인사이트 생성'));

      // Empty State 숨겨짐
      await waitFor(() => {
        expect(
          screen.queryByText('AI 인사이트를 생성해보세요')
        ).not.toBeInTheDocument();
      });
    });
  });

  // ========================================
  // Group 5: Multiple Generations
  // ========================================
  describe('Group 5: Multiple Generations', () => {
    it('여러 번 생성시 이전 결과 덮어쓰기', async () => {
      const firstInsight = {
        title: '첫 번째 분석',
        insights: ['인사이트 A'],
        recommendations: [],
        component_code: '<div>First</div>',
      };

      const secondInsight = {
        title: '두 번째 분석',
        insights: ['인사이트 B'],
        recommendations: [],
        component_code: '<div>Second</div>',
      };

      vi.mocked(tauriApi.generateBiInsight)
        .mockResolvedValueOnce(firstInsight)
        .mockResolvedValueOnce(secondInsight);

      renderWithQueryClient(<BiInsights />);

      const textarea = screen.getByRole('textbox');
      const generateButton = screen.getByText('AI 인사이트 생성');

      // 첫 번째 생성
      await user.type(textarea, '첫 번째 요청');
      await user.click(generateButton);

      await waitFor(() => {
        expect(screen.getByText('첫 번째 분석')).toBeInTheDocument();
        expect(screen.getByText('인사이트 A')).toBeInTheDocument();
      });

      // 두 번째 생성 (덮어쓰기)
      await user.clear(textarea);
      await user.type(textarea, '두 번째 요청');
      await user.click(generateButton);

      await waitFor(() => {
        expect(screen.getByText('두 번째 분석')).toBeInTheDocument();
        expect(screen.getByText('인사이트 B')).toBeInTheDocument();
      });

      // 첫 번째 결과 사라짐
      expect(screen.queryByText('첫 번째 분석')).not.toBeInTheDocument();
      expect(screen.queryByText('인사이트 A')).not.toBeInTheDocument();
    });
  });

  // ========================================
  // Group 6: Error Handling (Basic)
  // ========================================
  describe('Group 6: Error Handling', () => {
    it('API 오류시 에러 상태 (React Query)', async () => {
      vi.mocked(tauriApi.generateBiInsight).mockRejectedValueOnce(
        new Error('API error')
      );

      renderWithQueryClient(<BiInsights />);

      const textarea = screen.getByRole('textbox');
      await user.type(textarea, '분석');
      await user.click(screen.getByText('AI 인사이트 생성'));

      // React Query가 자동으로 에러 처리
      // (onError 핸들러가 없으므로 UI에 에러 메시지 없음)
      await waitFor(() => {
        // "생성 중..." 사라짐
        expect(screen.queryByText('생성 중...')).not.toBeInTheDocument();
      });

      // 여전히 Empty State 표시
      expect(screen.getByText('AI 인사이트를 생성해보세요')).toBeInTheDocument();
    });
  });
});
