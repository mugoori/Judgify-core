import React from 'react';
import { X, Copy, Check } from 'lucide-react';
import { useState } from 'react';
import { JsonView } from 'react-json-view-lite';
import 'react-json-view-lite/dist/index.css';

interface JsonViewerModalProps {
  isOpen: boolean;
  onClose: () => void;
  data: any;
  title?: string;
}

export const JsonViewerModal: React.FC<JsonViewerModalProps> = ({
  isOpen,
  onClose,
  data,
  title = 'JSON 데이터'
}) => {
  const [copied, setCopied] = useState(false);

  if (!isOpen) return null;

  const handleCopy = () => {
    navigator.clipboard.writeText(JSON.stringify(data, null, 2));
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const handleBackdropClick = (e: React.MouseEvent) => {
    if (e.target === e.currentTarget) {
      onClose();
    }
  };

  return (
    <div
      className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-[60]"
      onClick={handleBackdropClick}
    >
      <div className="bg-gray-900 rounded-lg w-[80%] max-w-4xl h-[70%] flex flex-col shadow-2xl">
        {/* 헤더 */}
        <div className="flex items-center justify-between p-4 border-b border-gray-700">
          <h3 className="text-lg font-semibold text-white">{title}</h3>
          <div className="flex items-center gap-2">
            <button
              onClick={handleCopy}
              className="px-3 py-1.5 bg-gray-700 hover:bg-gray-600 text-white text-sm rounded transition-colors flex items-center gap-2"
            >
              {copied ? (
                <>
                  <Check className="w-4 h-4" />
                  복사됨
                </>
              ) : (
                <>
                  <Copy className="w-4 h-4" />
                  복사
                </>
              )}
            </button>
            <button
              onClick={onClose}
              className="text-gray-400 hover:text-white transition-colors"
            >
              <X className="w-5 h-5" />
            </button>
          </div>
        </div>

        {/* JSON 뷰어 */}
        <div className="flex-1 overflow-auto p-4 json-viewer-container">
          <style>{`
            .json-viewer-container {
              background: #1a1a1a;
              font-family: 'Monaco', 'Courier New', monospace;
            }
            .json-viewer-container .json-container {
              background: transparent !important;
              font-size: 13px;
              line-height: 1.5;
            }
            .json-viewer-container .json-string {
              color: #98c379 !important;
            }
            .json-viewer-container .json-number {
              color: #e5c07b !important;
            }
            .json-viewer-container .json-boolean {
              color: #56b6c2 !important;
            }
            .json-viewer-container .json-null {
              color: #abb2bf !important;
            }
            .json-viewer-container .json-property {
              color: #e06c75 !important;
            }
            .json-viewer-container .json-index {
              color: #c678dd !important;
            }
            .json-viewer-container .json-expanded::before {
              color: #61afef !important;
            }
            .json-viewer-container .json-collapsed::before {
              color: #61afef !important;
            }
          `}</style>
          <JsonView
            data={data}
            shouldExpandNode={(level) => level < 3}
            style={{
              container: {
                background: 'transparent',
                fontSize: '13px'
              }
            }}
          />
        </div>
      </div>
    </div>
  );
};