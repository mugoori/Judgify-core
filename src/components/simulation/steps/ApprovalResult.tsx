/**
 * 승인 결과 카드 컴포넌트 (인터랙티브 버튼)
 */
import React, { useState } from 'react';
import { ApprovalData } from '../../../lib/mock-data/types';

interface ApprovalResultProps {
  data: ApprovalData;
  stepLabel?: string;
  onApprove?: () => void;
}

export const ApprovalResult: React.FC<ApprovalResultProps> = ({
  data,
  stepLabel,
  onApprove,
}) => {
  const [status, setStatus] = useState<'pending' | 'approved' | 'rejected'>(
    data.status
  );
  const [approvedAt, setApprovedAt] = useState<string | null>(data.approvedAt);

  const handleApprove = () => {
    const now = new Date().toISOString().replace('T', ' ').substring(0, 19);
    setStatus('approved');
    setApprovedAt(now);
    onApprove?.();
  };

  return (
    <div
      className={`border rounded-lg p-4 mb-3 ${
        status === 'approved'
          ? 'bg-green-50 border-green-200'
          : status === 'rejected'
          ? 'bg-red-50 border-red-200'
          : 'bg-amber-50 border-amber-200'
      }`}
    >
      <div className="flex items-center gap-2 mb-3">
        <span className="text-xl">✍️</span>
        <h4
          className={`font-semibold ${
            status === 'approved'
              ? 'text-green-800'
              : status === 'rejected'
              ? 'text-red-800'
              : 'text-amber-800'
          }`}
        >
          {stepLabel || '승인'}
        </h4>
      </div>

      <div className="space-y-3">
        {/* 승인자 정보 */}
        <div className="flex justify-between items-center text-sm">
          <span className="text-gray-600">승인자</span>
          <span className="font-medium">
            {data.approver} ({data.role})
          </span>
        </div>

        {/* 승인 버튼 또는 상태 */}
        <div className="flex justify-center py-2">
          {status === 'pending' ? (
            <button
              onClick={handleApprove}
              className="px-6 py-3 bg-amber-500 hover:bg-amber-600 text-white font-semibold rounded-lg transition-colors flex items-center gap-2 shadow-md"
            >
              <span>🖊️</span>
              <span>승인하기</span>
            </button>
          ) : status === 'approved' ? (
            <div className="inline-flex items-center gap-2 px-6 py-3 bg-green-100 text-green-700 font-semibold rounded-lg border border-green-300">
              <span className="text-xl">✅</span>
              <span>승인됨</span>
            </div>
          ) : (
            <div className="inline-flex items-center gap-2 px-6 py-3 bg-red-100 text-red-700 font-semibold rounded-lg border border-red-300">
              <span className="text-xl">❌</span>
              <span>반려됨</span>
            </div>
          )}
        </div>

        {/* 승인 시각 */}
        {approvedAt && (
          <div className="flex justify-between items-center text-sm">
            <span className="text-gray-600">승인 시각</span>
            <span className="font-medium text-green-700">{approvedAt}</span>
          </div>
        )}

        {/* 코멘트 */}
        {data.comment && (
          <div className="mt-2 pt-2 border-t border-gray-200">
            <div className="text-xs text-gray-500 mb-1">코멘트</div>
            <div className="text-sm text-gray-700 bg-white rounded p-2">
              {data.comment}
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
