import { useState, useEffect } from 'react'
import { Plus, Save, Play, Settings, FolderOpen, Trash2, History, Clock, CheckCircle, XCircle, AlertCircle, Timer, Power, PowerOff, FileText } from 'lucide-react'
import { DragDropContext, Droppable, Draggable, DropResult } from '@hello-pangea/dnd'
import { invoke } from '@tauri-apps/api/tauri'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Card } from '@/components/ui/card'
import { useToast } from '@/components/ui/use-toast'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import StepCard from '@/components/workflow/StepCard'
import { AiGenerator } from '@/components/workflow/AiGenerator'

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

interface StepExecutionResult {
  step_id: string
  step_type: string
  label: string
  status: 'success' | 'error' | 'skipped'
  input: any
  output: any | null
  error: string | null
  execution_time_ms: number
}

interface SimulationResult {
  workflow_id: string
  steps_executed: StepExecutionResult[]
  final_result: any
  total_execution_time_ms: number
  status: 'success' | 'partial_success' | 'error'
}

// 실행 이력 인터페이스
interface WorkflowExecution {
  id: string
  workflow_id: string
  status: 'success' | 'partial_success' | 'error'
  steps_executed: number
  final_result: any
  execution_time_ms: number
  created_at: string
}

interface WorkflowExecutionDetail {
  id: string
  workflow_id: string
  status: string
  steps_executed: StepExecutionResult[]
  final_result: any
  execution_time_ms: number
  created_at: string
}

