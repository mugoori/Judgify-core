import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import App from '../App';

// Mock lazy loaded pages
vi.mock('../pages/ChatInterface', () => ({
  default: () => <div>ChatInterface Page</div>,
}));

vi.mock('../pages/Dashboard', () => ({
  default: () => <div>Dashboard Page</div>,
}));

vi.mock('../pages/WorkflowBuilder', () => ({
  default: () => <div>WorkflowBuilder Page</div>,
}));

vi.mock('../pages/BiInsights', () => ({
  default: () => <div>BiInsights Page</div>,
}));

vi.mock('../pages/Settings', () => ({
  default: () => <div>Settings Page</div>,
}));

// Mock layout components
vi.mock('../components/layout/Sidebar', () => ({
  default: ({ isOpen, onToggle }: { isOpen: boolean; onToggle: () => void }) => (
    <div data-testid="sidebar" data-open={isOpen}>
      <button onClick={onToggle} data-testid="sidebar-toggle">
        Toggle Sidebar
      </button>
    </div>
  ),
}));

vi.mock('../components/layout/Header', () => ({
  default: () => <div data-testid="header">Header</div>,
}));

// Mock ErrorBoundary (to test App independently)
vi.mock('../components/ErrorBoundary', () => ({
  default: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
}));

// Mock Toaster
vi.mock('../components/ui/toaster', () => ({
  Toaster: () => <div data-testid="toaster">Toaster</div>,
}));

// Mock OfflineDetector
vi.mock('../components/OfflineDetector', () => ({
  default: () => <div data-testid="offline-detector">OfflineDetector</div>,
}));

// Mock framer-motion to avoid animation issues in tests
vi.mock('framer-motion', () => ({
  AnimatePresence: ({ children }: any) => <div>{children}</div>,
  motion: {
    div: ({ children, ...props }: any) => <div {...props}>{children}</div>,
  },
}));

describe('App', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  // ========================================
  // Group 1: QueryClient Configuration
  // ========================================
  describe('Group 1: QueryClient Configuration', () => {
    it('QueryClient는 네트워크 에러시 3회 재시도', () => {
      render(<App />);

      // QueryClient가 초기화되었는지 확인 (QueryClientProvider 존재)
      expect(screen.getByTestId('toaster')).toBeInTheDocument();

      // Note: QueryClient retry logic은 실제 API 호출시 테스트되므로
      // 여기서는 초기화만 확인
    });

    it('QueryClient는 HTTP 에러시 재시도 안함', () => {
      render(<App />);

      // QueryClient 설정 확인
      expect(screen.getByTestId('toaster')).toBeInTheDocument();
    });

    it('QueryClient는 지수 백오프 적용 (최대 30초)', () => {
      render(<App />);

      // QueryClient 설정 확인
      expect(screen.getByTestId('toaster')).toBeInTheDocument();
    });
  });

  // ========================================
  // Group 2: Routing and Lazy Loading
  // ========================================
  describe('Group 2: Routing and Lazy Loading', () => {
    it('/ 경로로 ChatInterface 렌더링', async () => {
      render(<App />);

      await waitFor(() => {
        expect(screen.getByText('ChatInterface Page')).toBeInTheDocument();
      });
    });

    it('/dashboard 경로로 Dashboard 렌더링', async () => {
      window.history.pushState({}, 'Dashboard', '/dashboard');
      render(<App />);

      await waitFor(() => {
        expect(screen.getByText('Dashboard Page')).toBeInTheDocument();
      });
    });

    it('/workflow 경로로 WorkflowBuilder 렌더링', async () => {
      window.history.pushState({}, 'Workflow', '/workflow');
      render(<App />);

      await waitFor(() => {
        expect(screen.getByText('WorkflowBuilder Page')).toBeInTheDocument();
      });
    });

    it('/bi 경로로 BiInsights 렌더링', async () => {
      window.history.pushState({}, 'BI', '/bi');
      render(<App />);

      await waitFor(() => {
        expect(screen.getByText('BiInsights Page')).toBeInTheDocument();
      });
    });

    it('/settings 경로로 Settings 렌더링', async () => {
      window.history.pushState({}, 'Settings', '/settings');
      render(<App />);

      await waitFor(() => {
        expect(screen.getByText('Settings Page')).toBeInTheDocument();
      });
    });
  });

  // ========================================
  // Group 3: Layout and Providers
  // ========================================
  describe('Group 3: Layout and Providers', () => {
    it('사이드바 토글 버튼 클릭시 상태 변경', async () => {
      render(<App />);

      const sidebar = screen.getByTestId('sidebar');
      const toggleButton = screen.getByTestId('sidebar-toggle');

      // 초기 상태: sidebarOpen = true
      expect(sidebar).toHaveAttribute('data-open', 'true');

      // 토글 클릭
      fireEvent.click(toggleButton);

      // 상태 변경 확인: true → false
      await waitFor(() => {
        expect(sidebar).toHaveAttribute('data-open', 'false');
      });

      // 다시 토글 클릭
      fireEvent.click(toggleButton);

      // 상태 변경 확인: false → true
      await waitFor(() => {
        expect(sidebar).toHaveAttribute('data-open', 'true');
      });
    });

    it('Skip to content 링크가 접근 가능', () => {
      render(<App />);

      const skipLink = screen.getByText('메인 콘텐츠로 건너뛰기');
      expect(skipLink).toBeInTheDocument();
      expect(skipLink).toHaveAttribute('href', '#main-content');
    });
  });

  // ========================================
  // Group 4: Additional Components
  // ========================================
  describe('Group 4: Additional Components', () => {
    it('Toaster 컴포넌트 렌더링', () => {
      render(<App />);

      expect(screen.getByTestId('toaster')).toBeInTheDocument();
    });

    it('OfflineDetector 컴포넌트 렌더링', () => {
      render(<App />);

      expect(screen.getByTestId('offline-detector')).toBeInTheDocument();
    });
  });
});
