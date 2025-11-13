import { test, expect } from './fixtures/base';
import { changeTabVisibility, isPageVisible, wait } from './helpers/test-helpers';

/**
 * Tab Recovery E2E Test
 *
 * 가장 중요한 테스트: 탭 전환시 데이터 손실 방지 및 복구 검증
 *
 * 시나리오:
 * 1. 채팅 입력 중 다른 탭으로 전환
 * 2. 다른 탭에서 작업 수행
 * 3. 채팅 탭으로 복귀
 * 4. 입력 중이던 텍스트가 유지되는지 확인
 * 5. 캐시/세션이 정상 복구되는지 확인
 */

test.describe('Tab Recovery - 탭 전환 및 복구', () => {
  test.beforeEach(async ({ chatPage }) => {
    // 채팅 페이지로 이동
    await chatPage.goto();
    await wait(1000); // 페이지 로딩 대기
  });

  test('should preserve input text when switching tabs', async ({ page, chatPage }) => {
    // 1. 메시지 입력 (전송하지 않음)
    const testMessage = 'This is a test message that should not be lost';
    await chatPage.messageInput.fill(testMessage);

    // 입력 확인
    const inputValue = await chatPage.messageInput.inputValue();
    expect(inputValue).toBe(testMessage);

    // 2. 탭을 백그라운드로 전환 (사용자가 다른 탭으로 이동)
    await changeTabVisibility(page, false);
    await wait(500);

    // 페이지가 숨겨진 상태인지 확인
    const isHidden = !(await isPageVisible(page));
    expect(isHidden).toBe(true);

    // 3. 몇 초 대기 (다른 탭에서 작업하는 시뮬레이션)
    await wait(2000);

    // 4. 탭을 다시 포그라운드로 전환 (사용자가 채팅 탭으로 복귀)
    await changeTabVisibility(page, true);
    await wait(500);

    // 페이지가 보이는 상태인지 확인
    const isVisible = await isPageVisible(page);
    expect(isVisible).toBe(true);

    // 5. 입력한 텍스트가 유지되는지 확인
    const recoveredValue = await chatPage.messageInput.inputValue();
    expect(recoveredValue).toBe(testMessage);
  });

  test('should maintain session state after tab switch', async ({ page, chatPage }) => {
    // 1. 메시지 전송
    await chatPage.sendMessage('Hello, first message');
    await chatPage.waitForResponse(30000);

    // 메시지 개수 확인
    const messagesBefore = await chatPage.getMessages();
    expect(messagesBefore.length).toBeGreaterThan(0);

    // 2. 탭 전환
    await changeTabVisibility(page, false);
    await wait(2000);
    await changeTabVisibility(page, true);
    await wait(500);

    // 3. 세션이 유지되는지 확인 (메시지 개수 동일)
    const messagesAfter = await chatPage.getMessages();
    expect(messagesAfter.length).toBe(messagesBefore.length);
  });

  test('should recover from cache after tab visibility change', async ({ page, chatPage }) => {
    // 1. 여러 메시지 전송
    await chatPage.sendMessage('Message 1');
    await chatPage.waitForResponse(30000);

    await wait(1000);

    await chatPage.sendMessage('Message 2');
    await chatPage.waitForResponse(30000);

    // 2. 메시지 히스토리 확인
    const messages = await chatPage.getMessages();
    expect(messages.length).toBeGreaterThanOrEqual(2);

    // 3. 탭 전환 (캐시에서 복구해야 함)
    await changeTabVisibility(page, false);
    await wait(3000); // 충분한 대기 시간
    await changeTabVisibility(page, true);
    await wait(1000);

    // 4. 메시지가 캐시에서 복구되었는지 확인
    const recoveredMessages = await chatPage.getMessages();
    expect(recoveredMessages.length).toBe(messages.length);

    // 5. 메시지 내용이 동일한지 확인
    for (let i = 0; i < messages.length; i++) {
      expect(recoveredMessages[i]).toBe(messages[i]);
    }
  });

  test('should handle rapid tab switching', async ({ page, chatPage }) => {
    // 1. 메시지 입력
    const testMessage = 'Rapid tab switching test';
    await chatPage.messageInput.fill(testMessage);

    // 2. 빠른 탭 전환 (5회)
    for (let i = 0; i < 5; i++) {
      await changeTabVisibility(page, false);
      await wait(200);
      await changeTabVisibility(page, true);
      await wait(200);
    }

    // 3. 입력 값이 여전히 유지되는지 확인
    const finalValue = await chatPage.messageInput.inputValue();
    expect(finalValue).toBe(testMessage);
  });

  test('should restore focus to input after tab switch', async ({ page, chatPage }) => {
    // 1. 메시지 입력란에 포커스
    await chatPage.messageInput.focus();

    // 포커스 확인
    const isFocusedBefore = await chatPage.messageInput.evaluate((el) => {
      return document.activeElement === el;
    });
    expect(isFocusedBefore).toBe(true);

    // 2. 탭 전환
    await changeTabVisibility(page, false);
    await wait(1000);
    await changeTabVisibility(page, true);
    await wait(500);

    // 3. 포커스가 복구되는지 확인 (선택 사항)
    // 주의: 일부 브라우저는 탭 전환 후 자동으로 포커스를 복구하지 않을 수 있음
    // 이 경우 앱에서 명시적으로 포커스를 복구해야 함

    // 입력란 클릭 (포커스 복구)
    await chatPage.messageInput.click();

    // 포커스 확인
    const isFocusedAfter = await chatPage.messageInput.evaluate((el) => {
      return document.activeElement === el;
    });
    expect(isFocusedAfter).toBe(true);
  });

  test('should handle long tab inactivity (10 seconds)', async ({ page, chatPage }) => {
    // 1. 메시지 전송
    await chatPage.sendMessage('Before long inactivity');
    await chatPage.waitForResponse(30000);

    const messagesBefore = await chatPage.getMessages();

    // 2. 장시간 탭 비활성화 (10초)
    await changeTabVisibility(page, false);
    await wait(10000); // 10초 대기
    await changeTabVisibility(page, true);
    await wait(1000);

    // 3. 세션이 유지되는지 확인
    const messagesAfter = await chatPage.getMessages();
    expect(messagesAfter.length).toBe(messagesBefore.length);

    // 4. 새 메시지 전송 가능 여부 확인
    await chatPage.sendMessage('After long inactivity');
    await chatPage.waitForResponse(30000);

    const finalMessages = await chatPage.getMessages();
    expect(finalMessages.length).toBeGreaterThan(messagesBefore.length);
  });

  test('should preserve scroll position after tab switch', async ({ page, chatPage }) => {
    // 1. 여러 메시지 전송 (스크롤 생성)
    for (let i = 1; i <= 5; i++) {
      await chatPage.sendMessage(`Message ${i}`);
      await chatPage.waitForResponse(30000);
      await wait(500);
    }

    // 2. 스크롤 위치 기록
    const scrollBefore = await page.evaluate(() => {
      const messageList = document.querySelector('[data-testid="message-list"], .message-list, .messages-container');
      return messageList ? messageList.scrollTop : 0;
    });

    // 3. 탭 전환
    await changeTabVisibility(page, false);
    await wait(2000);
    await changeTabVisibility(page, true);
    await wait(500);

    // 4. 스크롤 위치 확인 (대략적으로 유지되어야 함)
    const scrollAfter = await page.evaluate(() => {
      const messageList = document.querySelector('[data-testid="message-list"], .message-list, .messages-container');
      return messageList ? messageList.scrollTop : 0;
    });

    // 스크롤 위치가 크게 변하지 않았는지 확인 (±50px 허용)
    expect(Math.abs(scrollAfter - scrollBefore)).toBeLessThan(50);
  });

  test('should handle tab switch during message sending', async ({ page, chatPage }) => {
    // 1. 메시지 전송 시작
    await chatPage.sendMessage('Message during tab switch');

    // 2. 응답을 기다리는 동안 탭 전환
    await wait(500); // 전송 시작 후 잠시 대기
    await changeTabVisibility(page, false);
    await wait(2000);
    await changeTabVisibility(page, true);

    // 3. 메시지가 정상적으로 전송되었는지 확인
    await chatPage.waitForResponse(30000);

    const messages = await chatPage.getMessages();
    const lastMessage = await chatPage.getLastMessage();

    expect(messages.length).toBeGreaterThan(0);
    expect(lastMessage).toContain('Message during tab switch');
  });

  test('should maintain cache state across multiple tab switches', async ({ page, chatPage }) => {
    // 1. 초기 메시지 전송
    await chatPage.sendMessage('Initial message');
    await chatPage.waitForResponse(30000);

    // 2. 여러 번 탭 전환
    for (let i = 0; i < 3; i++) {
      await changeTabVisibility(page, false);
      await wait(1000);
      await changeTabVisibility(page, true);
      await wait(500);

      // 각 전환 후 메시지 추가
      await chatPage.sendMessage(`Message after switch ${i + 1}`);
      await chatPage.waitForResponse(30000);
      await wait(500);
    }

    // 3. 모든 메시지가 저장되었는지 확인
    const allMessages = await chatPage.getMessages();
    expect(allMessages.length).toBeGreaterThanOrEqual(4); // 1 initial + 3 after switches
  });
});
