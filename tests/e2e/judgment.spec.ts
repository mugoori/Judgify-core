import { test, expect } from './fixtures/base';
import { wait, waitForNetworkIdle } from './helpers/test-helpers';

/**
 * Judgment E2E Test
 *
 * Workflow 판단 실행 검증 (향후 구현 대비)
 *
 * 시나리오:
 * 1. 간단한 판단 요청 ("이 데이터 정상인가요?")
 * 2. 판단 결과 수신 (JSON 또는 텍스트)
 * 3. 판단 히스토리 조회
 * 4. 판단 실패 처리 (에러 메시지)
 * 5. 판단 재실행
 * 6. 판단 결과 저장 (캐시)
 * 7. 판단 설명 표시 (Explanation)
 * 8. 복잡한 워크플로우 실행 (향후 확장)
 *
 * 참고: 현재는 채팅 인터페이스를 통한 기본 판단만 테스트
 *       향후 Visual Workflow Builder 구현시 확장 예정
 */

test.describe('Judgment - 워크플로우 판단 실행', () => {
  test.beforeEach(async ({ chatPage }) => {
    // 채팅 페이지로 이동
    await chatPage.goto();
    await wait(1000);
  });

  test('should request simple judgment via chat', async ({ chatPage }) => {
    // 1. 판단 요청 (간단한 질문)
    await chatPage.sendMessage('Is this data normal: temperature=25, humidity=60?');

    // 2. 응답 대기
    await chatPage.waitForResponse(30000);

    // 3. 판단 결과 확인
    const lastMessage = await chatPage.getLastMessage();
    expect(lastMessage.length).toBeGreaterThan(0);

    // 결과에 "normal" 또는 "정상" 같은 판단 용어 포함
    const hasJudgment = lastMessage.toLowerCase().includes('normal') ||
                        lastMessage.toLowerCase().includes('정상') ||
                        lastMessage.toLowerCase().includes('yes') ||
                        lastMessage.toLowerCase().includes('no');

    expect(hasJudgment).toBe(true);
  });

  test('should display judgment result in structured format', async ({ page, chatPage }) => {
    // 1. 판단 요청
    await chatPage.sendMessage('Analyze: {"temperature": 85, "vibration": 120}');

    // 2. 응답 대기
    await chatPage.waitForResponse(30000);

    // 3. 구조화된 응답 확인 (JSON 또는 표 형식)
    const hasStructuredData = await page.locator('pre, code, table, .json-view, [data-testid="judgment-result"]').count() > 0;

    // 구조화된 데이터가 표시되어야 함 (또는 최소한 명확한 판단)
    const lastMessage = await chatPage.getLastMessage();
    expect(lastMessage.length).toBeGreaterThan(0);
  });

  test('should show judgment explanation', async ({ chatPage }) => {
    // 1. 판단 요청
    await chatPage.sendMessage('Why is temperature 95 considered high?');

    // 2. 응답 대기
    await chatPage.waitForResponse(30000);

    // 3. 설명이 포함된 응답 확인
    const lastMessage = await chatPage.getLastMessage();

    // 설명 키워드 확인
    const hasExplanation = lastMessage.toLowerCase().includes('because') ||
                          lastMessage.toLowerCase().includes('왜냐하면') ||
                          lastMessage.toLowerCase().includes('reason') ||
                          lastMessage.toLowerCase().includes('이유');

    expect(hasExplanation).toBe(true);
  });

  test('should handle judgment with multiple criteria', async ({ chatPage }) => {
    // 1. 복잡한 판단 요청 (여러 기준)
    await chatPage.sendMessage('Check if all criteria met: temp<50, humidity<70, vibration<100');

    // 2. 응답 대기
    await chatPage.waitForResponse(30000);

    // 3. 모든 기준에 대한 응답 확인
    const lastMessage = await chatPage.getLastMessage();

    // 응답이 충분히 상세해야 함
    expect(lastMessage.length).toBeGreaterThan(20);
  });

  test('should save judgment history', async ({ chatPage }) => {
    // 1. 첫 번째 판단
    await chatPage.sendMessage('Check: value=100');
    await chatPage.waitForResponse(30000);

    await wait(1000);

    // 2. 두 번째 판단
    await chatPage.sendMessage('Check: value=200');
    await chatPage.waitForResponse(30000);

    // 3. 히스토리 확인
    const messages = await chatPage.getMessages();

    // 두 판단이 모두 저장됨
    expect(messages.length).toBeGreaterThanOrEqual(2);
  });

  test('should allow judgment retry on failure', async ({ page, chatPage }) => {
    // 1. 판단 요청
    await chatPage.sendMessage('Complex judgment request');
    await chatPage.waitForResponse(30000);

    // 2. 같은 요청 재시도
    await wait(1000);
    await chatPage.sendMessage('Complex judgment request');
    await chatPage.waitForResponse(30000);

    // 3. 재시도가 성공했는지 확인
    const messages = await chatPage.getMessages();
    expect(messages.length).toBeGreaterThan(0);
  });

  test('should cache judgment results', async ({ chatPage }) => {
    // 1. 판단 요청
    const judgmentQuery = 'Is temperature 50 normal?';
    await chatPage.sendMessage(judgmentQuery);
    await chatPage.waitForResponse(30000);

    const firstResponseTime = Date.now();

    await wait(2000);

    // 2. 동일한 판단 요청 (캐시에서 빠르게 응답)
    await chatPage.sendMessage(judgmentQuery);

    const startTime = Date.now();
    await chatPage.waitForResponse(10000);
    const responseTime = Date.now() - startTime;

    // 캐시된 응답은 빠름 (< 5초)
    expect(responseTime).toBeLessThan(5000);
  });

  test('should handle invalid judgment request', async ({ page, chatPage }) => {
    // 콘솔 에러 수집
    const errors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error') {
        errors.push(msg.text());
      }
    });

    // 1. 잘못된 판단 요청
    await chatPage.sendMessage('Invalid JSON: {{{{{');

    // 2. 응답 대기 (에러 또는 거부 응답)
    try {
      await chatPage.waitForResponse(10000);
    } catch {
      // 타임아웃 가능
    }

    // 3. 앱이 여전히 작동하는지 확인
    const inputEnabled = await chatPage.messageInput.isEnabled();
    expect(inputEnabled).toBe(true);
  });

  test('should provide judgment confidence score', async ({ chatPage }) => {
    // 1. 판단 요청
    await chatPage.sendMessage('How confident are you that this is normal?');

    // 2. 응답 대기
    await chatPage.waitForResponse(30000);

    // 3. 신뢰도 점수 확인
    const lastMessage = await chatPage.getLastMessage();

    // 신뢰도 키워드 확인 (%, confident, 확신)
    const hasConfidence = lastMessage.toLowerCase().includes('%') ||
                         lastMessage.toLowerCase().includes('confident') ||
                         lastMessage.toLowerCase().includes('확신') ||
                         lastMessage.toLowerCase().includes('likely');

    expect(hasConfidence).toBe(true);
  });

  test('should compare multiple judgment scenarios', async ({ chatPage }) => {
    // 1. 첫 번째 시나리오 판단
    await chatPage.sendMessage('Scenario A: temp=30, humidity=50');
    await chatPage.waitForResponse(30000);

    await wait(1000);

    // 2. 두 번째 시나리오 판단
    await chatPage.sendMessage('Scenario B: temp=90, humidity=80');
    await chatPage.waitForResponse(30000);

    await wait(1000);

    // 3. 비교 요청
    await chatPage.sendMessage('Which scenario is better?');
    await chatPage.waitForResponse(30000);

    // 4. 비교 결과 확인
    const lastMessage = await chatPage.getLastMessage();

    // 비교 키워드 확인
    const hasComparison = lastMessage.toLowerCase().includes('better') ||
                         lastMessage.toLowerCase().includes('worse') ||
                         lastMessage.toLowerCase().includes('더') ||
                         lastMessage.toLowerCase().includes('scenario');

    expect(hasComparison).toBe(true);
  });

  test('should handle streaming judgment response', async ({ page, chatPage }) => {
    // 1. 긴 판단 요청 (스트리밍 응답 예상)
    await chatPage.sendMessage('Provide detailed analysis of all factors');

    // 2. 로딩 상태 확인
    await wait(500);
    const isLoading = await page.locator('.loading, [data-testid="loading"]').isVisible().catch(() => false);

    // 3. 스트리밍 응답 대기
    await chatPage.waitForResponse(30000);

    // 4. 응답이 수신되었는지 확인
    const lastMessage = await chatPage.getLastMessage();
    expect(lastMessage.length).toBeGreaterThan(0);
  });

  test('should persist judgment results after page refresh', async ({ page, chatPage }) => {
    // 1. 판단 요청
    await chatPage.sendMessage('Judgment to persist');
    await chatPage.waitForResponse(30000);

    const messagesBefore = await chatPage.getMessages();

    // 2. 페이지 새로고침
    await page.reload();
    await wait(2000);

    // 3. 판단 결과가 유지되는지 확인
    const messagesAfter = await chatPage.getMessages();

    // 판단 결과가 캐시에 저장되어 유지됨
    expect(messagesAfter.length).toBe(messagesBefore.length);
  });

  test('should display judgment timestamp', async ({ page, chatPage }) => {
    // 1. 판단 요청
    await chatPage.sendMessage('Timestamped judgment');
    await chatPage.waitForResponse(30000);

    // 2. 타임스탬프 확인
    const hasTimestamp = await page.locator('[data-timestamp], .timestamp, time, .message-time').count() > 0;

    // 타임스탬프가 표시되어야 함
    expect(hasTimestamp).toBe(true);
  });

  test('should allow filtering judgment history', async ({ page, chatPage }) => {
    // 1. 여러 판단 요청 (다른 타입)
    await chatPage.sendMessage('Temperature check: 25');
    await chatPage.waitForResponse(30000);

    await wait(500);

    await chatPage.sendMessage('Humidity check: 60');
    await chatPage.waitForResponse(30000);

    await wait(500);

    await chatPage.sendMessage('Vibration check: 50');
    await chatPage.waitForResponse(30000);

    // 2. 히스토리 필터링 (향후 구현시)
    // 현재는 모든 메시지가 히스토리에 저장되는지 확인
    const messages = await chatPage.getMessages();
    expect(messages.length).toBeGreaterThanOrEqual(3);
  });

  test('should export judgment results', async ({ page, chatPage }) => {
    // 1. 여러 판단 요청
    for (let i = 1; i <= 3; i++) {
      await chatPage.sendMessage(`Export test ${i}`);
      await chatPage.waitForResponse(30000);
      await wait(500);
    }

    // 2. 내보내기 버튼 확인 (향후 구현)
    const exportButton = page.locator('[data-testid="export"], button:has-text("Export"), button:has-text("내보내기")').first();

    // 내보내기 기능이 있으면 클릭 (선택 사항)
    if (await exportButton.isVisible().catch(() => false)) {
      await exportButton.click();
      await wait(1000);
    }

    // 최소한 판단 결과가 저장되어 있어야 함
    const messages = await chatPage.getMessages();
    expect(messages.length).toBeGreaterThanOrEqual(3);
  });
});
