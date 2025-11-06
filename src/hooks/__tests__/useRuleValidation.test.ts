import { renderHook, waitFor } from '@testing-library/react';
import { vi, describe, test, expect, beforeEach, afterEach } from 'vitest';
import { useRuleValidation } from '../useRuleValidation';
import { invoke } from '@tauri-apps/api/tauri';

// Mock Tauri API
vi.mock('@tauri-apps/api/tauri', () => ({
  invoke: vi.fn(),
}));

describe('useRuleValidation', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.clearAllTimers();
  });

  test('should return valid state for empty rule', async () => {
    const { result } = renderHook(() => useRuleValidation(''));

    // Empty rule should be valid by default
    expect(result.current.isValid).toBe(true);
    expect(result.current.errors).toEqual([]);
    expect(result.current.isValidating).toBe(false);
  });

  test('should validate a simple rule expression', async () => {
    vi.mocked(invoke).mockResolvedValue({
      isValid: true,
      errors: [],
    });

    const { result } = renderHook(() => useRuleValidation('temperature > 80', { debounceMs: 0 }));

    await waitFor(() => {
      expect(result.current.isValidating).toBe(false);
    });

    expect(result.current.isValid).toBe(true);
    expect(result.current.errors).toEqual([]);
    expect(invoke).toHaveBeenCalledWith('validate_rule_expression', {
      rule: 'temperature > 80',
    });
  });

  test('should validate a complex rule expression', async () => {
    vi.mocked(invoke).mockResolvedValue({
      isValid: true,
      errors: [],
    });

    const { result } = renderHook(() =>
      useRuleValidation('temperature > 80 && humidity < 50', { debounceMs: 0 })
    );

    await waitFor(() => {
      expect(result.current.isValidating).toBe(false);
    });

    expect(result.current.isValid).toBe(true);
    expect(invoke).toHaveBeenCalledWith('validate_rule_expression', {
      rule: 'temperature > 80 && humidity < 50',
    });
  });

  test('should return validation errors for invalid syntax', async () => {
    vi.mocked(invoke).mockResolvedValue({
      isValid: false,
      errors: ['Invalid syntax: unexpected token ">"'],
    });

    const { result } = renderHook(() => useRuleValidation('temperature >', { debounceMs: 0 }));

    await waitFor(() => {
      expect(result.current.isValidating).toBe(false);
    });

    expect(result.current.isValid).toBe(false);
    expect(result.current.errors).toContain('Invalid syntax: unexpected token ">"');
  });

  test('should provide suggestions for common mistakes', async () => {
    vi.mocked(invoke).mockResolvedValue({
      isValid: false,
      errors: ['Unmatched parentheses'],
      suggestions: ['Check opening/closing parentheses balance'],
    });

    const { result } = renderHook(() =>
      useRuleValidation('(temperature > 80', { debounceMs: 0 })
    );

    await waitFor(() => {
      expect(result.current.isValidating).toBe(false);
    });

    expect(result.current.isValid).toBe(false);
    expect(result.current.errors).toContain('Unmatched parentheses');
    expect(result.current.suggestions).toContain('Check opening/closing parentheses balance');
  });

  test('should debounce validation calls', async () => {
    vi.mocked(invoke).mockResolvedValue({
      isValid: true,
      errors: [],
    });

    const { result, rerender } = renderHook(
      ({ rule }) => useRuleValidation(rule, { debounceMs: 100 }),
      {
        initialProps: { rule: '' },
      }
    );

    // Change rule multiple times rapidly
    rerender({ rule: 'temperature > 80' });
    rerender({ rule: 'temperature > 85' });
    rerender({ rule: 'temperature > 90' });

    // Wait for debounce to complete
    await new Promise((resolve) => setTimeout(resolve, 150));

    await waitFor(() => {
      expect(result.current.isValidating).toBe(false);
    });

    // Should only validate the last rule (after debounce)
    expect(invoke).toHaveBeenCalledTimes(1);
    expect(invoke).toHaveBeenCalledWith('validate_rule_expression', {
      rule: 'temperature > 90',
    });
  });

  test('should handle validation errors gracefully', async () => {
    const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    vi.mocked(invoke).mockRejectedValue(new Error('Network error'));

    const { result } = renderHook(() => useRuleValidation('temperature > 80', { debounceMs: 50 }));

    await waitFor(
      () => {
        expect(result.current.isValidating).toBe(false);
        expect(result.current.isValid).toBe(false);
      },
      { timeout: 500 }
    );

    expect(result.current.errors).toContain('Network error');
    consoleErrorSpy.mockRestore();
  });

  test('should respect enabled option', () => {
    vi.mocked(invoke).mockResolvedValue({
      isValid: true,
      errors: [],
    });

    const { result } = renderHook(() =>
      useRuleValidation('temperature > 80', { enabled: false, debounceMs: 0 })
    );

    // Should not call invoke when disabled
    expect(invoke).not.toHaveBeenCalled();
    expect(result.current.isValid).toBe(true);
    expect(result.current.errors).toEqual([]);
  });
});
