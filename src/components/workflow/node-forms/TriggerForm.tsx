import { Label } from '@/components/ui/label'
import { Input } from '@/components/ui/input'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Textarea } from '@/components/ui/textarea'

/**
 * TRIGGER 노드 설정 폼
 *
 * 제조업 트리거 예시:
 * - 설비 온도 임계값 초과
 * - 불량률 증가 감지
 * - 재고 부족 알림
 * - 주기적 점검 시간 도래
 */

interface TriggerFormProps {
  config: {
    triggerType?: 'threshold' | 'scheduled' | 'event' | 'manual'
    condition?: string
    threshold?: number
    schedule?: string
    description?: string
  }
  onChange: (config: any) => void
}

export default function TriggerForm({ config, onChange }: TriggerFormProps) {
  const updateConfig = (key: string, value: any) => {
    onChange({ ...config, [key]: value })
  }

  return (
    <div className="space-y-4">
      {/* 트리거 타입 선택 */}
      <div className="space-y-2">
        <Label htmlFor="triggerType">트리거 타입</Label>
        <Select
          value={config.triggerType || 'threshold'}
          onValueChange={(value) => updateConfig('triggerType', value)}
        >
          <SelectTrigger id="triggerType">
            <SelectValue placeholder="트리거 타입 선택" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="threshold">임계값 초과</SelectItem>
            <SelectItem value="scheduled">주기적 실행</SelectItem>
            <SelectItem value="event">이벤트 감지</SelectItem>
            <SelectItem value="manual">수동 실행</SelectItem>
          </SelectContent>
        </Select>
      </div>

      {/* 임계값 조건 (threshold인 경우) */}
      {config.triggerType === 'threshold' && (
        <>
          <div className="space-y-2">
            <Label htmlFor="condition">조건식</Label>
            <Input
              id="condition"
              placeholder="예: temperature > 90"
              value={config.condition || ''}
              onChange={(e) => updateConfig('condition', e.target.value)}
            />
            <p className="text-xs text-muted-foreground">
              예시: temperature &gt; 90, defect_rate &gt; 0.05, inventory &lt; 100
            </p>
          </div>
          <div className="space-y-2">
            <Label htmlFor="threshold">임계값</Label>
            <Input
              id="threshold"
              type="number"
              placeholder="90"
              value={config.threshold || ''}
              onChange={(e) => updateConfig('threshold', parseFloat(e.target.value))}
            />
          </div>
        </>
      )}

      {/* 스케줄 설정 (scheduled인 경우) */}
      {config.triggerType === 'scheduled' && (
        <div className="space-y-2">
          <Label htmlFor="schedule">실행 주기</Label>
          <Input
            id="schedule"
            placeholder="예: 0 9 * * * (매일 오전 9시)"
            value={config.schedule || ''}
            onChange={(e) => updateConfig('schedule', e.target.value)}
          />
          <p className="text-xs text-muted-foreground">
            Cron 표현식 형식: 분 시 일 월 요일
          </p>
        </div>
      )}

      {/* 설명 */}
      <div className="space-y-2">
        <Label htmlFor="description">설명 (선택사항)</Label>
        <Textarea
          id="description"
          placeholder="이 트리거에 대한 설명을 입력하세요"
          value={config.description || ''}
          onChange={(e) => updateConfig('description', e.target.value)}
          className="min-h-[80px]"
        />
      </div>
    </div>
  )
}
