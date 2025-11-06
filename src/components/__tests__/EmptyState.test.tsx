import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Inbox } from 'lucide-react';
import EmptyState from '../EmptyState';

describe('EmptyState', () => {
  it('기본 렌더링 - 아이콘, 제목, 설명 표시', () => {
    render(
      <EmptyState
        icon={Inbox}
        title="비어있음"
        description="데이터가 없습니다"
      />
    );

    // 제목 확인
    expect(screen.getByText('비어있음')).toBeInTheDocument();

    // 설명 확인
    expect(screen.getByText('데이터가 없습니다')).toBeInTheDocument();

    // 아이콘이 렌더링되는지 확인 (SVG 요소)
    const svgElements = document.querySelectorAll('svg');
    expect(svgElements.length).toBeGreaterThan(0);
  });

  it('액션 버튼이 있을 때 렌더링', () => {
    const mockAction = vi.fn();

    render(
      <EmptyState
        icon={Inbox}
        title="비어있음"
        description="데이터가 없습니다"
        actionLabel="새로 만들기"
        onAction={mockAction}
      />
    );

    // 버튼이 표시되는지 확인
    const button = screen.getByRole('button', { name: '새로 만들기' });
    expect(button).toBeInTheDocument();
  });

  it('액션 버튼 클릭시 핸들러 호출', async () => {
    const user = userEvent.setup();
    const mockAction = vi.fn();

    render(
      <EmptyState
        icon={Inbox}
        title="비어있음"
        description="데이터가 없습니다"
        actionLabel="새로 만들기"
        onAction={mockAction}
      />
    );

    const button = screen.getByRole('button', { name: '새로 만들기' });
    await user.click(button);

    expect(mockAction).toHaveBeenCalledTimes(1);
  });

  it('액션 라벨만 있고 핸들러 없으면 버튼 미표시', () => {
    render(
      <EmptyState
        icon={Inbox}
        title="비어있음"
        description="데이터가 없습니다"
        actionLabel="새로 만들기"
        // onAction 없음
      />
    );

    // 버튼이 없어야 함
    const button = screen.queryByRole('button', { name: '새로 만들기' });
    expect(button).not.toBeInTheDocument();
  });

  it('핸들러만 있고 액션 라벨 없으면 버튼 미표시', () => {
    const mockAction = vi.fn();

    render(
      <EmptyState
        icon={Inbox}
        title="비어있음"
        description="데이터가 없습니다"
        // actionLabel 없음
        onAction={mockAction}
      />
    );

    // 버튼이 없어야 함
    expect(screen.queryByRole('button')).not.toBeInTheDocument();
  });

  it('children prop 렌더링', () => {
    render(
      <EmptyState
        icon={Inbox}
        title="비어있음"
        description="데이터가 없습니다"
      >
        <div>추가 콘텐츠</div>
      </EmptyState>
    );

    expect(screen.getByText('추가 콘텐츠')).toBeInTheDocument();
  });

  it('긴 설명 텍스트 렌더링', () => {
    const longDescription =
      '이것은 매우 긴 설명 텍스트입니다. ' +
      '여러 줄에 걸쳐 표시될 수 있으며 ' +
      '사용자에게 상세한 정보를 제공합니다.';

    render(
      <EmptyState
        icon={Inbox}
        title="비어있음"
        description={longDescription}
      />
    );

    expect(screen.getByText(longDescription)).toBeInTheDocument();
  });

  it('다양한 아이콘 타입 렌더링 가능', () => {
    const { rerender } = render(
      <EmptyState
        icon={Inbox}
        title="제목"
        description="설명"
      />
    );

    // 첫 번째 렌더링
    expect(screen.getByText('제목')).toBeInTheDocument();

    // 다른 아이콘으로 리렌더링
    rerender(
      <EmptyState
        icon={Inbox}
        title="새 제목"
        description="새 설명"
      />
    );

    expect(screen.getByText('새 제목')).toBeInTheDocument();
    expect(screen.getByText('새 설명')).toBeInTheDocument();
  });

  it('버튼과 children 동시 렌더링', async () => {
    const user = userEvent.setup();
    const mockAction = vi.fn();

    render(
      <EmptyState
        icon={Inbox}
        title="비어있음"
        description="데이터가 없습니다"
        actionLabel="새로 만들기"
        onAction={mockAction}
      >
        <p>도움말 텍스트</p>
      </EmptyState>
    );

    // 버튼 확인
    const button = screen.getByRole('button', { name: '새로 만들기' });
    expect(button).toBeInTheDocument();

    // children 확인
    expect(screen.getByText('도움말 텍스트')).toBeInTheDocument();

    // 버튼 클릭 동작 확인
    await user.click(button);
    expect(mockAction).toHaveBeenCalledTimes(1);
  });

  it('Card 컴포넌트 스타일 적용 확인', () => {
    const { container } = render(
      <EmptyState
        icon={Inbox}
        title="비어있음"
        description="데이터가 없습니다"
      />
    );

    // border-dashed 클래스 확인 (Card에 적용됨)
    const cardElement = container.querySelector('.border-dashed');
    expect(cardElement).toBeInTheDocument();
  });
});
