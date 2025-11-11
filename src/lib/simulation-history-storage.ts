/**
 * 시뮬레이션 히스토리 영구 저장 시스템 (IndexedDB)
 *
 * Week 6 Task 4: 시뮬레이션 패널을 닫아도 히스토리가 유지되며,
 * 과거 실행 기록을 재확인하고 재실행할 수 있습니다.
 */

import type { SimulationState, SimulationStep } from './workflow-simulator';

export interface SimulationHistory {
  id: string;                     // UUID
  workflow_id: string;             // 워크플로우 식별자
  workflow_name: string;           // 워크플로우 이름 (UI 표시용)
  timestamp: number;               // 실행 시작 시간 (Unix timestamp)
  initial_data: Record<string, any>; // 초기 입력 데이터
  steps: SimulationStep[];         // 실행 단계 목록
  final_state: SimulationState;    // 최종 상태
  duration_ms: number;             // 전체 실행 시간
  status: 'success' | 'error' | 'partial'; // 실행 결과
}

const DB_NAME = 'judgify_simulation_history';
const STORE_NAME = 'histories';
const DB_VERSION = 1;
const MAX_HISTORY_COUNT = 50;    // 최대 저장 개수

export class SimulationHistoryStorage {
  private db: IDBDatabase | null = null;

  /**
   * IndexedDB 초기화 및 연결
   */
  async initialize(): Promise<void> {
    return new Promise((resolve, reject) => {
      const request = indexedDB.open(DB_NAME, DB_VERSION);

      request.onerror = () => {
        console.error('[HistoryStorage] Failed to open DB:', request.error);
        reject(request.error);
      };

      request.onsuccess = () => {
        this.db = request.result;
        console.log('[HistoryStorage] DB opened successfully');
        resolve();
      };

      request.onupgradeneeded = (event) => {
        const db = (event.target as IDBOpenDBRequest).result;

        if (!db.objectStoreNames.contains(STORE_NAME)) {
          const objectStore = db.createObjectStore(STORE_NAME, { keyPath: 'id' });

          // 인덱스 생성 (검색 최적화)
          objectStore.createIndex('timestamp', 'timestamp', { unique: false });
          objectStore.createIndex('workflow_id', 'workflow_id', { unique: false });
          objectStore.createIndex('status', 'status', { unique: false });

          console.log('[HistoryStorage] Object store created');
        }
      };
    });
  }

