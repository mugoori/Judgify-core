import React from 'react';
import { Handle, Position, NodeProps } from 'reactflow';
import {
  FileText,
  GitBranch,
  Zap,
  CheckCircle,
  AlertCircle,
  Database,
  FileCode,
  Brain,
  Bell,
  BarChart,
} from 'lucide-react';
import { cn } from '@/lib/utils';
import { NodeType } from '@/types/workflow';

/**
 * 노드 타입별 스타일 및 아이콘 (v2 - 7가지 타입)
 */
const nodeStyles: Record<
  NodeType,
  {
    bg: string;
    icon: typeof FileText;
    iconColor: string;
  }
> = {
  // 기존 타입 (v1 호환)
  [NodeType.INPUT]: {
    bg: 'bg-blue-50 border-blue-500',
    icon: FileText,
    iconColor: 'text-blue-600',
  },
  [NodeType.DECISION]: {
    bg: 'bg-purple-50 border-purple-500',
    icon: GitBranch,
    iconColor: 'text-purple-600',
  },
  [NodeType.ACTION]: {
    bg: 'bg-yellow-50 border-yellow-500',
    icon: Zap,
    iconColor: 'text-yellow-600',
  },
  [NodeType.OUTPUT]: {
    bg: 'bg-green-50 border-green-500',
    icon: CheckCircle,
    iconColor: 'text-green-600',
  },

  // 신규 타입 (Week 5)
  [NodeType.DATA_INPUT]: {
    bg: 'bg-violet-50 border-violet-500',
    icon: Database,
    iconColor: 'text-violet-600',
  },
  [NodeType.RULE_JUDGMENT]: {
    bg: 'bg-emerald-50 border-emerald-500',
    icon: FileCode,
    iconColor: 'text-emerald-600',
  },
  [NodeType.LLM_JUDGMENT]: {
    bg: 'bg-cyan-50 border-cyan-500',
    icon: Brain,
    iconColor: 'text-cyan-600',
  },
  [NodeType.ACTION_EXECUTION]: {
    bg: 'bg-orange-50 border-orange-500',
    icon: Zap,
    iconColor: 'text-orange-600',
  },
  [NodeType.NOTIFICATION]: {
    bg: 'bg-pink-50 border-pink-500',
    icon: Bell,
    iconColor: 'text-pink-600',
  },
  [NodeType.DATA_AGGREGATION]: {
    bg: 'bg-teal-50 border-teal-500',
    icon: BarChart,
    iconColor: 'text-teal-600',
  },
};

/**
 * 기본 스타일 (알 수 없는 타입)
 */
const defaultStyle = {
  bg: 'bg-gray-50 border-gray-400',
  icon: AlertCircle,
  iconColor: 'text-gray-600',
};

/**
 * CustomNode 데이터 (v1 호환 + v2 확장)
 */
interface CustomNodeData {
  label: string;
  type?: string; // v1: 'input' | v2: NodeType enum
  description?: string;
  rule?: string; // v1 DECISION 노드의 조건식
  error?: string;
  highlighted?: boolean;

  // v2 확장 필드
  ruleExpression?: string; // RULE_JUDGMENT 노드의 AST 기반 표현식
  prompt?: string; // LLM_JUDGMENT 노드의 프롬프트
  service?: string; // ACTION_EXECUTION 노드의 서비스명
  channel?: string; // NOTIFICATION 노드의 채널명
  aggregationType?: string; // DATA_AGGREGATION 노드의 집계 타입
  source?: string; // DATA_INPUT 노드의 데이터 소스
}

const CustomNode = React.memo(({ data, selected }: NodeProps<CustomNodeData>) => {
  // 노드 타입 결정 (v1 문자열 → v2 Enum 지원)
  const nodeType: NodeType =
    data.type && Object.values(NodeType).includes(data.type as NodeType)
      ? (data.type as NodeType)
      : NodeType.INPUT; // 기본값

  // 스타일 가져오기
  const style = nodeStyles[nodeType] || defaultStyle;
  const IconComponent = style.icon;

  // 입력 핸들 표시 여부 (INPUT, DATA_INPUT 제외)
  const showInputHandle = ![NodeType.INPUT, NodeType.DATA_INPUT].includes(nodeType);

  // 출력 핸들 표시 여부 (OUTPUT 제외)
  const showOutputHandle = nodeType !== NodeType.OUTPUT;

  // 분기 핸들 표시 여부 (DECISION, RULE_JUDGMENT, LLM_JUDGMENT)
  const showBranchHandles = [
    NodeType.DECISION,
    NodeType.RULE_JUDGMENT,
    NodeType.LLM_JUDGMENT,
  ].includes(nodeType);

  /**
   * 노드 타입별 상세 정보 렌더링
   */
  const renderNodeDetails = () => {
    switch (nodeType) {
      case NodeType.DECISION:
      case NodeType.RULE_JUDGMENT:
        // 조건식 또는 Rule 표현식 표시
        const expression = data.ruleExpression || data.rule;
        if (expression) {
          return (
            <div className="mt-2 p-2 bg-white rounded border border-purple-200">
              <div className="text-xs font-mono text-purple-700 break-all">
                {expression}
              </div>
            </div>
          );
        }
        break;

      case NodeType.LLM_JUDGMENT:
        // LLM 프롬프트 미리보기
        if (data.prompt) {
          return (
            <div className="mt-2 p-2 bg-white rounded border border-cyan-200">
              <div className="text-xs text-cyan-700 line-clamp-2">{data.prompt}</div>
            </div>
          );
        }
        break;

      case NodeType.ACTION_EXECUTION:
        // 서비스명 표시
        if (data.service) {
          return (
            <div className="mt-2 flex items-center gap-1">
              <span className="text-xs bg-orange-100 text-orange-700 px-2 py-1 rounded">
                {data.service}
              </span>
            </div>
          );
        }
        break;

      case NodeType.NOTIFICATION:
        // 채널명 표시
        if (data.channel) {
          return (
            <div className="mt-2 flex items-center gap-1">
              <Bell className="w-3 h-3 text-pink-600" />
              <span className="text-xs text-pink-700">{data.channel}</span>
            </div>
          );
        }
        break;

      case NodeType.DATA_AGGREGATION:
        // 집계 타입 표시
        if (data.aggregationType) {
          return (
            <div className="mt-2 flex items-center gap-1">
              <span className="text-xs bg-teal-100 text-teal-700 px-2 py-1 rounded font-mono">
                {data.aggregationType.toUpperCase()}
              </span>
            </div>
          );
        }
        break;

      case NodeType.DATA_INPUT:
        // 데이터 소스 표시
        if (data.source) {
          return (
            <div className="mt-2 flex items-center gap-1">
              <Database className="w-3 h-3 text-violet-600" />
              <span className="text-xs text-violet-700">{data.source}</span>
            </div>
          );
        }
        break;

      default:
        return null;
    }
  };

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
      {showInputHandle && (
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
        <div className="text-xs text-muted-foreground mb-2 line-clamp-2">
          {data.description}
        </div>
      )}

      {/* 노드 타입별 상세 정보 */}
      {renderNodeDetails()}

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
      {showOutputHandle && !showBranchHandles && (
        <Handle
          type="source"
          position={Position.Bottom}
          className="w-3 h-3 border-2 border-white bg-primary"
        />
      )}

      {/* 분기 핸들 (DECISION, RULE_JUDGMENT, LLM_JUDGMENT) */}
      {showBranchHandles && (
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
