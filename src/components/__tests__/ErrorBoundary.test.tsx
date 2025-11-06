import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import ErrorBoundary from '../ErrorBoundary';

// Mock window.location
const mockReload = vi.fn();
const originalLocation = window.location;

beforeEach(() => {
  // Mock console.error to suppress error logs during tests
  vi.spyOn(console, 'error').mockImplementation(() => {});

  // Mock window.location
  delete (window as any).location;
  (window as any).location = {
    ...originalLocation,
    reload: mockReload,
    href: '/',
  };
});

afterEach(() => {
  vi.restoreAllMocks();
  window.location = originalLocation;
});

// Component that throws error on demand
const ThrowError = ({ shouldThrow }: { shouldThrow: boolean }) => {
  if (shouldThrow) {
    throw new Error('Test error message');
  }
  return <div>No error</div>;
};

// Component that throws error with custom message
const ThrowCustomError = ({ message }: { message: string }) => {
  throw new Error(message);
};

describe('ErrorBoundary', () => {
  // ========================================
  // Group 1: Error Catching and State Management
  // ========================================
  describe('Group 1: Error Catching and State Management', () => {
    it('자식 컴포넌트의 에러를 캐치', () => {
      render(
        <ErrorBoundary>
          <ThrowError shouldThrow={true} />
        </ErrorBoundary>
      );

      // 에러 UI가 렌더링됨
      expect(screen.getByText('앱에서 오류가 발생했습니다')).toBeInTheDocument();
      expect(screen.getByText('예상치 못한 오류가 발생했습니다. 앱을 다시 시작해 주세요.')).toBeInTheDocument();
    });

    it('componentDidCatch가 에러 정보를 상태에 저장', () => {
      render(
        <ErrorBoundary>
          <ThrowError shouldThrow={true} />
        </ErrorBoundary>
      );

      // 에러 메시지가 표시됨
      expect(screen.getByText('Test error message')).toBeInTheDocument();

      // console.error가 호출됨 (ErrorBoundary.tsx line 35)
      expect(console.error).toHaveBeenCalled();
    });

    it('에러가 없을 때 자식 컴포넌트를 정상 렌더링', () => {
      render(
        <ErrorBoundary>
          <ThrowError shouldThrow={false} />
        </ErrorBoundary>
      );

      // 자식 컴포넌트가 정상 렌더링됨
      expect(screen.getByText('No error')).toBeInTheDocument();

      // 에러 UI가 렌더링되지 않음
      expect(screen.queryByText('앱에서 오류가 발생했습니다')).not.toBeInTheDocument();
    });
  });

  // ========================================
  // Group 2: Error UI Display
  // ========================================
  describe('Group 2: Error UI Display', () => {
    it('에러 메시지를 UI에 표시', () => {
      render(
        <ErrorBoundary>
          <ThrowCustomError message="Custom error message" />
        </ErrorBoundary>
      );

      // 커스텀 에러 메시지 확인
      expect(screen.getByText('Custom error message')).toBeInTheDocument();
    });

    it('개발 모드에서만 에러 스택 표시', () => {
      // DEV 모드 설정
      const originalEnv = import.meta.env.DEV;
      import.meta.env.DEV = true;

      render(
        <ErrorBoundary>
          <ThrowError shouldThrow={true} />
        </ErrorBoundary>
      );

      // 개발자용 세부 정보 확인
      expect(screen.getByText('기술적 세부 정보 (개발자용)')).toBeInTheDocument();

      // 원래 환경 복원
      import.meta.env.DEV = originalEnv;
    });

    it('프로덕션 모드에서는 에러 스택 숨김', () => {
      // Production 모드 설정
      const originalEnv = import.meta.env.DEV;
      import.meta.env.DEV = false;

      render(
        <ErrorBoundary>
          <ThrowError shouldThrow={true} />
        </ErrorBoundary>
      );

      // 개발자용 세부 정보가 없음
      expect(screen.queryByText('기술적 세부 정보 (개발자용)')).not.toBeInTheDocument();

      // 원래 환경 복원
      import.meta.env.DEV = originalEnv;
    });
  });

  // ========================================
  // Group 3: Error Recovery Actions
  // ========================================
  describe('Group 3: Error Recovery Actions', () => {
    it('"앱 다시 시작" 버튼 클릭시 window.location.reload() 호출', () => {
      render(
        <ErrorBoundary>
          <ThrowError shouldThrow={true} />
        </ErrorBoundary>
      );

      // "앱 다시 시작" 버튼 찾기
      const resetButton = screen.getByText('앱 다시 시작');
      expect(resetButton).toBeInTheDocument();

      // 버튼 클릭
      fireEvent.click(resetButton);

      // window.location.reload() 호출 확인
      expect(mockReload).toHaveBeenCalledTimes(1);
    });

    it('"홈으로 이동" 버튼 클릭시 window.location.href 변경', () => {
      render(
        <ErrorBoundary>
          <ThrowError shouldThrow={true} />
        </ErrorBoundary>
      );

      // "홈으로 이동" 버튼 찾기
      const homeButton = screen.getByText('홈으로 이동');
      expect(homeButton).toBeInTheDocument();

      // 버튼 클릭
      fireEvent.click(homeButton);

      // window.location.href 변경 확인
      expect(window.location.href).toBe('/');
    });
  });
});
