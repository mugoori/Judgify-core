/**
 * 판정 결과 카드 컴포넌트
 */
import React from 'react';
import { JudgmentData } from '../../../lib/mock-data/types';

interface JudgmentResultProps {
  data: JudgmentData;
  stepLabel?: string;
}

export const JudgmentResult: React.FC<JudgmentResultProps> = ({ data, stepLabel }) => {
  const isPass = data.result;
  const confidencePercent = Math.round(data.confidence * 100);

  return (
    <div
      className={`border rounded-lg p-4 mb-3 ${
        isPass
          ? 'bg-green-50 border-green-200'
          : 'bg-red-50 border-red-200'
      }`}
    >
      <div className="flex items-center gap-2 mb-3">
        <span className="text-xl">⚖️</span>
        <h4
          className={`font-semibold ${
            isPass ? 'text-green-800' : 'text-red-800'
          }`}
        >
          {stepLabel || '판정'}
        </h4>
      </div>

      {/* 판정 결과 배지 */}
      <div className="flex justify-center mb-4">
        <div
          className={`inline-flex items-center gap-2 px-6 py-3 rounded-lg text-lg font-bold ${
            isPass
              ? 'bg-green-100 text-green-700 border border-green-300'
              : 'bg-red-100 text-red-700 border border-red-300'
          }`}
        >
          <span className="text-2xl">{isPass ? '✅' : '❌'}</span>
          <span>{isPass ? '적합' : '부적합'}</span>
          <span className="text-sm font-normal ml-2">
            (신뢰도 {confidencePercent}%)
          </span>
        </div>
      </div>

      {/* 판정 상세 */}
      <div className="space-y-2 text-sm">
        <div className="flex justify-between items-center">
          <span className="text-gray-600">판정 방법</span>
          <span
            className={`font-medium px-2 py-0.5 rounded ${
              data.method === 'Rule Engine'
                ? 'bg-blue-100 text-blue-700'
                : data.method === 'LLM'
                ? 'bg-purple-100 text-purple-700'
                : 'bg-orange-100 text-orange-700'
            }`}
          >
            {data.method}
          </span>
        </div>

        {/* 신뢰도 바 */}
        <div className="mt-3">
          <div className="flex justify-between text-xs text-gray-500 mb-1">
            <span>신뢰도</span>
            <span>{confidencePercent}%</span>
          </div>
          <div className="w-full bg-gray-200 rounded-full h-2">
            <div
              className={`h-2 rounded-full transition-all ${
                isPass ? 'bg-green-500' : 'bg-red-500'
              }`}
              style={{ width: `${confidencePercent}%` }}
            />
          </div>
        </div>

        {/* 근거 설명 */}
        <div className="mt-3 pt-3 border-t border-gray-200">
          <div className="text-gray-600 mb-1">판정 근거</div>
          <div className="bg-white rounded p-2 text-gray-700">
            {data.explanation}
          </div>
        </div>
      </div>
    </div>
  );
};