  /**
   * 시뮬레이션 히스토리 저장
   */
  async saveHistory(history: SimulationHistory): Promise<void> {
    if (!this.db) {
      await this.initialize();
    }

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([STORE_NAME], 'readwrite');
      const objectStore = transaction.objectStore(STORE_NAME);
      const request = objectStore.add(history);

      request.onsuccess = () => {
        console.log('[HistoryStorage] History saved:', history.id);

        // 저장 후 개수 제한 확인
        this.clearOldHistory(MAX_HISTORY_COUNT).catch(console.error);
        resolve();
      };

      request.onerror = () => {
        console.error('[HistoryStorage] Failed to save:', request.error);
        reject(request.error);
      };
    });
  }

  /**
   * 히스토리 목록 조회 (최신순)
   */
  async getHistoryList(limit: number = 20): Promise<SimulationHistory[]> {
    if (!this.db) {
      await this.initialize();
    }

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([STORE_NAME], 'readonly');
      const objectStore = transaction.objectStore(STORE_NAME);
      const index = objectStore.index('timestamp');

      // 최신순으로 정렬 (내림차순)
      const request = index.openCursor(null, 'prev');
      const histories: SimulationHistory[] = [];
      let count = 0;

      request.onsuccess = (event) => {
        const cursor = (event.target as IDBRequest).result;

        if (cursor && count < limit) {
          histories.push(cursor.value);
          count++;
          cursor.continue();
        } else {
          resolve(histories);
        }
      };

      request.onerror = () => {
        console.error('[HistoryStorage] Failed to get list:', request.error);
        reject(request.error);
      };
    });
  }

  /**
   * 특정 히스토리 조회
   */
  async getHistoryById(id: string): Promise<SimulationHistory | null> {
    if (!this.db) {
      await this.initialize();
    }

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([STORE_NAME], 'readonly');
      const objectStore = transaction.objectStore(STORE_NAME);
      const request = objectStore.get(id);

      request.onsuccess = () => {
        resolve(request.result || null);
      };

      request.onerror = () => {
        console.error('[HistoryStorage] Failed to get by ID:', request.error);
        reject(request.error);
      };
    });
  }

  /**
   * 히스토리 삭제
   */
  async deleteHistory(id: string): Promise<void> {
    if (!this.db) {
      await this.initialize();
    }

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([STORE_NAME], 'readwrite');
      const objectStore = transaction.objectStore(STORE_NAME);
      const request = objectStore.delete(id);

      request.onsuccess = () => {
        console.log('[HistoryStorage] History deleted:', id);
        resolve();
      };

      request.onerror = () => {
        console.error('[HistoryStorage] Failed to delete:', request.error);
        reject(request.error);
      };
    });
  }

  /**
   * 오래된 히스토리 자동 삭제 (최대 개수 유지)
   */
  async clearOldHistory(keepCount: number = 50): Promise<void> {
    if (!this.db) {
      await this.initialize();
    }

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([STORE_NAME], 'readwrite');
      const objectStore = transaction.objectStore(STORE_NAME);
      const index = objectStore.index('timestamp');

      // 오래된 순으로 정렬 (오름차순)
      const request = index.openCursor(null, 'next');
      const toDelete: string[] = [];
      let count = 0;

      // 먼저 전체 개수 확인
      const countRequest = objectStore.count();

      countRequest.onsuccess = () => {
        const totalCount = countRequest.result;

        if (totalCount <= keepCount) {
          resolve();
          return;
        }

        const deleteCount = totalCount - keepCount;

        request.onsuccess = (event) => {
          const cursor = (event.target as IDBRequest).result;

          if (cursor && count < deleteCount) {
            toDelete.push(cursor.value.id);
            count++;
            cursor.continue();
          } else {
            // 삭제 실행
            toDelete.forEach(id => {
              objectStore.delete(id);
            });

            console.log(`[HistoryStorage] Cleared ${toDelete.length} old histories`);
            resolve();
          }
        };

        request.onerror = () => {
          console.error('[HistoryStorage] Failed to clear old histories:', request.error);
          reject(request.error);
        };
      };

      countRequest.onerror = () => {
        reject(countRequest.error);
      };
    });
  }

  /**
   * 전체 히스토리 삭제 (초기화)
   */
  async clearAll(): Promise<void> {
    if (!this.db) {
      await this.initialize();
    }

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([STORE_NAME], 'readwrite');
      const objectStore = transaction.objectStore(STORE_NAME);
      const request = objectStore.clear();

      request.onsuccess = () => {
        console.log('[HistoryStorage] All histories cleared');
        resolve();
      };

      request.onerror = () => {
        console.error('[HistoryStorage] Failed to clear all:', request.error);
        reject(request.error);
      };
    });
  }

  /**
   * 워크플로우별 히스토리 조회
   */
  async getHistoriesByWorkflow(workflowId: string, limit: number = 10): Promise<SimulationHistory[]> {
    if (!this.db) {
      await this.initialize();
    }

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([STORE_NAME], 'readonly');
      const objectStore = transaction.objectStore(STORE_NAME);
      const index = objectStore.index('workflow_id');

      const request = index.openCursor(IDBKeyRange.only(workflowId));
      const histories: SimulationHistory[] = [];
      let count = 0;

      request.onsuccess = (event) => {
        const cursor = (event.target as IDBRequest).result;

        if (cursor && count < limit) {
          histories.push(cursor.value);
          count++;
          cursor.continue();
        } else {
          // 타임스탬프 기준 최신순 정렬
          histories.sort((a, b) => b.timestamp - a.timestamp);
          resolve(histories);
        }
      };

      request.onerror = () => {
        console.error('[HistoryStorage] Failed to get by workflow:', request.error);
        reject(request.error);
      };
    });
  }

  /**
   * 통계 정보 조회
   */
  async getStatistics(): Promise<{
    total: number;
    success: number;
    error: number;
    partial: number;
  }> {
    if (!this.db) {
      await this.initialize();
    }

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([STORE_NAME], 'readonly');
      const objectStore = transaction.objectStore(STORE_NAME);
      const request = objectStore.getAll();

      request.onsuccess = () => {
        const histories: SimulationHistory[] = request.result;

        const stats = {
          total: histories.length,
          success: histories.filter(h => h.status === 'success').length,
          error: histories.filter(h => h.status === 'error').length,
          partial: histories.filter(h => h.status === 'partial').length,
        };

        resolve(stats);
      };

      request.onerror = () => {
        console.error('[HistoryStorage] Failed to get statistics:', request.error);
        reject(request.error);
      };
    });
  }
}

// 싱글톤 인스턴스
export const historyStorage = new SimulationHistoryStorage();
