/**
 * Unit tests for ClaudeProvider (Week 5 Day 3)
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { ClaudeProvider } from '../claude-provider';
import type { WorkflowGenerationRequest } from '../llm-provider';
import Anthropic from '@anthropic-ai/sdk';

vi.mock('@anthropic-ai/sdk');

describe('ClaudeProvider', () => {
  let provider: ClaudeProvider;
  let mockClient: any;

  beforeEach(() => {
    provider = new ClaudeProvider();
    mockClient = {
      messages: {
        create: vi.fn(),
      },
    };

    // Mock Anthropic constructor
    (Anthropic as any).mockImplementation(() => mockClient);
  });

  describe('validateApiKey', () => {
    it('accepts valid Claude API key format', () => {
      const validKey = 'sk-ant-' + 'a'.repeat(95);
      expect(provider.validateApiKey(validKey)).toBe(true);
    });

    it('rejects invalid API key format', () => {
      expect(provider.validateApiKey('invalid-key')).toBe(false);
      expect(provider.validateApiKey('sk-ant-short')).toBe(false);
      expect(provider.validateApiKey('')).toBe(false);
    });
  });

  describe('generateWorkflow', () => {
    const mockRequest: WorkflowGenerationRequest = {
      description: 'Quality check workflow for manufacturing',
      context: {
        industry: 'manufacturing',
        complexity: 'simple',
      },
    };

    const mockConfig = {
      apiKey: 'sk-ant-' + 'a'.repeat(95),
      model: 'claude-3-5-sonnet-20241022',
      maxTokens: 4096,
      temperature: 0.7,
    };

    it('generates valid workflow from Claude response', async () => {
      mockClient.messages.create.mockResolvedValue({
        model: 'claude-3-5-sonnet-20241022',
        content: [
          {
            type: 'text',
            text: JSON.stringify({
              nodes: [
                { id: 'node-1', type: 'data-input', label: 'Input', config: {} },
              ],
              edges: [],
            }),
          },
        ],
      });

      const result = await provider.generateWorkflow(mockRequest, mockConfig);

      expect(result.nodes).toHaveLength(1);
      expect(result.metadata?.provider).toBe('Claude (Anthropic)');
      expect(result.metadata?.generationTime).toBeGreaterThan(0);
    });

    it('extracts JSON from markdown code blocks', async () => {
      mockClient.messages.create.mockResolvedValue({
        model: 'claude-3-5-sonnet-20241022',
        content: [
          {
            type: 'text',
            text:
              '```json\n' +
              JSON.stringify({
                nodes: [
                  {
                    id: 'node-1',
                    type: 'data-input',
                    label: 'Input',
                    config: {},
                  },
                ],
                edges: [],
              }) +
              '\n```',
          },
        ],
      });

      const result = await provider.generateWorkflow(mockRequest, mockConfig);
      expect(result.nodes).toHaveLength(1);
    });

    it('throws error for invalid API key', async () => {
      const invalidConfig = { ...mockConfig, apiKey: 'invalid-key' };

      await expect(
        provider.generateWorkflow(mockRequest, invalidConfig)
      ).rejects.toThrow('Invalid Claude API key format');
    });

    it('throws error for malformed JSON response', async () => {
      mockClient.messages.create.mockResolvedValue({
        model: 'claude-3-5-sonnet-20241022',
        content: [
          {
            type: 'text',
            text: 'Not valid JSON',
          },
        ],
      });

      await expect(
        provider.generateWorkflow(mockRequest, mockConfig)
      ).rejects.toThrow('Failed to parse workflow JSON');
    });

    it('throws error for missing required node fields', async () => {
      mockClient.messages.create.mockResolvedValue({
        model: 'claude-3-5-sonnet-20241022',
        content: [
          {
            type: 'text',
            text: JSON.stringify({
              nodes: [
                { id: 'node-1' }, // Missing type and label
              ],
              edges: [],
            }),
          },
        ],
      });

      await expect(
        provider.generateWorkflow(mockRequest, mockConfig)
      ).rejects.toThrow('missing required fields');
    });

    it('handles Claude API errors (401 Unauthorized)', async () => {
      const apiError = new Anthropic.APIError(
        401,
        {},
        'Unauthorized',
        {} as any
      );
      mockClient.messages.create.mockRejectedValue(apiError);

      await expect(
        provider.generateWorkflow(mockRequest, mockConfig)
      ).rejects.toThrow('Invalid Claude API key');
    });

    it('handles Claude API errors (429 Rate Limit)', async () => {
      const apiError = new Anthropic.APIError(
        429,
        {},
        'Rate limited',
        {} as any
      );
      mockClient.messages.create.mockRejectedValue(apiError);

      await expect(
        provider.generateWorkflow(mockRequest, mockConfig)
      ).rejects.toThrow('rate limit exceeded');
    });
  });

  describe('getErrorMessage', () => {
    it('returns user-friendly message for 401 error', () => {
      const error = new Anthropic.APIError(401, {}, 'Unauthorized', {} as any);
      expect(provider.getErrorMessage(error)).toContain(
        'Invalid Claude API key'
      );
    });

    it('returns generic message for unknown errors', () => {
      const error = new Error('Unknown error');
      expect(provider.getErrorMessage(error)).toContain('Unexpected error');
    });
  });
});
