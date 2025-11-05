import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/tauri';

interface ValidationResult {
  isValid: boolean;
  errors: string[];
  suggestions?: string[];
}

interface UseRuleValidationOptions {
  debounceMs?: number;
  enabled?: boolean;
}

/**
 * React Hook for real-time Rule expression validation
 *
 * @param rule - The Rule expression to validate
 * @param options - Configuration options
 * @returns Validation result with isValid, errors, isValidating state
 *
 * @example
 * const { isValid, errors, isValidating } = useRuleValidation(rule, { debounceMs: 500 });
 */
export function useRuleValidation(
  rule: string,
  options: UseRuleValidationOptions = {}
) {
  const { debounceMs = 500, enabled = true } = options;

  const [validation, setValidation] = useState<ValidationResult>({
    isValid: true,
    errors: [],
  });
  const [isValidating, setIsValidating] = useState(false);
  const [debouncedRule, setDebouncedRule] = useState(rule);

  // Debounce the rule input
  useEffect(() => {
    if (!enabled) return;

    const timer = setTimeout(() => {
      setDebouncedRule(rule);
    }, debounceMs);

    return () => clearTimeout(timer);
  }, [rule, debounceMs, enabled]);

  // Validate the rule expression
  const validateRule = useCallback(async (ruleExpression: string) => {
    if (!ruleExpression.trim()) {
      setValidation({
        isValid: true,
        errors: [],
      });
      return;
    }

    setIsValidating(true);

    try {
      const result = await invoke<ValidationResult>('validate_rule_expression', {
        rule: ruleExpression,
      });

      setValidation(result);
    } catch (error) {
      console.error('Rule validation error:', error);
      setValidation({
        isValid: false,
        errors: [error instanceof Error ? error.message : '알 수 없는 오류가 발생했습니다.'],
      });
    } finally {
      setIsValidating(false);
    }
  }, []);

  // Trigger validation when debounced rule changes
  useEffect(() => {
    if (!enabled) return;

    validateRule(debouncedRule);
  }, [debouncedRule, validateRule, enabled]);

  return {
    ...validation,
    isValidating,
  };
}
