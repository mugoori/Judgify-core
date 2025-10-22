# E2E 테스트 자동화 (Ver2.0 Final)

**목적**: Playwright를 활용한 End-to-End 테스트 자동화

**관련 MCP 도구**: `playwright-mcp-server`

---

## 🎯 Playwright E2E 테스트 패턴

### 대시보드 자동 생성 E2E 테스트

```python
# Claude가 구현하는 Playwright E2E 테스트

async def test_dashboard_auto_generation_e2e():
    """대시보드 자동 생성 E2E 테스트"""

    # 1. 사용자 요청 시뮬레이션
    page = await browser.new_page()
    await page.goto("http://localhost:3000/dashboard")

    # 2. 자연어 요청 입력
    await page.fill('[data-testid="dashboard-request"]',
                   "지난 주 워크플로우별 성공률을 보여줘")
    await page.click('[data-testid="generate-button"]')

    # 3. 대시보드 생성 확인
    await page.wait_for_selector('[data-testid="generated-dashboard"]')

    # 4. 차트 컴포넌트 로딩 확인
    chart = await page.query_selector('[data-testid="bar-chart"]')
    assert chart is not None

    # 5. 실시간 데이터 업데이트 확인
    await page.wait_for_function("() => document.querySelectorAll('.chart-data').length > 0")
```

---

## 🧪 서비스별 E2E 테스트 시나리오

### 1. Judgment Service E2E 테스트

```python
import pytest
from playwright.async_api import async_playwright

@pytest.mark.asyncio
async def test_hybrid_judgment_workflow():
    """하이브리드 판단 워크플로우 E2E 테스트"""

    async with async_playwright() as p:
        browser = await p.chromium.launch()
        page = await browser.new_page()

        # 1. Workflow Builder 접속
        await page.goto("http://localhost:3000/workflows")

        # 2. 새 워크플로우 생성
        await page.click('[data-testid="create-workflow"]')
        await page.fill('[data-testid="workflow-name"]', "품질 검사 워크플로우")

        # 3. Judgment 노드 추가
        await page.click('[data-testid="add-judgment-node"]')
        await page.fill('[data-testid="rule-expression"]', "temperature > 80 AND vibration > 50")

        # 4. 워크플로우 저장
        await page.click('[data-testid="save-workflow"]')
        await page.wait_for_selector('[data-testid="save-success"]')

        # 5. 테스트 실행
        await page.click('[data-testid="test-workflow"]')
        await page.fill('[data-testid="test-input"]', '{"temperature": 90, "vibration": 60}')
        await page.click('[data-testid="execute-test"]')

        # 6. 결과 검증
        result = await page.text_content('[data-testid="test-result"]')
        assert "판단 결과: true" in result
        assert "method_used: rule" in result
        assert "confidence:" in result

        await browser.close()
```

### 2. Chat Interface E2E 테스트

```python
@pytest.mark.asyncio
async def test_chat_interface_multitern_conversation():
    """Chat Interface 멀티턴 대화 E2E 테스트"""

    async with async_playwright() as p:
        browser = await p.chromium.launch()
        page = await browser.new_page()

        # 1. Chat Interface 접속
        await page.goto("http://localhost:3000/chat")

        # 2. 첫 번째 요청: 워크플로우 실행
        await page.fill('[data-testid="chat-input"]', "품질 검사 워크플로우 실행해줘")
        await page.click('[data-testid="send-button"]')
        await page.wait_for_selector('[data-testid="chat-response"]')

        response1 = await page.text_content('[data-testid="chat-response"]:last-child')
        assert "워크플로우를 실행하겠습니다" in response1

        # 3. 두 번째 요청: 결과 시각화
        await page.fill('[data-testid="chat-input"]', "결과를 차트로 보여줘")
        await page.click('[data-testid="send-button"]')
        await page.wait_for_selector('[data-testid="generated-chart"]')

        chart = await page.query_selector('[data-testid="generated-chart"]')
        assert chart is not None

        # 4. 컨텍스트 유지 확인 (이전 대화 참조)
        await page.fill('[data-testid="chat-input"]', "이 워크플로우의 성공률은?")
        await page.click('[data-testid="send-button"]')
        await page.wait_for_selector('[data-testid="chat-response"]')

        response3 = await page.text_content('[data-testid="chat-response"]:last-child')
        assert "품질 검사 워크플로우" in response3  # 컨텍스트 유지 확인
        assert "%" in response3  # 성공률 수치 포함

        await browser.close()
```

### 3. Learning Service E2E 테스트 (자동학습)

