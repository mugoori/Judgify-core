/**
 * Mock localStorage utility for testing
 *
 * Usage:
 * ```typescript
 * import { setupMockLocalStorage } from '@/__tests__/utils/mockLocalStorage';
 *
 * const mockLocalStorage = setupMockLocalStorage();
 * ```
 */

export const createMockLocalStorage = () => {
  let store: Record<string, string> = {};

  return {
    getItem: (key: string) => store[key] || null,
    setItem: (key: string, value: string) => {
      store[key] = value;
    },
    removeItem: (key: string) => {
      delete store[key];
    },
    clear: () => {
      store = {};
    },
    get length() {
      return Object.keys(store).length;
    },
    key: (index: number) => {
      const keys = Object.keys(store);
      return keys[index] || null;
    },
  };
};

export const setupMockLocalStorage = () => {
  const mockLocalStorage = createMockLocalStorage();

  Object.defineProperty(window, 'localStorage', {
    value: mockLocalStorage,
    writable: true,
    configurable: true,
  });

  return mockLocalStorage;
};
