import { Label } from '@/components/ui/label'
import { Input } from '@/components/ui/input'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Textarea } from '@/components/ui/textarea'

/**
 * CALC 노드 설정 폼
 *
 * 제조업 계산 예시:
 * - 불량률 계산 (불량품 수 / 전체 생산량)
 * - 설비 효율 계산 (실제 가동시간 / 계획 가동시간)
 * - 재고 회전율 계산
 * - 품질 지수 계산
 */

interface CalcFormProps {
  config: {
    calcType?: 'formula' | 'aggregate' | 'transform'
    formula?: string
    aggregateFunction?: 'sum' | 'avg' | 'min' | 'max' | 'count'
    targetField?: string
    outputField?: string
  }
  onChange: (config: any) => void
}

export default function CalcForm({ config, onChange }: CalcFormProps) {
  const updateConfig = (key: string, value: any) => {
    onChange({ ...config, [key]: value })
  }

  return (
    <div className="space-y-4">
      {/* 계산 타입 선택 */}
      <div className="space-y-2">
        <Label htmlFor="calcType">계산 타입</Label>
        <Select
          value={config.calcType || 'formula'}
          onValueChange={(value) => updateConfig('calcType', value)}
        >
          <SelectTrigger id="calcType">
            <SelectValue placeholder="계산 타입 선택" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="formula">수식 계산</SelectItem>
            <SelectItem value="aggregate">집계 함수</SelectItem>
            <SelectItem value="transform">데이터 변환</SelectItem>
          </SelectContent>
        </Select>
      </div>

      {/* 수식 입력 (formula인 경우) */}
      {config.calcType === 'formula' && (
        <div className="space-y-2">
          <Label htmlFor="formula">계산 수식</Label>
          <Textarea
            id="formula"
            placeholder="예: (defect_count / total_count) * 100"
            value={config.formula || ''}
            onChange={(e) => updateConfig('formula', e.target.value)}
            className="min-h-[100px] font-mono text-sm"
          />
          <p className="text-xs text-muted-foreground">
            지원 연산자: +, -, *, /, %, (), Math.함수들
          </p>
          <div className="p-3 bg-muted rounded-lg text-xs space-y-1">
            <p className="font-semibold">제조업 계산 예시:</p>
            <p>• 불량률: (defect_count / total_count) * 100</p>
            <p>• 설비 효율: (actual_time / planned_time) * 100</p>
            <p>• 가동률: (running_time / available_time) * 100</p>
          </div>
        </div>
      )}

      {/* 집계 함수 (aggregate인 경우) */}
      {config.calcType === 'aggregate' && (
        <>
          <div className="space-y-2">
            <Label htmlFor="aggregateFunction">집계 함수</Label>
            <Select
              value={config.aggregateFunction || 'avg'}
              onValueChange={(value) => updateConfig('aggregateFunction', value)}
            >
              <SelectTrigger id="aggregateFunction">
                <SelectValue placeholder="집계 함수 선택" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="sum">합계 (SUM)</SelectItem>
                <SelectItem value="avg">평균 (AVG)</SelectItem>
                <SelectItem value="min">최소값 (MIN)</SelectItem>
                <SelectItem value="max">최대값 (MAX)</SelectItem>
                <SelectItem value="count">개수 (COUNT)</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-2">
            <Label htmlFor="targetField">대상 필드</Label>
            <Input
              id="targetField"
              placeholder="예: temperature"
              value={config.targetField || ''}
              onChange={(e) => updateConfig('targetField', e.target.value)}
            />
          </div>
        </>
      )}

      {/* 출력 필드명 */}
      <div className="space-y-2">
        <Label htmlFor="outputField">출력 필드명</Label>
        <Input
          id="outputField"
          placeholder="예: defect_rate"
          value={config.outputField || ''}
          onChange={(e) => updateConfig('outputField', e.target.value)}
        />
        <p className="text-xs text-muted-foreground">
          계산 결과를 저장할 변수명
        </p>
      </div>
    </div>
  )
}
