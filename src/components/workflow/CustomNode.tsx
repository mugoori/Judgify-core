import React from 'react';
import { Handle, Position, NodeProps } from 'reactflow';
import { FileText, GitBranch, Zap, CheckCircle, AlertCircle } from 'lucide-react';
import { cn } from '@/lib/utils';

// 노드 타입별 스타일 및 아이콘
const nodeStyles = {
  input: {
    bg: 'bg-blue-50 border-blue-500',
    icon: FileText,
    iconColor: 'text-blue-600',
  },
  decision: {
    bg: 'bg-purple-50 border-purple-500',
    icon: GitBranch,
    iconColor: 'text-purple-600',
  },
  action: {
    bg: 'bg-yellow-50 border-yellow-500',
    icon: Zap,
    iconColor: 'text-yellow-600',
  },
  output: {
    bg: 'bg-green-50 border-green-500',
    icon: CheckCircle,
    iconColor: 'text-green-600',
  },
  default: {
    bg: 'bg-gray-50 border-gray-400',
    icon: AlertCircle,
    iconColor: 'text-gray-600',
  },
};

interface CustomNodeData {
  label: string;
  type?: 'input' | 'decision' | 'action' | 'output' | 'default';
  description?: string;
  rule?: string;
  error?: string;
  highlighted?: boolean;
}

const CustomNode = React.memo(({ data, selected }: NodeProps<CustomNodeData>) => {
  const nodeType = data.type || 'default';
  const style = nodeStyles[nodeType];
  const IconComponent = style.icon;

  return (
    <div
      className={cn(
        'relative px-4 py-3 rounded-lg border-2 transition-all duration-200',
        style.bg,
        selected
          ? 'shadow-[0_0_0_3px_rgba(59,124,245,0.4),0_0_20px_rgba(59,124,245,0.15)] scale-[1.02]'
          : 'shadow-lg hover:shadow-xl',
        data.error && 'border-red-500 bg-red-50',
        data.highlighted && 'ring-4 ring-yellow-400 ring-opacity-60 animate-pulse'
      )}
      style={{ minWidth: '200px' }}
    >
      {/* 선택 체크마크 */}
      {selected && (
        <div
          className="absolute -top-2 -right-2 w-6 h-6 bg-primary rounded-full flex items-center justify-center border-2 border-background shadow-md animate-in zoom-in-75 duration-200"
          aria-label="선택됨"
          role="status"
        >
          <CheckCircle className="w-4 h-4 text-primary-foreground" />
        </div>
      )}

      {/* Top Handle (입력) */}
      {nodeType !== 'input' && (
        <Handle
          type="target"
          position={Position.Top}
          className="w-3 h-3 border-2 border-white bg-primary"
        />
      )}

      {/* 노드 헤더 */}
      <div className="flex items-center gap-2 mb-2">
        <IconComponent className={cn('w-5 h-5', style.iconColor)} />
        <div className="text-sm font-semibold">{data.label}</div>
      </div>

      {/* 노드 설명 */}
      {data.description && (
        <div className="text-xs text-muted-foreground mb-2">{data.description}</div>
      )}

      {/* Rule 표현식 (Decision 노드) */}
      {nodeType === 'decision' && data.rule && (
        <div className="mt-2 p-2 bg-white rounded border border-purple-200">
          <div className="text-xs font-mono text-purple-700">{data.rule}</div>
        </div>
      )}

      {/* 에러 메시지 */}
      {data.error && (
        <div className="mt-2 p-2 bg-red-100 rounded border border-red-300">
          <div className="text-xs text-red-700 flex items-center gap-1">
            <AlertCircle className="w-3 h-3" />
            {data.error}
          </div>
        </div>
      )}

      {/* Bottom Handle (출력) */}
      {nodeType !== 'output' && (
        <Handle
          type="source"
          position={Position.Bottom}
          className="w-3 h-3 border-2 border-white bg-primary"
        />
      )}

      {/* Decision 노드는 양쪽에도 핸들 */}
      {nodeType === 'decision' && (
        <>
          <Handle
            type="source"
            position={Position.Right}
            id="true"
            className="w-3 h-3 border-2 border-white bg-green-500"
            style={{ top: '50%' }}
          />
          <Handle
            type="source"
            position={Position.Left}
            id="false"
            className="w-3 h-3 border-2 border-white bg-red-500"
            style={{ top: '50%' }}
          />
        </>
      )}
    </div>
  );
});

CustomNode.displayName = 'CustomNode';

export default CustomNode;
