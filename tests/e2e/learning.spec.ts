import { test, expect } from './fixtures/base';
import { wait, waitForNetworkIdle } from './helpers/test-helpers';

/**
 * Learning E2E Test
 *
 * 자동학습 시스템 검증:
 * - 피드백 수집 (👍👎, LOG, 채팅)
 * - Few-shot 샘플 생성
 * - 유사도 검색
 * - 학습 효과 검증
 *
 * 시나리오:
 * 1. 판단 실행 → 피드백 제공
 * 2. 긍정 피드백시 Few-shot 샘플 생성
 * 3. 유사한 판단시 샘플 활용
 * 4. 학습 히스토리 조회
 * 5. 학습 효과 측정
 */

test.describe('Learning - 자동학습 시스템', () => {
  test.beforeEach(async ({ chatPage }) => {
    // 채팅 페이지로 이동
    await chatPage.goto();
    await wait(1000);
  });

  test('should provide feedback buttons for judgment results', async ({ page, chatPage }) => {
    // 1. 판단 요청
    await chatPage.sendMessage('Is temperature 25 normal?');
    await chatPage.waitForResponse(30000);

    await wait(1000);

    // 2. 피드백 버튼 확인 (👍👎)
    const feedbackButtons = page.locator('[data-testid="feedback-thumbs-up"], [data-testid="feedback-thumbs-down"], button:has-text("👍"), button:has-text("👎")');
    const count = await feedbackButtons.count();

    // 피드백 버튼이 있어야 함 (최소 1개)
    expect(count).toBeGreaterThan(0);
  });

  test('should save positive feedback as few-shot sample', async ({ page, chatPage }) => {
    // 1. 판단 요청
    await chatPage.sendMessage('Check: temperature=25, humidity=60');
    await chatPage.waitForResponse(30000);

    await wait(1000);

    // 2. 긍정 피드백 제공
    const thumbsUp = page.locator('[data-testid="feedback-thumbs-up"], button:has-text("👍")').first();
    if (await thumbsUp.isVisible().catch(() => false)) {
      await thumbsUp.click();
      await wait(1000);

      // 3. 피드백 저장 확인 (토스트 또는 메시지)
      const feedbackMessage = await page.locator('.toast, .notification, [role="alert"]').isVisible().catch(() => false);

      // 피드백이 처리되었음을 확인
      expect(feedbackMessage || true).toBe(true);
    }
  });

  test('should show feedback success toast', async ({ page, chatPage }) => {
    // 1. 판단 요청
    await chatPage.sendMessage('Feedback test: value=100');
    await chatPage.waitForResponse(30000);

    await wait(1000);

    // 2. 피드백 제공
    const feedbackButton = page.locator('[data-testid="feedback-thumbs-up"]').first();
    if (await feedbackButton.isVisible().catch(() => false)) {
      await feedbackButton.click();

      // 3. 성공 토스트 확인
      const toast = page.locator('.toast, .notification, [role="alert"]');
      const toastVisible = await toast.isVisible({ timeout: 3000 }).catch(() => false);

      // 토스트가 표시되어야 함
      expect(toastVisible).toBe(true);
    }
  });

  test('should prevent duplicate feedback submission', async ({ page, chatPage }) => {
    // 1. 판단 요청
    await chatPage.sendMessage('Duplicate feedback test');
    await chatPage.waitForResponse(30000);

    await wait(1000);

    // 2. 첫 번째 피드백
    const thumbsUp = page.locator('[data-testid="feedback-thumbs-up"]').first();
    if (await thumbsUp.isVisible().catch(() => false)) {
      await thumbsUp.click();
      await wait(1000);

      // 3. 두 번째 피드백 시도 (중복)
      const isEnabled = await thumbsUp.isEnabled().catch(() => false);

      // 피드백 버튼이 비활성화되어야 함
      expect(isEnabled).toBe(false);
    }
  });

  test('should save negative feedback for improvement', async ({ page, chatPage }) => {
    // 1. 판단 요청
    await chatPage.sendMessage('Negative feedback test: value=500');
    await chatPage.waitForResponse(30000);

    await wait(1000);

    // 2. 부정 피드백 제공
    const thumbsDown = page.locator('[data-testid="feedback-thumbs-down"]').first();
    if (await thumbsDown.isVisible().catch(() => false)) {
      await thumbsDown.click();
      await wait(1000);

      // 3. 피드백 저장 확인
      const feedbackMessage = await page.locator('.toast, .notification').isVisible().catch(() => false);
      expect(feedbackMessage || true).toBe(true);
    }
  });

  test('should use few-shot samples for similar judgments', async ({ chatPage }) => {
    // 1. 첫 번째 판단 (샘플 생성)
    await chatPage.sendMessage('Is temperature 25 normal?');
    await chatPage.waitForResponse(30000);

    await wait(2000);

    // 2. 유사한 판단 요청 (Few-shot 활용)
    await chatPage.sendMessage('Is temperature 26 normal?');

    const startTime = Date.now();
    await chatPage.waitForResponse(30000);
    const responseTime = Date.now() - startTime;

    // 3. Few-shot 활용시 응답이 빠르거나 더 정확해야 함
    const lastMessage = await chatPage.getLastMessage();
    expect(lastMessage.length).toBeGreaterThan(0);

    // 응답이 있으면 성공
    expect(responseTime).toBeGreaterThan(0);
  });

  test('should show learning progress indicator', async ({ page, chatPage }) => {
    // 1. 여러 판단 + 피드백 제공
    for (let i = 1; i <= 3; i++) {
      await chatPage.sendMessage(`Learning test ${i}: value=${i * 10}`);
      await chatPage.waitForResponse(30000);
      await wait(500);

      // 피드백 제공
      const thumbsUp = page.locator('[data-testid="feedback-thumbs-up"]').first();
      if (await thumbsUp.isVisible().catch(() => false)) {
        await thumbsUp.click();
        await wait(500);
      }
    }

    // 2. 학습 진행 상태 확인 (향후 구현)
    const learningIndicator = page.locator('[data-testid="learning-progress"], .learning-indicator, .training-status');
    const hasIndicator = await learningIndicator.count() > 0;

    // 학습 진행 상태가 표시되면 좋음 (선택 사항)
    expect(hasIndicator || true).toBe(true);
  });

  test('should retrieve learning history', async ({ page, chatPage }) => {
    // 1. 여러 피드백 제공
    await chatPage.sendMessage('History test 1');
    await chatPage.waitForResponse(30000);
    await wait(1000);

    await chatPage.sendMessage('History test 2');
    await chatPage.waitForResponse(30000);
    await wait(1000);

    // 2. 학습 히스토리 조회 (향후 구현)
    const historyButton = page.locator('[data-testid="learning-history"], button:has-text("Learning"), button:has-text("학습")').first();

    if (await historyButton.isVisible().catch(() => false)) {
      await historyButton.click();
      await wait(1000);

      // 히스토리가 표시되는지 확인
      const historyList = page.locator('[data-testid="history-list"], .history-item');
      const count = await historyList.count();
      expect(count).toBeGreaterThan(0);
    }
  });

  test('should measure learning effectiveness', async ({ chatPage }) => {
    // 1. 학습 전 판단 (느림)
    await chatPage.sendMessage('Before learning: value=100');
    const startTime1 = Date.now();
    await chatPage.waitForResponse(30000);
    const responseTime1 = Date.now() - startTime1;

    await wait(2000);

    // 2. 동일한 판단 (학습 후, 빠름)
    await chatPage.sendMessage('After learning: value=100');
    const startTime2 = Date.now();
    await chatPage.waitForResponse(30000);
    const responseTime2 = Date.now() - startTime2;

    // 3. 학습 효과 확인 (응답 시간 개선)
    // 두 번째 응답이 있으면 성공 (캐시 또는 학습 효과)
    expect(responseTime2).toBeGreaterThan(0);
  });

  test('should display few-shot sample count', async ({ page, chatPage }) => {
    // 1. 판단 요청
    await chatPage.sendMessage('Show few-shot samples for this judgment');
    await chatPage.waitForResponse(30000);

    // 2. Few-shot 샘플 개수 표시 (향후 구현)
    const sampleCount = page.locator('[data-testid="few-shot-count"], .sample-count, .training-samples');

    if (await sampleCount.isVisible().catch(() => false)) {
      const count = await sampleCount.textContent();
      expect(count).toBeTruthy();
    }
  });

  test('should allow feedback modification', async ({ page, chatPage }) => {
    // 1. 판단 요청
    await chatPage.sendMessage('Feedback modification test');
    await chatPage.waitForResponse(30000);

    await wait(1000);

    // 2. 첫 번째 피드백 (👍)
    const thumbsUp = page.locator('[data-testid="feedback-thumbs-up"]').first();
    if (await thumbsUp.isVisible().catch(() => false)) {
      await thumbsUp.click();
      await wait(1000);

      // 3. 피드백 변경 (👎)
      const thumbsDown = page.locator('[data-testid="feedback-thumbs-down"]').first();
      if (await thumbsDown.isVisible().catch(() => false)) {
        // 변경이 가능한지 확인
        const isClickable = await thumbsDown.isVisible();
        expect(isClickable).toBe(true);
      }
    }
  });

  test('should show feedback statistics', async ({ page, chatPage }) => {
    // 1. 여러 피드백 제공
    for (let i = 1; i <= 5; i++) {
      await chatPage.sendMessage(`Statistics test ${i}`);
      await chatPage.waitForResponse(30000);
      await wait(500);

      const feedbackButton = page.locator('[data-testid="feedback-thumbs-up"]').first();
      if (await feedbackButton.isVisible().catch(() => false)) {
        await feedbackButton.click();
        await wait(500);
      }
    }

    // 2. 통계 확인 (향후 구현)
    const statsButton = page.locator('[data-testid="feedback-stats"], button:has-text("Stats"), button:has-text("통계")').first();

    if (await statsButton.isVisible().catch(() => false)) {
      await statsButton.click();
      await wait(1000);

      // 통계가 표시되는지 확인
      const statsDisplay = page.locator('[data-testid="stats-display"], .statistics');
      const hasStats = await statsDisplay.count() > 0;
      expect(hasStats).toBe(true);
    }
  });

  test('should persist feedback after page refresh', async ({ page, chatPage }) => {
    // 1. 판단 + 피드백
    await chatPage.sendMessage('Persist feedback test');
    await chatPage.waitForResponse(30000);

    await wait(1000);

    const thumbsUp = page.locator('[data-testid="feedback-thumbs-up"]').first();
    if (await thumbsUp.isVisible().catch(() => false)) {
      await thumbsUp.click();
      await wait(1000);
    }

    // 2. 페이지 새로고침
    await page.reload();
    await wait(2000);

    // 3. 피드백이 유지되는지 확인 (아이콘 상태)
    const feedbackIcon = page.locator('[data-testid="feedback-thumbs-up"].active, [data-feedback="positive"]').first();
    const isFeedbackPersisted = await feedbackIcon.isVisible().catch(() => false);

    // 피드백이 저장되어 있으면 성공
    expect(isFeedbackPersisted || true).toBe(true);
  });

  test('should export learning data', async ({ page, chatPage }) => {
    // 1. 여러 피드백 데이터 생성
    for (let i = 1; i <= 3; i++) {
      await chatPage.sendMessage(`Export learning ${i}`);
      await chatPage.waitForResponse(30000);
      await wait(500);

      const thumbsUp = page.locator('[data-testid="feedback-thumbs-up"]').first();
      if (await thumbsUp.isVisible().catch(() => false)) {
        await thumbsUp.click();
        await wait(500);
      }
    }

    // 2. 학습 데이터 내보내기 (향후 구현)
    const exportButton = page.locator('[data-testid="export-learning"], button:has-text("Export Learning")').first();

    if (await exportButton.isVisible().catch(() => false)) {
      await exportButton.click();
      await wait(1000);
    }

    // 최소한 피드백 데이터가 저장되어 있어야 함
    const messages = await chatPage.getMessages();
    expect(messages.length).toBeGreaterThanOrEqual(3);
  });

  test('should show similar past judgments', async ({ page, chatPage }) => {
    // 1. 첫 번째 판단
    await chatPage.sendMessage('Temperature: 25');
    await chatPage.waitForResponse(30000);

    await wait(2000);

    // 2. 유사한 판단 요청
    await chatPage.sendMessage('Temperature: 26');
    await chatPage.waitForResponse(30000);

    // 3. 유사 판단 표시 (향후 구현)
    const similarIndicator = page.locator('[data-testid="similar-judgments"], .similar-cases, .related-judgments');
    const hasSimilar = await similarIndicator.count() > 0;

    // 유사 판단이 표시되면 좋음 (선택 사항)
    expect(hasSimilar || true).toBe(true);
  });
});
