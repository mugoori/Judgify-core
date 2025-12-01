// useState not needed currently but kept for future expansion
import { ChevronDown, ChevronRight, GripVertical, Trash2, Copy } from 'lucide-react'
import { Card } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'
import TriggerForm from './node-forms/TriggerForm'
import QueryForm from './node-forms/QueryForm'
import CalcForm from './node-forms/CalcForm'
import JudgmentForm from './node-forms/JudgmentForm'
import ApprovalForm from './node-forms/ApprovalForm'
import AlertForm from './node-forms/AlertForm'

/**
 * Phase 9 StepCard Component
 *
 * 두 가지 상태:
 * - Collapsed: 스텝 번호 + 타입 + 라벨만 표시
 * - Expanded: 전체 설정 폼 표시 (NodeType별로 다름)
 */

interface StepCardProps {
  id: string
  index: number
  type: 'TRIGGER' | 'QUERY' | 'CALC' | 'JUDGMENT' | 'APPROVAL' | 'ALERT'
  label: string
  config: Record<string, any>
  isExpanded?: boolean
  onToggleExpand?: () => void
  onDelete?: () => void
  onDuplicate?: () => void
  onConfigChange?: (config: Record<string, any>) => void
  dragHandleProps?: any
}

const NODE_TYPE_CONFIG = {
  TRIGGER: {
    icon: '⚡',
    color: 'text-yellow-600 bg-yellow-50 border-yellow-200',
    label: '트리거'
  },
  QUERY: {
    icon: '🔍',
    color: 'text-blue-600 bg-blue-50 border-blue-200',
    label: '데이터 조회'
  },
  CALC: {
    icon: '🧮',
    color: 'text-purple-600 bg-purple-50 border-purple-200',
    label: '계산'
  },
  JUDGMENT: {
    icon: '⚖️',
    color: 'text-green-600 bg-green-50 border-green-200',
    label: 'AI 판단'
  },
  APPROVAL: {
    icon: '✅',
    color: 'text-indigo-600 bg-indigo-50 border-indigo-200',
    label: '승인'
  },
  ALERT: {
    icon: '🔔',
    color: 'text-red-600 bg-red-50 border-red-200',
    label: '알림'
  }
}

export default function StepCard({
  id: _id,
  index,
  type,
  label,
  config,
  isExpanded = false,
  onToggleExpand,
  onDelete,
  onDuplicate,
  onConfigChange,
  dragHandleProps
}: StepCardProps) {
  // _id is used for key prop by parent, kept in props for consistency
  const typeConfig = NODE_TYPE_CONFIG[type]

  // NodeType별 폼 컴포넌트 렌더링
  const renderConfigForm = () => {
    if (!onConfigChange) return null

    const formProps = {
      config,
      onChange: onConfigChange
    }

    switch (type) {
      case 'TRIGGER':
        return <TriggerForm {...formProps} />
      case 'QUERY':
        return <QueryForm {...formProps} />
      case 'CALC':
        return <CalcForm {...formProps} />
      case 'JUDGMENT':
        return <JudgmentForm {...formProps} />
      case 'APPROVAL':
        return <ApprovalForm {...formProps} />
      case 'ALERT':
        return <AlertForm {...formProps} />
      default:
        return null
    }
  }

  return (
    <Card className={cn(
      'transition-all duration-200',
      isExpanded ? 'shadow-lg' : 'shadow-sm hover:shadow-md'
    )}>
      {/* Header (항상 표시) */}
      <div className="p-4">
        <div className="flex items-center gap-3">
          {/* Drag Handle */}
          <div
            {...dragHandleProps}
            className="cursor-grab active:cursor-grabbing text-muted-foreground hover:text-foreground transition-colors"
          >
            <GripVertical className="w-5 h-5" />
          </div>

          {/* Step Number */}
          <div className="flex items-center justify-center w-8 h-8 rounded-full bg-primary/10 text-primary font-semibold text-sm">
            {index + 1}
          </div>

          {/* Type Badge */}
          <div className={cn(
            'flex items-center gap-2 px-3 py-1 rounded-full border',
            typeConfig.color
          )}>
            <span className="text-base">{typeConfig.icon}</span>
            <span className="text-sm font-medium">{typeConfig.label}</span>
          </div>

          {/* Step Label */}
          <div className="flex-1 font-medium truncate">
            {label}
          </div>

          {/* Actions */}
          <div className="flex items-center gap-1">
            <Button
              variant="ghost"
              size="sm"
              onClick={onDuplicate}
              className="h-8 w-8 p-0"
            >
              <Copy className="w-4 h-4" />
            </Button>
            <Button
              variant="ghost"
              size="sm"
              onClick={onDelete}
              className="h-8 w-8 p-0 text-destructive hover:text-destructive"
            >
              <Trash2 className="w-4 h-4" />
            </Button>
            <Button
              variant="ghost"
              size="sm"
              onClick={onToggleExpand}
              className="h-8 w-8 p-0"
            >
              {isExpanded ? (
                <ChevronDown className="w-5 h-5" />
              ) : (
                <ChevronRight className="w-5 h-5" />
              )}
            </Button>
          </div>
        </div>
      </div>

      {/* Expanded Content - NodeType별 폼 */}
      {isExpanded && (
        <div className="px-4 pb-4 pt-2 border-t">
          <div className="space-y-4">
            {renderConfigForm()}
          </div>
        </div>
      )}
    </Card>
  )
}