```python
@pytest.mark.asyncio
async def test_learning_service_feedback_flow():
    """Learning Service 피드백 수집 및 Few-shot 학습 E2E 테스트"""

    async with async_playwright() as p:
        browser = await p.chromium.launch()
        page = await browser.new_page()

        # 1. 판단 실행 페이지 접속
        await page.goto("http://localhost:3000/judgment")

        # 2. 판단 실행
        await page.fill('[data-testid="workflow-id"]', "quality-check-workflow")
        await page.fill('[data-testid="input-data"]', '{"temperature": 85, "vibration": 55}')
        await page.click('[data-testid="execute-judgment"]')
        await page.wait_for_selector('[data-testid="judgment-result"]')

        # 3. 피드백 제공 (긍정적 피드백)
        await page.click('[data-testid="thumbs-up"]')
        await page.wait_for_selector('[data-testid="feedback-success"]')

        # 4. Learning Service 페이지 이동
        await page.goto("http://localhost:3000/learning")

        # 5. Few-shot 샘플 확인
        await page.click('[data-testid="view-samples"]')
        await page.wait_for_selector('[data-testid="sample-list"]')

        samples = await page.query_selector_all('[data-testid="sample-item"]')
        assert len(samples) > 0  # 샘플이 추가되었는지 확인

        # 6. 자동 Rule 추출 실행
        await page.click('[data-testid="extract-rules"]')
        await page.wait_for_selector('[data-testid="extraction-result"]')

        result = await page.text_content('[data-testid="extraction-result"]')
        assert "알고리즘" in result  # 3개 알고리즘 중 하나 실행 확인
        assert "Rule" in result

        await browser.close()
```

---

## 🔧 Playwright 설정

### playwright.config.ts

```typescript
import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './tests/e2e',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: 'html',

  use: {
    baseURL: 'http://localhost:3000',
    trace: 'on-first-retry',
  },

  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
    {
      name: 'firefox',
      use: { ...devices['Desktop Firefox'] },
    },
    {
      name: 'webkit',
      use: { ...devices['Desktop Safari'] },
    },
  ],

  webServer: {
    command: 'npm run dev',
    url: 'http://localhost:3000',
    reuseExistingServer: !process.env.CI,
  },
});
```

---

## 📊 테스트 커버리지 목표

| 서비스 | E2E 시나리오 | 목표 커버리지 |
|--------|-------------|--------------|
| **Judgment Service** | 하이브리드 판단 워크플로우 | 90% |
| **Learning Service** | 피드백 수집 + Few-shot 학습 | 85% |
| **Chat Interface** | 멀티턴 대화 + 의도 분류 | 85% |
| **BI Service** | 자동 대시보드 생성 | 80% |
| **Workflow Builder** | n8n 스타일 드래그앤드롭 | 75% |

---

## 🚀 실행 방법

```bash
# 전체 E2E 테스트 실행
npx playwright test

# 특정 브라우저만 실행
npx playwright test --project=chromium

# UI 모드로 실행 (디버깅)
npx playwright test --ui

# 헤드리스 모드 비활성화 (브라우저 보기)
npx playwright test --headed

# 특정 테스트만 실행
npx playwright test tests/e2e/judgment-service.spec.ts

# HTML 리포트 생성
npx playwright show-report
```

---

## 💡 테스트 작성 가이드

### 1. Page Object Model (POM) 패턴

```typescript
// pages/DashboardPage.ts
export class DashboardPage {
  constructor(private page: Page) {}

  async goto() {
    await this.page.goto('/dashboard');
  }

  async generateDashboard(request: string) {
    await this.page.fill('[data-testid="dashboard-request"]', request);
    await this.page.click('[data-testid="generate-button"]');
  }

  async waitForDashboard() {
    await this.page.waitForSelector('[data-testid="generated-dashboard"]');
  }

  async getChartElement(chartType: string) {
    return await this.page.querySelector(`[data-testid="${chartType}"]`);
  }
}

// tests/e2e/dashboard.spec.ts
test('대시보드 자동 생성', async ({ page }) => {
  const dashboardPage = new DashboardPage(page);
  await dashboardPage.goto();
  await dashboardPage.generateDashboard("지난 주 워크플로우별 성공률");
  await dashboardPage.waitForDashboard();

  const chart = await dashboardPage.getChartElement('bar-chart');
  expect(chart).not.toBeNull();
});
```

### 2. 테스트 격리 (Isolation)

```typescript
test.beforeEach(async ({ page }) => {
  // 각 테스트 전 초기화
  await page.goto('/');
  await page.evaluate(() => localStorage.clear());
  await page.evaluate(() => sessionStorage.clear());
});

test.afterEach(async ({ page }) => {
  // 각 테스트 후 정리
  await page.close();
});
```

### 3. 네트워크 모킹 (Mocking)

```typescript
test('API 응답 모킹', async ({ page }) => {
  // API 응답 가로채기
  await page.route('**/api/v2/judgment/execute', route => {
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        result: true,
        method_used: 'rule',
        confidence: 0.95
      })
    });
  });

  await page.goto('/judgment');
  // 테스트 계속...
});
```

---

## 🔗 관련 문서

- [CLAUDE.md](../../CLAUDE.md) - 섹션 9 (개발 검증 및 테스트 전략)
- [docs/operations/deployment_strategy.md](../operations/deployment_strategy.md) - CI/CD 통합
- **MCP 도구**: `playwright-mcp-server` 활용 가이드

---

## 🆘 트러블슈팅

### 문제 1: 타임아웃 에러

```typescript
// 해결: 타임아웃 시간 증가
test('긴 작업 테스트', async ({ page }) => {
  test.setTimeout(60000); // 60초
  // ...
});
```

### 문제 2: 플레이크 테스트 (Flaky Tests)

```typescript
// 해결: 명시적 대기 추가
await page.waitForSelector('[data-testid="result"]', { state: 'visible' });
await page.waitForLoadState('networkidle');
```

### 문제 3: 브라우저 호환성

```bash
# 브라우저 재설치
npx playwright install --with-deps chromium
```
