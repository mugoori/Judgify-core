import { useState } from 'react'
import { Sparkles } from 'lucide-react'
import { invoke } from '@tauri-apps/api/tauri'
import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/textarea'
import { useToast } from '@/components/ui/use-toast'

/**
 * AI Workflow Generator 컴포넌트 (Phase 9-2)
 *
 * 자연어 입력으로 워크플로우 초안을 자동 생성합니다.
 *
 * 기능:
 * - 한글 자연어 입력 (예: "1호선 불량률 3% 초과시 알림")
 * - Claude API를 통한 워크플로우 생성
 * - 6개 NodeType 지원 (TRIGGER, QUERY, CALC, JUDGMENT, APPROVAL, ALERT)
 * - 생성된 스텝을 부모 컴포넌트로 전달
 *
 * @example
 * <AiGenerator onGenerate={(steps) => setSteps(steps)} />
 */

export interface WorkflowStep {
  id: string
  type: 'TRIGGER' | 'QUERY' | 'CALC' | 'JUDGMENT' | 'APPROVAL' | 'ALERT'
  label: string
  config: Record<string, any>
}

interface AiGeneratorProps {
  onGenerate: (steps: WorkflowStep[]) => void
  disabled?: boolean
}

export function AiGenerator({ onGenerate, disabled = false }: AiGeneratorProps) {
  const { toast } = useToast()
  const [prompt, setPrompt] = useState('')
  const [isGenerating, setIsGenerating] = useState(false)

  const handleGenerate = async () => {
    // 입력 검증
    if (!prompt.trim()) {
      toast({
        title: '입력 필요',
        description: '워크플로우 설명을 입력해주세요.',
        variant: 'destructive',
      })
      return
    }

    setIsGenerating(true)
    try {
      // Tauri 백엔드 호출
      const steps = await invoke<WorkflowStep[]>('generate_workflow_draft', {
        userPrompt: prompt
      })

      // 생성 성공
      onGenerate(steps)
      setPrompt('')  // 입력 초기화

      toast({
        title: 'AI 생성 완료',
        description: `${steps.length}개 스텝이 생성되었습니다!`,
      })
    } catch (error) {
      // 생성 실패
      console.error('AI workflow generation failed:', error)
      toast({
        title: 'AI 생성 실패',
        description: error instanceof Error ? error.message : '알 수 없는 오류가 발생했습니다.',
        variant: 'destructive',
      })
    } finally {
      setIsGenerating(false)
    }
  }

  // Enter 키로 생성 (Shift+Enter는 줄바꿈)
  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      if (!isGenerating && prompt.trim()) {
        handleGenerate()
      }
    }
  }

  return (
    <div className="border-b bg-gradient-to-r from-purple-50 to-blue-50 dark:from-purple-950/20 dark:to-blue-950/20 p-4">
      <div className="container mx-auto max-w-6xl">
        {/* 헤더 */}
        <div className="flex items-center gap-2 mb-3">
          <Sparkles className="w-5 h-5 text-purple-600 dark:text-purple-400" />
          <h3 className="font-semibold text-purple-900 dark:text-purple-100">
            AI 워크플로우 생성기
          </h3>
          <span className="text-xs text-muted-foreground ml-2">
            (Claude Sonnet 4.5)
          </span>
        </div>

        {/* 입력 영역 */}
        <div className="flex items-start gap-4">
          <Textarea
            value={prompt}
            onChange={(e) => setPrompt(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="예: 1호선 불량률이 3% 초과하면 알림 보내기"
            className="flex-1 min-h-[80px] max-h-[200px] bg-white dark:bg-gray-800 resize-y"
            disabled={isGenerating || disabled}
          />
          <Button
            onClick={handleGenerate}
            disabled={isGenerating || disabled || !prompt.trim()}
            className="bg-purple-600 hover:bg-purple-700 dark:bg-purple-500 dark:hover:bg-purple-600 whitespace-nowrap"
            size="lg"
          >
            <Sparkles className="w-4 h-4 mr-2" />
            {isGenerating ? (
              <>
                <span className="animate-pulse">AI 생성 중...</span>
              </>
            ) : (
              'AI로 생성'
            )}
          </Button>
        </div>

        {/* 힌트 */}
        <div className="mt-2 text-xs text-muted-foreground">
          💡 팁: "불량률 모니터링", "설비 가동률 분석", "재고 부족 알림" 등 제조업 시나리오를 자연어로 입력하세요.
        </div>
      </div>
    </div>
  )
}
