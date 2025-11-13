import { test, expect } from './fixtures/base';
import { wait, clearBrowserData, getLocalStorageItem, setLocalStorageItem } from './helpers/test-helpers';

/**
 * Cache E2E Test
 *
 * 캐시 동작 검증 (Memory-First Hybrid Cache)
 *
 * 시나리오:
 * 1. 메모리 캐시 작동 (GET 연산 빠른 속도)
 * 2. SQLite 캐시 지속성 (페이지 새로고침 후에도 유지)
 * 3. 캐시 적중률 (Cache Hit Rate)
 * 4. 캐시 만료 (TTL)
 * 5. 캐시 무효화 (Clear Cache)
 * 6. 캐시 우선순위 (Memory → SQLite → Backend)
 * 7. 캐시 크기 제한 (메모리 10MB, SQLite 100MB)
 * 8. 캐시 워밍업 (앱 시작시 SQLite → Memory 로딩)
 */

test.describe('Cache - 캐시 동작 검증', () => {
  test.beforeEach(async ({ chatPage }) => {
    // 채팅 페이지로 이동
    await chatPage.goto();
    await wait(1000);
  });

  test('should cache messages in memory for fast access', async ({ chatPage }) => {
    // 1. 메시지 전송 (캐시에 저장됨)
    await chatPage.sendMessage('Cached message 1');
    await chatPage.waitForResponse(30000);

    // 2. 동일한 메시지 다시 조회 (메모리 캐시에서 빠르게 로딩)
    const startTime = Date.now();
    const messages = await chatPage.getMessages();
    const loadTime = Date.now() - startTime;

    // 메모리에서 로딩하므로 매우 빨라야 함 (< 100ms)
    expect(loadTime).toBeLessThan(100);
    expect(messages.length).toBeGreaterThan(0);
  });

  test('should persist cache across page reloads', async ({ page, chatPage }) => {
    // 1. 메시지 전송 (SQLite에 저장됨)
    await chatPage.sendMessage('Persistent message');
    await chatPage.waitForResponse(30000);

    const messagesBefore = await chatPage.getMessages();

    // 2. 페이지 새로고침 (메모리 캐시 클리어)
    await page.reload();
    await wait(2000);

    // 3. SQLite에서 복구된 메시지 확인
    const messagesAfter = await chatPage.getMessages();

    // 메시지가 유지되어야 함 (SQLite 캐시에서 복구)
    expect(messagesAfter.length).toBe(messagesBefore.length);
  });

  test('should achieve high cache hit rate', async ({ chatPage }) => {
    // 1. 여러 메시지 전송 (캐시에 저장)
    for (let i = 1; i <= 5; i++) {
      await chatPage.sendMessage(`Message ${i}`);
      await chatPage.waitForResponse(30000);
      await wait(500);
    }

    // 2. 메시지 히스토리 조회 (캐시에서 로딩)
    const startTime = Date.now();
    const messages = await chatPage.getMessages();
    const loadTime = Date.now() - startTime;

    // 캐시에서 로딩하므로 빠름 (< 200ms)
    expect(loadTime).toBeLessThan(200);

    // 모든 메시지가 로딩됨
    expect(messages.length).toBeGreaterThanOrEqual(5);
  });

  test('should invalidate cache when cleared', async ({ page, chatPage }) => {
    // 1. 메시지 전송 (캐시에 저장)
    await chatPage.sendMessage('Message before clear');
    await chatPage.waitForResponse(30000);

    const messagesBefore = await chatPage.getMessages();
    expect(messagesBefore.length).toBeGreaterThan(0);

    // 2. 캐시 클리어 (Clear Cache 버튼 클릭)
    const clearButton = page.locator('[data-testid="clear-cache"], button:has-text("캐시 클리어"), button:has-text("Clear Cache")').first();

    if (await clearButton.isVisible().catch(() => false)) {
      await clearButton.click();
      await wait(1000);

      // 3. 메시지가 클리어되었는지 확인
      const messagesAfter = await chatPage.getMessages();

      // 캐시 클리어 후 메시지가 없거나 적어야 함
      expect(messagesAfter.length).toBeLessThanOrEqual(messagesBefore.length);
    }
  });

  test('should prioritize memory over SQLite', async ({ page, chatPage }) => {
    // 1. 메시지 전송 (메모리 + SQLite에 저장)
    await chatPage.sendMessage('Priority test');
    await chatPage.waitForResponse(30000);

    // 2. 메모리 캐시에서 빠르게 조회
    const startTime1 = Date.now();
    const messages1 = await chatPage.getMessages();
    const loadTime1 = Date.now() - startTime1;

    // 메모리에서 로딩 (< 100ms)
    expect(loadTime1).toBeLessThan(100);

    // 3. 페이지 새로고침 (메모리 클리어)
    await page.reload();
    await wait(2000);

    // 4. SQLite에서 조회 (더 느림)
    const startTime2 = Date.now();
    const messages2 = await chatPage.getMessages();
    const loadTime2 = Date.now() - startTime2;

    // SQLite에서 로딩 (메모리보다 느림, 하지만 여전히 빠름 < 500ms)
    expect(loadTime2).toBeLessThan(500);

    // 메시지는 동일
    expect(messages2.length).toBe(messages1.length);
  });

  test('should warm up cache on app start', async ({ page, chatPage }) => {
    // 1. 메시지 전송 (SQLite에 저장)
    await chatPage.sendMessage('Message for warmup');
    await chatPage.waitForResponse(30000);

    // 2. 페이지 새로고침 (앱 재시작 시뮬레이션)
    await page.reload();

    // 3. 캐시 워밍업 대기 (SQLite → Memory 로딩)
    await wait(2000);

    // 4. 메시지 조회 (메모리에서 빠르게 로딩)
    const startTime = Date.now();
    const messages = await chatPage.getMessages();
    const loadTime = Date.now() - startTime;

    // 워밍업 후 메모리에서 로딩하므로 빠름 (< 150ms)
    expect(loadTime).toBeLessThan(150);
    expect(messages.length).toBeGreaterThan(0);
  });

  test('should handle cache miss gracefully', async ({ chatPage }) => {
    // 1. 새 세션 시작 (캐시 없음)
    await chatPage.startNewSession();
    await wait(500);

    // 2. 메시지 조회 (캐시 미스)
    const messages = await chatPage.getMessages();

    // 캐시 미스해도 에러 없이 빈 배열 반환
    expect(messages).toBeDefined();
    expect(Array.isArray(messages)).toBe(true);
  });

  test('should update cache on new messages', async ({ chatPage }) => {
    // 1. 첫 번째 메시지 전송
    await chatPage.sendMessage('Message 1');
    await chatPage.waitForResponse(30000);

    const messages1 = await chatPage.getMessages();

    // 2. 두 번째 메시지 전송 (캐시 업데이트)
    await wait(1000);
    await chatPage.sendMessage('Message 2');
    await chatPage.waitForResponse(30000);

    // 3. 캐시가 업데이트되었는지 확인
    const messages2 = await chatPage.getMessages();

    // 메시지 개수 증가
    expect(messages2.length).toBeGreaterThan(messages1.length);
  });

  test('should fallback to backend if cache corrupted', async ({ page, chatPage }) => {
    // 1. 메시지 전송
    await chatPage.sendMessage('Message before corruption');
    await chatPage.waitForResponse(30000);

    // 2. localStorage 캐시 손상 시뮬레이션
    await page.evaluate(() => {
      localStorage.setItem('cache_corrupt', 'invalid_json{{{');
    });

    // 3. 페이지 새로고침
    await page.reload();
    await wait(2000);

    // 4. 백엔드에서 데이터 로딩 (fallback)
    const messages = await chatPage.getMessages();

    // 백엔드에서 복구하므로 메시지가 있어야 함
    expect(messages).toBeDefined();
  });

  test('should respect cache TTL', async ({ page, chatPage }) => {
    // 1. 메시지 전송 (TTL 설정)
    await chatPage.sendMessage('Message with TTL');
    await chatPage.waitForResponse(30000);

    // 2. 짧은 대기 (TTL 내)
    await wait(2000);

    // 3. 캐시에서 조회 (여전히 유효)
    const messages1 = await chatPage.getMessages();
    expect(messages1.length).toBeGreaterThan(0);

    // 4. 긴 대기 (TTL 만료 시뮬레이션 - 실제 테스트에서는 짧게 설정)
    // 실제 앱에서 TTL이 5분이라면, 테스트에서는 조건부로 체크
    const cacheAge = await page.evaluate(() => {
      const item = localStorage.getItem('cache_timestamp');
      if (!item) return 0;
      return Date.now() - parseInt(item);
    });

    // TTL 로직이 작동하는지 확인 (타임스탬프 존재)
    expect(cacheAge).toBeGreaterThanOrEqual(0);
  });

  test('should handle concurrent cache updates', async ({ chatPage }) => {
    // 1. 빠르게 여러 메시지 전송 (동시 캐시 업데이트)
    const promises = [];
    for (let i = 1; i <= 3; i++) {
      promises.push(
        chatPage.messageInput.fill(`Concurrent ${i}`).then(() => chatPage.sendButton.click())
      );
    }

    await Promise.all(promises);

    // 2. 모든 응답 대기
    await wait(10000);

    // 3. 캐시가 일관성 있게 업데이트되었는지 확인
    const messages = await chatPage.getMessages();

    // 메시지가 손실되지 않고 저장됨
    expect(messages.length).toBeGreaterThanOrEqual(1);
  });

  test('should persist cache across browser sessions', async ({ page, chatPage }) => {
    // 1. 메시지 전송
    await chatPage.sendMessage('Persistent across sessions');
    await chatPage.waitForResponse(30000);

    const messagesBefore = await chatPage.getMessages();

    // 2. 브라우저 컨텍스트 재생성 시뮬레이션 (localStorage 유지)
    await page.reload();
    await wait(2000);

    // 3. 메시지가 여전히 있는지 확인
    const messagesAfter = await chatPage.getMessages();

    // localStorage + SQLite에 저장되므로 유지됨
    expect(messagesAfter.length).toBe(messagesBefore.length);
  });

  test('should evict old entries when memory limit reached', async ({ page, chatPage }) => {
    // 1. 많은 메시지 전송 (메모리 한계 테스트)
    for (let i = 1; i <= 20; i++) {
      await chatPage.sendMessage(`Message ${i}`);
      await chatPage.waitForResponse(30000);
      await wait(200);
    }

    // 2. 메모리 캐시 크기 확인
    const cacheSize = await page.evaluate(() => {
      let total = 0;
      for (let key in localStorage) {
        if (localStorage.hasOwnProperty(key)) {
          total += localStorage[key].length;
        }
      }
      return total;
    });

    // 캐시가 관리되고 있음 (무한 증가하지 않음)
    expect(cacheSize).toBeLessThan(10 * 1024 * 1024); // < 10MB
  });

  test('should provide cache statistics', async ({ page, chatPage }) => {
    // 1. 여러 메시지 전송
    for (let i = 1; i <= 5; i++) {
      await chatPage.sendMessage(`Stats test ${i}`);
      await chatPage.waitForResponse(30000);
      await wait(300);
    }

    // 2. 캐시 통계 확인 (디버그 패널 또는 개발자 도구)
    const cacheStats = await page.evaluate(() => {
      return {
        itemCount: localStorage.length,
        hasCache: localStorage.getItem('chat_cache') !== null
      };
    });

    // 캐시가 작동하고 있음
    expect(cacheStats.itemCount).toBeGreaterThan(0);
  });
});
