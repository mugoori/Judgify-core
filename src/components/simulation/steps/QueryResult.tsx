/**
 * 조회 결과 카드 컴포넌트 (테이블 형태)
 */
import React from 'react';
import { QueryData } from '../../../lib/mock-data/types';

interface QueryResultProps {
  data: QueryData;
  stepLabel?: string;
}

export const QueryResult: React.FC<QueryResultProps> = ({ data, stepLabel }) => {
  return (
    <div className="bg-indigo-50 border border-indigo-200 rounded-lg p-4 mb-3">
      <div className="flex items-center gap-2 mb-3">
        <span className="text-xl">📊</span>
        <h4 className="font-semibold text-indigo-800">
          {stepLabel || '조회'}: {data.tableName}
        </h4>
        <span className="text-xs bg-indigo-100 text-indigo-600 px-2 py-0.5 rounded">
          {data.totalCount}건
        </span>
      </div>

      <div className="overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-indigo-200">
              <th className="text-left py-2 px-2 text-gray-600 font-medium">항목</th>
              <th className="text-left py-2 px-2 text-gray-600 font-medium">기준값</th>
              <th className="text-left py-2 px-2 text-gray-600 font-medium">측정값</th>
              <th className="text-center py-2 px-2 text-gray-600 font-medium">결과</th>
            </tr>
          </thead>
          <tbody>
            {data.results.map((row, index) => (
              <tr
                key={index}
                className={`border-b border-indigo-100 ${
                  !row.pass ? 'bg-red-50' : ''
                }`}
              >
                <td className="py-2 px-2 font-medium">{row.item}</td>
                <td className="py-2 px-2 text-gray-600">{row.standard}</td>
                <td className="py-2 px-2">{row.actual}</td>
                <td className="py-2 px-2 text-center">
                  {row.pass ? (
                    <span className="text-green-600 text-lg">✅</span>
                  ) : (
                    <span className="text-red-600 text-lg">❌</span>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
};
