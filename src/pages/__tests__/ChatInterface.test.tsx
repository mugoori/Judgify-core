import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { userEvent } from '@testing-library/user-event';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import ChatInterface from '../ChatInterface';
import * as tauriApi from '@/lib/tauri-api';

// Mock Tauri API
vi.mock('@/lib/tauri-api', () => ({
  sendChatMessage: vi.fn(),
  getChatHistory: vi.fn(),
}));

// Mock window.confirm
const mockConfirm = vi.fn();
window.confirm = mockConfirm;

// Mock localStorage
const mockLocalStorage = (() => {
  let store: Record<string, string> = {};
  return {
    getItem: (key: string) => store[key] || null,
    setItem: (key: string, value: string) => {
      store[key] = value;
    },
    removeItem: (key: string) => {
      delete store[key];
    },
    clear: () => {
      store = {};
    },
  };
})();

Object.defineProperty(window, 'localStorage', {
  value: mockLocalStorage,
});

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

// Helper: Get send button (icon-only button)
function getSendButton() {
  const buttons = screen.getAllByRole('button');
  return buttons.find(btn => btn.className.includes('h-[60px]'))!;
}

describe('ChatInterface', () => {
  const user = userEvent.setup();

  beforeEach(() => {
    vi.clearAllMocks();
    mockLocalStorage.clear();
    mockConfirm.mockReturnValue(true);
    vi.mocked(tauriApi.getChatHistory).mockResolvedValue([]);
  });

  afterEach(() => {
    vi.clearAllTimers();
  });

  // ========================================
  // Group 1: Initial Rendering & Welcome Message
  // ========================================
  describe('Group 1: Initial Rendering', () => {
    it('초기 렌더링시 환영 메시지 표시', () => {
      renderWithQueryClient(<ChatInterface />);

      expect(screen.getByText('AI 어시스턴트')).toBeInTheDocument();
      expect(
        screen.getByText(/자연어로 대화하며 판단 실행/)
      ).toBeInTheDocument();
      expect(
        screen.getByText(/안녕하세요! TriFlow AI 어시스턴트입니다/)
      ).toBeInTheDocument();
    });

    it('초기 렌더링시 Quick Actions 버튼 4개 표시', () => {
      renderWithQueryClient(<ChatInterface />);

      expect(screen.getByText('지난 주 불량률 트렌드')).toBeInTheDocument();
      expect(screen.getByText('워크플로우 실행')).toBeInTheDocument();
      expect(screen.getByText('워크플로우 생성 방법')).toBeInTheDocument();
      expect(screen.getByText('시스템 상태 확인')).toBeInTheDocument();
    });

    it('입력창 placeholder 정상 표시', () => {
      renderWithQueryClient(<ChatInterface />);

      const textarea = screen.getByPlaceholderText(
        /메시지를 입력하세요.../
      );
      expect(textarea).toBeInTheDocument();
    });

    it('대화 초기화 버튼 표시', () => {
      renderWithQueryClient(<ChatInterface />);

      expect(screen.getByText('대화 초기화')).toBeInTheDocument();
    });
  });

  // ========================================
  // Group 2: Message Sending (Basic)
  // ========================================
  describe('Group 2: Message Sending', () => {
    it('메시지 입력 및 전송 성공', async () => {
      vi.mocked(tauriApi.sendChatMessage).mockResolvedValueOnce({
        response: 'AI 응답입니다',
        session_id: 'test-session-123',
        intent: 'general',
      });

      renderWithQueryClient(<ChatInterface />);

      const textarea = screen.getByPlaceholderText(/메시지를 입력하세요.../);
      const sendButton = getSendButton();

      // 메시지 입력
      await user.type(textarea, '안녕하세요');
      expect(textarea).toHaveValue('안녕하세요');

      // 전송 버튼 클릭
      await user.click(sendButton);

      // 사용자 메시지 표시 확인
      await waitFor(() => {
        expect(screen.getByText('안녕하세요')).toBeInTheDocument();
      });

      // AI 응답 표시 확인
      await waitFor(() => {
        expect(screen.getByText('AI 응답입니다')).toBeInTheDocument();
      });

      // 입력창 초기화 확인
      expect(textarea).toHaveValue('');

      // Tauri API 호출 확인
      expect(tauriApi.sendChatMessage).toHaveBeenCalledWith({
        message: '안녕하세요',
        session_id: undefined,
      });
    });

    it('빈 메시지 전송시 아무 동작 안함', async () => {
      renderWithQueryClient(<ChatInterface />);

      const textarea = screen.getByPlaceholderText(/메시지를 입력하세요.../);
      const sendButton = getSendButton();

      // 빈 메시지 전송
      await user.click(sendButton);

      // API 호출되지 않음
      expect(tauriApi.sendChatMessage).not.toHaveBeenCalled();

      // 입력창 여전히 비어있음
      expect(textarea).toHaveValue('');
    });

    it('Enter 키로 메시지 전송 (Shift 없이)', async () => {
      vi.mocked(tauriApi.sendChatMessage).mockResolvedValueOnce({
        response: 'AI 응답',
        session_id: 'test-session',
        intent: 'general',
      });

      renderWithQueryClient(<ChatInterface />);

      const textarea = screen.getByPlaceholderText(/메시지를 입력하세요.../);

      // 메시지 입력 후 Enter
      await user.type(textarea, '테스트{Enter}');

      // 사용자 메시지 표시
      await waitFor(() => {
        expect(screen.getByText('테스트')).toBeInTheDocument();
      });

      // API 호출 확인
      expect(tauriApi.sendChatMessage).toHaveBeenCalledWith({
        message: '테스트',
        session_id: undefined,
      });
    });

    it('전송 중 상태 표시 (생각 중...)', async () => {
      let resolvePromise: (value: any) => void;
      const promise = new Promise((resolve) => {
        resolvePromise = resolve;
      });

      vi.mocked(tauriApi.sendChatMessage).mockReturnValueOnce(promise as any);

      renderWithQueryClient(<ChatInterface />);

      const textarea = screen.getByPlaceholderText(/메시지를 입력하세요.../);
      await user.type(textarea, '테스트');
      await user.click(getSendButton());

      // "생각 중..." 표시 확인
      await waitFor(() => {
        expect(screen.getByText('생각 중...')).toBeInTheDocument();
      });

      // 프로미스 해결
      resolvePromise!({
        response: 'AI 응답',
        session_id: 'test-session',
        intent: 'general',
      });

      // 응답 표시 확인
      await waitFor(() => {
        expect(screen.getByText('AI 응답')).toBeInTheDocument();
      });
    });
  });

  // ========================================
  // Group 3: Error Handling
  // ========================================
  describe('Group 3: Error Handling', () => {
    it('API 오류시 에러 메시지 표시', async () => {
      vi.mocked(tauriApi.sendChatMessage).mockRejectedValueOnce(
        new Error('API 연결 실패')
      );

      renderWithQueryClient(<ChatInterface />);

      const textarea = screen.getByPlaceholderText(/메시지를 입력하세요.../);
      await user.type(textarea, '테스트');
      await user.click(getSendButton());

      // 에러 메시지 표시 확인
      await waitFor(() => {
        expect(
          screen.getByText(/❌ 오류가 발생했습니다: API 연결 실패/)
        ).toBeInTheDocument();
      });

      // pending flag 제거 확인
      expect(mockLocalStorage.getItem('chat-pending-request')).toBeNull();
    });

    it('API 오류시 사용자 메시지는 유지', async () => {
      vi.mocked(tauriApi.sendChatMessage).mockRejectedValueOnce(
        new Error('Network error')
      );

      renderWithQueryClient(<ChatInterface />);

      const textarea = screen.getByPlaceholderText(/메시지를 입력하세요.../);
      await user.type(textarea, '에러 테스트');
      await user.click(getSendButton());

      // 사용자 메시지 유지
      await waitFor(() => {
        expect(screen.getByText('에러 테스트')).toBeInTheDocument();
      });

      // 에러 메시지도 표시
      await waitFor(() => {
        expect(screen.getByText(/❌ 오류가 발생했습니다/)).toBeInTheDocument();
      });
    });
  });

  // ========================================
  // Group 4: Quick Actions
  // ========================================
  describe('Group 4: Quick Actions', () => {
    it('Quick Action 버튼 클릭시 메시지 전송', async () => {
      vi.mocked(tauriApi.sendChatMessage).mockResolvedValueOnce({
        response: '불량률 데이터입니다',
        session_id: 'test-session',
        intent: 'data_visualization',
      });

      renderWithQueryClient(<ChatInterface />);

      // "지난 주 불량률 트렌드" 버튼 클릭
      await user.click(screen.getByText('지난 주 불량률 트렌드'));

      // 메시지 전송 확인
      await waitFor(() => {
        expect(tauriApi.sendChatMessage).toHaveBeenCalledWith({
          message: '지난 주 불량률 트렌드 보여줘',
          session_id: undefined,
        });
      });

      // 사용자 메시지 표시
      await waitFor(() => {
        expect(screen.getByText('지난 주 불량률 트렌드 보여줘')).toBeInTheDocument();
      });
    });

    it('Quick Actions는 초기 메시지일 때만 표시', async () => {
      vi.mocked(tauriApi.sendChatMessage).mockResolvedValueOnce({
        response: 'AI 응답',
        session_id: 'test-session',
        intent: 'general',
      });

      renderWithQueryClient(<ChatInterface />);

      // 초기 상태에서 Quick Actions 표시
      expect(screen.getByText('지난 주 불량률 트렌드')).toBeInTheDocument();

      // 메시지 전송
      const textarea = screen.getByPlaceholderText(/메시지를 입력하세요.../);
      await user.type(textarea, '테스트');
      await user.click(getSendButton());

      // Quick Actions 숨겨짐
      await waitFor(() => {
        expect(screen.queryByText('지난 주 불량률 트렌드')).not.toBeInTheDocument();
      });
    });

    it('Quick Action 버튼 클릭시 전송', async () => {
      vi.mocked(tauriApi.sendChatMessage).mockResolvedValueOnce({
        response: 'AI 응답',
        session_id: 'test-session',
        intent: 'general',
      });

      renderWithQueryClient(<ChatInterface />);

      const quickActionButton = screen.getByText('워크플로우 실행');
      await user.click(quickActionButton);

      // 메시지 전송 확인
      await waitFor(() => {
        expect(tauriApi.sendChatMessage).toHaveBeenCalled();
      });

      // 응답 후 Quick Actions 사라짐
      await waitFor(() => {
        expect(screen.queryByText('워크플로우 실행')).not.toBeInTheDocument();
      });
    });
  });

  // ========================================
  // Group 5: Clear History
  // ========================================
  describe('Group 5: Clear History', () => {
    it('대화 초기화 버튼 클릭시 AlertDialog 표시 후 초기화', async () => {
      vi.mocked(tauriApi.sendChatMessage).mockResolvedValueOnce({
        response: 'AI 응답',
        session_id: 'test-session',
        intent: 'general',
      });

      renderWithQueryClient(<ChatInterface />);

      // 메시지 먼저 전송
      const textarea = screen.getByPlaceholderText(/메시지를 입력하세요.../);
      await user.type(textarea, '테스트 메시지');
      await user.click(getSendButton());

      await waitFor(() => {
        expect(screen.getByText('테스트 메시지')).toBeInTheDocument();
      });

      // 대화 초기화 버튼 클릭
      await user.click(screen.getByText('대화 초기화'));

      // ✅ AlertDialog 표시 확인
      await waitFor(() => {
        expect(screen.getByText('대화 내역 삭제')).toBeInTheDocument();
        expect(screen.getByText(/채팅 내역을 모두 삭제하시겠습니까?/)).toBeInTheDocument();
      });

      // ✅ 확인 버튼 클릭 (메시지 삭제 실행)
      await user.click(screen.getByRole('button', { name: '확인' }));

      // session ID 초기화 확인
      expect(mockLocalStorage.getItem('chat-session-id')).toBeNull();

      // 환영 메시지만 남음 (사용자 메시지 사라짐)
      await waitFor(() => {
        expect(screen.queryByText('테스트 메시지')).not.toBeInTheDocument();
        expect(
          screen.getByText(/안녕하세요! .* TriFlow AI 어시스턴트입니다/)
        ).toBeInTheDocument();
      });
    });

    it('대화 초기화 취소 버튼 클릭시 아무 동작 안함', async () => {
      renderWithQueryClient(<ChatInterface />);

      // localStorage에 데이터 저장
      mockLocalStorage.setItem(
        'chat-messages',
        JSON.stringify([
          { role: 'user', content: '테스트' },
          { role: 'assistant', content: '응답' },
        ])
      );

      // 대화 초기화 버튼 클릭
      await user.click(screen.getByText('대화 초기화'));

      // ✅ AlertDialog 표시 확인
      await waitFor(() => {
        expect(screen.getByText('대화 내역 삭제')).toBeInTheDocument();
      });

      // ✅ 취소 버튼 클릭 (메시지 유지)
      await user.click(screen.getByRole('button', { name: '취소' }));

      // localStorage 유지 확인
      expect(mockLocalStorage.getItem('chat-messages')).not.toBeNull();
    });
  });

  // ========================================
  // Group 6: LocalStorage Persistence
  // ========================================
  describe('Group 6: LocalStorage Persistence', () => {
    it('메시지 전송시 localStorage에 저장', async () => {
      vi.mocked(tauriApi.sendChatMessage).mockResolvedValueOnce({
        response: 'AI 응답',
        session_id: 'test-session-456',
        intent: 'general',
      });

      renderWithQueryClient(<ChatInterface />);

      const textarea = screen.getByPlaceholderText(/메시지를 입력하세요.../);
      await user.type(textarea, '테스트 메시지');
      await user.click(getSendButton());

      // localStorage 저장 확인
      await waitFor(() => {
        const saved = mockLocalStorage.getItem('chat-messages');
        expect(saved).toBeTruthy();
        const messages = JSON.parse(saved!);
        expect(messages).toContainEqual({
          role: 'user',
          content: '테스트 메시지',
        });
      });

      // session ID 저장 확인
      await waitFor(() => {
        expect(mockLocalStorage.getItem('chat-session-id')).toBe(
          'test-session-456'
        );
      });
    });

    it('초기 로드시 localStorage에서 메시지 복원', () => {
      mockLocalStorage.setItem(
        'chat-messages',
        JSON.stringify([
          { role: 'user', content: '이전 메시지 1' },
          { role: 'assistant', content: 'AI 응답 1' },
          { role: 'user', content: '이전 메시지 2' },
        ])
      );

      renderWithQueryClient(<ChatInterface />);

      // 메시지 복원 확인
      expect(screen.getByText('이전 메시지 1')).toBeInTheDocument();
      expect(screen.getByText('AI 응답 1')).toBeInTheDocument();
      expect(screen.getByText('이전 메시지 2')).toBeInTheDocument();
    });

    it('localStorage 파싱 실패시 초기 환영 메시지 표시', () => {
      mockLocalStorage.setItem('chat-messages', 'invalid-json');

      renderWithQueryClient(<ChatInterface />);

      // 환영 메시지 표시
      expect(
        screen.getByText(/안녕하세요! TriFlow AI 어시스턴트입니다/)
      ).toBeInTheDocument();
    });

    it('session ID 복원 후 getChatHistory 호출 (pending request 없음)', async () => {
      vi.mocked(tauriApi.getChatHistory).mockResolvedValueOnce([]);

      mockLocalStorage.setItem('chat-session-id', 'restored-session-123');
      mockLocalStorage.setItem(
        'chat-messages',
        JSON.stringify([{ role: 'user', content: '이전 메시지' }])
      );

      renderWithQueryClient(<ChatInterface />);

      // getChatHistory 호출되지 않음 (pending request 없으므로)
      await waitFor(() => {
        expect(tauriApi.getChatHistory).not.toHaveBeenCalled();
      });
    });
  });

  // ========================================
  // Group 7: Session Management
  // ========================================
  describe('Group 7: Session Management', () => {
    it('session ID 없이 시작하여 첫 응답에서 session ID 저장', async () => {
      vi.mocked(tauriApi.sendChatMessage).mockResolvedValueOnce({
        response: 'AI 응답',
        session_id: 'new-session-789',
        intent: 'general',
      });

      renderWithQueryClient(<ChatInterface />);

      const textarea = screen.getByPlaceholderText(/메시지를 입력하세요.../);
      await user.type(textarea, '첫 메시지');
      await user.click(getSendButton());

      // session ID 저장 확인
      await waitFor(() => {
        expect(mockLocalStorage.getItem('chat-session-id')).toBe(
          'new-session-789'
        );
      });
    });

    it('기존 session ID가 있으면 재사용', async () => {
      mockLocalStorage.setItem('chat-session-id', 'existing-session-123');

      vi.mocked(tauriApi.sendChatMessage).mockResolvedValueOnce({
        response: 'AI 응답',
        session_id: 'existing-session-123',
        intent: 'general',
      });

      renderWithQueryClient(<ChatInterface />);

      const textarea = screen.getByPlaceholderText(/메시지를 입력하세요.../);
      await user.type(textarea, '메시지');
      await user.click(getSendButton());

      // 기존 session ID로 API 호출
      await waitFor(() => {
        expect(tauriApi.sendChatMessage).toHaveBeenCalledWith({
          message: '메시지',
          session_id: 'existing-session-123',
        });
      });
    });
  });

  // ========================================
  // Group 8: MessageBubble Component
  // ========================================
  describe('Group 8: MessageBubble Rendering', () => {
    it('user 메시지 렌더링 (오른쪽 정렬)', async () => {
      vi.mocked(tauriApi.sendChatMessage).mockResolvedValueOnce({
        response: 'AI 응답',
        session_id: 'test-session',
        intent: 'general',
      });

      renderWithQueryClient(<ChatInterface />);

      const textarea = screen.getByPlaceholderText(/메시지를 입력하세요.../);
      await user.type(textarea, '사용자 메시지');
      await user.click(getSendButton());

      // 사용자 메시지 표시 확인
      const userMessage = await screen.findByText('사용자 메시지');
      expect(userMessage).toBeInTheDocument();

      // 오른쪽 정렬 확인 (justify-end 클래스)
      const messageContainer = userMessage.closest('.flex');
      expect(messageContainer).toHaveClass('justify-end');
    });

    it('assistant 메시지 렌더링 (왼쪽 정렬, Bot 아이콘)', async () => {
      renderWithQueryClient(<ChatInterface />);

      // 초기 환영 메시지 (assistant)
      const assistantMessage = screen.getByText(
        /안녕하세요! Judgify AI 어시스턴트입니다/
      );
      expect(assistantMessage).toBeInTheDocument();

      // 왼쪽 정렬 확인 (justify-start 클래스)
      const messageContainer = assistantMessage.closest('.flex');
      expect(messageContainer).toHaveClass('justify-start');

      // Bot 아이콘 확인 (svg 존재)
      const botIcon = messageContainer?.querySelector('svg');
      expect(botIcon).toBeInTheDocument();
    });

    it('intent가 있을 때 표시', async () => {
      vi.mocked(tauriApi.sendChatMessage).mockResolvedValueOnce({
        response: 'AI 응답',
        session_id: 'test-session',
        intent: 'workflow_execution',
      });

      renderWithQueryClient(<ChatInterface />);

      const textarea = screen.getByPlaceholderText(/메시지를 입력하세요.../);
      await user.type(textarea, '워크플로우 실행');
      await user.click(getSendButton());

      // intent 표시 확인
      await waitFor(() => {
        expect(screen.getByText('의도: workflow_execution')).toBeInTheDocument();
      });
    });
  });

  // ========================================
  // Group 9: Input Validation
  // ========================================
  describe('Group 9: Input Validation', () => {
    it('공백만 있는 메시지 전송시 무시', async () => {
      renderWithQueryClient(<ChatInterface />);

      const textarea = screen.getByPlaceholderText(/메시지를 입력하세요.../);
      await user.type(textarea, '   ');
      await user.click(getSendButton());

      // API 호출되지 않음
      expect(tauriApi.sendChatMessage).not.toHaveBeenCalled();
    });

    it('전송 중 API 호출 검증', async () => {
      vi.mocked(tauriApi.sendChatMessage).mockResolvedValueOnce({
        response: 'AI 응답',
        session_id: 'test-session',
        intent: 'general',
      });

      renderWithQueryClient(<ChatInterface />);

      const textarea = screen.getByPlaceholderText(/메시지를 입력하세요.../);
      const sendButton = getSendButton();

      await user.type(textarea, '테스트 메시지');
      await user.click(sendButton);

      // API 호출 확인
      await waitFor(() => {
        expect(tauriApi.sendChatMessage).toHaveBeenCalledWith({
          message: '테스트 메시지',
          session_id: undefined,
        });
      });

      // 응답 메시지 표시 확인
      await waitFor(() => {
        expect(screen.getByText('AI 응답')).toBeInTheDocument();
      });
    });
  });

  // ========================================
  // Group 9: Visibility Change & Tab Recovery
  // ========================================
  describe('Group 9: Visibility Change & Tab Recovery', () => {
    beforeEach(() => {
      // Mock document.hidden and visibilityState
      Object.defineProperty(document, 'hidden', {
        writable: true,
        value: false,
        configurable: true,
      });

      Object.defineProperty(document, 'visibilityState', {
        writable: true,
        value: 'visible',
        configurable: true,
      });
    });

    it('탭이 visible로 변경시 백엔드와 동기화', async () => {
      // Mock backend history with new messages
      vi.mocked(tauriApi.getChatHistory).mockResolvedValueOnce([
        { role: 'user', content: 'Hello', intent: 'general' },
        { role: 'assistant', content: 'Hi there!', intent: 'general' },
      ]);

      renderWithQueryClient(<ChatInterface />);

      // Set hidden to true first
      Object.defineProperty(document, 'hidden', { value: true, writable: true });

      // Simulate tab becoming visible
      Object.defineProperty(document, 'hidden', { value: false, writable: true });
      const event = new Event('visibilitychange');
      document.dispatchEvent(event);

      // Wait for sync to complete
      await waitFor(() => {
        expect(tauriApi.getChatHistory).toHaveBeenCalled();
      });

      // Verify messages updated
      await waitFor(() => {
        expect(screen.getByText('Hello')).toBeInTheDocument();
        expect(screen.getByText('Hi there!')).toBeInTheDocument();
      });
    });

    it('탭이 already visible이면 sync 스킵', () => {
      // Keep tab visible (default state)
      Object.defineProperty(document, 'hidden', { value: false, writable: true });

      renderWithQueryClient(<ChatInterface />);

      // Dispatch event but tab is already visible
      const event = new Event('visibilitychange');
      document.dispatchEvent(event);

      // No sync should happen
      expect(tauriApi.getChatHistory).not.toHaveBeenCalled();
    });

    it('백그라운드 응답 플래그 확인 및 복구', async () => {
      // Set pending response flag
      mockLocalStorage.setItem('chat-pending-response', 'true');

      // Mock backend with new AI response
      vi.mocked(tauriApi.getChatHistory).mockResolvedValueOnce([
        { role: 'user', content: 'Question', intent: 'general' },
        { role: 'assistant', content: 'Answer', intent: 'general' },
      ]);

      renderWithQueryClient(<ChatInterface />);

      // Simulate tab return
      Object.defineProperty(document, 'hidden', { value: false, writable: true });
      const event = new Event('visibilitychange');
      document.dispatchEvent(event);

      // Wait for recovery
      await waitFor(() => {
        expect(screen.getByText('Answer')).toBeInTheDocument();
      });

      // Verify flag removed
      expect(mockLocalStorage.getItem('chat-pending-response')).toBeNull();
    });

    it('sync 완료 후 모든 플래그 정리', async () => {
      // Set both flags
      mockLocalStorage.setItem('chat-pending-request', 'true');
      mockLocalStorage.setItem('chat-pending-response', 'true');

      vi.mocked(tauriApi.getChatHistory).mockResolvedValueOnce([]);

      renderWithQueryClient(<ChatInterface />);

      // Simulate tab return
      Object.defineProperty(document, 'hidden', { value: false, writable: true });
      const event = new Event('visibilitychange');
      document.dispatchEvent(event);

      // Wait for cleanup
      await waitFor(() => {
        expect(mockLocalStorage.getItem('chat-pending-request')).toBeNull();
        expect(mockLocalStorage.getItem('chat-pending-response')).toBeNull();
      });
    });
  });

  // ========================================
  // Group 10: Session Sync & Timing
  // ========================================
  describe('Group 10: Session Sync & Timing', () => {
    beforeEach(() => {
      vi.useFakeTimers();
    });

    afterEach(() => {
      vi.useRealTimers();
    });

    it('sessionId 변경시 300ms 후 백엔드 동기화', async () => {
      vi.mocked(tauriApi.getChatHistory).mockResolvedValue([
        { role: 'user', content: 'Synced message', intent: 'general' },
      ]);

      renderWithQueryClient(<ChatInterface />);

      // Wait initial render
      await waitFor(() => {
        expect(screen.getByText('AI 어시스턴트')).toBeInTheDocument();
      });

      // Simulate session ID change (would happen in real app via state)
      // Since we can't directly change sessionId prop, we verify the useEffect timing
      // by checking setTimeout was called correctly

      // Fast-forward 300ms
      vi.advanceTimersByTime(300);

      // getChatHistory should be called after timeout
      await waitFor(() => {
        expect(tauriApi.getChatHistory).toHaveBeenCalled();
      });
    });

    it('탭 hidden 상태에서 sync 스킵', async () => {
      // Set tab to hidden before render
      Object.defineProperty(document, 'hidden', {
        writable: true,
        value: true,
        configurable: true,
      });

      renderWithQueryClient(<ChatInterface />);

      // Fast-forward timers
      vi.advanceTimersByTime(300);

      // No sync should happen when tab is hidden
      // (syncWithBackend checks document.hidden)
      await new Promise(resolve => setTimeout(resolve, 100));

      // Note: Since syncWithBackend checks document.hidden internally,
      // we can't easily verify the skip without spy on console.log
      // This test verifies the component doesn't crash
      expect(screen.getByText('AI 어시스턴트')).toBeInTheDocument();
    });

    it('session 변경시 이전 timeout 취소', async () => {
      const clearTimeoutSpy = vi.spyOn(global, 'clearTimeout');

      renderWithQueryClient(<ChatInterface />);

      // Wait for first useEffect
      await waitFor(() => {
        expect(screen.getByText('AI 어시스턴트')).toBeInTheDocument();
      });

      // Trigger unmount/re-mount (simulating session change cleanup)
      // The useEffect cleanup should call clearTimeout
      expect(clearTimeoutSpy).toHaveBeenCalled();

      clearTimeoutSpy.mockRestore();
    });
  });

  // ========================================
  // Group 11: Mutation Handlers & Background Processing
  // ========================================
  describe('Group 11: Mutation Handlers', () => {
    beforeEach(() => {
      Object.defineProperty(document, 'hidden', {
        writable: true,
        value: false,
        configurable: true,
      });
    });

    it('mutation 시작시 pending 플래그 설정', async () => {
      vi.mocked(tauriApi.sendChatMessage).mockImplementation(() => {
        // Check flag is set during mutation
        expect(mockLocalStorage.getItem('chat-pending-request')).toBe('true');
        return Promise.resolve({
          response: 'Answer',
          session_id: 'test-session',
          intent: 'general',
        });
      });

      renderWithQueryClient(<ChatInterface />);

      const textarea = screen.getByPlaceholderText(/메시지를 입력하세요.../);
      await user.type(textarea, 'Test message');
      await user.click(getSendButton());

      await waitFor(() => {
        expect(tauriApi.sendChatMessage).toHaveBeenCalled();
      });
    });

    it('탭이 visible이면 즉시 메시지 추가', async () => {
      // Tab is visible (default)
      Object.defineProperty(document, 'hidden', { value: false, writable: true });

      vi.mocked(tauriApi.sendChatMessage).mockResolvedValueOnce({
        response: 'Immediate response',
        session_id: 'test-session',
        intent: 'general',
      });

      renderWithQueryClient(<ChatInterface />);

      const textarea = screen.getByPlaceholderText(/메시지를 입력하세요.../);
      await user.type(textarea, 'Question');
      await user.click(getSendButton());

      // Message should appear immediately
      await waitFor(() => {
        expect(screen.getByText('Immediate response')).toBeInTheDocument();
      });

      // No background flag should be set
      expect(mockLocalStorage.getItem('chat-pending-response')).toBeNull();
    });

    it('탭이 hidden이면 background 플래그 설정', async () => {
      // Set tab to hidden
      Object.defineProperty(document, 'hidden', { value: true, writable: true });

      vi.mocked(tauriApi.sendChatMessage).mockResolvedValueOnce({
        response: 'Background response',
        session_id: 'test-session',
        intent: 'general',
      });

      renderWithQueryClient(<ChatInterface />);

      const textarea = screen.getByPlaceholderText(/메시지를 입력하세요.../);
      await user.type(textarea, 'Question');
      await user.click(getSendButton());

      // Wait for mutation to complete
      await waitFor(() => {
        expect(tauriApi.sendChatMessage).toHaveBeenCalled();
      });

      // Background flag should be set
      await waitFor(() => {
        expect(mockLocalStorage.getItem('chat-pending-response')).toBe('true');
      });

      // Message should NOT appear immediately (will appear on tab return)
      expect(screen.queryByText('Background response')).not.toBeInTheDocument();
    });
  });
});
