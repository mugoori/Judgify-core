# Judgify-core 테스트 가이드

**작성일**: 2025-11-06
**버전**: 1.0
**대상**: 개발팀, 신규 기여자
**프로젝트**: Judgify-core Desktop App (Tauri + React + Rust)

---

## 📑 목차

1. [테스트 철학 (Why Test?)](#1-테스트-철학-why-test)
2. [프로젝트 테스트 구조](#2-프로젝트-테스트-구조)
3. [TypeScript 유닛 테스트 패턴](#3-typescript-유닛-테스트-패턴)
4. [Rust 통합 테스트 패턴](#4-rust-통합-테스트-패턴)
5. [E2E 테스트 패턴](#5-e2e-테스트-패턴)
6. [CI/CD 통합](#6-cicd-통합)
7. [커버리지 목표 및 측정 방법](#7-커버리지-목표-및-측정-방법)

---

## 1. 테스트 철학 (Why Test?)

### 🎯 핵심 원칙

**"테스트는 코드의 신뢰를 보장하고, 리팩토링의 안전망을 제공한다"**

### 프로젝트 테스트 목표

1. **신뢰성 확보**: 사용자가 믿고 쓸 수 있는 Desktop App
2. **빠른 피드백**: 버그를 커밋 전에 발견
3. **리팩토링 안전성**: 코드 변경 시 회귀(regression) 방지
4. **문서화 효과**: 테스트 코드가 곧 사용 예시

### 테스트 계층 구조 (Testing Pyramid)

```
      /\
     /  \     E2E Tests (5개 시나리오)
    /    \    - 전체 워크플로우 검증
   /------\
  /        \  Integration Tests (37개 Rust 테스트)
 /          \ - 서비스 간 통신 검증
/------------\
              Unit Tests (48개 TypeScript 테스트)
              - 개별 함수/컴포넌트 검증
```

**피라미드 원칙**:
- **기반**: 유닛 테스트가 가장 많음 (빠르고 저렴)
- **중간**: 통합 테스트 (의존성 포함)
- **상단**: E2E 테스트 (느리지만 실제 사용자 시나리오)

---

## 2. 프로젝트 테스트 구조

### 📂 디렉토리 구조

```
Judgify-core/
├── src/                          # Frontend (TypeScript + React)
│   ├── hooks/__tests__/          # React Hooks 테스트
│   │   └── useRuleValidation.test.ts (8 tests)
│   ├── lib/__tests__/            # Utils 테스트
│   │   ├── tauri-api.test.ts (21 tests)
│   │   └── sample-data.test.ts (9 tests)
│   ├── components/__tests__/     # Component 테스트
│   │   └── EmptyState.test.tsx (10 tests)
│   └── pages/                    # (E2E 테스트로 커버)
│
├── src-tauri/                    # Backend (Rust)
│   ├── src/
│   │   ├── cache_service.rs      # CacheService 구현
│   │   ├── memory_manager.rs     # MemoryManager 구현
│   │   └── judgment.rs           # Judgment 로직
│   └── tests/                    # Rust 통합 테스트
│       ├── cache_service_test.rs (37 tests)
│       └── memory_manager_test.rs (예정)
│
├── tests-e2e/                    # E2E 테스트
│   └── workflow.spec.ts (5 scenarios)
│
├── vitest.config.ts              # Vitest 설정
├── playwright.config.ts          # Playwright 설정
└── Cargo.toml                    # Rust 의존성 (dev-dependencies)
```

### 🛠️ 테스트 프레임워크

| 계층 | 프레임워크 | 용도 | 실행 명령어 |
|------|-----------|------|------------|
| **TypeScript 유닛** | [Vitest](https://vitest.dev/) | React Hooks, Utils, Components | `npm run test` |
| **Rust 통합** | [Criterion.rs](https://github.com/bheisler/criterion.rs) | Benchmark + 통합 테스트 | `cargo test` |
| **E2E** | [Playwright](https://playwright.dev/) | 전체 워크플로우 | `npm run test:e2e` |
| **성능** | Criterion.rs | 벤치마킹 (평균 0.001ms) | `cargo bench` |

### 📊 현재 커버리지 현황 (2025-11-06)

| 언어 | 커버리지 | 파일 수 | 테스트 수 | 목표 |
|------|---------|--------|---------|------|
| **TypeScript** | 17.02% | 4 files | 48 tests | 70% |
| **Rust** | 48% | 2 files | 37 tests | 80% |
| **E2E** | 100% (5 scenarios) | - | 5 tests | 100% |

---

## 3. TypeScript 유닛 테스트 패턴

### 3.1 공통 설정 및 모범 사례

#### 필수 Import 구조

```typescript
import { describe, it, test, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { invoke } from '@tauri-apps/api/tauri';

// 테스트 대상
import { 테스트대상 } from '../파일경로';
```

#### Tauri API Mocking 표준 패턴

**모든 Tauri 통신 테스트는 다음 패턴을 따릅니다**:

```typescript
// 1. Tauri invoke 모킹
vi.mock('@tauri-apps/api/tauri', () => ({
  invoke: vi.fn(),
}));

describe('테스트 그룹', () => {
  beforeEach(() => {
    vi.clearAllMocks(); // 각 테스트 전에 모든 모킹 초기화
  });

  afterEach(() => {
    vi.clearAllTimers(); // 타이머 정리 (debounce 등)
  });

  test('테스트 케이스', async () => {
    // 모킹 설정
    vi.mocked(invoke).mockResolvedValue({ /* 응답 데이터 */ });

    // 테스트 로직
    // ...

    // 검증
    expect(invoke).toHaveBeenCalledWith('tauri_command', { args });
  });
});
```

---

### 3.2 React Hooks 테스트 패턴

**파일**: [src/hooks/__tests__/useRuleValidation.test.ts](../../src/hooks/__tests__/useRuleValidation.test.ts)

#### 핵심 패턴: `renderHook` + `waitFor`

```typescript
import { renderHook, waitFor } from '@testing-library/react';
import { useRuleValidation } from '../useRuleValidation';

test('should validate a simple rule expression', async () => {
  // 1. Tauri 응답 모킹
  vi.mocked(invoke).mockResolvedValue({
    isValid: true,
    errors: [],
  });

  // 2. Hook 렌더링
  const { result } = renderHook(() =>
    useRuleValidation('temperature > 80', { debounceMs: 0 })
  );

  // 3. 비동기 상태 대기
  await waitFor(() => {
    expect(result.current.isValidating).toBe(false);
  });

  // 4. 최종 상태 검증
  expect(result.current.isValid).toBe(true);
  expect(result.current.errors).toEqual([]);
  expect(invoke).toHaveBeenCalledWith('validate_rule_expression', {
    rule: 'temperature > 80',
  });
});
```

#### Debounce 테스트 패턴

```typescript
test('should debounce validation calls', async () => {
  vi.mocked(invoke).mockResolvedValue({
    isValid: true,
    errors: [],
  });

  // 동적 props를 위한 rerender
  const { result, rerender } = renderHook(
    ({ rule }) => useRuleValidation(rule, { debounceMs: 100 }),
    {
      initialProps: { rule: '' },
    }
  );

  // 빠른 연속 변경
  rerender({ rule: 'temperature > 80' });
  rerender({ rule: 'temperature > 85' });
  rerender({ rule: 'temperature > 90' });

  // Debounce 대기
  await new Promise((resolve) => setTimeout(resolve, 150));

  await waitFor(() => {
    expect(result.current.isValidating).toBe(false);
  });

  // 마지막 호출만 실행되어야 함
  expect(invoke).toHaveBeenCalledTimes(1);
  expect(invoke).toHaveBeenCalledWith('validate_rule_expression', {
    rule: 'temperature > 90',
  });
});
```

#### 에러 핸들링 테스트 패턴

```typescript
test('should handle validation errors gracefully', async () => {
  // console.error 모킹 (테스트 로그 정리)
  const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

  // Tauri 에러 시뮬레이션
  vi.mocked(invoke).mockRejectedValue(new Error('Network error'));

  const { result } = renderHook(() =>
    useRuleValidation('temperature > 80', { debounceMs: 50 })
  );

  await waitFor(
    () => {
      expect(result.current.isValidating).toBe(false);
      expect(result.current.isValid).toBe(false);
    },
    { timeout: 500 }
  );

  expect(result.current.errors).toContain('Network error');

  // 모킹 복원
  consoleErrorSpy.mockRestore();
});
```

---

### 3.3 Utils 테스트 패턴

**파일**: [src/lib/__tests__/tauri-api.test.ts](../../src/lib/__tests__/tauri-api.test.ts) (21 tests)

#### API 함수 테스트 표준 패턴

```typescript
describe('Judgment API', () => {
  it('executeJudgment - 판단 실행 성공', async () => {
    // 1. 요청 데이터 준비
    const mockRequest: ExecuteJudgmentRequest = {
      workflow_id: 'workflow-123',
      input_data: { temperature: 90 },
      method: 'hybrid',
    };

    // 2. 응답 데이터 준비
    const mockResult: JudgmentResult = {
      id: 'judgment-456',
      workflow_id: 'workflow-123',
      result: true,
      confidence: 0.95,
      method_used: 'rule',
      explanation: 'Temperature exceeds threshold',
      created_at: '2025-11-06T10:00:00Z',
    };

    // 3. Tauri 응답 모킹
    vi.mocked(invoke).mockResolvedValue(mockResult);

    // 4. API 함수 호출
    const result = await executeJudgment(mockRequest);

    // 5. Tauri 호출 검증
    expect(invoke).toHaveBeenCalledWith('execute_judgment', { request: mockRequest });

    // 6. 반환값 검증
    expect(result).toEqual(mockResult);
    expect(result.confidence).toBeGreaterThanOrEqual(0.9);
  });
});
```

#### 배열 응답 테스트 패턴

```typescript
it('getJudgmentHistory - 히스토리 조회 성공', async () => {
  const mockHistory: JudgmentResult[] = [
    {
      id: 'judgment-1',
      workflow_id: 'workflow-123',
      result: true,
      confidence: 0.92,
      method_used: 'hybrid',
      explanation: 'Test 1',
      created_at: '2025-11-06T09:00:00Z',
    },
    {
      id: 'judgment-2',
      workflow_id: 'workflow-123',
      result: false,
      confidence: 0.88,
      method_used: 'rule',
      explanation: 'Test 2',
      created_at: '2025-11-06T10:00:00Z',
    },
  ];

  vi.mocked(invoke).mockResolvedValue(mockHistory);

  const result = await getJudgmentHistory('workflow-123', 10);

  expect(invoke).toHaveBeenCalledWith('get_judgment_history', {
    workflowId: 'workflow-123',
    limit: 10,
  });
  expect(result).toHaveLength(2);
  expect(result[0].workflow_id).toBe('workflow-123');
});
```

#### 에러 시나리오 테스트

```typescript
describe('Error Handling', () => {
  it('네트워크 타임아웃 처리', async () => {
    vi.mocked(invoke).mockRejectedValue(new Error('Timeout'));

    await expect(getSystemStatus()).rejects.toThrow('Timeout');
  });

  it('잘못된 응답 형식 처리', async () => {
    vi.mocked(invoke).mockResolvedValue(null);

    const result = await getSystemStatus();

    expect(result).toBeNull();
  });
});
```

---

### 3.4 데이터 생성 함수 테스트 패턴

**파일**: [src/lib/__tests__/sample-data.test.ts](../../src/lib/__tests__/sample-data.test.ts) (9 tests)

#### 복잡한 비동기 흐름 테스트

```typescript
it('샘플 워크플로우 3개 생성 성공', async () => {
  // 1. 여러 번의 연속 호출 모킹
  vi.mocked(invoke)
    .mockResolvedValueOnce({ id: 'workflow-1' })  // 첫 번째 호출
    .mockResolvedValueOnce({ id: 'workflow-2' })  // 두 번째 호출
    .mockResolvedValueOnce({ id: 'workflow-3' }); // 세 번째 호출

  // 2. 후속 호출 모킹 (판단 실행 37개)
  for (let i = 0; i < 37; i++) {
    vi.mocked(invoke).mockResolvedValueOnce({ id: `judgment-${i}` });
  }

  // 3. 함수 실행
  const result = await generateSampleData();

  // 4. 결과 검증
  expect(result.workflows).toBe(3);
  expect(result.judgments).toBeGreaterThan(0);
  expect(result.judgments).toBeLessThanOrEqual(37);
});
```

#### 부분 실패 시나리오 테스트

```typescript
it('워크플로우 생성 실패시 계속 진행', async () => {
  // 첫 번째 호출 실패
  vi.mocked(invoke).mockRejectedValueOnce(new Error('Create failed'));

  // 나머지 호출 성공
  vi.mocked(invoke)
    .mockResolvedValueOnce({ id: 'workflow-2' })
    .mockResolvedValueOnce({ id: 'workflow-3' });

  for (let i = 0; i < 22; i++) {
    vi.mocked(invoke).mockResolvedValueOnce({ id: `judgment-${i}` });
  }

  const result = await generateSampleData();

  // 2개만 성공 (1개 실패)
  expect(result.workflows).toBe(2);
});
```

#### 데이터 구조 검증 패턴

```typescript
it('생성된 워크플로우 구조 확인', async () => {
  vi.mocked(invoke)
    .mockResolvedValueOnce({ id: 'workflow-1' })
    .mockResolvedValueOnce({ id: 'workflow-2' })
    .mockResolvedValueOnce({ id: 'workflow-3' });

  for (let i = 0; i < 37; i++) {
    vi.mocked(invoke).mockResolvedValueOnce({ id: `judgment-${i}` });
  }

  await generateSampleData();

  // expect.objectContaining으로 부분 검증
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
```

---

### 3.5 React Component 테스트 패턴

**파일**: [src/components/__tests__/EmptyState.test.tsx](../../src/components/__tests__/EmptyState.test.tsx) (10 tests)

#### 기본 렌더링 테스트

```typescript
import { render, screen } from '@testing-library/react';
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

    // 텍스트 검증
    expect(screen.getByText('비어있음')).toBeInTheDocument();
    expect(screen.getByText('데이터가 없습니다')).toBeInTheDocument();

    // SVG 아이콘 검증
    const svgElements = document.querySelectorAll('svg');
    expect(svgElements.length).toBeGreaterThan(0);
  });
});
```

#### 사용자 인터랙션 테스트 (User Event)

```typescript
import userEvent from '@testing-library/user-event';

it('액션 버튼 클릭시 핸들러 호출', async () => {
  // 1. userEvent 초기화
  const user = userEvent.setup();
  const mockAction = vi.fn();

  // 2. 컴포넌트 렌더링
  render(
    <EmptyState
      icon={Inbox}
      title="비어있음"
      description="데이터가 없습니다"
      actionLabel="새로 만들기"
      onAction={mockAction}
    />
  );

  // 3. 버튼 찾기
  const button = screen.getByRole('button', { name: '새로 만들기' });
  expect(button).toBeInTheDocument();

  // 4. 클릭 이벤트
  await user.click(button);

  // 5. 핸들러 호출 검증
  expect(mockAction).toHaveBeenCalledTimes(1);
});
```

#### 조건부 렌더링 테스트

```typescript
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

  // queryByRole: 요소가 없으면 null 반환
  const button = screen.queryByRole('button', { name: '새로 만들기' });
  expect(button).not.toBeInTheDocument();
});
```

#### 리렌더링 테스트

```typescript
it('다양한 아이콘 타입 렌더링 가능', () => {
  const { rerender } = render(
    <EmptyState
      icon={Inbox}
      title="제목"
      description="설명"
    />
  );

  // 첫 번째 렌더링 검증
  expect(screen.getByText('제목')).toBeInTheDocument();

  // props 변경 후 리렌더링
  rerender(
    <EmptyState
      icon={Inbox}
      title="새 제목"
      description="새 설명"
    />
  );

  // 변경된 내용 검증
  expect(screen.getByText('새 제목')).toBeInTheDocument();
  expect(screen.getByText('새 설명')).toBeInTheDocument();
});
```

#### 스타일 검증 테스트

```typescript
it('Card 컴포넌트 스타일 적용 확인', () => {
  const { container } = render(
    <EmptyState
      icon={Inbox}
      title="비어있음"
      description="데이터가 없습니다"
    />
  );

  // querySelector로 CSS 클래스 검증
  const cardElement = container.querySelector('.border-dashed');
  expect(cardElement).toBeInTheDocument();
});
```

---

## 4. Rust 통합 테스트 패턴

### 4.1 Rust 테스트 구조

**파일**: [src-tauri/tests/cache_service_test.rs](../../src-tauri/tests/cache_service_test.rs) (37 tests)

#### 기본 테스트 구조

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn test_cache_set_and_get() {
        // 1. 테스트 환경 초기화
        let cache = Arc::new(Mutex::new(CacheService::new(100)));

        // 2. 테스트 실행
        let key = "test_key";
        let value = "test_value";
        cache.lock().await.set(key, value).await;

        // 3. 검증
        let result = cache.lock().await.get(key).await;
        assert_eq!(result, Some(value.to_string()));
    }
}
```

#### 비동기 테스트 패턴

```rust
#[tokio::test]
async fn test_async_operation() {
    let service = create_test_service().await;

    // 비동기 작업 실행
    let result = service.execute_async_task().await;

    // 결과 검증
    assert!(result.is_ok());
}
```

#### 에러 처리 테스트

```rust
#[tokio::test]
async fn test_error_handling() {
    let service = create_test_service().await;

    // 실패 예상 작업
    let result = service.fail_operation().await;

    // 에러 타입 검증
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "Expected error message"
    );
}
```

### 4.2 Criterion.rs 벤치마킹 패턴

**파일**: [src-tauri/benches/cache_bench.rs](../../src-tauri/benches/cache_bench.rs)

#### 기본 벤치마크 구조

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn benchmark_cache_get(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_operations");

    // 벤치마크 실행
    group.bench_function("get", |b| {
        b.iter(|| {
            let cache = CacheService::new(100);
            cache.get(black_box("key"))
        });
    });

    group.finish();
}

criterion_group!(benches, benchmark_cache_get);
criterion_main!(benches);
```

#### 실측 성능 데이터 (2025-11-06)

```
CacheService Benchmarks:
├── cache_get_hit      : 0.001 ms (평균)
├── cache_get_miss     : 0.0008 ms
├── cache_set          : 0.0012 ms
└── memory_update      : 0.0015 ms

목표:
- GET 작업: < 1ms ✅ 달성
- SET 작업: < 2ms ✅ 달성
- 캐시 적중률: > 85% ✅ 90% 달성
```

---

## 5. E2E 테스트 패턴

### 5.1 Playwright 테스트 구조

**파일**: [tests-e2e/workflow.spec.ts](../../tests-e2e/workflow.spec.ts) (5 scenarios)

#### 기본 E2E 테스트 패턴

```typescript
import { test, expect } from '@playwright/test';

test.describe('Workflow Management', () => {
  test('사용자가 새 워크플로우를 생성할 수 있다', async ({ page }) => {
    // 1. 페이지 이동
    await page.goto('http://localhost:1420');

    // 2. 워크플로우 페이지로 이동
    await page.click('text=Workflow Builder');

    // 3. 새 워크플로우 생성 버튼 클릭
    await page.click('button:has-text("New Workflow")');

    // 4. 폼 입력
    await page.fill('input[name="workflow-name"]', 'Test Workflow');
    await page.fill('textarea[name="rule-expression"]', 'temperature > 80');

    // 5. 저장
    await page.click('button:has-text("Save")');

    // 6. 검증
    await expect(page.locator('text=Test Workflow')).toBeVisible();
  });
});
```

#### 실제 5개 E2E 시나리오

1. **워크플로우 생성**: 사용자가 새 워크플로우를 생성
2. **판단 실행**: 워크플로우를 통한 판단 실행
3. **피드백 제공**: 판단 결과에 대한 사용자 피드백
4. **대시보드 조회**: BI 대시보드에서 데이터 확인
5. **채팅 인터랙션**: AI 채팅으로 워크플로우 제어

---

## 6. CI/CD 통합

### 6.1 GitHub Actions 워크플로우

**파일**: [.github/workflows/test.yml](../../.github/workflows/test.yml)

#### 테스트 실행 단계

```yaml
name: Test

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]

    steps:
      # 1. 코드 체크아웃
      - uses: actions/checkout@v3

      # 2. Node.js 설정
      - uses: actions/setup-node@v3
        with:
          node-version: '18'

      # 3. Rust 설정
      - uses: dtolnay/rust-toolchain@stable

      # 4. Tauri 시스템 의존성 설치 (Linux only)
      - name: Install Tauri system dependencies (Linux)
        if: runner.os == 'Linux'
        run: |
          sudo apt-get update
          sudo apt-get install -y \
            libwebkit2gtk-4.0-dev \
            build-essential \
            curl \
            wget \
            libssl-dev \
            libgtk-3-dev \
            libayatana-appindicator3-dev \
            librsvg2-dev

      # 5. 의존성 설치
      - run: npm install

      # 6. TypeScript 테스트
      - run: npm run test

      # 7. Rust 테스트
      - run: cargo test --manifest-path src-tauri/Cargo.toml

      # 8. E2E 테스트 (Linux only)
      - name: E2E Tests
        if: runner.os == 'Linux'
        run: npm run test:e2e

      # 9. Lighthouse CI (성능 테스트)
      - name: Lighthouse CI
        if: runner.os == 'Linux'
        run: |
          npm install -g @lhci/cli@0.12.x
          lhci autorun
```

### 6.2 로컬 테스트 명령어

```bash
# TypeScript 유닛 테스트
npm run test                  # 전체 실행
npm run test:watch            # Watch 모드
npm run test:coverage         # 커버리지 리포트

# Rust 통합 테스트
cargo test                    # 전체 실행
cargo test cache_service      # 특정 모듈만
cargo test -- --nocapture     # stdout 출력 보기

# Rust 벤치마크
cargo bench                   # 전체 벤치마크
cargo bench cache_operations  # 특정 그룹만

# E2E 테스트
npm run test:e2e              # Headed 모드
npm run test:e2e:headless     # Headless 모드
npm run test:e2e:debug        # Debug 모드

# Lighthouse 성능 테스트
npm run build                 # 프로덕션 빌드
npm run preview               # 미리보기 서버
lhci autorun                  # Lighthouse CI 실행
```

---

## 7. 커버리지 목표 및 측정 방법

### 7.1 현재 커버리지 현황

#### TypeScript 커버리지 (17.02%)

```
File                               | % Stmts | % Branch | % Funcs | % Lines
-----------------------------------|---------|----------|---------|--------
hooks/useRuleValidation.ts         | 100     | 100      | 100     | 100
lib/tauri-api.ts                   | 100     | 100      | 100     | 100
lib/sample-data.ts                 | 100     | 100      | 100     | 100
components/EmptyState.tsx          | 100     | 100      | 100     | 100
-----------------------------------|---------|----------|---------|--------
전체 (src/)                        | 17.02   | 12.5     | 15.3    | 17.02
```

#### Rust 커버리지 (48%)

```
Module                    | Lines Covered | Total Lines | Coverage
--------------------------|---------------|-------------|----------
cache_service.rs          | 120           | 250         | 48%
memory_manager.rs         | 0             | 180         | 0%
judgment.rs               | 0             | 320         | 0%
--------------------------|---------------|-------------|----------
전체 (src-tauri/src/)     | 120           | 750         | 48%
```

### 7.2 커버리지 목표

| 항목 | 현재 | 단기 목표 (1개월) | 장기 목표 (3개월) |
|------|------|------------------|-------------------|
| **TypeScript** | 17.02% | 40% | 70% |
| **Rust** | 48% | 60% | 80% |
| **E2E** | 100% | 100% | 100% |

### 7.3 우선순위 테스트 대상

#### Phase 1 (다음 주)
1. **Workflow 모듈** (TypeScript)
   - `WorkflowBuilder.tsx`
   - `src/lib/workflow-generator.ts`
2. **Memory Manager** (Rust)
   - `memory_manager.rs` (현재 0%)

#### Phase 2 (2주차)
3. **Page Components** (TypeScript)
   - `Dashboard.tsx`
   - `BiInsights.tsx`
   - `Settings.tsx`
4. **Judgment Service** (Rust)
   - `judgment.rs` (현재 0%)

#### Phase 3 (3-4주차)
5. **나머지 Component** (TypeScript)
   - `Sidebar.tsx`, `Header.tsx`
   - UI 컴포넌트들

### 7.4 커버리지 측정 명령어

```bash
# TypeScript 커버리지
npm run test:coverage

# 결과:
# - 터미널 요약 출력
# - coverage/ 디렉토리에 HTML 리포트 생성
# - coverage/lcov-report/index.html 브라우저로 열기

# Rust 커버리지 (Tarpaulin)
cargo install cargo-tarpaulin
cargo tarpaulin --manifest-path src-tauri/Cargo.toml --out Html

# 결과:
# - 터미널 요약 출력
# - tarpaulin-report.html 생성
```

### 7.5 커버리지 개선 전략

#### 1. 테스트 우선순위 결정
- **높은 우선순위**: 핵심 비즈니스 로직 (Judgment, Workflow)
- **중간 우선순위**: UI 컴포넌트 (Page, Component)
- **낮은 우선순위**: 단순 유틸리티 함수

#### 2. 테스트 작성 가이드라인
- **신규 기능**: 테스트 커버리지 90% 이상 필수
- **버그 수정**: 회귀 테스트 필수 작성
- **리팩토링**: 기존 테스트 유지 및 개선

#### 3. CI/CD 통합
- **PR 체크**: 커버리지 감소시 경고
- **주간 리포트**: 커버리지 트렌드 모니터링

---

## 📚 참고 자료

### 공식 문서
- [Vitest 공식 문서](https://vitest.dev/)
- [Playwright 공식 문서](https://playwright.dev/)
- [Criterion.rs 공식 문서](https://github.com/bheisler/criterion.rs)
- [Testing Library 공식 문서](https://testing-library.com/)

### 프로젝트 문서
- [TASKS.md](../../TASKS.md) - 전체 작업 진행 현황
- [CLAUDE.md](../../CLAUDE.md) - 프로젝트 개발 가이드
- [docs/development/plan.md](../development/plan.md) - 개발 계획

### 관련 파일
- [vitest.config.ts](../../vitest.config.ts)
- [playwright.config.ts](../../playwright.config.ts)
- [.github/workflows/test.yml](../../.github/workflows/test.yml)

---

**작성자**: Claude (AI Assistant)
**최종 수정일**: 2025-11-06
**문서 버전**: 1.0

**다음 업데이트 예정**:
- Rust Memory Manager 테스트 패턴 추가 (Task 4.2-Full 완료 후)
- TypeScript Workflow 모듈 테스트 패턴 추가 (Task 4.2-Full 완료 후)
- 커버리지 40% 달성 후 실측 데이터 업데이트
