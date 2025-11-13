/**
 * Tauri 환경 감지 유틸리티
 *
 * 웹 브라우저와 Tauri WebView 환경을 구분하여
 * 적절한 API (Mock vs Real)를 선택하도록 지원합니다.
 */

/**
 * 현재 환경이 Tauri WebView인지 확인
 *
 * @returns {boolean} Tauri 환경이면 true, 웹 브라우저면 false
 *
 * @example
 * if (isTauri()) {
 *   // Tauri API 사용
 *   await invoke('get_system_stats');
 * } else {
 *   // Mock API 사용
 *   return mockSystemStats;
 * }
 */
export const isTauri = (): boolean => {
  // 프로덕션 빌드는 항상 Tauri 환경으로 간주
  if (import.meta.env.PROD) {
    return true;
  }

  // 개발 환경: window.__TAURI__ 존재 여부로 판단
  return typeof window !== 'undefined' && '__TAURI__' in window;
};

/**
 * 현재 환경 정보 출력 (디버깅용)
 *
 * @example
 * logEnvironmentInfo();
 * // 출력: [Environment] Tauri: true, Mode: development
 */
export const logEnvironmentInfo = (): void => {
  console.log(`[Environment] Tauri: ${isTauri()}, Mode: ${import.meta.env.MODE}`);
};
