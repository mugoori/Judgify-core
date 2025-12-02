/**
 * 계산 결과 카드 컴포넌트
 */
import React from 'react';
import { CalcData } from '../../../lib/mock-data/types';

interface CalcResultProps {
  data: CalcData;
  stepLabel?: string;
}

export const CalcResult: React.FC<CalcResultProps> = ({ data, stepLabel }) => {
  return (
    <div className="bg-purple-50 border border-purple-200 rounded-lg p-4 mb-3">
      <div className="flex items-center gap-2 mb-3">
        <span className="text-xl">🔢</span>
        <h4 className="font-semibold text-purple-800">
          {stepLabel || '계산'}
        </h4>
      </div>

      <div className="space-y-3">
        {/* 수식 */}
        <div className="bg-white rounded p-3 border border-purple-100">
          <div className="text-xs text-gray-500 mb-1">수식</div>
          <div className="font-mono text-sm text-purple-700">{data.formula}</div>
        </div>

        {/* 입력값 */}
        <div className="grid grid-cols-2 gap-2">
          {Object.entries(data.inputs).map(([key, value]) => (
            <div key={key} className="bg-white rounded p-2 border border-purple-100">
              <div className="text-xs text-gray-500">{key}</div>
              <div className="font-medium text-purple-700">
                {typeof value === 'number' ? value.toLocaleString() : value}
              </div>
            </div>
          ))}
        </div>

        {/* 결과 */}
        <div className="bg-purple-100 rounded-lg p-3 text-center">
          <div className="text-xs text-purple-600 mb-1">계산 결과</div>
          <div className="text-2xl font-bold text-purple-800">
            {typeof data.result === 'number' ? data.result.toLocaleString() : data.result}
            <span className="text-sm font-normal ml-1">{data.unit}</span>
          </div>
        </div>

        {/* 설명 */}
        <div className="text-sm text-gray-600 text-center">
          {data.description}
        </div>
      </div>
    </div>
  );
};
