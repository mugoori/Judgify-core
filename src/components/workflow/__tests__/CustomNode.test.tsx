/**
 * CustomNode Backward Compatibility Tests (Week 5 Day 1)
 *
 * v1 워크플로우(4가지 레거시 노드 타입)가 새로운 CustomNode 컴포넌트에서
 * 정상 작동하는지 검증합니다.
 */

import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { ReactFlowProvider } from 'reactflow';
import CustomNode from '../CustomNode';
import { NodeType } from '@/types/workflow';

// React Flow Provider로 감싼 헬퍼 함수
const renderNode = (data: any, selected = false) => {
  return render(
    <ReactFlowProvider>
      <CustomNode
        id="test-node"
        type="custom"
        data={data}
        selected={selected}
        isConnectable={true}
        xPos={0}
        yPos={0}
        zIndex={0}
        dragging={false}
      />
    </ReactFlowProvider>
  );
};

describe('CustomNode - Backward Compatibility (v1 Legacy Types)', () => {
  describe('1. v1 노드 타입 렌더링', () => {
    it('INPUT 노드를 정상 렌더링한다', () => {
      renderNode({
        label: 'Input Node',
        type: 'input', // v1 문자열 타입
        description: 'Legacy input node',
      });

      expect(screen.getByText('Input Node')).toBeInTheDocument();
      expect(screen.getByText('Legacy input node')).toBeInTheDocument();
    });

    it('DECISION 노드를 정상 렌더링한다', () => {
      renderNode({
        label: 'Decision Node',
        type: 'decision', // v1 문자열 타입
        rule: 'temperature > 90',
      });

      expect(screen.getByText('Decision Node')).toBeInTheDocument();
      expect(screen.getByText('temperature > 90')).toBeInTheDocument();
    });

    it('ACTION 노드를 정상 렌더링한다', () => {
      renderNode({
        label: 'Action Node',
        type: 'action', // v1 문자열 타입
        description: 'Send notification',
      });

      expect(screen.getByText('Action Node')).toBeInTheDocument();
      expect(screen.getByText('Send notification')).toBeInTheDocument();
    });

    it('OUTPUT 노드를 정상 렌더링한다', () => {
      renderNode({
        label: 'Output Node',
        type: 'output', // v1 문자열 타입
      });

      expect(screen.getByText('Output Node')).toBeInTheDocument();
    });
  });

  describe('2. 노드 타입별 스타일 정확성', () => {
    it('INPUT 노드는 파란색 스타일을 가진다', () => {
      const { container } = renderNode({
        label: 'Input',
        type: 'input',
      });

      const nodeElement = container.querySelector('.bg-blue-50');
      expect(nodeElement).toBeInTheDocument();
    });

    it('DECISION 노드는 보라색 스타일을 가진다', () => {
      const { container } = renderNode({
        label: 'Decision',
        type: 'decision',
        rule: 'test',
      });

      const nodeElement = container.querySelector('.bg-purple-50');
      expect(nodeElement).toBeInTheDocument();
    });

    it('ACTION 노드는 노란색 스타일을 가진다', () => {
      const { container } = renderNode({
        label: 'Action',
        type: 'action',
      });

      const nodeElement = container.querySelector('.bg-yellow-50');
      expect(nodeElement).toBeInTheDocument();
    });

    it('OUTPUT 노드는 초록색 스타일을 가진다', () => {
      const { container } = renderNode({
        label: 'Output',
        type: 'output',
      });

      const nodeElement = container.querySelector('.bg-green-50');
      expect(nodeElement).toBeInTheDocument();
    });
  });

  describe('3. 레거시 문자열 → NodeType Enum 자동 변환', () => {
    it('문자열 "input"이 NodeType.INPUT으로 처리된다', () => {
      const { container } = renderNode({
        label: 'Test',
        type: 'input', // 문자열
      });

      // INPUT 노드 특성: 입력 핸들 없음, 출력 핸들 있음
      const outputHandles = container.querySelectorAll('[data-handlepos="bottom"]');
      expect(outputHandles.length).toBeGreaterThan(0);
    });

    it('문자열 "decision"이 NodeType.DECISION으로 처리된다', () => {
      const { container } = renderNode({
        label: 'Test',
        type: 'decision', // 문자열
        rule: 'x > 0',
      });

      // DECISION 노드 특성: 분기 핸들 (true/false)
      const branchHandles = container.querySelectorAll('.bg-green-500, .bg-red-500');
      expect(branchHandles.length).toBe(2); // true + false 핸들
    });

    it('문자열 "output"이 NodeType.OUTPUT으로 처리된다', () => {
      const { container } = renderNode({
        label: 'Test',
        type: 'output', // 문자열
      });

      // OUTPUT 노드 특성: 출력 핸들 없음, 입력 핸들 있음
      const inputHandles = container.querySelectorAll('[data-handlepos="top"]');
      expect(inputHandles.length).toBeGreaterThan(0);
    });
  });

  describe('4. decision 노드의 rule 표시', () => {
    it('rule이 있으면 표시한다', () => {
      renderNode({
        label: 'Decision',
        type: 'decision',
        rule: 'temperature > 90 && pressure < 50',
      });

      expect(screen.getByText('temperature > 90 && pressure < 50')).toBeInTheDocument();
    });

    it('rule이 없으면 표시하지 않는다', () => {
      const { container } = renderNode({
        label: 'Decision',
        type: 'decision',
      });

      const ruleBox = container.querySelector('.border-purple-200');
      expect(ruleBox).not.toBeInTheDocument();
    });
  });

  describe('5. 분기 핸들 표시 (DECISION 노드)', () => {
    it('DECISION 노드는 true/false 핸들을 가진다', () => {
      const { container } = renderNode({
        label: 'Decision',
        type: 'decision',
        rule: 'test',
      });

      // React Flow 핸들 구조 확인 (data-handlepos 속성 사용)
      const rightHandle = container.querySelector('[data-handlepos="right"]');
      const leftHandle = container.querySelector('[data-handlepos="left"]');

      expect(rightHandle).toBeInTheDocument();
      expect(leftHandle).toBeInTheDocument();
    });

    it('right 핸들(true)은 초록색이다', () => {
      const { container } = renderNode({
        label: 'Decision',
        type: 'decision',
        rule: 'test',
      });

      const rightHandle = container.querySelector('[data-handlepos="right"]');
      expect(rightHandle).toHaveClass('bg-green-500');
    });

    it('left 핸들(false)은 빨간색이다', () => {
      const { container } = renderNode({
        label: 'Decision',
        type: 'decision',
        rule: 'test',
      });

      const leftHandle = container.querySelector('[data-handlepos="left"]');
      expect(leftHandle).toHaveClass('bg-red-500');
    });
  });

  describe('6. 입력/출력 핸들 위치', () => {
    it('INPUT 노드는 출력 핸들만 가진다 (하단)', () => {
      const { container } = renderNode({
        label: 'Input',
        type: 'input',
      });

      const topHandles = container.querySelectorAll('[data-handlepos="top"]');
      const bottomHandles = container.querySelectorAll('[data-handlepos="bottom"]');

      expect(topHandles.length).toBe(0);
      expect(bottomHandles.length).toBeGreaterThan(0);
    });

    it('OUTPUT 노드는 입력 핸들만 가진다 (상단)', () => {
      const { container } = renderNode({
        label: 'Output',
        type: 'output',
      });

      const topHandles = container.querySelectorAll('[data-handlepos="top"]');
      const bottomHandles = container.querySelectorAll('[data-handlepos="bottom"]');

      expect(topHandles.length).toBeGreaterThan(0);
      expect(bottomHandles.length).toBe(0);
    });

    it('ACTION 노드는 입력/출력 핸들을 모두 가진다', () => {
      const { container } = renderNode({
        label: 'Action',
        type: 'action',
      });

      const topHandles = container.querySelectorAll('[data-handlepos="top"]');
      const bottomHandles = container.querySelectorAll('[data-handlepos="bottom"]');

      expect(topHandles.length).toBeGreaterThan(0);
      expect(bottomHandles.length).toBeGreaterThan(0);
    });
  });

  describe('7. 선택 상태 표시', () => {
    it('선택되지 않은 노드는 체크마크가 없다', () => {
      const { container } = renderNode(
        {
          label: 'Test',
          type: 'input',
        },
        false
      );

      // 체크마크는 .bg-primary.rounded-full 클래스 조합으로 식별
      const checkmark = container.querySelector('.rounded-full.bg-primary');
      expect(checkmark).not.toBeInTheDocument();
    });

    it('선택된 노드는 체크마크를 표시한다', () => {
      const { container } = renderNode(
        {
          label: 'Test',
          type: 'input',
        },
        true
      );

      // 체크마크는 .bg-primary.rounded-full 클래스 조합으로 식별
      const checkmark = container.querySelector('.rounded-full.bg-primary');
      expect(checkmark).toBeInTheDocument();
    });
  });

  describe('8. 에러 표시', () => {
    it('에러가 있으면 에러 메시지를 표시한다', () => {
      renderNode({
        label: 'Test',
        type: 'input',
        error: 'Invalid input data',
      });

      expect(screen.getByText('Invalid input data')).toBeInTheDocument();
    });

    it('에러가 있으면 빨간색 테두리를 표시한다', () => {
      const { container } = renderNode({
        label: 'Test',
        type: 'input',
        error: 'Error',
      });

      const nodeElement = container.querySelector('.border-red-500');
      expect(nodeElement).toBeInTheDocument();
    });
  });

  describe('9. 하이라이트 상태', () => {
    it('highlighted가 true이면 노란색 링을 표시한다', () => {
      const { container } = renderNode({
        label: 'Test',
        type: 'input',
        highlighted: true,
      });

      const nodeElement = container.querySelector('.ring-yellow-400');
      expect(nodeElement).toBeInTheDocument();
    });

    it('highlighted가 false이면 링이 없다', () => {
      const { container } = renderNode({
        label: 'Test',
        type: 'input',
        highlighted: false,
      });

      const nodeElement = container.querySelector('.ring-yellow-400');
      expect(nodeElement).not.toBeInTheDocument();
    });
  });

  describe('10. 알 수 없는 타입 처리', () => {
    it('알 수 없는 타입은 INPUT으로 기본값 처리한다', () => {
      const { container } = renderNode({
        label: 'Unknown',
        type: 'unknown_type', // 알 수 없는 타입
      });

      // INPUT 노드 스타일 (파란색) 확인
      const nodeElement = container.querySelector('.bg-blue-50');
      expect(nodeElement).toBeInTheDocument();
    });
  });
});
