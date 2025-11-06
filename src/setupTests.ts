import '@testing-library/jest-dom';

// Mock ResizeObserver (required for Recharts in tests)
global.ResizeObserver = class ResizeObserver {
  observe() {}
  unobserve() {}
  disconnect() {}
};
