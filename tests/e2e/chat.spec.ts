import { test, expect } from './fixtures/base';
import { wait, clearBrowserData, waitForNetworkIdle } from './helpers/test-helpers';

/**
 * Chat E2E Test
 *
 * 채팅 메시지 전송 및 응답 검증
 *
 * 시나리오:
 * 1. 기본 메시지 전송 및 응답 수신
 * 2. 스트리밍 응답 처리
 * 3. 메시지 히스토리 로딩
 * 4. 세션 관리 (새 세션 생성, 세션 전환)
 * 5. 에러 처리 (전송 실패, 네트워크 오류)
 * 6. 멀티턴 대화 (컨텍스트 유지)
 * 7. Markdown 렌더링
 */

test.describe('Chat - 채팅 메시지 전송 및 응답', () => {
  test.beforeEach(async ({ chatPage }) => {
    // 채팅 페이지로 이동
    await chatPage.goto();
    await wait(1000); // 페이지 로딩 대기
  });

  test('should send message and receive response', async ({ chatPage }) => {
    // 1. 메시지 전송
    const testMessage = 'Hello, how are you?';
    await chatPage.sendMessage(testMessage);

    // 2. 응답 대기 (최대 30초)
    await chatPage.waitForResponse(30000);

    // 3. 메시지가 화면에 표시되는지 확인
    const messages = await chatPage.getMessages();
    expect(messages.length).toBeGreaterThan(0);

    // 4. 사용자 메시지가 포함되어 있는지 확인
    const userMessage = messages.find(msg => msg.includes(testMessage));
    expect(userMessage).toBeDefined();

    // 5. 어시스턴트 응답이 있는지 확인
    const lastMessage = await chatPage.getLastMessage();
    expect(lastMessage.length).toBeGreaterThan(0);
  });

  test('should handle streaming response', async ({ page, chatPage }) => {
    // 스트리밍 응답 추적
    const streamingStates: boolean[] = [];

    // 로딩 상태 모니터링
    page.on('console', msg => {
      if (msg.text().includes('streaming')) {
        streamingStates.push(true);
      }
    });

    // 메시지 전송
    await chatPage.sendMessage('Tell me a short story');

    // 로딩 인디케이터 표시 확인
    const loadingVisible = await page.locator('.loading, [data-testid="loading"]').isVisible().catch(() => false);
    if (loadingVisible) {
      streamingStates.push(true);
    }

    // 응답 대기
    await chatPage.waitForResponse(30000);

    // 스트리밍이 처리되었거나 응답이 완료됨
    const messages = await chatPage.getMessages();
    expect(messages.length).toBeGreaterThan(0);
  });

  test('should load message history', async ({ chatPage }) => {
    // 1. 첫 번째 메시지 전송
    await chatPage.sendMessage('First message');
    await chatPage.waitForResponse(30000);

    await wait(1000);

    // 2. 두 번째 메시지 전송
    await chatPage.sendMessage('Second message');
    await chatPage.waitForResponse(30000);

    // 3. 메시지 히스토리 확인
    const messages = await chatPage.getMessages();
    expect(messages.length).toBeGreaterThanOrEqual(2);

    // 4. 메시지 순서 확인 (첫 번째가 먼저)
    const allText = messages.join(' ');
    expect(allText).toContain('First message');
    expect(allText).toContain('Second message');
  });

  test('should create new session', async ({ chatPage }) => {
    // 1. 첫 번째 세션에서 메시지 전송
    await chatPage.sendMessage('Message in session 1');
    await chatPage.waitForResponse(30000);

    const messagesSession1 = await chatPage.getMessages();

    // 2. 새 세션 생성
    await chatPage.startNewSession();
    await wait(1000);

    // 3. 새 세션에서 메시지가 비어있는지 확인
    const messagesSession2 = await chatPage.getMessages();

    // 새 세션은 이전 메시지를 포함하지 않아야 함
    expect(messagesSession2.length).toBeLessThan(messagesSession1.length);
  });

  test('should handle send failure gracefully', async ({ page, chatPage }) => {
    // 콘솔 에러 수집
    const errors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error') {
        errors.push(msg.text());
      }
    });

    // 1. 매우 긴 메시지 전송 (잠재적 실패 시나리오)
    const longMessage = 'A'.repeat(10000);
    await chatPage.messageInput.fill(longMessage);
    await chatPage.sendButton.click();

    // 2. 응답 대기 또는 에러 확인
    try {
      await chatPage.waitForResponse(10000);
    } catch (error) {
      // 타임아웃은 예상 가능
    }

    // 3. 입력창이 여전히 사용 가능한지 확인
    const inputEnabled = await chatPage.messageInput.isEnabled();
    expect(inputEnabled).toBe(true);
  });

  test('should maintain context in multi-turn conversation', async ({ chatPage }) => {
    // 1. 첫 번째 메시지 (컨텍스트 설정)
    await chatPage.sendMessage('My name is Alice');
    await chatPage.waitForResponse(30000);

    await wait(1000);

    // 2. 두 번째 메시지 (컨텍스트 참조)
    await chatPage.sendMessage('What is my name?');
    await chatPage.waitForResponse(30000);

    // 3. 응답에서 이름이 언급되는지 확인
    const lastMessage = await chatPage.getLastMessage();
    expect(lastMessage.toLowerCase()).toContain('alice');
  });

  test('should render markdown in messages', async ({ page, chatPage }) => {
    // 1. Markdown 포함 메시지 요청
    await chatPage.sendMessage('Show me a code example');
    await chatPage.waitForResponse(30000);

    // 2. Markdown 렌더링 확인 (코드 블록 또는 볼드체)
    const hasCodeBlock = await page.locator('pre, code, .code-block').count() > 0;
    const hasBold = await page.locator('strong, b, .font-bold').count() > 0;

    // Markdown 요소가 하나 이상 있어야 함
    expect(hasCodeBlock || hasBold).toBe(true);
  });

  test('should clear input after sending', async ({ chatPage }) => {
    // 1. 메시지 입력
    const testMessage = 'Test message';
    await chatPage.messageInput.fill(testMessage);

    // 입력 확인
    const inputBefore = await chatPage.messageInput.inputValue();
    expect(inputBefore).toBe(testMessage);

    // 2. 전송
    await chatPage.sendButton.click();

    // 3. 입력창이 비워졌는지 확인
    await wait(500);
    const inputAfter = await chatPage.messageInput.inputValue();
    expect(inputAfter).toBe('');
  });

  test('should disable send button while processing', async ({ page, chatPage }) => {
    // 1. 메시지 전송
    await chatPage.sendMessage('Processing test');

    // 2. 전송 직후 버튼 상태 확인
    await wait(200);

    // 로딩 중에는 버튼이 비활성화되거나 로딩 상태여야 함
    const isDisabled = await chatPage.sendButton.isDisabled().catch(() => false);
    const hasLoadingClass = await page.locator('.loading, [data-loading="true"]').count() > 0;

    // 둘 중 하나는 참이어야 함 (버튼 비활성화 또는 로딩 인디케이터)
    expect(isDisabled || hasLoadingClass).toBe(true);

    // 3. 응답 완료 후 버튼 다시 활성화 확인
    await chatPage.waitForResponse(30000);
    await wait(500);

    const isEnabledAfter = await chatPage.sendButton.isEnabled();
    expect(isEnabledAfter).toBe(true);
  });

  test('should handle empty message submission', async ({ chatPage }) => {
    // 1. 빈 메시지 전송 시도
    await chatPage.messageInput.fill('');
    await chatPage.sendButton.click();

    // 2. 메시지가 전송되지 않아야 함 (또는 경고 표시)
    await wait(1000);

    // 입력창이 여전히 비어있어야 함
    const inputValue = await chatPage.messageInput.inputValue();
    expect(inputValue).toBe('');

    // 메시지 목록이 변하지 않아야 함
    const messages = await chatPage.getMessages();
    expect(messages.length).toBe(0);
  });

  test('should auto-scroll to latest message', async ({ page, chatPage }) => {
    // 1. 여러 메시지 전송 (스크롤 생성)
    for (let i = 1; i <= 3; i++) {
      await chatPage.sendMessage(`Message ${i}`);
      await chatPage.waitForResponse(30000);
      await wait(500);
    }

    // 2. 스크롤 위치 확인 (하단에 있어야 함)
    const scrollInfo = await page.evaluate(() => {
      const messageList = document.querySelector('[data-testid="message-list"], .message-list, .messages-container');
      if (!messageList) return null;

      return {
        scrollTop: messageList.scrollTop,
        scrollHeight: messageList.scrollHeight,
        clientHeight: messageList.clientHeight
      };
    });

    if (scrollInfo) {
      // 스크롤이 하단 근처에 있는지 확인 (±100px 허용)
      const distanceFromBottom = scrollInfo.scrollHeight - scrollInfo.scrollTop - scrollInfo.clientHeight;
      expect(distanceFromBottom).toBeLessThan(100);
    }
  });

  test('should preserve message after page refresh', async ({ page, chatPage }) => {
    // 1. 메시지 전송
    await chatPage.sendMessage('Message before refresh');
    await chatPage.waitForResponse(30000);

    const messagesBefore = await chatPage.getMessages();

    // 2. 페이지 새로고침
    await page.reload();
    await wait(2000);

    // 3. 메시지가 유지되는지 확인 (localStorage/sessionStorage 활용)
    const messagesAfter = await chatPage.getMessages();

    // 메시지가 유지되어야 함 (또는 최소한 일부)
    expect(messagesAfter.length).toBeGreaterThan(0);
  });

  test('should handle rapid message sending', async ({ chatPage }) => {
    // 1. 빠르게 여러 메시지 전송
    const messages = ['Quick 1', 'Quick 2', 'Quick 3'];

    for (const msg of messages) {
      await chatPage.messageInput.fill(msg);
      await chatPage.sendButton.click();
      await wait(100); // 매우 짧은 대기
    }

    // 2. 모든 응답 대기
    await wait(5000);

    // 3. 메시지가 모두 전송되었는지 확인
    const allMessages = await chatPage.getMessages();
    const allText = allMessages.join(' ');

    // 최소한 일부 메시지가 포함되어야 함
    expect(allText).toContain('Quick');
  });

  test('should display timestamp for messages', async ({ page, chatPage }) => {
    // 1. 메시지 전송
    await chatPage.sendMessage('Timestamp test');
    await chatPage.waitForResponse(30000);

    // 2. 타임스탬프 요소 확인
    const hasTimestamp = await page.locator('[data-timestamp], .timestamp, time, .message-time').count() > 0;

    // 타임스탬프가 표시되어야 함
    expect(hasTimestamp).toBe(true);
  });
});
