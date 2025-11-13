import { Page } from '@playwright/test';

/**
 * Test Helper Utilities
 *
 * Common utility functions for E2E tests
 */

/**
 * Wait for a specific timeout
 */
export async function wait(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

/**
 * Mock network conditions (offline/online)
 */
export async function setNetworkCondition(page: Page, condition: 'offline' | 'online') {
  if (condition === 'offline') {
    await page.context().setOffline(true);
  } else {
    await page.context().setOffline(false);
  }
}

/**
 * Clear browser cache and cookies
 */
export async function clearBrowserData(page: Page) {
  await page.context().clearCookies();
  await page.evaluate(() => {
    localStorage.clear();
    sessionStorage.clear();
  });
}

/**
 * Get localStorage item
 */
export async function getLocalStorageItem(page: Page, key: string): Promise<string | null> {
  return await page.evaluate((k) => localStorage.getItem(k), key);
}

/**
 * Set localStorage item
 */
export async function setLocalStorageItem(page: Page, key: string, value: string) {
  await page.evaluate(
    ({ k, v }) => localStorage.setItem(k, v),
    { k: key, v: value }
  );
}

/**
 * Check if element is in viewport
 */
export async function isElementInViewport(page: Page, selector: string): Promise<boolean> {
  return await page.evaluate((sel) => {
    const element = document.querySelector(sel);
    if (!element) return false;

    const rect = element.getBoundingClientRect();
    return (
      rect.top >= 0 &&
      rect.left >= 0 &&
      rect.bottom <= (window.innerHeight || document.documentElement.clientHeight) &&
      rect.right <= (window.innerWidth || document.documentElement.clientWidth)
    );
  }, selector);
}

/**
 * Scroll element into view
 */
export async function scrollIntoView(page: Page, selector: string) {
  await page.evaluate((sel) => {
    const element = document.querySelector(sel);
    if (element) {
      element.scrollIntoView({ behavior: 'smooth', block: 'center' });
    }
  }, selector);
}

/**
 * Wait for all pending network requests to complete
 */
export async function waitForNetworkIdle(page: Page, timeout: number = 5000) {
  await page.waitForLoadState('networkidle', { timeout });
}

/**
 * Take screenshot with timestamp
 */
export async function takeTimestampedScreenshot(page: Page, name: string) {
  const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
  await page.screenshot({
    path: `test-results/screenshots/${name}-${timestamp}.png`,
    fullPage: true
  });
}

/**
 * Get current tab title
 */
export async function getTabTitle(page: Page): Promise<string> {
  return await page.title();
}

/**
 * Check if page is currently visible (tab focus)
 */
export async function isPageVisible(page: Page): Promise<boolean> {
  return await page.evaluate(() => document.visibilityState === 'visible');
}

/**
 * Simulate tab visibility change
 */
export async function changeTabVisibility(page: Page, visible: boolean) {
  await page.evaluate((vis) => {
    Object.defineProperty(document, 'visibilityState', {
      writable: true,
      configurable: true,
      value: vis ? 'visible' : 'hidden'
    });
    document.dispatchEvent(new Event('visibilitychange'));
  }, visible);
}

/**
 * Get console logs from the page
 */
export function collectConsoleLogs(page: Page): string[] {
  const logs: string[] = [];

  page.on('console', msg => {
    logs.push(`[${msg.type()}] ${msg.text()}`);
  });

  return logs;
}

/**
 * Check if Tauri app is ready
 */
export async function waitForTauriApp(page: Page, timeout: number = 30000) {
  await page.waitForFunction(
    () => {
      return (window as any).__TAURI__ !== undefined;
    },
    { timeout }
  );
}

/**
 * Get current route/path
 */
export async function getCurrentRoute(page: Page): Promise<string> {
  return await page.evaluate(() => window.location.pathname);
}
