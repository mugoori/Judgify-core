# Judgify Desktop App - E2E Tests

Playwright 기반 E2E 테스트 프레임워크입니다.

## 📦 설치

이미 설치되어 있습니다. 추가 설치가 필요한 경우:

```bash
npm install -D @playwright/test playwright
npx playwright install chromium
```

## 🚀 테스트 실행

### 기본 실행 (Headless)
```bash
npm run test:e2e
```

### UI 모드 (추천)
```bash
npm run test:e2e:ui
```

### Headed 모드 (브라우저 보이기)
```bash
npm run test:e2e:headed
```

### 디버그 모드
```bash
npm run test:e2e:debug
```

### 테스트 리포트 보기
```bash
npm run test:e2e:report
```

## 📁 디렉토리 구조

```
tests/e2e/
├── pages/              # Page Object Models (POM)
│   ├── BasePage.ts     # 기본 페이지 클래스
│   └── ChatPage.ts     # Chat 페이지 클래스
├── fixtures/           # 테스트 픽스쳐
│   └── base.ts         # 커스텀 픽스쳐
├── helpers/            # 헬퍼 유틸리티
│   └── test-helpers.ts # 공통 유틸리티 함수
└── *.spec.ts           # 테스트 파일
```

## 📝 테스트 작성 가이드

### 1. Page Object Model 사용

```typescript
import { test, expect } from './fixtures/base';

test('should send a message', async ({ chatPage }) => {
  await chatPage.goto();
  await chatPage.sendMessage('Hello!');
  await chatPage.waitForResponse();

  const lastMessage = await chatPage.getLastMessage();
  expect(lastMessage).toContain('Hello');
});
```

### 2. 헬퍼 함수 활용

```typescript
import { setNetworkCondition, waitForNetworkIdle } from './helpers/test-helpers';

test('should handle offline mode', async ({ page, chatPage }) => {
  await chatPage.goto();

  // 오프라인 모드 설정
  await setNetworkCondition(page, 'offline');

  await chatPage.sendMessage('Test message');

  // 오프라인 배너 확인
  const isOffline = await chatPage.isOfflineBannerVisible();
  expect(isOffline).toBe(true);
});
```

### 3. 새로운 Page Object 추가

```typescript
// tests/e2e/pages/DashboardPage.ts
import { Page, Locator } from '@playwright/test';
import { BasePage } from './BasePage';

export class DashboardPage extends BasePage {
  readonly metricCard: Locator;

  constructor(page: Page) {
    super(page);
    this.metricCard = page.locator('[data-testid="metric-card"]');
  }

  async goto() {
    await super.goto('/dashboard');
    await this.waitForLoad();
  }

  async getMetricValue(name: string): Promise<string> {
    const card = this.page.locator(`[data-metric="${name}"]`);
    return await card.textContent() || '';
  }
}
```

### 4. 픽스쳐에 추가

```typescript
// tests/e2e/fixtures/base.ts
import { DashboardPage } from '../pages/DashboardPage';

type Fixtures = {
  chatPage: ChatPage;
  dashboardPage: DashboardPage; // 추가
};

export const test = base.extend<Fixtures>({
  chatPage: async ({ page }, use) => {
    const chatPage = new ChatPage(page);
    await use(chatPage);
  },
  dashboardPage: async ({ page }, use) => {
    const dashboardPage = new DashboardPage(page);
    await use(dashboardPage);
  },
});
```

## 🎯 현재 테스트 시나리오

### health.spec.ts (6개 테스트)
- ✅ Tauri 앱 로딩 확인
- ✅ 메인 네비게이션 렌더링
- ✅ Chat 페이지 이동
- ✅ 페이지 구조 확인
- ✅ 콘솔 에러 없음 확인
- ✅ 반응형 레이아웃 확인

## 📊 다음 구현 예정

### Day 2-3: 5개 핵심 시나리오
1. **chat.spec.ts** - 채팅 메시지 전송 및 응답
2. **tab-recovery.spec.ts** - 탭 전환 및 복구 (중요!)
3. **offline.spec.ts** - 오프라인 처리
4. **cache.spec.ts** - 캐시 동작 검증
5. **judgment.spec.ts** - Judgment 실행

## 🔧 트러블슈팅

### Tauri 앱이 시작되지 않음
```bash
# Tauri 개발 서버가 실행 중인지 확인
npm run tauri:dev

# 다른 터미널에서 테스트 실행
npm run test:e2e
```

### 타임아웃 에러
- `playwright.config.ts`에서 `timeout` 값 증가
- 또는 개별 테스트에서 `{ timeout: 60000 }` 지정

### Chromium 다운로드 실패
```bash
npx playwright install chromium --force
```

## 📚 참고 자료

- [Playwright 공식 문서](https://playwright.dev/)
- [Tauri E2E Testing](https://tauri.app/v1/guides/testing/webdriver/introduction)
- [Page Object Model 패턴](https://playwright.dev/docs/pom)
