/**
 * 트리거 결과 카드 컴포넌트
 */
import React from 'react';
import { TriggerData } from '../../../lib/mock-data/types';

interface TriggerResultProps {
  data: TriggerData;
  stepLabel?: string;
}

const TRIGGER_TYPE_LABELS: Record<string, string> = {
  event: '이벤트',
  schedule: '스케줄',
  manual: '수동',
};

export const TriggerResult: React.FC<TriggerResultProps> = ({ data, stepLabel }) => {
  return (
    <div className="bg-blue-50 border border-blue-200 rounded-lg p-4 mb-3">
      <div className="flex items-center gap-2 mb-3">
        <span className="text-xl">🎯</span>
        <h4 className="font-semibold text-blue-800">
          {stepLabel || '트리거'}: {data.eventName}
        </h4>
      </div>

      <div className="space-y-2 text-sm">
        <div className="flex justify-between">
          <span className="text-gray-600">발생 시각</span>
          <span className="font-medium">{data.timestamp}</span>
        </div>
        <div className="flex justify-between">
          <span className="text-gray-600">트리거 유형</span>
          <span className="font-medium">{TRIGGER_TYPE_LABELS[data.type] || data.type}</span>
        </div>
        <div className="flex justify-between">
          <span className="text-gray-600">대상</span>
          <span className="font-medium">{data.target}</span>
        </div>

        {data.data && Object.keys(data.data).length > 0 && (
          <div className="mt-3 pt-3 border-t border-blue-100">
            <div className="text-gray-600 mb-2">추가 정보</div>
            <div className="grid grid-cols-2 gap-2">
              {Object.entries(data.data).map(([key, value]) => (
                <div key={key} className="flex justify-between bg-white px-2 py-1 rounded">
                  <span className="text-gray-500">{key}</span>
                  <span className="font-medium">{value}</span>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
