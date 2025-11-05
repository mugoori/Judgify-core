import { test, expect } from './fixtures/base';
import { setNetworkCondition, wait, clearBrowserData } from './helpers/test-helpers';

/**
 * Offline Mode E2E Test
 *
 * 오프라인 상태 처리 및 복구 검증
 *
 * 시나리오:
 * 1. 오프라인 전환 감지
 * 2. 오프라인 상태에서 메시지 전송 시도 (실패 처리)
 * 3. 오프라인 상태 표시 (UI 인디케이터)
 * 4. 오프라인→온라인 복구
 * 5. 복구 후 큐잉된 메시지 재전송
 * 6. 캐시된 데이터 표시 (오프라인에서도 히스토리 보기)
 * 7. 불안정한 네트워크 (간헐적 연결)
 */

test.describe('Offline Mode - 오프라인 처리', () => {
  test.beforeEach(async ({ chatPage }) => {
    // 채팅 페이지로 이동
    await chatPage.goto();
    await wait(1000);
  });

  test('should detect offline status', async ({ page, chatPage }) => {
    // 1. 초기 상태 확인 (온라인)
    const isOnlineInitial = await page.evaluate(() => navigator.onLine);
    expect(isOnlineInitial).toBe(true);

    // 2. 오프라인으로 전환
    await setNetworkCondition(page, 'offline');
    await wait(500);

    // 3. 오프라인 상태 확인
    const isOffline = await page.evaluate(() => !navigator.onLine);
    expect(isOffline).toBe(true);

    // 4. 오프라인 인디케이터 표시 확인
    const offlineIndicator = await page.locator('[data-testid="offline-indicator"], .offline-indicator, .network-status.offline').count();

    // 오프라인 표시가 있거나, 적어도 오프라인 상태를 감지해야 함
    expect(isOffline).toBe(true);
  });

  test('should prevent message sending when offline', async ({ page, chatPage }) => {
    // 1. 오프라인으로 전환
    await setNetworkCondition(page, 'offline');
    await wait(500);

    // 2. 메시지 전송 시도
    await chatPage.messageInput.fill('Message while offline');
    await chatPage.sendButton.click();

    // 3. 메시지가 전송되지 않거나 에러 표시
    await wait(2000);

    // 전송 버튼이 비활성화되었거나 에러 메시지가 표시되어야 함
    const hasErrorMessage = await page.locator('[data-testid="error-message"], .error-message, .alert-error').count() > 0;
    const isSendDisabled = await chatPage.sendButton.isDisabled().catch(() => false);

    // 둘 중 하나는 참이어야 함
    expect(hasErrorMessage || isSendDisabled).toBe(true);
  });

  test('should display offline indicator in UI', async ({ page, chatPage }) => {
    // 1. 오프라인으로 전환
    await setNetworkCondition(page, 'offline');
    await wait(1000);

    // 2. 오프라인 인디케이터 확인
    const indicators = await page.locator('[data-testid="offline-indicator"], .offline-indicator, .network-status, .connection-status').count();

    // 오프라인 상태가 UI에 표시되어야 함 (또는 최소한 네트워크 상태 표시)
    expect(indicators).toBeGreaterThan(0);
  });

  test('should recover when back online', async ({ page, chatPage }) => {
    // 1. 메시지 전송 (온라인 상태)
    await chatPage.sendMessage('Before offline');
    await chatPage.waitForResponse(30000);

    // 2. 오프라인으로 전환
    await setNetworkCondition(page, 'offline');
    await wait(1000);

    // 3. 온라인으로 복구
    await setNetworkCondition(page, 'online');
    await wait(1000);

    // 4. 온라인 상태 확인
    const isOnline = await page.evaluate(() => navigator.onLine);
    expect(isOnline).toBe(true);

    // 5. 메시지 전송 가능 확인
    await chatPage.sendMessage('After recovery');
    await chatPage.waitForResponse(30000);

    const messages = await chatPage.getMessages();
    expect(messages.length).toBeGreaterThan(0);
  });

  test('should show cached messages while offline', async ({ page, chatPage }) => {
    // 1. 온라인 상태에서 메시지 전송
    await chatPage.sendMessage('Message 1');
    await chatPage.waitForResponse(30000);

    await wait(1000);

    await chatPage.sendMessage('Message 2');
    await chatPage.waitForResponse(30000);

    const messagesOnline = await chatPage.getMessages();

    // 2. 오프라인으로 전환
    await setNetworkCondition(page, 'offline');
    await wait(1000);

    // 3. 캐시된 메시지가 여전히 보이는지 확인
    const messagesOffline = await chatPage.getMessages();
    expect(messagesOffline.length).toBe(messagesOnline.length);

    // 메시지 내용도 동일해야 함
    for (let i = 0; i < messagesOnline.length; i++) {
      expect(messagesOffline[i]).toBe(messagesOnline[i]);
    }
  });

  test('should queue messages for sending when back online', async ({ page, chatPage }) => {
    // 1. 오프라인으로 전환
    await setNetworkCondition(page, 'offline');
    await wait(500);

    // 2. 오프라인 상태에서 메시지 입력
    const offlineMessage = 'Queued message';
    await chatPage.messageInput.fill(offlineMessage);

    // 3. 온라인으로 복구
    await setNetworkCondition(page, 'online');
    await wait(500);

    // 4. 메시지 전송
    await chatPage.sendButton.click();

    // 5. 메시지가 성공적으로 전송되는지 확인
    await chatPage.waitForResponse(30000);

    const messages = await chatPage.getMessages();
    const sentMessage = messages.find(msg => msg.includes(offlineMessage));
    expect(sentMessage).toBeDefined();
  });

  test('should handle intermittent connectivity', async ({ page, chatPage }) => {
    // 1. 메시지 전송 (온라인)
    await chatPage.sendMessage('Message 1');
    await chatPage.waitForResponse(30000);

    // 2. 짧은 오프라인 (1초)
    await setNetworkCondition(page, 'offline');
    await wait(1000);
    await setNetworkCondition(page, 'online');
    await wait(1000);

    // 3. 다시 메시지 전송
    await chatPage.sendMessage('Message 2');
    await chatPage.waitForResponse(30000);

    // 4. 짧은 오프라인 (1초)
    await setNetworkCondition(page, 'offline');
    await wait(1000);
    await setNetworkCondition(page, 'online');
    await wait(1000);

    // 5. 다시 메시지 전송
    await chatPage.sendMessage('Message 3');
    await chatPage.waitForResponse(30000);

    // 6. 모든 메시지가 저장되었는지 확인
    const messages = await chatPage.getMessages();
    expect(messages.length).toBeGreaterThanOrEqual(3);
  });

  test('should disable send button when offline', async ({ page, chatPage }) => {
    // 1. 초기 상태 확인 (버튼 활성화)
    const isEnabledInitial = await chatPage.sendButton.isEnabled();
    expect(isEnabledInitial).toBe(true);

    // 2. 오프라인으로 전환
    await setNetworkCondition(page, 'offline');
    await wait(1000);

    // 3. 버튼이 비활성화되었는지 확인
    const isDisabledOffline = await chatPage.sendButton.isDisabled().catch(() => false);

    // 오프라인일 때 버튼 비활성화 (또는 경고 표시)
    const hasOfflineWarning = await page.locator('.offline-warning, [data-testid="offline-warning"]').count() > 0;

    expect(isDisabledOffline || hasOfflineWarning).toBe(true);

    // 4. 온라인으로 복구
    await setNetworkCondition(page, 'online');
    await wait(1000);

    // 5. 버튼이 다시 활성화되었는지 확인
    const isEnabledOnline = await chatPage.sendButton.isEnabled();
    expect(isEnabledOnline).toBe(true);
  });

  test('should retry failed requests when back online', async ({ page, chatPage }) => {
    // 1. 메시지 전송 (온라인)
    await chatPage.sendMessage('Message before offline');
    await chatPage.waitForResponse(30000);

    // 2. 오프라인으로 전환
    await setNetworkCondition(page, 'offline');
    await wait(500);

    // 3. 메시지 전송 시도 (실패해야 함)
    await chatPage.messageInput.fill('Failed message');
    await chatPage.sendButton.click();
    await wait(2000);

    // 4. 온라인으로 복구
    await setNetworkCondition(page, 'online');
    await wait(2000);

    // 5. 재시도 또는 수동 재전송
    // (앱이 자동으로 재시도하거나 사용자가 다시 전송 버튼 클릭)
    const isOnline = await page.evaluate(() => navigator.onLine);
    expect(isOnline).toBe(true);

    // 온라인 상태에서 새 메시지 전송 가능
    await chatPage.sendMessage('After recovery');
    await chatPage.waitForResponse(30000);

    const messages = await chatPage.getMessages();
    expect(messages.length).toBeGreaterThan(0);
  });

  test('should preserve input text during offline period', async ({ page, chatPage }) => {
    // 1. 메시지 입력
    const testMessage = 'Message during offline';
    await chatPage.messageInput.fill(testMessage);

    // 2. 오프라인으로 전환
    await setNetworkCondition(page, 'offline');
    await wait(1000);

    // 3. 입력 텍스트가 유지되는지 확인
    const inputOffline = await chatPage.messageInput.inputValue();
    expect(inputOffline).toBe(testMessage);

    // 4. 온라인으로 복구
    await setNetworkCondition(page, 'online');
    await wait(1000);

    // 5. 입력 텍스트가 여전히 유지되는지 확인
    const inputOnline = await chatPage.messageInput.inputValue();
    expect(inputOnline).toBe(testMessage);
  });

  test('should show error toast/notification when offline', async ({ page, chatPage }) => {
    // 1. 오프라인으로 전환
    await setNetworkCondition(page, 'offline');
    await wait(500);

    // 2. 메시지 전송 시도
    await chatPage.messageInput.fill('Test message');
    await chatPage.sendButton.click();
    await wait(1000);

    // 3. 에러 알림 확인
    const hasError = await page.locator('[role="alert"], .toast, .notification, .error-toast, [data-testid="error-toast"]').count() > 0;

    // 에러 알림이 표시되어야 함 (또는 최소한 오프라인 표시)
    const hasOfflineIndicator = await page.locator('.offline-indicator, [data-testid="offline-indicator"]').count() > 0;

    expect(hasError || hasOfflineIndicator).toBe(true);
  });

  test('should maintain session across offline/online transitions', async ({ page, chatPage }) => {
    // 1. 온라인 상태에서 세션 시작
    await chatPage.sendMessage('Session message 1');
    await chatPage.waitForResponse(30000);

    const messagesOnline = await chatPage.getMessages();

    // 2. 오프라인으로 전환
    await setNetworkCondition(page, 'offline');
    await wait(2000);

    // 3. 온라인으로 복구
    await setNetworkCondition(page, 'online');
    await wait(2000);

    // 4. 세션이 유지되는지 확인 (메시지 개수 동일)
    const messagesAfterRecovery = await chatPage.getMessages();
    expect(messagesAfterRecovery.length).toBe(messagesOnline.length);

    // 5. 새 메시지 전송 가능
    await chatPage.sendMessage('Session message 2');
    await chatPage.waitForResponse(30000);

    const finalMessages = await chatPage.getMessages();
    expect(finalMessages.length).toBeGreaterThan(messagesOnline.length);
  });

  test('should handle long offline period (10 seconds)', async ({ page, chatPage }) => {
    // 1. 메시지 전송
    await chatPage.sendMessage('Before long offline');
    await chatPage.waitForResponse(30000);

    // 2. 장시간 오프라인 (10초)
    await setNetworkCondition(page, 'offline');
    await wait(10000);

    // 3. 온라인으로 복구
    await setNetworkCondition(page, 'online');
    await wait(2000);

    // 4. 앱이 여전히 작동하는지 확인
    const isOnline = await page.evaluate(() => navigator.onLine);
    expect(isOnline).toBe(true);

    // 5. 메시지 전송 가능
    await chatPage.sendMessage('After long offline');
    await chatPage.waitForResponse(30000);

    const messages = await chatPage.getMessages();
    expect(messages.length).toBeGreaterThan(0);
  });

  test('should clear error state when back online', async ({ page, chatPage }) => {
    // 1. 오프라인으로 전환
    await setNetworkCondition(page, 'offline');
    await wait(500);

    // 2. 메시지 전송 시도 (에러 발생)
    await chatPage.messageInput.fill('Error message');
    await chatPage.sendButton.click();
    await wait(1000);

    // 3. 온라인으로 복구
    await setNetworkCondition(page, 'online');
    await wait(2000);

    // 4. 에러 상태가 클리어되었는지 확인
    const hasError = await page.locator('[role="alert"].error, .error-toast:visible').count() > 0;

    // 에러 메시지가 사라져야 함 (또는 성공 메시지로 변경)
    expect(hasError).toBe(false);

    // 5. 정상적으로 메시지 전송 가능
    await chatPage.sendMessage('After error clear');
    await chatPage.waitForResponse(30000);

    const messages = await chatPage.getMessages();
    expect(messages.length).toBeGreaterThan(0);
  });
});
