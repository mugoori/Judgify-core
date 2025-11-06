/**
 * Claude (Anthropic) LLM Provider Implementation (Week 5 Day 3)
 */

import Anthropic from '@anthropic-ai/sdk';
import type {
  LLMProvider,
  LLMProviderConfig,
  WorkflowGenerationRequest,
  WorkflowGenerationResponse,
} from './llm-provider';
import { LLMProviderError } from './llm-provider';

export class ClaudeProvider implements LLMProvider {
  readonly name = 'Claude (Anthropic)';
  readonly defaultModel = 'claude-3-5-sonnet-20241022';

  private client: Anthropic | null = null;

  /**
   * Initialize Anthropic client (lazy)
   */
  private getClient(apiKey: string): Anthropic {
    if (!this.client || (this.client as any).apiKey !== apiKey) {
      this.client = new Anthropic({ apiKey, dangerouslyAllowBrowser: true });
    }
    return this.client;
  }

  validateApiKey(apiKey: string): boolean {
    // Claude API keys start with "sk-ant-"
    return /^sk-ant-[a-zA-Z0-9\-_]{95,}$/.test(apiKey);
  }

  async generateWorkflow(
    request: WorkflowGenerationRequest,
    config: LLMProviderConfig
  ): Promise<WorkflowGenerationResponse> {
    const startTime = Date.now();

    if (!this.validateApiKey(config.apiKey)) {
      throw new LLMProviderError(
        'Invalid Claude API key format. Expected: sk-ant-...',
        this.name,
        400
      );
    }

    try {
      const client = this.getClient(config.apiKey);
      const response = await client.messages.create({
        model: config.model || this.defaultModel,
        max_tokens: config.maxTokens || 4096,
        temperature: config.temperature || 0.7,
        messages: [
          {
            role: 'user',
            content: this.buildPrompt(request),
          },
        ],
      });

      const content = response.content[0];
      if (content.type !== 'text') {
        throw new Error('Unexpected response type from Claude');
      }

      const workflow = this.parseWorkflow(content.text);
      const generationTime = Date.now() - startTime;

      return {
        ...workflow,
        metadata: {
          provider: this.name,
          model: response.model,
          confidence: 0.85, // Claude doesn't provide confidence scores
          generationTime,
        },
      };
    } catch (error) {
      throw new LLMProviderError(
        this.getErrorMessage(error),
        this.name,
        (error as any).status,
        error
      );
    }
  }

  private buildPrompt(request: WorkflowGenerationRequest): string {
    const { description, context } = request;

    let prompt = `Generate a workflow JSON for the following requirement:

Description: ${description}
`;

    if (context?.industry) {
      prompt += `Industry: ${context.industry}\n`;
    }

    if (context?.complexity) {
      prompt += `Complexity: ${context.complexity}\n`;
    }

    prompt += `
IMPORTANT: Return ONLY valid JSON in this exact format:
{
  "nodes": [
    {
      "id": "node-1",
      "type": "data-input",
      "label": "Node Label",
      "config": {},
      "position": { "x": 100, "y": 100 }
    }
  ],
  "edges": [
    {
      "id": "edge-1",
      "source": "node-1",
      "target": "node-2"
    }
  ]
}

Available node types: data-input, condition, action, notification, data-output

Rules:
1. First node must be data-input
2. Last node must be data-output
3. All nodes must be connected
4. Use descriptive labels
5. Position nodes left-to-right (increment x by 250)
`;

    return prompt;
  }

  private parseWorkflow(
    text: string
  ): Omit<WorkflowGenerationResponse, 'metadata'> {
    // Extract JSON from markdown code blocks if present
    const jsonMatch = text.match(/```(?:json)?\s*([\s\S]*?)```/);
    const jsonText = jsonMatch ? jsonMatch[1] : text;

    try {
      const parsed = JSON.parse(jsonText.trim());

      // Validate structure
      if (!Array.isArray(parsed.nodes) || !Array.isArray(parsed.edges)) {
        throw new Error(
          'Invalid workflow structure: missing nodes or edges array'
        );
      }

      // Validate required fields
      parsed.nodes.forEach((node: any, idx: number) => {
        if (!node.id || !node.type || !node.label) {
          throw new Error(
            `Node ${idx} missing required fields (id, type, label)`
          );
        }
      });

      return {
        nodes: parsed.nodes,
        edges: parsed.edges,
      };
    } catch (error) {
      throw new Error(
        `Failed to parse workflow JSON: ${(error as Error).message}`
      );
    }
  }

  getErrorMessage(error: unknown): string {
    if (error instanceof Anthropic.APIError) {
      switch (error.status) {
        case 401:
          return 'Invalid Claude API key. Please check your Settings.';
        case 429:
          return 'Claude API rate limit exceeded. Please try again later.';
        case 500:
          return 'Claude API server error. Please try again.';
        default:
          return `Claude API error: ${error.message}`;
      }
    }
    return `Unexpected error: ${(error as Error).message}`;
  }
}
