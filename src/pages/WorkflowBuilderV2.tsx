import { useState } from 'react'
import { Plus, Save, Play, Settings } from 'lucide-react'
import { DragDropContext, Droppable, Draggable, DropResult } from '@hello-pangea/dnd'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import { Card } from '@/components/ui/card'
import StepCard from '@/components/workflow/StepCard'

/**
 * Phase 9 Manufacturing Workflow Builder (Vertical List UI)
 *
 * 제조업 특화 워크플로우 빌더:
 * - Vertical List 레이아웃 (Canvas 없음)
 * - 6개 NodeType: TRIGGER, QUERY, CALC, JUDGMENT, APPROVAL, ALERT
 * - 드래그앤드롭 재정렬
 * - 인라인 시뮬레이션
 */

interface WorkflowMetadata {
  name: string
  description: string
  isActive: boolean
}

interface WorkflowStep {
  id: string
  type: 'TRIGGER' | 'QUERY' | 'CALC' | 'JUDGMENT' | 'APPROVAL' | 'ALERT'
  label: string
  config: Record<string, any>
}

export default function WorkflowBuilderV2() {
  // 워크플로우 메타데이터 상태
  const [metadata, setMetadata] = useState<WorkflowMetadata>({
    name: '새 워크플로우',
    description: '',
    isActive: false
  })

  // 워크플로우 스텝 배열
  const [steps, setSteps] = useState<WorkflowStep[]>([])

  // 현재 확장된 스텝 ID
  const [expandedStepId, setExpandedStepId] = useState<string | null>(null)

  // 시뮬레이션 모드
  const [isSimulating, setIsSimulating] = useState(false)

  // 스텝 추가 핸들러
  const handleAddStep = (type: WorkflowStep['type']) => {
    const newStep: WorkflowStep = {
      id: `step-${Date.now()}`,
      type,
      label: `${type} 스텝`,
      config: {}
    }
    setSteps([...steps, newStep])
    // 새로 추가된 스텝 자동 확장
    setExpandedStepId(newStep.id)
  }

  // 스텝 삭제 핸들러
  const handleDeleteStep = (stepId: string) => {
    setSteps(steps.filter(step => step.id !== stepId))
    if (expandedStepId === stepId) {
      setExpandedStepId(null)
    }
  }

  // 스텝 복제 핸들러
  const handleDuplicateStep = (stepId: string) => {
    const stepToDuplicate = steps.find(step => step.id === stepId)
    if (stepToDuplicate) {
      const duplicatedStep: WorkflowStep = {
        ...stepToDuplicate,
        id: `step-${Date.now()}`,
        label: `${stepToDuplicate.label} (복사)`
      }
      const index = steps.findIndex(step => step.id === stepId)
      const newSteps = [...steps]
      newSteps.splice(index + 1, 0, duplicatedStep)
      setSteps(newSteps)
    }
  }

  // 스텝 설정 변경 핸들러
  const handleStepConfigChange = (stepId: string, config: Record<string, any>) => {
    setSteps(steps.map(step =>
      step.id === stepId ? { ...step, config } : step
    ))
  }

  // 드래그앤드롭 종료 핸들러
  const handleDragEnd = (result: DropResult) => {
    if (!result.destination) return

    const items = Array.from(steps)
    const [reorderedItem] = items.splice(result.source.index, 1)
    items.splice(result.destination.index, 0, reorderedItem)

    setSteps(items)
  }

  // 워크플로우 저장 핸들러
  const handleSaveWorkflow = async () => {
    // TODO: Backend API 호출
    console.log('워크플로우 저장:', { metadata, steps })
  }

  // 시뮬레이션 실행 핸들러
  const handleRunSimulation = () => {
    setIsSimulating(true)
    // TODO: 시뮬레이션 로직 구현
  }

  return (
    <div className="flex flex-col h-full bg-background">
      {/* Header Section */}
      <div className="border-b bg-card">
        <div className="container mx-auto p-4 space-y-4">
          {/* Workflow Name */}
          <div className="flex items-center gap-4">
            <Input
              value={metadata.name}
              onChange={(e) => setMetadata({ ...metadata, name: e.target.value })}
              className="text-2xl font-bold border-none shadow-none focus-visible:ring-0 px-0"
              placeholder="워크플로우 이름을 입력하세요"
            />
            <div className="flex gap-2 ml-auto">
              <Button
                variant="outline"
                size="sm"
                onClick={handleRunSimulation}
                disabled={steps.length === 0}
              >
                <Play className="w-4 h-4 mr-2" />
                시뮬레이션
              </Button>
              <Button
                variant="default"
                size="sm"
                onClick={handleSaveWorkflow}
              >
                <Save className="w-4 h-4 mr-2" />
                저장
              </Button>
              <Button
                variant="ghost"
                size="sm"
              >
                <Settings className="w-4 h-4" />
              </Button>
            </div>
          </div>

          {/* Workflow Description */}
          <Textarea
            value={metadata.description}
            onChange={(e) => setMetadata({ ...metadata, description: e.target.value })}
            placeholder="워크플로우 설명을 입력하세요 (선택사항)"
            className="min-h-[60px] resize-none border-dashed"
          />
        </div>
      </div>

      {/* Main Content - Scrollable Step List */}
      <div className="flex-1 overflow-auto">
        <div className="container mx-auto p-4">
          {/* Empty State */}
          {steps.length === 0 ? (
            <Card className="p-12 text-center border-dashed">
              <div className="space-y-4">
                <div className="text-muted-foreground">
                  <p className="text-lg font-medium">워크플로우 스텝이 없습니다</p>
                  <p className="text-sm mt-2">
                    아래 버튼을 클릭하여 첫 번째 스텝을 추가하세요
                  </p>
                </div>
              </div>
            </Card>
          ) : (
            /* Step List with Drag and Drop */
            <DragDropContext onDragEnd={handleDragEnd}>
              <Droppable droppableId="workflow-steps">
                {(provided, snapshot) => (
                  <div
                    {...provided.droppableProps}
                    ref={provided.innerRef}
                    className="space-y-3"
                  >
                    {steps.map((step, index) => (
                      <Draggable
                        key={step.id}
                        draggableId={step.id}
                        index={index}
                      >
                        {(provided, snapshot) => (
                          <div
                            ref={provided.innerRef}
                            {...provided.draggableProps}
                          >
                            <StepCard
                              id={step.id}
                              index={index}
                              type={step.type}
                              label={step.label}
                              config={step.config}
                              isExpanded={expandedStepId === step.id}
                              onToggleExpand={() => setExpandedStepId(
                                expandedStepId === step.id ? null : step.id
                              )}
                              onDelete={() => handleDeleteStep(step.id)}
                              onDuplicate={() => handleDuplicateStep(step.id)}
                              onConfigChange={(config) => handleStepConfigChange(step.id, config)}
                              dragHandleProps={provided.dragHandleProps}
                            />
                          </div>
                        )}
                      </Draggable>
                    ))}
                    {provided.placeholder}
                  </div>
                )}
              </Droppable>
            </DragDropContext>
          )}
        </div>
      </div>

      {/* Footer - Add Step Buttons */}
      <div className="border-t bg-card">
        <div className="container mx-auto p-4">
          <div className="flex flex-wrap gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={() => handleAddStep('TRIGGER')}
            >
              <Plus className="w-4 h-4 mr-2" />
              트리거
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={() => handleAddStep('QUERY')}
            >
              <Plus className="w-4 h-4 mr-2" />
              데이터 조회
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={() => handleAddStep('CALC')}
            >
              <Plus className="w-4 h-4 mr-2" />
              계산
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={() => handleAddStep('JUDGMENT')}
            >
              <Plus className="w-4 h-4 mr-2" />
              판단
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={() => handleAddStep('APPROVAL')}
            >
              <Plus className="w-4 h-4 mr-2" />
              승인
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={() => handleAddStep('ALERT')}
            >
              <Plus className="w-4 h-4 mr-2" />
              알림
            </Button>
          </div>
        </div>
      </div>
    </div>
  )
}
