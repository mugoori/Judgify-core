/**
 * 알림 결과 카드 컴포넌트
 */
import React from 'react';
import { AlertData } from '../../../lib/mock-data/types';

interface AlertResultProps {
  data: AlertData;
  stepLabel?: string;
  isPending?: boolean; // 승인 대기 중일 때 true
}

const CHANNEL_LABELS: Record<string, { icon: string; label: string }> = {
  email: { icon: '📧', label: '이메일' },
  slack: { icon: '💬', label: 'Slack' },
  sms: { icon: '📱', label: 'SMS' },
  system: { icon: '🔔', label: '시스템' },
};

export const AlertResult: React.FC<AlertResultProps> = ({ data, stepLabel, isPending = false }) => {
  const channel = CHANNEL_LABELS[data.channel] || CHANNEL_LABELS.system;

  return (
    <div className={`rounded-lg p-4 mb-3 ${isPending ? 'bg-gray-50 border border-gray-200' : 'bg-teal-50 border border-teal-200'}`}>
      <div className="flex items-center gap-2 mb-3">
        <span className="text-xl">🔔</span>
        <h4 className={`font-semibold ${isPending ? 'text-gray-600' : 'text-teal-800'}`}>
          {stepLabel || '알림'}
        </h4>
      </div>

      {/* 전송 상태 메시지 */}
      <div className="flex justify-center py-3 mb-3">
        {isPending ? (
          <div className="inline-flex items-center gap-2 px-6 py-3 bg-gray-100 text-gray-500 font-semibold rounded-lg border border-gray-300">
            <span className="text-xl">⏳</span>
            <span>승인 대기 중...</span>
          </div>
        ) : (
          <div className="inline-flex items-center gap-2 px-6 py-3 bg-teal-100 text-teal-700 font-semibold rounded-lg border border-teal-300">
            <span className="text-xl">✅</span>
            <span>알림이 전송되었습니다</span>
          </div>
        )}
      </div>

      <div className="space-y-2 text-sm">
        {/* 수신자 */}
        <div className="flex justify-between items-start">
          <span className="text-gray-600">수신자</span>
          <div className="text-right">
            {data.recipients.map((r, i) => (
              <span
                key={i}
                className={`inline-block px-2 py-0.5 rounded text-xs ml-1 mb-1 ${isPending ? 'bg-gray-100 text-gray-500' : 'bg-teal-100 text-teal-700'}`}
              >
                {r}
              </span>
            ))}
          </div>
        </div>

        {/* 전송 채널 */}
        <div className="flex justify-between items-center">
          <span className="text-gray-600">전송 채널</span>
          <span className="font-medium">
            {channel.icon} {channel.label}
          </span>
        </div>

        {/* 전송 시각 - 대기 중일 때는 숨김 */}
        {!isPending && (
          <div className="flex justify-between items-center">
            <span className="text-gray-600">전송 시각</span>
            <span className="font-medium">{data.sentAt}</span>
          </div>
        )}

        {/* 메시지 내용 */}
        {data.message && (
          <div className={`mt-3 pt-3 ${isPending ? 'border-t border-gray-200' : 'border-t border-teal-200'}`}>
            <div className="text-gray-600 mb-1">메시지</div>
            <div className={`rounded p-2 text-sm ${isPending ? 'bg-gray-100 text-gray-500' : 'bg-white text-gray-700'}`}>
              {data.message}
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
