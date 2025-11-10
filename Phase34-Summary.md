# Phase 34: ReactFlow Rendering Detection Attempt (Failed)

## 📊 최종 결과
- **E2E 테스트**: 5/8 통과 (62.5%)
- **퇴보**: Phase 33 (6/8) → Phase 34 (5/8) ❌
- **Test 6**: ✅ → ❌ 새로운 실패 (strict mode violation)
- **Tests 2,4**: ❌ 여전히 실패

## 🎯 목표 및 달성률
| 목표 | 달성 상태 |
|------|------------|
| ReactFlow 렌더링 타이밍 문제 해결 | ❌ 실패 |
| 8/8 E2E 테스트 통과 | ❌ 실패 (5/8, 62.5%) |
| Test 6 유지 | ❌ 새로운 버그 발생 |

## 🔧 Phase 34 시도한 구현

### 1. WorkflowBuilder.tsx - 렌더링 감지 로직 추가

```typescript
// 추가된 상태
const [isReactFlowReady, setIsReactFlowReady] = useState(false);

// useEffect로 nodes 변경 추적
useEffect(() => {
  // Reset ready flag when nodes change
  setIsReactFlowReady(false);
  document.body.removeAttribute('data-reactflow-ready');

  // Wait for ReactFlow to render the new nodes
  const timer = setTimeout(() => {
    setIsReactFlowReady(true);
    document.body.setAttribute('data-reactflow-ready', 'true');
    console.log('[Phase 34] ReactFlow rendering complete:', { nodeCount: nodes.length });
  }, 100);

  return () => clearTimeout(timer);
}, [nodes]);

// Enhanced ReactFlow init handler
const handleReactFlowInit = useCallback((instance: ReactFlowInstance) => {
  setReactFlowInstance(instance);

  // Double requestAnimationFrame ensures DOM is fully painted
  requestAnimationFrame(() => {
    requestAnimationFrame(() => {
      setIsReactFlowReady(true);
      document.body.setAttribute('data-reactflow-ready', 'true');
      console.log('[Phase 34] ReactFlow initialized and ready');
    });
  });
}, []);

// ReactFlow component
<ReactFlow
  onInit={handleReactFlowInit}  // Changed from setReactFlowInstance
  nodes={nodes}
  edges={edges}
  // ... other props
>
```

### 2. E2E Tests - waitForFunction 사용

```typescript
// Before (Phase 33):
await page.waitForTimeout(3000);

// After (Phase 34):
await page.waitForFunction(() => {
  return document.body.getAttribute('data-reactflow-ready') === 'true';
}, { timeout: 10000 });
```

## 📈 테스트 결과 분석

### ✅ 여전히 통과한 테스트 (5/8)
1. **Test 1**: Pattern 모드 - 간단한 워크플로우 생성
3. **Test 3**: Hybrid 모드 - Pattern 성공 케이스
5. **Test 5**: API 키 없이 Pattern 모드 정상 작동
7. **Test 7**: 샘플 시나리오 버튼 동작
8. **Test 8**: 생성 중 상태 표시

### ❌ 실패한 테스트 (3/8)

#### Test 2: LLM 모드 - 복잡한 워크플로우 생성
```
Error: expect(received).toBeGreaterThanOrEqual(expected)

Expected: >= 4
Received:    2
```
- **문제**: Mock이 5개 노드 생성했지만 ReactFlow가 2개만 렌더링
- **원인**: `data-reactflow-ready` 플래그는 `nodes` 상태 변경에 반응하지만, ReactFlow 내부 렌더링 큐는 별개
- **근본 원인**: ReactFlow의 비동기 렌더링 파이프라인이 DOM 속성 설정보다 늦음

#### Test 4: Hybrid 모드 - LLM 보완 케이스
```
Error: strict mode violation: locator('text=워크플로우 생성 완료') resolved to 2 elements
```
- **문제**: 토스트 메시지가 2개 렌더링됨
- **원인**: 이전 토스트가 제거되기 전에 새 토스트가 추가됨
- **새로운 버그**: Phase 33에서는 없었던 문제

#### Test 6: 잘못된 API 키 에러 처리 (NEW FAILURE!)
```
Error: strict mode violation: locator('text=생성 실패') resolved to 2 elements
```
- **문제**: 에러 토스트가 2개 렌더링됨
- **원인**: 동일 - 토스트 중복 렌더링 문제
- **퇴보**: Phase 33에서 통과했던 테스트가 실패

## 🚨 Phase 34의 핵심 문제점

