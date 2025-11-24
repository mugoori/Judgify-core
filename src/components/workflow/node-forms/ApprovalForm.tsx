import { Label } from '@/components/ui/label'
import { Input } from '@/components/ui/input'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Textarea } from '@/components/ui/textarea'
import { Switch } from '@/components/ui/switch'

/**
 * APPROVAL 노드 설정 폼
 *
 * 제조업 승인 예시:
 * - 생산 계획 변경 승인
 * - 설비 가동 승인
 * - 불량품 폐기 승인
 * - 긴급 발주 승인
 */

interface ApprovalFormProps {
  config: {
    approvalType?: 'manual' | 'auto' | 'conditional'
    approvers?: string
    autoApproveCondition?: string
    timeoutMinutes?: number
    requireComment?: boolean
    notifyOnPending?: boolean
  }
  onChange: (config: any) => void
}

export default function ApprovalForm({ config, onChange }: ApprovalFormProps) {
  const updateConfig = (key: string, value: any) => {
    onChange({ ...config, [key]: value })
  }

  return (
    <div className="space-y-4">
      {/* 승인 타입 선택 */}
      <div className="space-y-2">
        <Label htmlFor="approvalType">승인 타입</Label>
        <Select
          value={config.approvalType || 'manual'}
          onValueChange={(value) => updateConfig('approvalType', value)}
        >
          <SelectTrigger id="approvalType">
            <SelectValue placeholder="승인 타입 선택" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="manual">수동 승인</SelectItem>
            <SelectItem value="auto">자동 승인</SelectItem>
            <SelectItem value="conditional">조건부 승인</SelectItem>
          </SelectContent>
        </Select>
      </div>

      {/* 승인자 목록 (manual 또는 conditional인 경우) */}
      {(config.approvalType === 'manual' || config.approvalType === 'conditional') && (
        <div className="space-y-2">
          <Label htmlFor="approvers">승인자 목록</Label>
          <Textarea
            id="approvers"
            placeholder="이메일 주소를 쉼표로 구분 (예: manager@company.com, supervisor@company.com)"
            value={config.approvers || ''}
            onChange={(e) => updateConfig('approvers', e.target.value)}
            className="min-h-[80px]"
          />
          <p className="text-xs text-muted-foreground">
            여러 승인자 중 한 명만 승인하면 진행됩니다
          </p>
        </div>
      )}

      {/* 자동 승인 조건 (conditional인 경우) */}
      {config.approvalType === 'conditional' && (
        <div className="space-y-2">
          <Label htmlFor="autoApproveCondition">자동 승인 조건</Label>
          <Textarea
            id="autoApproveCondition"
            placeholder="예: amount < 100000 && priority != 'high'"
            value={config.autoApproveCondition || ''}
            onChange={(e) => updateConfig('autoApproveCondition', e.target.value)}
            className="min-h-[80px] font-mono text-sm"
          />
          <p className="text-xs text-muted-foreground">
            조건 충족시 자동 승인, 아니면 수동 승인 대기
          </p>
        </div>
      )}

      {/* 타임아웃 설정 */}
      <div className="space-y-2">
        <Label htmlFor="timeoutMinutes">승인 대기 시간 (분)</Label>
        <Input
          id="timeoutMinutes"
          type="number"
          placeholder="60"
          value={config.timeoutMinutes || ''}
          onChange={(e) => updateConfig('timeoutMinutes', parseInt(e.target.value))}
        />
        <p className="text-xs text-muted-foreground">
          시간 초과시 자동으로 거부됩니다 (0 = 무제한)
        </p>
      </div>

      {/* 옵션 체크박스들 */}
      <div className="space-y-3">
        <div className="flex items-center space-x-2">
          <Switch
            id="requireComment"
            checked={config.requireComment || false}
            onCheckedChange={(checked) => updateConfig('requireComment', checked)}
          />
          <Label
            htmlFor="requireComment"
            className="text-sm font-normal cursor-pointer"
          >
            승인/거부시 코멘트 필수
          </Label>
        </div>

        <div className="flex items-center space-x-2">
          <Switch
            id="notifyOnPending"
            checked={config.notifyOnPending || false}
            onCheckedChange={(checked) => updateConfig('notifyOnPending', checked)}
          />
          <Label
            htmlFor="notifyOnPending"
            className="text-sm font-normal cursor-pointer"
          >
            승인 대기시 즉시 알림 전송
          </Label>
        </div>
      </div>
    </div>
  )
}
