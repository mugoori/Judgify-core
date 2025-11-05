import { Page, Locator } from '@playwright/test';
import { BasePage } from './BasePage';

/**
 * Page Object Model for Chat Interface
 *
 * Encapsulates all interactions with the Chat page
 */
export class ChatPage extends BasePage {
  // Locators
  readonly messageInput: Locator;
  readonly sendButton: Locator;
  readonly messageList: Locator;
  readonly newSessionButton: Locator;
  readonly sessionSidebar: Locator;
  readonly clearCacheButton: Locator;

  constructor(page: Page) {
    super(page);

    // Initialize locators
    this.messageInput = page.locator('textarea[placeholder*="메시지"]');
    this.sendButton = page.locator('button:has-text("전송"), button[type="submit"]').first();
    this.messageList = page.locator('[data-testid="message-list"], .message-list, .messages-container').first();
    this.newSessionButton = page.locator('button:has-text("새 대화"), button:has-text("New Chat")');
    this.sessionSidebar = page.locator('[data-testid="session-sidebar"], .sidebar, aside').first();
    this.clearCacheButton = page.locator('button:has-text("캐시 삭제"), button:has-text("Clear Cache")');
  }

  /**
   * Navigate to Chat page
   */
  async goto() {
    await super.goto('/chat');
    await this.waitForLoad();
  }

  /**
   * Send a message
   */
  async sendMessage(message: string) {
    await this.messageInput.fill(message);
    await this.sendButton.click();
  }

  /**
   * Wait for assistant response
   */
  async waitForResponse(timeout: number = 30000) {
    // Wait for loading indicator to disappear or new message to appear
    await this.page.waitForSelector('.loading, [data-testid="loading"]', {
      state: 'hidden',
      timeout
    }).catch(() => {}); // Ignore if loading indicator doesn't exist

    // Wait for at least one assistant message
    await this.page.waitForSelector('[data-role="assistant"], .assistant-message', {
      state: 'visible',
      timeout
    });
  }

  /**
   * Get all messages
   */
  async getMessages(): Promise<string[]> {
    const messages = await this.page.locator('[data-role], .message').allTextContents();
    return messages;
  }

  /**
   * Get last message text
   */
  async getLastMessage(): Promise<string> {
    const messages = await this.page.locator('[data-role], .message').all();
    if (messages.length === 0) return '';

    return await messages[messages.length - 1].textContent() || '';
  }

  /**
   * Check if message input is disabled (offline mode)
   */
  async isInputDisabled(): Promise<boolean> {
    return await this.messageInput.isDisabled();
  }

  /**
   * Create a new chat session
   */
  async createNewSession() {
    await this.newSessionButton.click();
    await this.waitForLoad();
  }

  /**
   * Get session count in sidebar
   */
  async getSessionCount(): Promise<number> {
    const sessions = await this.sessionSidebar.locator('.session-item, [data-testid="session"]').count();
    return sessions;
  }

  /**
   * Select a session by index (0-based)
   */
  async selectSession(index: number) {
    const sessions = this.sessionSidebar.locator('.session-item, [data-testid="session"]');
    await sessions.nth(index).click();
    await this.waitForLoad();
  }

  /**
   * Clear chat cache
   */
  async clearCache() {
    await this.clearCacheButton.click();
    // Wait for confirmation or cache cleared message
    await this.page.waitForTimeout(1000);
  }

  /**
   * Check if offline banner is visible
   */
  async isOfflineBannerVisible(): Promise<boolean> {
    const banner = this.page.locator('[data-testid="offline-banner"], .offline-indicator').first();
    return await banner.isVisible().catch(() => false);
  }

  /**
   * Wait for message to be sent (check for user message in list)
   */
  async waitForMessageSent(messageText: string, timeout: number = 5000) {
    await this.page.waitForSelector(`[data-role="user"]:has-text("${messageText}")`, {
      state: 'visible',
      timeout
    });
  }
}
