import { test as base } from '@playwright/test';
import { ChatPage } from '../pages/ChatPage';

/**
 * Custom test fixtures for Judgify Desktop App
 *
 * Provides commonly used page objects and utilities
 */

type Fixtures = {
  chatPage: ChatPage;
};

/**
 * Extend base test with custom fixtures
 */
export const test = base.extend<Fixtures>({
  /**
   * Chat Page fixture
   */
  chatPage: async ({ page }, use) => {
    const chatPage = new ChatPage(page);
    await use(chatPage);
  },
});

export { expect } from '@playwright/test';
