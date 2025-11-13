import { test, expect } from './fixtures/base';
import { waitForTauriApp } from './helpers/test-helpers';

/**
 * Health Check E2E Test
 *
 * Basic test to verify Tauri app starts and loads correctly
 */

test.describe('Health Check', () => {
  test('should load Tauri application successfully', async ({ page }) => {
    // Navigate to root
    await page.goto('/');

    // Wait for Tauri API to be available
    await waitForTauriApp(page);

    // Verify page title
    const title = await page.title();
    expect(title).toContain('Judgify');

    // Verify Tauri is loaded
    const tauriLoaded = await page.evaluate(() => {
      return (window as any).__TAURI__ !== undefined;
    });
    expect(tauriLoaded).toBe(true);
  });

  test('should render main navigation', async ({ page }) => {
    await page.goto('/');
    await waitForTauriApp(page);

    // Check for main navigation tabs
    const chatTab = page.locator('a[href="/chat"], button:has-text("Chat"), button:has-text("채팅")').first();
    const dashboardTab = page.locator('a[href="/dashboard"], button:has-text("Dashboard"), button:has-text("대시보드")').first();
    const workflowTab = page.locator('a[href="/workflow-builder"], button:has-text("Workflow"), button:has-text("워크플로우")').first();

    // At least one navigation element should be visible
    const navVisible = await chatTab.isVisible().catch(() => false) ||
                       await dashboardTab.isVisible().catch(() => false) ||
                       await workflowTab.isVisible().catch(() => false);

    expect(navVisible).toBe(true);
  });

  test('should be able to navigate to Chat page', async ({ chatPage }) => {
    await chatPage.goto();

    // Verify we're on the chat page
    const url = await chatPage.getCurrentUrl();
    expect(url).toContain('/chat');

    // Verify message input is present
    await expect(chatPage.messageInput).toBeVisible({ timeout: 10000 });
  });

  test('should display proper page structure', async ({ page }) => {
    await page.goto('/');
    await waitForTauriApp(page);

    // Check for main app container
    const appContainer = page.locator('body, #root, #app').first();
    await expect(appContainer).toBeVisible();

    // Check that page has loaded content (not blank)
    const hasContent = await page.evaluate(() => {
      return document.body.textContent && document.body.textContent.trim().length > 0;
    });
    expect(hasContent).toBe(true);
  });

  test('should not have console errors on initial load', async ({ page }) => {
    const errors: string[] = [];

    // Collect console errors
    page.on('console', msg => {
      if (msg.type() === 'error') {
        errors.push(msg.text());
      }
    });

    await page.goto('/');
    await waitForTauriApp(page);

    // Wait a bit for any delayed errors
    await page.waitForTimeout(2000);

    // Filter out expected errors (if any)
    const criticalErrors = errors.filter(err => {
      // Ignore certain non-critical errors
      return !err.includes('favicon') &&
             !err.includes('DevTools') &&
             !err.includes('[HMR]');
    });

    expect(criticalErrors.length).toBe(0);
  });

  test('should have responsive layout', async ({ page }) => {
    await page.goto('/');
    await waitForTauriApp(page);

    // Test different viewport sizes
    const viewports = [
      { width: 1920, height: 1080 }, // Desktop
      { width: 1366, height: 768 },  // Laptop
      { width: 1024, height: 768 },  // Tablet landscape
    ];

    for (const viewport of viewports) {
      await page.setViewportSize(viewport);
      await page.waitForTimeout(500);

      // Verify content is still visible
      const hasContent = await page.evaluate(() => {
        return document.body.textContent && document.body.textContent.trim().length > 0;
      });
      expect(hasContent).toBe(true);
    }
  });
});
