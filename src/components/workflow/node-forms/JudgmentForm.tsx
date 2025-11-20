import { Label } from '@/components/ui/label'
import { Input } from '@/components/ui/input'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Textarea } from '@/components/ui/textarea'
import { Slider } from '@/components/ui/slider'

/**
 * JUDGMENT 노드 설정 폼
 *
 * 제조업 AI 판단 예시:
 * - 품질 불량 원인 분석
 * - 설비 고장 예측
 * - 생산 계획 최적화 제안
 * - 이상 징후 감지 및 분류
 */

interface JudgmentFormProps {
  config: {
    judgmentMethod?: 'rule' | 'llm' | 'hybrid'
    ruleExpression?: string
    llmPrompt?: string
    confidenceThreshold?: number
    model?: string
    temperature?: number
  }
  onChange: (config: any) => void
}

export default function JudgmentForm({ config, onChange }: JudgmentFormProps) {
  const updateConfig = (key: string, value: any) => {
    onChange({ ...config, [key]: value })
  }

  return (
    <div className="space-y-4">
      {/* 판단 방식 선택 */}
      <div className="space-y-2">
        <Label htmlFor="judgmentMethod">판단 방식</Label>
        <Select
          value={config.judgmentMethod || 'hybrid'}
          onValueChange={(value) => updateConfig('judgmentMethod', value)}
        >
          <SelectTrigger id="judgmentMethod">
            <SelectValue placeholder="판단 방식 선택" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="rule">Rule Engine (빠름)</SelectItem>
            <SelectItem value="llm">AI 판단 (정확)</SelectItem>
            <SelectItem value="hybrid">하이브리드 (권장)</SelectItem>
          </SelectContent>
        </Select>
        <p className="text-xs text-muted-foreground">
          하이브리드: Rule 우선 실행 → 실패시 AI 보완
        </p>
      </div>

      {/* Rule Expression (rule 또는 hybrid인 경우) */}
      {(config.judgmentMethod === 'rule' || config.judgmentMethod === 'hybrid') && (
        <div className="space-y-2">
          <Label htmlFor="ruleExpression">Rule 표현식</Label>
          <Textarea
            id="ruleExpression"
            placeholder="예: defect_rate > 0.05 && temperature > 90"
            value={config.ruleExpression || ''}
            onChange={(e) => updateConfig('ruleExpression', e.target.value)}
            className="min-h-[80px] font-mono text-sm"
          />
          <div className="p-3 bg-muted rounded-lg text-xs space-y-1">
            <p className="font-semibold">Rule 예시:</p>
            <p>• defect_rate &gt; 0.05 && temperature &gt; 90</p>
            <p>• inventory &lt; 100 || lead_time &gt; 7</p>
            <p>• equipment_status == "warning"</p>
          </div>
        </div>
      )}

      {/* LLM Prompt (llm 또는 hybrid인 경우) */}
      {(config.judgmentMethod === 'llm' || config.judgmentMethod === 'hybrid') && (
        <>
          <div className="space-y-2">
            <Label htmlFor="llmPrompt">AI 판단 프롬프트</Label>
            <Textarea
              id="llmPrompt"
              placeholder="예: 다음 설비 데이터를 분석하여 이상 여부를 판단하세요..."
              value={config.llmPrompt || ''}
              onChange={(e) => updateConfig('llmPrompt', e.target.value)}
              className="min-h-[120px]"
            />
            <div className="p-3 bg-muted rounded-lg text-xs space-y-1">
              <p className="font-semibold">프롬프트 작성 팁:</p>
              <p>• 명확한 판단 기준 제시</p>
              <p>• 입력 데이터 형식 설명</p>
              <p>• 원하는 출력 형식 지정 (JSON 권장)</p>
            </div>
          </div>

          <div className="space-y-2">
            <Label htmlFor="model">AI 모델</Label>
            <Select
              value={config.model || 'claude-3-5-sonnet-20241022'}
              onValueChange={(value) => updateConfig('model', value)}
            >
              <SelectTrigger id="model">
                <SelectValue placeholder="모델 선택" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="claude-3-5-sonnet-20241022">Claude 3.5 Sonnet (권장)</SelectItem>
                <SelectItem value="claude-3-opus-20240229">Claude 3 Opus (고성능)</SelectItem>
                <SelectItem value="claude-3-haiku-20240307">Claude 3 Haiku (경제적)</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div className="space-y-2">
            <Label htmlFor="temperature">
              Temperature: {config.temperature || 0.7}
            </Label>
            <Slider
              id="temperature"
              min={0}
              max={2}
              step={0.1}
              value={[config.temperature || 0.7]}
              onValueChange={(value) => updateConfig('temperature', value[0])}
            />
            <p className="text-xs text-muted-foreground">
              낮을수록 일관된 답변, 높을수록 창의적 답변
            </p>
          </div>
        </>
      )}

      {/* 신뢰도 임계값 */}
      <div className="space-y-2">
        <Label htmlFor="confidenceThreshold">
          신뢰도 임계값: {config.confidenceThreshold || 0.7}
        </Label>
        <Slider
          id="confidenceThreshold"
          min={0}
          max={1}
          step={0.05}
          value={[config.confidenceThreshold || 0.7]}
          onValueChange={(value) => updateConfig('confidenceThreshold', value[0])}
        />
        <p className="text-xs text-muted-foreground">
          하이브리드 모드: Rule 신뢰도가 이 값 이하면 AI 실행
        </p>
      </div>
    </div>
  )
}