export default function WorkflowBuilderV2() {
  const { toast } = useToast()

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
  const [simulationResult, setSimulationResult] = useState<SimulationResult | null>(null)
  const [showSimulationDialog, setShowSimulationDialog] = useState(false)
  const [testData, setTestData] = useState<string>('{}')

  // 저장 중 상태
  const [isSaving, setIsSaving] = useState(false)

  // 실행 이력 상태
  const [executionHistory, setExecutionHistory] = useState<WorkflowExecution[]>([])
  const [selectedExecution, setSelectedExecution] = useState<WorkflowExecutionDetail | null>(null)
  const [showHistoryDialog, setShowHistoryDialog] = useState(false)
  const [isLoadingHistory, setIsLoadingHistory] = useState(false)

  // 스케줄러 상태
  const [showSchedulerDialog, setShowSchedulerDialog] = useState(false)
  const [cronExpression, setCronExpression] = useState('0 * * * *')
  const [isScheduleActive, setIsScheduleActive] = useState(false)
  const [isScheduleLoading, setIsScheduleLoading] = useState(false)

  // 템플릿 상태
  const [showTemplateDialog, setShowTemplateDialog] = useState(false)

  // 워크플로우 목록 상태
  const [workflowList, setWorkflowList] = useState<Array<{
    id: string
    name: string
    description: string
    stepCount: number
    createdAt: string
  }>>([])
  const [isLoadingList, setIsLoadingList] = useState(false)
  const [showLoadDialog, setShowLoadDialog] = useState(false)

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
    try {
      setIsSaving(true)

      // 입력 검증
      if (!metadata.name.trim()) {
        toast({
          title: '저장 실패',
          description: '워크플로우 이름을 입력해주세요.',
          variant: 'destructive',
        })
        return
      }

      if (steps.length === 0) {
        toast({
          title: '저장 실패',
          description: '최소 1개 이상의 스텝을 추가해주세요.',
          variant: 'destructive',
        })
        return
      }

      // Tauri 백엔드 API 호출
      const response = await invoke<{ id: string; version: number; message: string }>(
        'save_workflow_v2',
        {
          request: {
            metadata: metadata,
            steps: steps,
          },
        }
      )

      toast({
        title: '저장 성공',
        description: `${response.message} (ID: ${response.id})`,
      })

      console.log('✅ 워크플로우 저장 완료:', response)
    } catch (error) {
      console.error('❌ 워크플로우 저장 실패:', error)
      toast({
        title: '저장 실패',
        description: error instanceof Error ? error.message : '알 수 없는 오류가 발생했습니다.',
        variant: 'destructive',
      })
    } finally {
      setIsSaving(false)
    }
  }

  // 워크플로우 목록 불러오기
  const handleLoadWorkflowList = async () => {
    try {
      setIsLoadingList(true)
      const list = await invoke<Array<{
        id: string
        name: string
        description: string
        stepCount: number
        createdAt: string
      }>>('list_workflows_v2')

      setWorkflowList(list)
      setShowLoadDialog(true)
    } catch (error) {
      console.error('❌ 워크플로우 목록 조회 실패:', error)
      toast({
        title: '목록 조회 실패',
        description: error instanceof Error ? error.message : '알 수 없는 오류가 발생했습니다.',
        variant: 'destructive',
      })
    } finally {
      setIsLoadingList(false)
    }
  }

  // 워크플로우 불러오기
  const handleLoadWorkflow = async (workflowId: string) => {
    try {
      const response = await invoke<{
        id: string
        metadata: WorkflowMetadata
        steps: WorkflowStep[]
      }>('load_workflow_v2', { workflowId })

      setMetadata(response.metadata)
      setSteps(response.steps)
      setShowLoadDialog(false)

      toast({
        title: '불러오기 성공',
        description: `"${response.metadata.name}" 워크플로우를 불러왔습니다.`,
      })

      console.log('✅ 워크플로우 불러오기 완료:', response)
    } catch (error) {
      console.error('❌ 워크플로우 불러오기 실패:', error)
      toast({
        title: '불러오기 실패',
        description: error instanceof Error ? error.message : '알 수 없는 오류가 발생했습니다.',
        variant: 'destructive',
      })
    }
  }

  // 워크플로우 삭제
  const handleDeleteWorkflow = async (workflowId: string, workflowName: string, e: React.MouseEvent) => {
    // 이벤트 전파 방지 (카드 클릭 이벤트가 발생하지 않도록)
    e.stopPropagation()

    try {
      await invoke('delete_workflow_v2', { workflowId })

      toast({
        title: '삭제 성공',
        description: `"${workflowName}" 워크플로우를 삭제했습니다.`,
      })

      // 목록 새로고침
      handleLoadWorkflowList()

      console.log('✅ 워크플로우 삭제 완료:', workflowId)
    } catch (error) {
      console.error('❌ 워크플로우 삭제 실패:', error)
      toast({
        title: '삭제 실패',
        description: error instanceof Error ? error.message : '알 수 없는 오류가 발생했습니다.',
        variant: 'destructive',
      })
    }
  }

  // 시뮬레이션 실행 핸들러
  const handleRunSimulation = async () => {
    try {
      // 입력 검증
      if (steps.length === 0) {
        toast({
          title: '시뮬레이션 실패',
          description: '최소 1개 이상의 스텝을 추가해주세요.',
          variant: 'destructive',
        })
        return
      }

      // 테스트 데이터 검증
      let parsedTestData
      try {
        parsedTestData = JSON.parse(testData)
      } catch (e) {
        toast({
          title: '시뮬레이션 실패',
          description: '테스트 데이터가 올바른 JSON 형식이 아닙니다.',
          variant: 'destructive',
        })
        return
      }

      setIsSimulating(true)
      setShowSimulationDialog(true)

      // Tauri 백엔드 API 호출
      const result = await invoke<SimulationResult>('simulate_workflow_v2', {
        request: {
          workflow_id: 'test-workflow-' + Date.now(),
          steps: steps.map(step => ({
            id: step.id,
            step_type: step.type,
            label: step.label,
            config: step.config
          })),
          test_data: parsedTestData
        }
      })

      setSimulationResult(result)

      toast({
        title: '시뮬레이션 완료',
        description: `${result.steps_executed.length}개 스텝 실행 완료 (${result.total_execution_time_ms}ms)`,
      })

      console.log('✅ 워크플로우 시뮬레이션 완료:', result)
    } catch (error) {
      console.error('❌ 워크플로우 시뮬레이션 실패:', error)
      toast({
        title: '시뮬레이션 실패',
        description: error instanceof Error ? error.message : '알 수 없는 오류가 발생했습니다.',
        variant: 'destructive',
      })
    } finally {
      setIsSimulating(false)
    }
  }

  /**
   * AI 워크플로우 생성 핸들러 (Phase 9-2)
   *
   * AI가 생성한 스텝을 기존 스텝 목록에 추가
   */
  const handleAiGenerate = (generatedSteps: WorkflowStep[]) => {
    setSteps((prevSteps) => [...prevSteps, ...generatedSteps])
    toast({
      title: '워크플로우 추가됨',
      description: `${generatedSteps.length}개 스텝이 추가되었습니다. 저장 버튼을 눌러 저장하세요.`,
    })
  }

  // 실행 이력 조회 핸들러
  const handleLoadExecutionHistory = async () => {
    try {
      setIsLoadingHistory(true)
      const history = await invoke<WorkflowExecution[]>('get_workflow_executions', {
        workflowId: metadata.name,
        limit: 20
      })
      setExecutionHistory(history)
      setShowHistoryDialog(true)
      setSelectedExecution(null)
    } catch (error) {
      console.error('❌ 실행 이력 조회 실패:', error)
      toast({
        title: '이력 조회 실패',
        description: error instanceof Error ? error.message : '알 수 없는 오류가 발생했습니다.',
        variant: 'destructive',
      })
    } finally {
      setIsLoadingHistory(false)
    }
  }

  // 실행 이력 상세 조회 핸들러
  const handleLoadExecutionDetail = async (executionId: string) => {
    try {
      const detail = await invoke<WorkflowExecutionDetail>('get_workflow_execution_detail', {
        executionId
      })
      setSelectedExecution(detail)
    } catch (error) {
      console.error('❌ 실행 이력 상세 조회 실패:', error)
      toast({
        title: '상세 조회 실패',
        description: error instanceof Error ? error.message : '알 수 없는 오류가 발생했습니다.',
        variant: 'destructive',
      })
    }
  }

  // 스케줄 등록 핸들러
  const handleRegisterSchedule = async () => {
    if (!metadata.name.trim()) {
      toast({
        title: '스케줄 등록 실패',
        description: '워크플로우 이름을 먼저 입력해주세요.',
        variant: 'destructive',
      })
      return
    }

    try {
      setIsScheduleLoading(true)
      await invoke('register_workflow_schedule', {
        workflowId: metadata.name,
        cronExpression: cronExpression
      })
      setIsScheduleActive(true)
      toast({
        title: '스케줄 등록 완료',
        description: `"${metadata.name}" 워크플로우가 스케줄에 등록되었습니다.`,
      })
    } catch (error) {
      console.error('❌ 스케줄 등록 실패:', error)
      toast({
        title: '스케줄 등록 실패',
        description: error instanceof Error ? error.message : '알 수 없는 오류가 발생했습니다.',
        variant: 'destructive',
      })
    } finally {
      setIsScheduleLoading(false)
    }
  }

  // 스케줄 해제 핸들러
  const handleUnregisterSchedule = async () => {
    try {
      setIsScheduleLoading(true)
      await invoke('unregister_workflow_schedule', {
        workflowId: metadata.name
      })
      setIsScheduleActive(false)
      toast({
        title: '스케줄 해제 완료',
        description: `"${metadata.name}" 워크플로우의 스케줄이 해제되었습니다.`,
      })
    } catch (error) {
      console.error('❌ 스케줄 해제 실패:', error)
      toast({
        title: '스케줄 해제 실패',
        description: error instanceof Error ? error.message : '알 수 없는 오류가 발생했습니다.',
        variant: 'destructive',
      })
    } finally {
      setIsScheduleLoading(false)
    }
  }

  // 워크플로우 템플릿 정의
  const workflowTemplates = [
    {
      id: 'defect-rate-monitoring',
      name: '불량률 모니터링',
      description: '생산라인 불량률이 임계값 초과시 알림',
      steps: [
        { id: 'trigger_1', type: 'TRIGGER' as const, label: '불량률 임계값 감지', config: { triggerType: 'threshold', metric: '불량률', threshold: 3 } },
        { id: 'query_1', type: 'QUERY' as const, label: '최근 불량 데이터 조회', config: { table: 'defects', period: '1h' } },
        { id: 'calc_1', type: 'CALC' as const, label: '불량률 계산', config: { formula: '(불량수/총생산수)*100' } },
        { id: 'judgment_1', type: 'JUDGMENT' as const, label: '불량률 판정', config: { condition: '> 3%', action: 'alert' } },
        { id: 'alert_1', type: 'ALERT' as const, label: '팀장 알림', config: { channel: 'slack', recipient: '품질팀장' } },
      ]
    },
    {
      id: 'equipment-anomaly',
      name: '설비 이상 감지',
      description: '센서 데이터 기반 설비 이상 탐지',
      steps: [
        { id: 'trigger_1', type: 'TRIGGER' as const, label: '센서 데이터 수신', config: { triggerType: 'event', source: 'sensor' } },
        { id: 'query_1', type: 'QUERY' as const, label: '센서 이력 조회', config: { table: 'sensor_data', limit: 100 } },
        { id: 'calc_1', type: 'CALC' as const, label: '표준편차 계산', config: { formula: 'std_dev(temperature)' } },
        { id: 'judgment_1', type: 'JUDGMENT' as const, label: 'AI 이상 탐지', config: { model: 'anomaly_detection', threshold: 0.8 } },
        { id: 'approval_1', type: 'APPROVAL' as const, label: '담당자 확인', config: { approver: '설비담당자', timeout: '30m' } },
        { id: 'alert_1', type: 'ALERT' as const, label: '유지보수 요청', config: { channel: 'email', recipient: '유지보수팀' } },
      ]
    },
    {
      id: 'quality-inspection',
      name: '품질 검사 워크플로우',
      description: '제품 품질 검사 및 등급 분류',
      steps: [
        { id: 'trigger_1', type: 'TRIGGER' as const, label: '검사 요청 수신', config: { triggerType: 'manual' } },
        { id: 'query_1', type: 'QUERY' as const, label: '검사 데이터 조회', config: { table: 'inspection_data' } },
        { id: 'judgment_1', type: 'JUDGMENT' as const, label: '품질 등급 판정', config: { rules: ['A등급: 95+', 'B등급: 85+', 'C등급: 70+'] } },
        { id: 'alert_1', type: 'ALERT' as const, label: '검사 결과 통보', config: { channel: 'system', recipient: '품질관리팀' } },
      ]
    },
  ]

  // 템플릿 로드 핸들러
  const handleLoadTemplate = (templateId: string) => {
    const template = workflowTemplates.find(t => t.id === templateId)
    if (template) {
      setSteps(template.steps)
      setMetadata({ ...metadata, name: template.name, description: template.description })
      setShowTemplateDialog(false)
      toast({
        title: '템플릿 로드 완료',
        description: `"${template.name}" 템플릿이 적용되었습니다.`,
      })
    }
  }

  return (
    <div className="flex flex-col h-full bg-background">
      {/* Header Section */}
      <div className="border-b bg-card">
        <div className="container mx-auto p-4">
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
                onClick={() => setShowTemplateDialog(true)}
              >
                <FileText className="w-4 h-4 mr-2" />
                템플릿
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={handleLoadWorkflowList}
                disabled={isLoadingList}
              >
                <FolderOpen className="w-4 h-4 mr-2" />
                {isLoadingList ? '불러오는 중...' : '불러오기'}
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={handleLoadExecutionHistory}
                disabled={isLoadingHistory}
              >
                <History className="w-4 h-4 mr-2" />
                {isLoadingHistory ? '조회 중...' : '실행 이력'}
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={() => setShowSchedulerDialog(true)}
              >
                <Timer className="w-4 h-4 mr-2" />
                스케줄러
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={handleRunSimulation}
                disabled={steps.length === 0 || isSimulating}
              >
                <Play className="w-4 h-4 mr-2" />
                {isSimulating ? '실행 중...' : '시뮬레이션'}
              </Button>
              <Button
                variant="default"
                size="sm"
                onClick={handleSaveWorkflow}
                disabled={isSaving}
              >
                <Save className="w-4 h-4 mr-2" />
                {isSaving ? '저장 중...' : '저장'}
              </Button>
              <Button
                variant="ghost"
                size="sm"
              >
                <Settings className="w-4 h-4" />
              </Button>
            </div>
          </div>
        </div>
      </div>

      {/* AI Workflow Generator (Phase 9-2) */}
      <AiGenerator onGenerate={handleAiGenerate} />

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

      {/* 워크플로우 불러오기 Dialog */}
      <Dialog open={showLoadDialog} onOpenChange={setShowLoadDialog}>
        <DialogContent className="max-w-2xl max-h-[80vh] overflow-auto">
          <DialogHeader>
            <DialogTitle>워크플로우 불러오기</DialogTitle>
            <DialogDescription>
              저장된 워크플로우 목록에서 선택하세요.
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-2">
            {workflowList.length === 0 ? (
              <div className="text-center py-12 text-muted-foreground">
                <p>저장된 워크플로우가 없습니다.</p>
                <p className="text-sm mt-2">워크플로우를 만들고 저장 버튼을 눌러보세요.</p>
              </div>
            ) : (
              workflowList.map((workflow) => (
                <Card
                  key={workflow.id}
                  className="p-4 cursor-pointer hover:bg-accent transition-colors"
                  onClick={() => handleLoadWorkflow(workflow.id)}
                >
                  <div className="flex items-start justify-between">
                    <div className="flex-1">
                      <h3 className="font-semibold text-lg">{workflow.name}</h3>
                      {workflow.description && (
                        <p className="text-sm text-muted-foreground mt-1">
                          {workflow.description}
                        </p>
                      )}
                      <div className="flex gap-4 mt-2 text-xs text-muted-foreground">
                        <span>{workflow.stepCount}개 스텝</span>
                        <span>{new Date(workflow.createdAt).toLocaleString('ko-KR')}</span>
                      </div>
                    </div>
                    <Button
                      variant="ghost"
                      size="sm"
                      className="text-destructive hover:text-destructive hover:bg-destructive/10"
                      onClick={(e) => handleDeleteWorkflow(workflow.id, workflow.name, e)}
                    >
                      <Trash2 className="w-4 h-4" />
                    </Button>
                  </div>
                </Card>
              ))
            )}
          </div>
        </DialogContent>
      </Dialog>

      {/* 시뮬레이션 결과 Dialog */}
      <Dialog open={showSimulationDialog} onOpenChange={setShowSimulationDialog}>
        <DialogContent className="max-w-4xl max-h-[80vh] overflow-auto">
          <DialogHeader>
            <DialogTitle>워크플로우 시뮬레이션 결과</DialogTitle>
            <DialogDescription>
              각 스텝의 실행 결과를 확인하세요.
            </DialogDescription>
          </DialogHeader>

          {simulationResult && (
            <div className="space-y-4">
              {/* 전체 실행 결과 요약 */}
              <Card className="p-4 bg-muted/50">
                <div className="flex items-center justify-between">
                  <div>
                    <h3 className="font-semibold text-lg">
                      실행 상태: {
                        simulationResult.status === 'success' ? '✅ 성공' :
                        simulationResult.status === 'partial_success' ? '⚠️ 부분 성공' :
                        '❌ 실패'
                      }
                    </h3>
                    <p className="text-sm text-muted-foreground mt-1">
                      총 실행 시간: {simulationResult.total_execution_time_ms}ms
                    </p>
                  </div>
                  <div className="text-right">
                    <p className="text-sm font-medium">
                      {simulationResult.steps_executed.filter(s => s.status === 'success').length} / {simulationResult.steps_executed.length} 스텝 성공
                    </p>
                  </div>
                </div>
              </Card>

              {/* 각 스텝별 실행 결과 */}
              <div className="space-y-3">
                <h3 className="font-semibold">스텝별 실행 결과</h3>
                {simulationResult.steps_executed.map((step, index) => (
                  <Card key={step.step_id} className={`p-4 ${
                    step.status === 'success' ? 'border-green-500/50' :
                    step.status === 'error' ? 'border-red-500/50' :
                    'border-gray-500/50'
                  }`}>
                    <div className="space-y-3">
                      {/* 스텝 헤더 */}
                      <div className="flex items-start justify-between">
                        <div className="flex-1">
                          <div className="flex items-center gap-2">
                            <span className="text-lg font-semibold">
                              {index + 1}. {step.label}
                            </span>
                            <span className={`text-xs px-2 py-1 rounded ${
                              step.status === 'success' ? 'bg-green-500/20 text-green-700' :
                              step.status === 'error' ? 'bg-red-500/20 text-red-700' :
                              'bg-gray-500/20 text-gray-700'
                            }`}>
                              {step.step_type}
                            </span>
                          </div>
                          <p className="text-xs text-muted-foreground mt-1">
                            실행 시간: {step.execution_time_ms}ms
                          </p>
                        </div>
                        <div>
                          {step.status === 'success' && (
                            <span className="text-2xl">✅</span>
                          )}
                          {step.status === 'error' && (
                            <span className="text-2xl">❌</span>
                          )}
                          {step.status === 'skipped' && (
                            <span className="text-2xl">⏭️</span>
                          )}
                        </div>
                      </div>

                      {/* 입력 데이터 */}
                      <div>
                        <p className="text-sm font-medium mb-1">입력:</p>
                        <pre className="text-xs bg-muted p-2 rounded overflow-x-auto">
                          {JSON.stringify(step.input, null, 2)}
                        </pre>
                      </div>

                      {/* 출력 데이터 */}
                      {step.output && (
                        <div>
                          <p className="text-sm font-medium mb-1">출력:</p>
                          <pre className="text-xs bg-muted p-2 rounded overflow-x-auto">
                            {JSON.stringify(step.output, null, 2)}
                          </pre>
                        </div>
                      )}

                      {/* 에러 메시지 */}
                      {step.error && (
                        <div className="bg-red-500/10 p-3 rounded">
                          <p className="text-sm font-medium text-red-700 mb-1">에러:</p>
                          <p className="text-sm text-red-600">{step.error}</p>
                        </div>
                      )}
                    </div>
                  </Card>
                ))}
              </div>

              {/* 최종 결과 */}
              <Card className="p-4 bg-muted/50">
                <div>
                  <h3 className="font-semibold mb-2">최종 결과 데이터</h3>
                  <pre className="text-xs bg-background p-3 rounded overflow-x-auto">
                    {JSON.stringify(simulationResult.final_result, null, 2)}
                  </pre>
                </div>
              </Card>
            </div>
          )}
        </DialogContent>
      </Dialog>

      {/* 실행 이력 Dialog */}
      <Dialog open={showHistoryDialog} onOpenChange={setShowHistoryDialog}>
        <DialogContent className="max-w-4xl max-h-[80vh] overflow-auto">
          <DialogHeader>
            <DialogTitle>워크플로우 실행 이력</DialogTitle>
            <DialogDescription>
              과거 시뮬레이션 및 실행 결과를 확인하세요.
            </DialogDescription>
          </DialogHeader>

          <div className="flex gap-4">
            {/* 실행 이력 목록 */}
            <div className="w-1/3 space-y-2 border-r pr-4">
              <h3 className="font-semibold text-sm mb-2">실행 목록</h3>
              {executionHistory.length === 0 ? (
                <div className="text-center py-8 text-muted-foreground">
                  <Clock className="w-8 h-8 mx-auto mb-2 opacity-50" />
                  <p className="text-sm">실행 이력이 없습니다.</p>
                </div>
              ) : (
                executionHistory.map((exec) => (
                  <Card
                    key={exec.id}
                    className={`p-3 cursor-pointer hover:bg-accent transition-colors ${
                      selectedExecution?.id === exec.id ? 'bg-accent border-primary' : ''
                    }`}
                    onClick={() => handleLoadExecutionDetail(exec.id)}
                  >
                    <div className="flex items-center gap-2">
                      {exec.status === 'success' && <CheckCircle className="w-4 h-4 text-green-500" />}
                      {exec.status === 'error' && <XCircle className="w-4 h-4 text-red-500" />}
                      {exec.status === 'partial_success' && <AlertCircle className="w-4 h-4 text-yellow-500" />}
                      <div className="flex-1 min-w-0">
                        <p className="text-xs font-medium truncate">{exec.workflow_id}</p>
                        <p className="text-xs text-muted-foreground">
                          {new Date(exec.created_at).toLocaleString('ko-KR')}
                        </p>
                      </div>
                    </div>
                    <div className="flex gap-2 mt-1 text-xs text-muted-foreground">
                      <span>{exec.steps_executed}개 스텝</span>
                      <span>{exec.execution_time_ms}ms</span>
                    </div>
                  </Card>
                ))
              )}
            </div>

            {/* 실행 상세 정보 */}
            <div className="flex-1">
              {selectedExecution ? (
                <div className="space-y-4">
                  {/* 상세 요약 */}
                  <Card className="p-4 bg-muted/50">
                    <div className="flex items-center justify-between">
                      <div>
                        <h3 className="font-semibold">
                          실행 상태: {
                            selectedExecution.status === 'success' ? '✅ 성공' :
                            selectedExecution.status === 'partial_success' ? '⚠️ 부분 성공' :
                            '❌ 실패'
                          }
                        </h3>
                        <p className="text-sm text-muted-foreground">
                          실행 시간: {selectedExecution.execution_time_ms}ms
                        </p>
                        <p className="text-xs text-muted-foreground mt-1">
                          {new Date(selectedExecution.created_at).toLocaleString('ko-KR')}
                        </p>
                      </div>
                    </div>
                  </Card>

                  {/* 스텝별 결과 */}
                  <div className="space-y-2">
                    <h3 className="font-semibold text-sm">스텝별 결과</h3>
                    {Array.isArray(selectedExecution.steps_executed) ? (
                      selectedExecution.steps_executed.map((step, idx) => (
                        <Card key={step.step_id} className={`p-3 ${
                          step.status === 'success' ? 'border-green-500/50' :
                          step.status === 'error' ? 'border-red-500/50' : ''
                        }`}>
                          <div className="flex items-center gap-2">
                            <span className="text-sm font-medium">{idx + 1}. {step.label}</span>
                            <span className={`text-xs px-1.5 py-0.5 rounded ${
                              step.status === 'success' ? 'bg-green-500/20 text-green-700' :
                              step.status === 'error' ? 'bg-red-500/20 text-red-700' : ''
                            }`}>
                              {step.step_type}
                            </span>
                            <span className="ml-auto text-xs text-muted-foreground">
                              {step.execution_time_ms}ms
                            </span>
                          </div>
                          {step.error && (
                            <p className="text-xs text-red-600 mt-1">{step.error}</p>
                          )}
                        </Card>
                      ))
                    ) : (
                      <p className="text-sm text-muted-foreground">스텝 상세 정보 없음</p>
                    )}
                  </div>

                  {/* 최종 결과 */}
                  <Card className="p-3">
                    <h3 className="font-semibold text-sm mb-2">최종 결과</h3>
                    <pre className="text-xs bg-muted p-2 rounded overflow-x-auto max-h-32">
                      {JSON.stringify(selectedExecution.final_result, null, 2)}
                    </pre>
                  </Card>
                </div>
              ) : (
                <div className="flex items-center justify-center h-full text-muted-foreground">
                  <div className="text-center">
                    <History className="w-12 h-12 mx-auto mb-2 opacity-30" />
                    <p>실행 이력을 선택하세요</p>
                  </div>
                </div>
              )}
            </div>
          </div>
        </DialogContent>
      </Dialog>

      {/* 스케줄러 Dialog */}
      <Dialog open={showSchedulerDialog} onOpenChange={setShowSchedulerDialog}>
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle>워크플로우 스케줄러</DialogTitle>
            <DialogDescription>
              Cron 표현식으로 워크플로우 자동 실행을 설정하세요.
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-4">
            {/* 스케줄 상태 */}
            <Card className={`p-4 ${isScheduleActive ? 'bg-green-500/10 border-green-500/50' : 'bg-muted/50'}`}>
              <div className="flex items-center gap-3">
                {isScheduleActive ? (
                  <Power className="w-5 h-5 text-green-500" />
                ) : (
                  <PowerOff className="w-5 h-5 text-muted-foreground" />
                )}
                <div>
                  <p className="font-medium">
                    {isScheduleActive ? '스케줄 활성화됨' : '스케줄 비활성화'}
                  </p>
                  <p className="text-xs text-muted-foreground">
                    {isScheduleActive ? `Cron: ${cronExpression}` : '스케줄이 등록되지 않았습니다.'}
                  </p>
                </div>
              </div>
            </Card>

            {/* Cron 표현식 입력 */}
            <div className="space-y-2">
              <label className="text-sm font-medium">Cron 표현식</label>
              <Input
                value={cronExpression}
                onChange={(e) => setCronExpression(e.target.value)}
                placeholder="0 * * * *"
                disabled={isScheduleActive}
              />
              <p className="text-xs text-muted-foreground">
                예시: "0 * * * *" (매시 정각), "*/5 * * * *" (5분마다), "0 9 * * *" (매일 9시)
              </p>
            </div>

            {/* 버튼 */}
            <div className="flex gap-2">
              {isScheduleActive ? (
                <Button
                  variant="destructive"
                  className="flex-1"
                  onClick={handleUnregisterSchedule}
                  disabled={isScheduleLoading}
                >
                  <PowerOff className="w-4 h-4 mr-2" />
                  {isScheduleLoading ? '처리 중...' : '스케줄 해제'}
                </Button>
              ) : (
                <Button
                  variant="default"
                  className="flex-1"
                  onClick={handleRegisterSchedule}
                  disabled={isScheduleLoading || steps.length === 0}
                >
                  <Power className="w-4 h-4 mr-2" />
                  {isScheduleLoading ? '처리 중...' : '스케줄 등록'}
                </Button>
              )}
            </div>

            {steps.length === 0 && (
              <p className="text-xs text-center text-destructive">
                워크플로우에 스텝을 추가해야 스케줄을 등록할 수 있습니다.
              </p>
            )}
          </div>
        </DialogContent>
      </Dialog>

      {/* 템플릿 선택 Dialog */}
      <Dialog open={showTemplateDialog} onOpenChange={setShowTemplateDialog}>
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle>워크플로우 템플릿</DialogTitle>
            <DialogDescription>
              미리 정의된 템플릿을 선택하여 빠르게 워크플로우를 구성하세요.
            </DialogDescription>
          </DialogHeader>

          <div className="grid gap-4 py-4">
            {workflowTemplates.map((template) => (
              <Card
                key={template.id}
                className="p-4 cursor-pointer hover:bg-accent transition-colors"
                onClick={() => handleLoadTemplate(template.id)}
              >
                <div className="flex items-start justify-between">
                  <div className="flex-1">
                    <h3 className="font-semibold">{template.name}</h3>
                    <p className="text-sm text-muted-foreground mt-1">
                      {template.description}
                    </p>
                    <div className="flex items-center gap-2 mt-2">
                      <span className="text-xs bg-primary/10 text-primary px-2 py-0.5 rounded">
                        {template.steps.length}개 스텝
                      </span>
                      <span className="text-xs text-muted-foreground">
                        {template.steps.map(s => s.type).join(' → ')}
                      </span>
                    </div>
                  </div>
                  <Button variant="ghost" size="sm">
                    적용
                  </Button>
                </div>
              </Card>
            ))}
          </div>
        </DialogContent>
      </Dialog>
    </div>
  )
}
