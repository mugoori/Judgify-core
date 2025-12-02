/**
 * Mock 데이터 생성기 통합 모듈
 */
import { StepMockData, TemplateId } from '../types';
import { generatePasteurizationCcpMock } from './templates/pasteurization-ccp';
import { generateMetalDetectionCcpMock } from './templates/metal-detection-ccp';
import { generateMaterialInspectionMock } from './templates/material-inspection';
import { generateReleaseApprovalMock } from './templates/release-approval';
import { generateShelfLifeMonitoringMock } from './templates/shelf-life-monitoring';
import { generateInventoryForecastMock } from './templates/inventory-forecast';
import { generatePreventiveMaintenanceMock } from './templates/preventive-maintenance';
import { generateMrpCalculationMock } from './templates/mrp-calculation';
import { generateBomCostManagementMock } from './templates/bom-cost-management';
import { generateProductionForecastMock } from './templates/production-forecast';

// 템플릿 ID → 생성기 매핑
const GENERATORS: Record<TemplateId, () => StepMockData[]> = {
  'pasteurization-ccp': generatePasteurizationCcpMock,
  'metal-detection-ccp': generateMetalDetectionCcpMock,
  'material-inspection': generateMaterialInspectionMock,
  'release-approval': generateReleaseApprovalMock,
  'shelf-life-monitoring': generateShelfLifeMonitoringMock,
  'inventory-forecast': generateInventoryForecastMock,
  'preventive-maintenance': generatePreventiveMaintenanceMock,
  'mrp-calculation': generateMrpCalculationMock,
  'bom-cost-management': generateBomCostManagementMock,
  'production-forecast': generateProductionForecastMock,
};

// 템플릿 ID → 한글명 매핑
export const TEMPLATE_NAMES: Record<TemplateId, string> = {
  'pasteurization-ccp': '살균온도 CCP',
  'metal-detection-ccp': '금속검출 CCP',
  'material-inspection': '원료입고검사',
  'release-approval': '출하승인',
  'shelf-life-monitoring': '유통기한 모니터링',
  'inventory-forecast': '재고예측',
  'preventive-maintenance': '예방정비',
  'mrp-calculation': 'MRP 계산',
  'bom-cost-management': 'BOM 원가관리',
  'production-forecast': '생산예측',
};

/**
 * 템플릿 ID에 해당하는 Mock 데이터 생성
 */
export function generateMockDataForTemplate(templateId: string): StepMockData[] {
  const generator = GENERATORS[templateId as TemplateId];

  if (!generator) {
    console.warn(`Unknown template ID: ${templateId}, using default mock data`);
    return generateDefaultMock();
  }

  return generator();
}

/**
 * 기본 Mock 데이터 (알 수 없는 템플릿용)
 */
function generateDefaultMock(): StepMockData[] {
  return [
    {
      trigger: {
        type: 'manual',
        eventName: '수동 실행',
        timestamp: new Date().toISOString().replace('T', ' ').substring(0, 19),
        target: '시스템',
      },
    },
    {
      query: {
        tableName: '기본 조회',
        results: [
          { item: '항목 1', standard: '기준값', actual: '측정값', pass: true },
          { item: '항목 2', standard: '기준값', actual: '측정값', pass: true },
        ],
        totalCount: 2,
      },
    },
    {
      judgment: {
        result: true,
        method: 'Rule Engine',
        confidence: 0.95,
        explanation: '시뮬레이션 판정 결과입니다.',
      },
    },
    {
      alert: {
        sent: true,
        recipients: ['관리자'],
        channel: 'system',
        sentAt: new Date().toISOString().replace('T', ' ').substring(0, 19),
        message: '시뮬레이션 완료',
      },
    },
  ];
}

/**
 * 단계 타입에 맞는 Mock 데이터 추출
 */
export function getMockDataForStep(
  stepType: string,
  stepIndex: number,
  allMockData: StepMockData[]
): StepMockData | undefined {
  // 단계 인덱스에 해당하는 mock 데이터 반환
  // 또는 단계 타입에 맞는 첫 번째 mock 데이터 반환
  const mockAtIndex = allMockData[stepIndex];
  if (mockAtIndex) {
    return mockAtIndex;
  }

  // 타입별 매칭
  const typeKey = stepType.toLowerCase();
  return allMockData.find(m => {
    if (typeKey === 'trigger' && m.trigger) return true;
    if (typeKey === 'query' && m.query) return true;
    if (typeKey === 'calc' && m.calc) return true;
    if (typeKey === 'judgment' && m.judgment) return true;
    if (typeKey === 'approval' && m.approval) return true;
    if (typeKey === 'alert' && m.alert) return true;
    return false;
  });
}

// Re-export common utilities
export * from './common';
