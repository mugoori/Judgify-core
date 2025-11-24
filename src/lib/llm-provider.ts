/**
 * LLM Provider Abstraction Layer (Week 5 Day 3)
 *
 * Allows easy swapping of LLM providers (Claude, OpenAI, Local LLM, etc.)
 * Interface-based design for maximum flexibility
 */

export interface WorkflowGenerationRequest {
  description: string;
  context?: {
    existingNodes?: string[];
    industry?: string;
    complexity?: 'simple' | 'medium' | 'complex';
  };
}

export interface WorkflowGenerationResponse {
  nodes: Array<{
    id: string;
    type: string;
    label?: string;
    config?: Record<string, any>;
    position?: { x: number; y: number };
    data?: {
      label: string;
      description?: string;
      config?: Record<string, any>;
    };
  }>;
  edges: Array<{
    id: string;
    source: string;
    target: string;
    sourceHandle?: string;
    targetHandle?: string;
  }>;
  metadata?: {
    provider: string;
    model: string;
    confidence: number;
    generationTime: number;
  };
}

export interface LLMProviderConfig {
  apiKey: string;
  model?: string;
  maxTokens?: number;
  temperature?: number;
}

export interface LLMProvider {
  readonly name: string;
  readonly defaultModel: string;

  /**
   * Generate workflow from natural language description
   */
  generateWorkflow(
    request: WorkflowGenerationRequest,
    config: LLMProviderConfig
  ): Promise<WorkflowGenerationResponse>;

  /**
   * Validate API key format (before API call)
   */
  validateApiKey(apiKey: string): boolean;

  /**
   * Get provider-specific error message
   */
  getErrorMessage(error: unknown): string;
}

export class LLMProviderError extends Error {
  constructor(
    message: string,
    public readonly provider: string,
    public readonly statusCode?: number,
    public readonly originalError?: unknown
  ) {
    super(message);
    this.name = 'LLMProviderError';
  }
}