### 1. 잘못된 가정
```
가정: `data-reactflow-ready` 속성이 설정되면 모든 노드가 렌더링 완료
현실: React 상태 변경 ≠ DOM 반영 완료
```

### 2. 타이밍 문제의 본질
```
React State Update (nodes 변경)
  ↓ (즉시)
useEffect 실행
  ↓ (100ms 타이머)
data-reactflow-ready 설정
  ↓ (??ms - 예측 불가능)
ReactFlow 내부 렌더링 큐 처리
  ↓ (??ms - 예측 불가능)
실제 DOM에 노드 반영
```

### 3. 새로운 버그 발생
- Phase 33에서 통과했던 Test 6이 Phase 34에서 실패
- 토스트 중복 렌더링 문제 발생
- 전체 통과율 하락: 75% → 62.5%

## 💡 교훈 및 다음 단계

### 실패 원인 분석
1. ❌ **DOM 속성 기반 감지**: ReactFlow 내부 상태와 DOM이 동기화되지 않음
2. ❌ **고정 타이머 (100ms)**: 환경에 따라 부족하거나 과도함
3. ❌ **useEffect 의존성**: `nodes` 변경은 ReactFlow 렌더링 완료를 보장하지 않음

### 근본 원인
```
ReactFlow는 자체 렌더링 큐를 가지고 있으며:
- React state 변경과 독립적으로 동작
- 비동기 렌더링 파이프라인 사용
- DOM 반영 시점을 외부에서 예측 불가능
```

### 제안: Phase 35 전략 변경

**Option 1: ReactFlow onNodesChange 이벤트 활용**
```typescript
const handleNodesChange = useCallback((changes: NodeChange[]) => {
  onNodesChange(changes);

  // Wait for next tick to ensure DOM is updated
  setTimeout(() => {
    document.body.setAttribute('data-reactflow-ready', 'true');
  }, 0);
}, [onNodesChange]);

<ReactFlow
  onNodesChange={handleNodesChange}
  // ...
>
```

**Option 2: E2E 테스트에서 노드 개수 직접 폴링**
```typescript
// Phase 34 (실패):
await page.waitForFunction(() => {
  return document.body.getAttribute('data-reactflow-ready') === 'true';
}, { timeout: 10000 });

// Phase 35 (제안):
await page.waitForFunction((expectedCount) => {
  const nodes = document.querySelectorAll('.react-flow__node');
  return nodes.length >= expectedCount;
}, expectedNodeCount, { timeout: 10000 });
```

**Option 3: MutationObserver로 DOM 변경 감지**
```typescript
useEffect(() => {
  const observer = new MutationObserver(() => {
    const nodeCount = document.querySelectorAll('.react-flow__node').length;
    if (nodeCount === nodes.length) {
      document.body.setAttribute('data-reactflow-ready', 'true');
    }
  });

  observer.observe(document.querySelector('.react-flow'), {
    childList: true,
    subtree: true
  });

  return () => observer.disconnect();
}, [nodes]);
```

**Option 4: 토스트 중복 문제 먼저 해결**
```typescript
// Sonner 토스트 설정에서 중복 방지
<Toaster
  richColors
  position="top-right"
  expand={false}
  limit={1}  // 한 번에 1개만 표시
  duration={5000}
/>
```

### 우선순위
1. **즉시**: 토스트 중복 문제 해결 (Test 4, 6 회복)
2. **다음**: Option 2 (노드 개수 직접 폴링) 시도
3. **최후**: Tauri 환경에서 실제 Claude API 테스트

## 📊 성과 지표
- **코드 변경**: ~200줄
- **작업 시간**: ~2.5시간
- **테스트 퇴보**: 75% → 62.5% (❌ -12.5%p)
- **새로운 버그**: Test 6 실패 (토스트 중복)
- **학습**: ReactFlow 렌더링 파이프라인 이해 증가

## 🔗 관련 파일
- [src/pages/WorkflowBuilder.tsx](src/pages/WorkflowBuilder.tsx) - 렌더링 감지 로직 (실패)
- [tests/e2e/workflow-generation.spec.ts](tests/e2e/workflow-generation.spec.ts) - waitForFunction (부분 성공)
- [Phase33-Summary.md](Phase33-Summary.md) - 이전 Phase (6/8 통과)
- [phase34-test.log](phase34-test.log) - 최종 테스트 로그

---
*작성일: 2025-11-10*
*Phase 34 완료 (실패, 퇴보 발생)*
*다음 목표: 토스트 중복 해결 후 노드 폴링 전략 시도*
