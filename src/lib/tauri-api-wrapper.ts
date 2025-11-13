/**
 * Tauri API Wrapper with Automatic Environment Detection
 *
 * 환경에 따라 자동으로 Real Tauri API 또는 Mock API를 선택합니다.
 * - Tauri WebView: 실제 Rust 백엔드 API 사용
 * - 웹 브라우저: Mock API 사용 (개발용)
 *
 * 사용법:
 * ```typescript
 * import { api } from '@/lib/tauri-api-wrapper';
 *
 * // 환경에 맞는 API가 자동으로 선택됨
 * const stats = await api.getSystemStats();
 * ```
 */

import { isTauri } from './environment';
import * as TauriAPI from './tauri-api';
import * as MockAPI from './mock-api';

/**
 * 환경 기반 API 자동 선택 Proxy
 *
 * Tauri 환경이면 실제 API, 웹 브라우저면 Mock API 반환
 */
export const api = new Proxy({} as typeof TauriAPI, {
  get(target, prop: string) {
    if (isTauri()) {
      // Tauri 환경: 실제 백엔드 API 사용
      return (TauriAPI as any)[prop];
    } else {
      // 웹 브라우저: Mock API 사용
      if ((MockAPI as any)[prop]) {
        return (MockAPI as any)[prop];
      }

      // Mock에 없는 함수 요청시 경고
      console.warn(
        `[API Wrapper] Mock API for "${prop}" not implemented. ` +
        `Consider adding it to mock-api.ts for better development experience.`
      );

      // 기본 Promise 반환 (에러 대신)
      return async (...args: any[]) => {
        console.error(`[API Wrapper] Function "${prop}" not available in web browser mode.`);
        throw new Error(
          `API function "${prop}" requires Tauri environment. ` +
          `Please run "npm run tauri:dev" instead of "npm run dev".`
        );
      };
    }
  },
});

/**
 * 개별 함수 Export (타입 안전성 유지)
 *
 * 아래 export를 사용하면 IDE 자동완성 및 타입 체크가 작동합니다.
 */

// Judgment API
export const executeJudgment = api.executeJudgment;
export const getJudgmentHistory = api.getJudgmentHistory;

// Learning API
export const saveFeedback = api.saveFeedback;
export const getFewShotSamples = api.getFewShotSamples;
export const extractRules = api.extractRules;

// BI API
export const generateBiInsight = api.generateBiInsight;

// Chat API
export const sendChatMessage = api.sendChatMessage;
export const getChatHistory = api.getChatHistory;
export const testClaudeApi = api.testClaudeApi;

// Workflow API
export const createWorkflow = api.createWorkflow;
export const getWorkflow = api.getWorkflow;
export const getAllWorkflows = api.getAllWorkflows;
export const updateWorkflow = api.updateWorkflow;
export const deleteWorkflow = api.deleteWorkflow;
export const validateWorkflow = api.validateWorkflow;

// System API
export const getSystemStatus = api.getSystemStatus;
export const getSystemStats = api.getSystemStats;
export const getDataDirectory = api.getDataDirectory;
export const exportDatabase = api.exportDatabase;

// Token Metrics API
export const getTokenMetrics = api.getTokenMetrics;

// Type re-exports (타입 정의도 함께 export)
export type {
  ExecuteJudgmentRequest,
  JudgmentResult,
  SaveFeedbackRequest,
  BiInsightResponse,
  ChatMessageRequest,
  ChatMessageResponse,
  CreateWorkflowRequest,
  WorkflowResponse,
  SystemStatus,
  SystemStats,
  TokenMetrics,
} from './tauri-api';
