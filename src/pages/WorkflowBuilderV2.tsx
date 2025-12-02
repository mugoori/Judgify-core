import { useState } from 'react'
import { Plus, Save, Play, Settings, FolderOpen, Trash2, FileText, ChevronDown, ChevronRight } from 'lucide-react'
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
// 시뮬레이션 결과 컴포넌트
import {
  TriggerResult,
  QueryResult,
  CalcResult,
  JudgmentResult,
  ApprovalResult,
  AlertResult,
} from '@/components/simulation'
import { generateMockDataForTemplate, TEMPLATE_NAMES } from '@/lib/mock-data/generators'
import type { StepMockData } from '@/lib/mock-data/types'

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
  const [testData, setTestData] = useState<string>('{\n  "defect_rate": 5,\n  "temperature": 25,\n  "production_count": 100\n}')

  // 저장 중 상태
  const [isSaving, setIsSaving] = useState(false)

  // 템플릿 상태
  const [showTemplateDialog, setShowTemplateDialog] = useState(false)
  const [currentTemplateId, setCurrentTemplateId] = useState<string | null>(null)

  // Mock 데이터 (시뮬레이션용)
  const [mockData, setMockData] = useState<StepMockData[]>([])
  const [showVisualResults, setShowVisualResults] = useState(true)
  const [isApproved, setIsApproved] = useState(false) // 승인 상태

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
        description: response.message,
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
        version: number
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
      setIsApproved(false) // 승인 상태 초기화

      // Tauri 백엔드 API 호출
      const result = await invoke<SimulationResult>('simulate_workflow_v2', {
        request: {
          workflow_id: 'test-workflow-' + Date.now(),
          steps: steps.map(step => ({
            id: step.id,
            type: step.type,
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

  // 워크플로우 템플릿 정의 (퓨어웰 음료㈜ HACCP/CCP 기준)
  // UI Form 필드와 정확히 매치되는 config 키 사용
  const workflowTemplates = [
    {
      id: 'pasteurization-ccp',
      name: '살균 CCP 모니터링',
      description: 'HACCP CCP1: 살균온도 85°C 이상, 유지시간 15초 이상 실시간 모니터링 (SOP-04)',
      steps: [
        { id: 'trigger_1', type: 'TRIGGER' as const, label: '살균 공정 시작', config: {
          triggerType: 'event',
          condition: 'ccp_type == "PASTEURIZATION"',
          description: '살균기(EQ-004) 공정 시작 이벤트 감지. CCP 온도/시간 모니터링 트리거.'
        }},
        { id: 'query_1', type: 'QUERY' as const, label: 'CCP 데이터 조회', config: {
          dataSource: 'database',
          queryType: 'sql',
          query: `SELECT pr.id as check_id, pr.batch_lot_no as lot_id, pr.process_type,
  85.0 as target_temp, pr.actual_temp,
  15 as target_time_sec, pr.actual_time_sec as holding_time_sec,
  CASE WHEN pr.result = 'OK' THEN 1 ELSE 0 END as is_passed,
  pr.start_time as checked_at,
  bm.fg_item_cd as item_cd, i.item_nm
FROM process_result pr
JOIN batch_lot bl ON pr.batch_lot_no = bl.batch_lot_no
JOIN bom_mst bm ON bl.bom_cd = bm.bom_cd
JOIN item_mst i ON bm.fg_item_cd = i.item_cd
WHERE pr.process_type = 'PASTEURIZATION'
  AND pr.start_time >= datetime('now', '-1 hour')
ORDER BY pr.start_time DESC`,
          parameters: '{"process_type": "PASTEURIZATION", "period_hours": 1}',
          resultMapping: 'data.rows'
        }},
        { id: 'calc_1', type: 'CALC' as const, label: '온도/시간 이탈 계산', config: {
          calcType: 'formula',
          formula: 'actual_temp >= 85 AND holding_time_sec >= 15',
          outputField: 'ccp_pass_status'
        }},
        { id: 'judgment_1', type: 'JUDGMENT' as const, label: '살균 CCP 판정', config: {
          judgmentMethod: 'rule',
          ruleExpression: 'actual_temp >= 85 && holding_time_sec >= 15',
          llmPrompt: `살균 CCP 판정 기준 (SOP-04):
- 살균 온도: 85°C 이상 (Critical Limit)
- 유지 시간: 15초 이상
입력 데이터를 검토하여 CCP 기준 충족 여부를 판단하세요.
JSON 형식: {"passed": true/false, "reason": "판단 근거"}`,
          confidenceThreshold: 0.9,
          model: 'claude-3-5-sonnet-20241022',
          temperature: 0.3
        }},
        { id: 'approval_1', type: 'APPROVAL' as const, label: 'HACCP 팀장 확인', config: {
          approvalType: 'conditional',
          approvers: 'haccp_manager@purewell.co.kr, quality_supervisor@purewell.co.kr',
          autoApproveCondition: 'ccp_pass_status == 1',
          timeoutMinutes: 15,
          requireComment: true,
          notifyOnPending: true
        }},
        { id: 'alert_1', type: 'ALERT' as const, label: 'CCP 이탈 알림', config: {
          channels: ['slack', 'email'],
          recipients: '#haccp-alerts, haccp_team@purewell.co.kr',
          subject: '[긴급] 살균 CCP 이탈 발생 - {lot_id}',
          messageTemplate: `🚨 살균 CCP 이탈 알림
LOT: {lot_id} | 제품: {item_nm}
목표: {target_temp}°C/{target_time_sec}초
실측: {actual_temp}°C/{holding_time_sec}초
상태: {is_passed ? '정상' : '이탈'}
SOP-04 절차에 따라 조치 필요`,
          priority: 'critical',
          includeData: true
        }},
      ]
    },
    {
      id: 'metal-detection-ccp',
      name: '금속검출 CCP 검증',
      description: 'HACCP CCP2: Fe 1.5mm, Sus 2.0mm 검출 기준 모니터링 (SOP-09)',
      steps: [
        { id: 'trigger_1', type: 'TRIGGER' as const, label: '금속검출 이벤트', config: {
          triggerType: 'event',
          condition: 'ccp_type == "METAL_DETECTION"',
          description: '금속검출기(EQ-007) 검사 이벤트 감지. 완제품 금속 이물 검출 모니터링.'
        }},
        { id: 'query_1', type: 'QUERY' as const, label: '검출 이력 조회', config: {
          dataSource: 'database',
          queryType: 'sql',
          query: `SELECT qc.qc_no as check_id, qc.lot_no as lot_id, qc.test_type,
  CASE WHEN qc.result = 'PASS' THEN 1 ELSE 0 END as is_passed,
  qc.test_date as checked_at, qc.remark as remarks,
  qc.item_cd, i.item_nm
FROM qc_test qc
JOIN item_mst i ON qc.item_cd = i.item_cd
WHERE qc.test_type = 'FINAL'
  AND qc.test_date >= datetime('now', '-24 hours')
ORDER BY qc.test_date DESC`,
          parameters: '{"test_type": "FINAL", "period_hours": 24}',
          resultMapping: 'data.rows'
        }},
        { id: 'judgment_1', type: 'JUDGMENT' as const, label: '검출 기준 판정', config: {
          judgmentMethod: 'rule',
          ruleExpression: 'is_passed == true',
          llmPrompt: `금속검출 CCP 판정 (SOP-09):
- Fe(철): 1.5mm 이하 검출 시 불합격
- Sus(스테인리스): 2.0mm 이하 검출 시 불합격
검출 결과를 분석하여 합격/불합격 판정 및 조치사항 권고.`,
          confidenceThreshold: 0.95,
          model: 'claude-3-5-sonnet-20241022',
          temperature: 0.2
        }},
        { id: 'alert_1', type: 'ALERT' as const, label: '금속 검출 알림', config: {
          channels: ['slack', 'email'],
          recipients: '#quality-alerts, qc_team@purewell.co.kr',
          subject: '[긴급] 금속 이물 검출 - LOT {lot_id}',
          messageTemplate: `🔴 금속 이물 검출 알림
LOT: {lot_id} | 제품: {item_nm}
판정: {is_passed ? '합격' : '불합격'}
조치: 해당 LOT 즉시 격리, SOP-09 절차 수행`,
          priority: 'critical',
          includeData: true
        }},
      ]
    },
    {
      id: 'material-inspection',
      name: '원료 입고검사',
      description: '원료 입고시 품질검사 및 적합 판정 (SOP-01, SOP-02)',
      steps: [
        { id: 'trigger_1', type: 'TRIGGER' as const, label: '입고 등록', config: {
          triggerType: 'event',
          condition: 'inbound_type == "NORMAL"',
          description: '원료 입고 등록 이벤트. 유산균, 비타민 등 원료 입고 시 품질검사 시작.'
        }},
        { id: 'query_1', type: 'QUERY' as const, label: '입고 원료 정보', config: {
          dataSource: 'database',
          queryType: 'sql',
          query: `SELECT ib.inbound_no, ib.po_no, v.vendor_nm, ib.inbound_date, ib.status,
  ibd.item_cd, i.item_nm, i.item_type, ibd.qty, ibd.lot_no
FROM inbound ib
JOIN inbound_dtl ibd ON ib.inbound_no = ibd.inbound_no
JOIN item_mst i ON ibd.item_cd = i.item_cd
JOIN vendor_mst v ON ib.vendor_cd = v.vendor_cd
WHERE ib.inbound_date >= date('now', '-1 day')
  AND ib.status IN ('PENDING', 'INSPECTING')
ORDER BY ib.inbound_date DESC`,
          parameters: '{"status": ["PENDING", "INSPECTING"]}',
          resultMapping: 'data.rows'
        }},
        { id: 'calc_1', type: 'CALC' as const, label: '검사항목 체크', config: {
          calcType: 'formula',
          formula: '(inspection_passed / total_items) * 100',
          outputField: 'pass_rate'
        }},
        { id: 'judgment_1', type: 'JUDGMENT' as const, label: '입고 적합 판정', config: {
          judgmentMethod: 'hybrid',
          ruleExpression: 'pass_rate >= 100 && status != "REJECTED"',
          llmPrompt: `원료 입고검사 판정 (SOP-01):
- 유산균(RM-001): 생균수, 수분, 유해균
- 비타민(RM-002): 함량, 순도
입고 데이터 검토 후 적합 여부 판정.`,
          confidenceThreshold: 0.85,
          model: 'claude-3-5-sonnet-20241022',
          temperature: 0.4
        }},
        { id: 'approval_1', type: 'APPROVAL' as const, label: 'QC 담당자 승인', config: {
          approvalType: 'manual',
          approvers: 'qc_inspector@purewell.co.kr, quality_manager@purewell.co.kr',
          timeoutMinutes: 120,
          requireComment: true,
          notifyOnPending: true
        }},
        { id: 'alert_1', type: 'ALERT' as const, label: '입고 결과 통보', config: {
          channels: ['email'],
          recipients: 'purchasing@purewell.co.kr, warehouse@purewell.co.kr',
          subject: '[입고검사] {inbound_no} - {item_nm} 결과',
          messageTemplate: `📦 입고검사 결과
입고번호: {inbound_no} | 발주: {po_no}
공급업체: {vendor_nm}
품목: {item_nm} ({item_cd}) | 수량: {qty}
결과: {status} | 합격률: {pass_rate}%`,
          priority: 'medium',
          includeData: false
        }},
      ]
    },
    {
      id: 'release-approval',
      name: '완제품 출하 승인',
      description: '완제품 품질검사 완료 후 출하 승인 프로세스 (SOP-10, SOP-11)',
      steps: [
        { id: 'trigger_1', type: 'TRIGGER' as const, label: '출하 요청', config: {
          triggerType: 'manual',
          description: '영업/물류팀 출하 요청 접수. 품질 적합 확인 후 출하 승인.'
        }},
        { id: 'query_1', type: 'QUERY' as const, label: '완제품 LOT 조회', config: {
          dataSource: 'database',
          queryType: 'sql',
          query: `SELECT f.fg_lot_no, f.fg_item_cd, i.item_nm, f.qty, f.mfg_date, f.exp_date,
  f.qc_status, f.storage_loc, inv.qty as stock_qty
FROM fg_lot f
JOIN item_mst i ON f.fg_item_cd = i.item_cd
LEFT JOIN inventory inv ON f.fg_item_cd = inv.item_cd
WHERE f.qc_status = 'PASSED'
  AND f.exp_date > date('now', '+30 days')
ORDER BY f.mfg_date ASC`,
          parameters: '{"qc_status": "PASSED", "min_shelf_days": 30}',
          resultMapping: 'data.rows'
        }},
        { id: 'query_2', type: 'QUERY' as const, label: 'CCP 이력 확인', config: {
          dataSource: 'database',
          queryType: 'sql',
          query: `SELECT qc.lot_no, qc.test_type,
  SUM(CASE WHEN qc.result = 'PASS' THEN 1 ELSE 0 END) as passed,
  SUM(CASE WHEN qc.result = 'FAIL' THEN 1 ELSE 0 END) as failed
FROM qc_test qc
WHERE qc.lot_no = '{fg_lot_no}'
GROUP BY qc.lot_no, qc.test_type`,
          parameters: '{"fg_lot_no": "{fg_lot_no}"}',
          resultMapping: 'data.ccp_summary'
        }},
        { id: 'judgment_1', type: 'JUDGMENT' as const, label: '출하 적합 판정', config: {
          judgmentMethod: 'rule',
          ruleExpression: 'inspection_status == "PASSED" && failed == 0',
          llmPrompt: `출하 적합 판정 (SOP-10):
조건: 품질검사 합격, CCP 통과, 유통기한 30일+
제품별 특이사항 확인 후 출하 적합 여부 판정.`,
          confidenceThreshold: 0.9,
          model: 'claude-3-5-sonnet-20241022',
          temperature: 0.3
        }},
        { id: 'approval_1', type: 'APPROVAL' as const, label: '품질책임자 최종 승인', config: {
          approvalType: 'manual',
          approvers: 'quality_director@purewell.co.kr, plant_manager@purewell.co.kr',
          timeoutMinutes: 240,
          requireComment: true,
          notifyOnPending: true
        }},
        { id: 'alert_1', type: 'ALERT' as const, label: '출하 승인 완료', config: {
          channels: ['email', 'slack'],
          recipients: 'sales@purewell.co.kr, logistics@purewell.co.kr, #shipping',
          subject: '[출하 승인] {item_nm} - LOT {lot_id}',
          messageTemplate: `✅ 출하 승인 완료
제품: {item_nm} ({item_cd})
LOT: {lot_id} | 수량: {qty}
생산일: {prod_date} | 유통기한: {exp_date}
위치: {location} | 재고: {stock_qty}
물류팀 배송 준비 진행하세요.`,
          priority: 'medium',
          includeData: true
        }},
      ]
    },
    {
      id: 'shelf-life-monitoring',
      name: '유통기한 관리',
      description: '제품별 유통기한 임박 재고 모니터링 및 선입선출 관리',
      steps: [
        { id: 'trigger_1', type: 'TRIGGER' as const, label: '일일 스케줄', config: {
          triggerType: 'scheduled',
          schedule: '0 9 * * *',
          description: '매일 오전 9시 유통기한 임박 재고 점검 자동 실행.'
        }},
        { id: 'query_1', type: 'QUERY' as const, label: '재고 유통기한 조회', config: {
          dataSource: 'database',
          queryType: 'sql',
          query: `SELECT inv.item_cd, i.item_nm, inv.lot_no, inv.qty, inv.exp_date, inv.location,
  julianday(inv.exp_date) - julianday('now') as days_left,
  CASE
    WHEN julianday(inv.exp_date) - julianday('now') <= 7 THEN '긴급'
    WHEN julianday(inv.exp_date) - julianday('now') <= 14 THEN '주의'
    WHEN julianday(inv.exp_date) - julianday('now') <= 30 THEN '임박'
    ELSE '정상'
  END as status
FROM inventory inv
JOIN item_mst i ON inv.item_cd = i.item_cd
WHERE inv.exp_date IS NOT NULL AND inv.qty > 0
ORDER BY inv.exp_date ASC`,
          parameters: '{"min_qty": 0}',
          resultMapping: 'data.rows'
        }},
        { id: 'calc_1', type: 'CALC' as const, label: 'D-Day 계산', config: {
          calcType: 'aggregate',
          aggregateFunction: 'count',
          targetField: 'status',
          outputField: 'expiry_summary'
        }},
        { id: 'judgment_1', type: 'JUDGMENT' as const, label: '임박 재고 판정', config: {
          judgmentMethod: 'rule',
          ruleExpression: 'days_left <= 30',
          llmPrompt: `유통기한 분석:
- 긴급(7일-): 할인판매/샘플 전환
- 주의(14일-): 판촉 우선
- 임박(30일-): FIFO 재확인
재고 현황 분석 후 조치사항 권고.`,
          confidenceThreshold: 0.8,
          model: 'claude-3-5-sonnet-20241022',
          temperature: 0.5
        }},
        { id: 'alert_1', type: 'ALERT' as const, label: '유통기한 임박 알림', config: {
          channels: ['slack', 'email'],
          recipients: '#inventory-alerts, sales@purewell.co.kr, logistics@purewell.co.kr',
          subject: '[유통기한] 긴급 {urgent}건 / 주의 {warning}건',
          messageTemplate: `⏰ 유통기한 임박 현황
🔴 긴급(7일-): {urgent}건
🟠 주의(14일-): {warning}건
🟡 임박(30일-): {near}건

영업팀: 긴급재고 판촉 계획
물류팀: FIFO 점검`,
          priority: 'high',
          includeData: true
        }},
      ]
    },
    {
      id: 'preventive-maintenance',
      name: '설비 예방정비',
      description: '생산설비 정기점검 및 예방정비 일정 관리',
      steps: [
        { id: 'trigger_1', type: 'TRIGGER' as const, label: '정비 스케줄', config: {
          triggerType: 'scheduled',
          schedule: '0 8 * * 1',
          description: '매주 월요일 오전 8시 설비 예방정비 일정 점검.'
        }},
        { id: 'query_1', type: 'QUERY' as const, label: '설비 정비 이력', config: {
          dataSource: 'database',
          queryType: 'sql',
          query: `SELECT lm.line_id as equip_cd, lm.line_name as equip_nm,
  'PRODUCTION' as equip_type, 'PLANT-1' as location, 'ACTIVE' as status,
  date('now', '-7 days') as last_maintenance,
  date('now', '+7 days') as next_maintenance,
  7 as days_until,
  '예정' as maint_status
FROM line_mst lm
UNION ALL
SELECT 'EQ-004' as equip_cd, '살균기' as equip_nm,
  'PASTEURIZER' as equip_type, 'PLANT-1' as location, 'ACTIVE' as status,
  date('now', '-14 days') as last_maintenance,
  date('now', '+1 days') as next_maintenance,
  1 as days_until,
  '긴급' as maint_status
UNION ALL
SELECT 'EQ-007' as equip_cd, '금속검출기' as equip_nm,
  'DETECTOR' as equip_type, 'PLANT-1' as location, 'ACTIVE' as status,
  date('now', '-30 days') as last_maintenance,
  date('now', '-1 days') as next_maintenance,
  -1 as days_until,
  '지연' as maint_status`,
          parameters: '{"status": "ACTIVE"}',
          resultMapping: 'data.rows'
        }},
        { id: 'calc_1', type: 'CALC' as const, label: '정비 우선순위', config: {
          calcType: 'formula',
          formula: 'days_until <= 7',
          outputField: 'needs_attention'
        }},
        { id: 'judgment_1', type: 'JUDGMENT' as const, label: '정비 필요 판정', config: {
          judgmentMethod: 'rule',
          ruleExpression: 'days_until <= 7 || maint_status == "지연"',
          llmPrompt: `설비 정비 판정:
- EQ-004 살균기: 월2회 온도센서 교정
- EQ-005 충진기: 주1회 노즐 세척
- EQ-007 금속검출기: 월1회 감도 검증
지연 및 예정 정비 분석 후 일정 최적화 권고.`,
          confidenceThreshold: 0.75,
          model: 'claude-3-5-sonnet-20241022',
          temperature: 0.5
        }},
        { id: 'approval_1', type: 'APPROVAL' as const, label: '생산팀장 승인', config: {
          approvalType: 'conditional',
          approvers: 'production_manager@purewell.co.kr, plant_engineer@purewell.co.kr',
          autoApproveCondition: 'maint_status != "지연"',
          timeoutMinutes: 1440,
          requireComment: false,
          notifyOnPending: true
        }},
        { id: 'alert_1', type: 'ALERT' as const, label: '정비 일정 통보', config: {
          channels: ['email', 'slack'],
          recipients: 'maintenance@purewell.co.kr, production@purewell.co.kr, #equipment',
          subject: '[주간 정비] {scheduled}건 예정 / {overdue}건 지연',
          messageTemplate: `🔧 주간 설비 정비 일정
🔴 지연: {overdue}건 (즉시 조치)
🟠 긴급(3일-): {urgent}건
🟡 예정(7일-): {scheduled}건

시설팀: 정비 인력 배정
생산팀: 설비 사용 일정 조율`,
          priority: 'high',
          includeData: true
        }},
      ]
    },
    // =============================================
    // 제조업 워크플로우 (MRP, 배합비, 재고예측, 생산량 예측)
    // =============================================
    {
      id: 'mrp-calculation',
      name: 'MRP 자재소요계획',
      description: '생산계획 기반 BOM 전개 → 자재 소요량 계산 → 발주 제안',
      steps: [
        { id: 'trigger_1', type: 'TRIGGER' as const, label: '생산계획 확정', config: {
          triggerType: 'event',
          condition: 'production_order.status == "RELEASED"',
          description: '생산지시가 확정(RELEASED)되면 MRP 계산 시작. BOM 기반 자재 소요량 산출.'
        }},
        { id: 'query_1', type: 'QUERY' as const, label: '생산계획 조회', config: {
          dataSource: 'database',
          queryType: 'sql',
          query: `SELECT po.prod_order_no, po.bom_cd, po.plan_date, po.plan_qty,
  bm.fg_item_cd, bm.batch_size, i.item_nm as fg_name
FROM production_order po
JOIN bom_mst bm ON po.bom_cd = bm.bom_cd
JOIN item_mst i ON bm.fg_item_cd = i.item_cd
WHERE po.status = 'RELEASED'
  AND po.plan_date BETWEEN date('now') AND date('now', '+7 days')
ORDER BY po.plan_date ASC`,
          parameters: '{"status": "RELEASED", "period_days": 7}',
          resultMapping: 'data.production_plans'
        }},
        { id: 'query_2', type: 'QUERY' as const, label: 'BOM 전개 (자재 소요량)', config: {
          dataSource: 'database',
          queryType: 'sql',
          query: `SELECT bd.bom_cd, bd.item_cd, i.item_nm, bd.qty as unit_qty, bd.unit,
  bd.loss_rate, i.item_type,
  (bd.qty * (1 + bd.loss_rate/100)) as gross_qty
FROM bom_dtl bd
JOIN item_mst i ON bd.item_cd = i.item_cd
WHERE bd.bom_cd IN (SELECT bom_cd FROM production_order WHERE status = 'RELEASED')
ORDER BY bd.bom_cd, bd.seq`,
          parameters: '{"bom_cd": "{bom_cd}"}',
          resultMapping: 'data.bom_details'
        }},
        { id: 'query_3', type: 'QUERY' as const, label: '현재 재고 조회', config: {
          dataSource: 'database',
          queryType: 'sql',
          query: `SELECT inv.item_cd, i.item_nm, SUM(inv.qty) as stock_qty,
  SUM(inv.reserved_qty) as reserved_qty,
  SUM(inv.qty - inv.reserved_qty) as available_qty
FROM inventory inv
JOIN item_mst i ON inv.item_cd = i.item_cd
WHERE i.item_type IN ('RM', 'PKG')
GROUP BY inv.item_cd, i.item_nm`,
          parameters: '{"item_type": ["RM", "PKG"]}',
          resultMapping: 'data.inventory'
        }},
        { id: 'calc_1', type: 'CALC' as const, label: '순소요량 계산', config: {
          calcType: 'formula',
          formula: 'gross_requirement - available_qty - scheduled_receipts',
          outputField: 'net_requirement'
        }},
        { id: 'judgment_1', type: 'JUDGMENT' as const, label: '발주 필요 판정', config: {
          judgmentMethod: 'rule',
          ruleExpression: 'net_requirement > 0 && available_qty < safety_stock',
          llmPrompt: `MRP 발주 판정:
- 순소요량(Net Requirement) = 총소요량 - 가용재고 - 입고예정
- 안전재고 미달 시 즉시 발주 권고
- Lead Time 고려하여 발주일자 역산
자재별 상황 분석 후 발주 우선순위 권고.`,
          confidenceThreshold: 0.85,
          model: 'claude-3-5-sonnet-20241022',
          temperature: 0.4
        }},
        { id: 'approval_1', type: 'APPROVAL' as const, label: '구매팀 발주 승인', config: {
          approvalType: 'conditional',
          approvers: 'purchasing@purewell.co.kr, scm_manager@purewell.co.kr',
          autoApproveCondition: 'net_requirement > 0 && unit_price < 1000000',
          timeoutMinutes: 240,
          requireComment: false,
          notifyOnPending: true
        }},
        { id: 'alert_1', type: 'ALERT' as const, label: 'MRP 발주 제안', config: {
          channels: ['email', 'slack'],
          recipients: 'purchasing@purewell.co.kr, production@purewell.co.kr, #mrp-alerts',
          subject: '[MRP] 자재 발주 제안 - {item_count}품목',
          messageTemplate: `📦 MRP 자재소요계획 결과
생산계획: {plan_date} ~ {plan_end_date}
대상 제품: {fg_count}종

발주 필요 자재:
{shortage_items}

총 발주 예상금액: {total_amount}원
구매팀 검토 후 발주 진행하세요.`,
          priority: 'high',
          includeData: true
        }},
      ]
    },
    {
      id: 'bom-cost-management',
      name: '배합비 관리',
      description: '제품별 BOM(배합비) 관리 및 원가 계산, 표준 대비 실적 분석',
      steps: [
        { id: 'trigger_1', type: 'TRIGGER' as const, label: '배합 완료', config: {
          triggerType: 'event',
          condition: 'batch_lot.status == "COMPLETED"',
          description: '배합 LOT 완료 시 실제 투입량 대비 표준 배합비 비교 분석.'
        }},
        { id: 'query_1', type: 'QUERY' as const, label: '표준 BOM 조회', config: {
          dataSource: 'database',
          queryType: 'sql',
          query: `SELECT bm.bom_cd, bm.fg_item_cd, i.item_nm as fg_name, bm.batch_size,
  bd.seq, bd.item_cd as rm_cd, rm.item_nm as rm_name,
  bd.qty as std_qty, bd.unit, bd.loss_rate
FROM bom_mst bm
JOIN bom_dtl bd ON bm.bom_cd = bd.bom_cd
JOIN item_mst i ON bm.fg_item_cd = i.item_cd
JOIN item_mst rm ON bd.item_cd = rm.item_cd
WHERE bm.is_active = 1
ORDER BY bm.bom_cd, bd.seq`,
          parameters: '{"is_active": 1}',
          resultMapping: 'data.standard_bom'
        }},
        { id: 'query_2', type: 'QUERY' as const, label: '실제 투입량 조회', config: {
          dataSource: 'database',
          queryType: 'sql',
          query: `SELECT mi.batch_lot_no, mi.item_cd, i.item_nm,
  mi.plan_qty, mi.actual_qty,
  (mi.actual_qty - mi.plan_qty) as variance,
  ROUND((mi.actual_qty - mi.plan_qty) / mi.plan_qty * 100, 2) as variance_pct
FROM material_issue mi
JOIN item_mst i ON mi.item_cd = i.item_cd
WHERE mi.batch_lot_no = '{batch_lot_no}'
ORDER BY mi.seq`,
          parameters: '{"batch_lot_no": "{batch_lot_no}"}',
          resultMapping: 'data.actual_issue'
        }},
        { id: 'query_3', type: 'QUERY' as const, label: '원자재 단가 조회', config: {
          dataSource: 'database',
          queryType: 'sql',
          query: `SELECT pod.item_cd, i.item_nm,
  AVG(pod.unit_price) as avg_price,
  MAX(pod.unit_price) as max_price,
  MIN(pod.unit_price) as min_price
FROM purchase_order_dtl pod
JOIN item_mst i ON pod.item_cd = i.item_cd
WHERE pod.po_no IN (SELECT po_no FROM purchase_order
  WHERE order_date >= date('now', '-3 months'))
GROUP BY pod.item_cd, i.item_nm`,
          parameters: '{"period_months": 3}',
          resultMapping: 'data.material_prices'
        }},
        { id: 'calc_1', type: 'CALC' as const, label: '원가 계산', config: {
          calcType: 'formula',
          formula: 'actual_qty * avg_price',
          outputField: 'batch_cost'
        }},
        { id: 'calc_2', type: 'CALC' as const, label: '표준 대비 차이', config: {
          calcType: 'formula',
          formula: '(actual_cost - standard_cost) / standard_cost * 100',
          outputField: 'cost_variance_pct'
        }},
        { id: 'judgment_1', type: 'JUDGMENT' as const, label: '배합비 이탈 판정', config: {
          judgmentMethod: 'hybrid',
          ruleExpression: 'ABS(variance_pct) > 5 || ABS(cost_variance_pct) > 3',
          llmPrompt: `배합비 이탈 분석:
- 투입량 기준: ±5% 이내 정상
- 원가 기준: ±3% 이내 정상
이탈 원인 분석:
1. 계량 오차
2. 원료 품질 변동
3. 공정 손실
4. 시스템 오류
배합 데이터 분석 후 개선 방안 권고.`,
          confidenceThreshold: 0.8,
          model: 'claude-3-5-sonnet-20241022',
          temperature: 0.4
        }},
        { id: 'alert_1', type: 'ALERT' as const, label: '배합비 분석 리포트', config: {
          channels: ['email'],
          recipients: 'production@purewell.co.kr, qa@purewell.co.kr, cost_accounting@purewell.co.kr',
          subject: '[배합비] {fg_name} - Batch {batch_lot_no} 분석',
          messageTemplate: `📊 배합비 분석 리포트
제품: {fg_name} ({fg_item_cd})
Batch: {batch_lot_no} | 배합일: {batch_date}

▶ 표준 원가: {standard_cost}원
▶ 실제 원가: {actual_cost}원
▶ 원가 차이: {cost_variance_pct}%

투입량 이탈 품목:
{variance_items}

품질팀 확인 필요.`,
          priority: 'medium',
          includeData: true
        }},
      ]
    },
    {
      id: 'inventory-forecast',
      name: '재고 예측',
      description: '과거 출하 데이터 기반 수요 예측 → 안전재고/발주점 계산',
      steps: [
        { id: 'trigger_1', type: 'TRIGGER' as const, label: '주간 분석', config: {
          triggerType: 'scheduled',
          schedule: '0 7 * * 1',
          description: '매주 월요일 오전 7시 재고 수요 예측 및 발주점 분석 실행.'
        }},
        { id: 'query_1', type: 'QUERY' as const, label: '출하 이력 조회 (3개월)', config: {
          dataSource: 'database',
          queryType: 'sql',
          query: `SELECT sod.item_cd, i.item_nm, i.item_type,
  strftime('%Y-%W', so.order_date) as week,
  SUM(sod.shipped_qty) as weekly_shipped
FROM sales_order_dtl sod
JOIN sales_order so ON sod.so_no = so.so_no
JOIN item_mst i ON sod.item_cd = i.item_cd
WHERE so.order_date >= date('now', '-3 months')
  AND so.status IN ('SHIPPED', 'COMPLETED')
GROUP BY sod.item_cd, i.item_nm, i.item_type, strftime('%Y-%W', so.order_date)
ORDER BY sod.item_cd, week`,
          parameters: '{"period_months": 3, "status": ["SHIPPED", "COMPLETED"]}',
          resultMapping: 'data.shipment_history'
        }},
        { id: 'query_2', type: 'QUERY' as const, label: '현재 재고 현황', config: {
          dataSource: 'database',
          queryType: 'sql',
          query: `SELECT inv.item_cd, i.item_nm,
  SUM(inv.qty) as current_stock,
  SUM(inv.reserved_qty) as reserved,
  SUM(inv.qty - inv.reserved_qty) as available
FROM inventory inv
JOIN item_mst i ON inv.item_cd = i.item_cd
WHERE i.item_type = 'FG'
GROUP BY inv.item_cd, i.item_nm
ORDER BY available ASC`,
          parameters: '{"item_type": "FG"}',
          resultMapping: 'data.current_inventory'
        }},
        { id: 'query_3', type: 'QUERY' as const, label: '입고 예정 조회', config: {
          dataSource: 'database',
          queryType: 'sql',
          query: `SELECT po.prod_order_no, bm.fg_item_cd as item_cd, po.plan_qty,
  po.plan_date as expected_date
FROM production_order po
JOIN bom_mst bm ON po.bom_cd = bm.bom_cd
WHERE po.status IN ('PLANNED', 'RELEASED', 'IN_PROGRESS')
  AND po.plan_date <= date('now', '+14 days')
ORDER BY po.plan_date ASC`,
          parameters: '{"status": ["PLANNED", "RELEASED", "IN_PROGRESS"]}',
          resultMapping: 'data.scheduled_production'
        }},
        { id: 'calc_1', type: 'CALC' as const, label: '평균 주간 출하량', config: {
          calcType: 'aggregate',
          aggregateFunction: 'avg',
          targetField: 'weekly_shipped',
          outputField: 'avg_weekly_demand'
        }},
        { id: 'calc_2', type: 'CALC' as const, label: '안전재고 계산', config: {
          calcType: 'formula',
          formula: 'avg_weekly_demand * 1.5 + std_deviation * 1.65',
          outputField: 'safety_stock'
        }},
        { id: 'calc_3', type: 'CALC' as const, label: '재고 회전일수', config: {
          calcType: 'formula',
          formula: 'current_stock / (avg_weekly_demand / 7)',
          outputField: 'days_of_supply'
        }},
        { id: 'judgment_1', type: 'JUDGMENT' as const, label: '재고 위험 판정', config: {
          judgmentMethod: 'hybrid',
          ruleExpression: 'available < safety_stock || days_of_supply < 7',
          llmPrompt: `재고 예측 분석:
- 안전재고 미달: 즉시 생산/발주 필요
- 7일분 미만: 주의 (생산계획 점검)
- 30일분 초과: 과잉재고 (판촉 검토)
출하 트렌드와 계절성 고려하여 최적 재고 수준 권고.`,
          confidenceThreshold: 0.75,
          model: 'claude-3-5-sonnet-20241022',
          temperature: 0.5
        }},
        { id: 'alert_1', type: 'ALERT' as const, label: '재고 예측 리포트', config: {
          channels: ['email', 'slack'],
          recipients: 'scm@purewell.co.kr, sales@purewell.co.kr, #inventory-alerts',
          subject: '[재고예측] 주간 리포트 - 부족 {shortage}건 / 과잉 {excess}건',
          messageTemplate: `📈 주간 재고 예측 리포트

🔴 재고 부족 위험 ({shortage}품목):
{shortage_items}

🟡 안전재고 임박 ({warning}품목):
{warning_items}

🟢 과잉 재고 ({excess}품목):
{excess_items}

평균 재고회전일: {avg_days_of_supply}일
SCM팀 검토 후 생산/구매 계획 조정.`,
          priority: 'high',
          includeData: true
        }},
      ]
    },
    {
      id: 'production-forecast',
      name: '생산량 예측',
      description: '수주/판매 트렌드 기반 생산량 예측 및 생산계획 제안',
      steps: [
        { id: 'trigger_1', type: 'TRIGGER' as const, label: '월간 분석', config: {
          triggerType: 'scheduled',
          schedule: '0 8 1 * *',
          description: '매월 1일 오전 8시 수주 트렌드 분석 및 익월 생산량 예측.'
        }},
        { id: 'query_1', type: 'QUERY' as const, label: '월별 수주 실적 (12개월)', config: {
          dataSource: 'database',
          queryType: 'sql',
          query: `SELECT sod.item_cd, i.item_nm,
  strftime('%Y-%m', so.order_date) as month,
  SUM(sod.qty) as monthly_order_qty,
  SUM(sod.amount) as monthly_order_amount,
  COUNT(DISTINCT so.so_no) as order_count
FROM sales_order_dtl sod
JOIN sales_order so ON sod.so_no = so.so_no
JOIN item_mst i ON sod.item_cd = i.item_cd
WHERE so.order_date >= date('now', '-12 months')
  AND so.status NOT IN ('CANCELLED', 'DRAFT')
GROUP BY sod.item_cd, i.item_nm, strftime('%Y-%m', so.order_date)
ORDER BY sod.item_cd, month`,
          parameters: '{"period_months": 12}',
          resultMapping: 'data.sales_history'
        }},
        { id: 'query_2', type: 'QUERY' as const, label: '월별 생산 실적 (12개월)', config: {
          dataSource: 'database',
          queryType: 'sql',
          query: `SELECT bm.fg_item_cd as item_cd, i.item_nm,
  strftime('%Y-%m', po.plan_date) as month,
  SUM(po.plan_qty) as planned_qty,
  SUM(po.actual_qty) as actual_qty,
  ROUND(SUM(po.actual_qty) * 100.0 / NULLIF(SUM(po.plan_qty), 0), 1) as achievement_rate
FROM production_order po
JOIN bom_mst bm ON po.bom_cd = bm.bom_cd
JOIN item_mst i ON bm.fg_item_cd = i.item_cd
WHERE po.plan_date >= date('now', '-12 months')
  AND po.status = 'COMPLETED'
GROUP BY bm.fg_item_cd, i.item_nm, strftime('%Y-%m', po.plan_date)
ORDER BY bm.fg_item_cd, month`,
          parameters: '{"period_months": 12, "status": "COMPLETED"}',
          resultMapping: 'data.production_history'
        }},
        { id: 'query_3', type: 'QUERY' as const, label: '설비 가동 현황', config: {
          dataSource: 'database',
          queryType: 'sql',
          query: `SELECT lm.line_id as equip_cd, lm.line_name as equip_nm,
  'PRODUCTION' as equip_type,
  50000 as daily_capacity,
  'ACTIVE' as status
FROM line_mst lm
UNION ALL
SELECT 'EQ-005' as equip_cd, '충진기' as equip_nm,
  'FILLER' as equip_type, 60000 as daily_capacity, 'ACTIVE' as status
UNION ALL
SELECT 'EQ-004' as equip_cd, '살균기' as equip_nm,
  'PASTEURIZER' as equip_type, 80000 as daily_capacity, 'ACTIVE' as status`,
          parameters: '{"equip_type": ["FILLER", "MIXER", "PASTEURIZER"]}',
          resultMapping: 'data.equipment_capacity'
        }},
        { id: 'calc_1', type: 'CALC' as const, label: '수요 성장률', config: {
          calcType: 'formula',
          formula: '(recent_3m_avg - prev_3m_avg) / prev_3m_avg * 100',
          outputField: 'demand_growth_rate'
        }},
        { id: 'calc_2', type: 'CALC' as const, label: '익월 예측 생산량', config: {
          calcType: 'formula',
          formula: 'recent_3m_avg * (1 + demand_growth_rate/100) * seasonal_factor',
          outputField: 'forecast_qty'
        }},
        { id: 'calc_3', type: 'CALC' as const, label: '설비 가동률', config: {
          calcType: 'formula',
          formula: 'forecast_qty / (daily_capacity * working_days) * 100',
          outputField: 'capacity_utilization'
        }},
        { id: 'judgment_1', type: 'JUDGMENT' as const, label: '생산계획 적정성', config: {
          judgmentMethod: 'hybrid',
          ruleExpression: 'capacity_utilization > 90 || capacity_utilization < 60',
          llmPrompt: `생산량 예측 분석:
- 가동률 90% 초과: 설비 증설 또는 외주 검토
- 가동률 60% 미만: 생산라인 통합 또는 판촉 강화
- 적정 가동률: 70-85%
계절성, 트렌드, 특수 이벤트 고려하여 생산계획 최적화 권고.`,
          confidenceThreshold: 0.75,
          model: 'claude-3-5-sonnet-20241022',
          temperature: 0.5
        }},
        { id: 'approval_1', type: 'APPROVAL' as const, label: '생산관리자 검토', config: {
          approvalType: 'manual',
          approvers: 'production_manager@purewell.co.kr, plant_manager@purewell.co.kr',
          timeoutMinutes: 480,
          requireComment: true,
          notifyOnPending: true
        }},
        { id: 'alert_1', type: 'ALERT' as const, label: '생산량 예측 리포트', config: {
          channels: ['email', 'slack'],
          recipients: 'production@purewell.co.kr, sales@purewell.co.kr, management@purewell.co.kr, #production-planning',
          subject: '[생산예측] {month} 월간 생산계획 제안',
          messageTemplate: `📊 월간 생산량 예측 리포트
기준월: {analysis_month} | 예측월: {forecast_month}

▶ 수요 트렌드:
  - 최근 3개월 평균 수주: {recent_avg}병
  - 전년 동기 대비: {yoy_growth}%
  - 예측 수요: {forecast_demand}병

▶ 생산계획 제안:
{production_plan}

▶ 설비 가동률: {capacity_utilization}%
▶ 추가 필요 용량: {additional_capacity}병

생산관리팀 검토 후 확정.`,
          priority: 'high',
          includeData: true
        }},
      ]
    },
  ]

  // 템플릿별 샘플 테스트 데이터
  const templateSampleData: Record<string, object> = {
    'pasteurization-ccp': { lot_id: 'LOT-2024-1201-001', actual_temp: 86.5, holding_time_sec: 18, target_temp: 85, target_time: 15 },
    'metal-detection-ccp': { lot_id: 'LOT-2024-1201-002', metal_detected: false, detection_count: 0 },
    'material-inspection': { inbound_no: 'IN-2024-1201-001', vendor: 'V-001', items: ['RM-WATER', 'RM-BASE-A', 'RM-SUGAR'], pass_count: 15, total_count: 15 },
    'release-approval': { request_id: 'REQ-2024-1201-001', customer: 'CUST-003', available_lots: 8, total_qty: 45000 },
    'shelf-life-monitoring': { urgent_count: 3, warning_count: 8, near_count: 15, normal_count: 130 },
    'preventive-maintenance': { overdue: 1, urgent: 2, scheduled: 5, normal: 4 },
    'mrp-calculation': { gross_requirement: { 'RM-WATER': 15000, 'RM-BASE-A': 500, 'RM-SUGAR': 2500 }, available: { 'RM-WATER': 20000, 'RM-BASE-A': 200, 'RM-SUGAR': 3000 } },
    'bom-cost-management': { batch_lot_no: 'B-20241201-001', bom_cd: 'BOM-PB-100', standard_cost: 125000, actual_cost: 128500 },
    'inventory-forecast': { avg_weekly_demand: 12500, safety_stock: 21803, days_of_supply: 43.7, available_stock: 78000 },
    'production-forecast': { forecast_qty: 135625, capacity_utilization: 75.3, demand_growth_rate: 8.5 },
  }

  // 템플릿 로드 핸들러
  const handleLoadTemplate = (templateId: string) => {
    const template = workflowTemplates.find(t => t.id === templateId)
    if (template) {
      setSteps(template.steps)
      setMetadata({ ...metadata, name: template.name, description: template.description })
      // 템플릿에 맞는 샘플 테스트 데이터 자동 설정
      const sampleData = templateSampleData[templateId] || {}
      setTestData(JSON.stringify(sampleData, null, 2))
      // 시뮬레이션용 Mock 데이터 자동 생성
      setCurrentTemplateId(templateId)
      setMockData(generateMockDataForTemplate(templateId))
      setShowTemplateDialog(false)
      toast({
        title: '템플릿 로드 완료',
        description: `"${template.name}" 템플릿이 적용되었습니다. 상단의 시뮬레이션 버튼을 눌러 결과를 확인하세요.`,
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
                onClick={() => {
                  setSimulationResult(null)
                  setShowSimulationDialog(true)
                  setIsApproved(false) // 승인 상태 초기화
                }}
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
                {(provided) => (
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
                        {(provided) => (
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
      <Dialog open={showSimulationDialog} onOpenChange={(open) => {
        setShowSimulationDialog(open)
        if (!open) {
          // Dialog 닫을 때 Mock 데이터 새로 생성 (다음 실행시 다른 값 표시)
          if (currentTemplateId) {
            setMockData(generateMockDataForTemplate(currentTemplateId))
          }
        }
      }}>
        <DialogContent className="max-w-4xl max-h-[85vh] overflow-auto">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-3">
              워크플로우 시뮬레이션 결과
              {currentTemplateId && TEMPLATE_NAMES[currentTemplateId as keyof typeof TEMPLATE_NAMES] && (
                <span className="text-sm font-normal text-muted-foreground px-2 py-1 bg-primary/10 rounded">
                  {TEMPLATE_NAMES[currentTemplateId as keyof typeof TEMPLATE_NAMES]}
                </span>
              )}
            </DialogTitle>
            <DialogDescription>
              각 스텝의 실행 결과를 확인하세요.
            </DialogDescription>
          </DialogHeader>

          {/* 시각적 결과 vs JSON 결과 토글 */}
          {mockData.length > 0 && (
            <div className="flex items-center gap-2 mb-4">
              <Button
                variant={showVisualResults ? 'default' : 'outline'}
                size="sm"
                onClick={() => setShowVisualResults(true)}
              >
                시각적 결과
              </Button>
              <Button
                variant={!showVisualResults ? 'default' : 'outline'}
                size="sm"
                onClick={() => setShowVisualResults(false)}
              >
                JSON 결과
              </Button>
              <Button
                variant="ghost"
                size="sm"
                className="ml-auto"
                onClick={() => {
                  if (currentTemplateId) {
                    setMockData(generateMockDataForTemplate(currentTemplateId))
                    toast({
                      title: 'Mock 데이터 새로고침',
                      description: '새로운 시뮬레이션 데이터가 생성되었습니다.',
                    })
                  }
                }}
              >
                🔄 새로고침
              </Button>
            </div>
          )}

          {/* 시각적 카드형 결과 */}
          {showVisualResults && mockData.length > 0 && (
            <div className="space-y-4">
              {/* 전체 실행 요약 */}
              <Card className="p-4 bg-gradient-to-r from-green-500/10 to-blue-500/10 border-green-500/30">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    <span className="text-3xl">✅</span>
                    <div>
                      <h3 className="font-semibold text-lg text-green-700 dark:text-green-400">
                        시뮬레이션 완료
                      </h3>
                      <p className="text-sm text-muted-foreground">
                        {steps.length}개 스텝이 정상적으로 실행되었습니다.
                      </p>
                    </div>
                  </div>
                  <div className="text-right">
                    <p className="text-2xl font-bold text-green-600">{steps.length}/{steps.length}</p>
                    <p className="text-xs text-muted-foreground">스텝 성공</p>
                  </div>
                </div>
              </Card>

              {/* 스텝별 시각적 결과 */}
              <div className="space-y-3">
                <h3 className="font-semibold text-lg">스텝별 실행 결과</h3>
                {steps.map((step, index) => {
                  // 해당 타입의 Mock 데이터 추출
                  const triggerData = step.type === 'TRIGGER' ? mockData.find(m => m.trigger)?.trigger : undefined
                  const queryData = step.type === 'QUERY' ? mockData.find(m => m.query)?.query : undefined
                  const calcData = step.type === 'CALC' ? mockData.find(m => m.calc)?.calc : undefined
                  const judgmentData = step.type === 'JUDGMENT' ? mockData.find(m => m.judgment)?.judgment : undefined
                  const approvalData = step.type === 'APPROVAL' ? mockData.find(m => m.approval)?.approval : undefined
                  const alertData = step.type === 'ALERT' ? mockData.find(m => m.alert)?.alert : undefined

                  return (
                    <div key={step.id} className="relative">
                      {/* 연결선 */}
                      {index < steps.length - 1 && (
                        <div className="absolute left-6 top-full w-0.5 h-3 bg-border z-0" />
                      )}

                      {/* 스텝 결과 카드 */}
                      <Card className="p-4 border-l-4 border-l-primary relative z-10">
                        <div className="flex items-start gap-3">
                          {/* 스텝 번호 */}
                          <div className="w-8 h-8 rounded-full bg-primary text-primary-foreground flex items-center justify-center text-sm font-bold shrink-0">
                            {index + 1}
                          </div>

                          {/* 스텝 내용 */}
                          <div className="flex-1 space-y-3">
                            <div className="flex items-center gap-2">
                              <span className="font-semibold">{step.label}</span>
                              <span className="text-xs px-2 py-0.5 rounded bg-muted">
                                {step.type}
                              </span>
                            </div>

                            {/* 스텝 타입별 결과 렌더링 */}
                            {step.type === 'TRIGGER' && triggerData && (
                              <TriggerResult data={triggerData} />
                            )}
                            {step.type === 'QUERY' && queryData && (
                              <QueryResult data={queryData} />
                            )}
                            {step.type === 'CALC' && calcData && (
                              <CalcResult data={calcData} />
                            )}
                            {step.type === 'JUDGMENT' && judgmentData && (
                              <JudgmentResult data={judgmentData} />
                            )}
                            {step.type === 'APPROVAL' && approvalData && (
                              <ApprovalResult
                                data={approvalData}
                                onApprove={() => {
                                  setIsApproved(true)
                                  toast({
                                    title: '알림 전송 완료',
                                    description: '알람이 전송 됐습니다.',
                                    duration: 3000,
                                  })
                                }}
                              />
                            )}
                            {step.type === 'ALERT' && alertData && (
                              <AlertResult
                                data={alertData}
                                isPending={steps.some(s => s.type === 'APPROVAL') && !isApproved}
                              />
                            )}
                          </div>
                        </div>
                      </Card>
                    </div>
                  )
                })}
              </div>

              {/* 닫기 버튼 */}
              <div className="flex justify-end pt-4">
                <Button onClick={() => setShowSimulationDialog(false)}>
                  닫기
                </Button>
              </div>
            </div>
          )}

          {/* 기존 JSON 결과 (토글 시) */}
          {!showVisualResults && (
            <>
              {/* 테스트 데이터 입력 영역 */}
              {!simulationResult && (
                <div className="space-y-4">
                  <div className="space-y-2">
                    <label className="text-sm font-medium">테스트 데이터 (JSON)</label>
                    <textarea
                      className="w-full h-40 p-3 font-mono text-sm border rounded-md bg-muted/30 focus:outline-none focus:ring-2 focus:ring-primary"
                      value={testData}
                      onChange={(e) => setTestData(e.target.value)}
                      placeholder='{ "defect_rate": 5, "temperature": 25 }'
                    />
                    <p className="text-xs text-muted-foreground">
                      워크플로우에서 사용할 테스트 데이터를 JSON 형식으로 입력하세요.
                    </p>
                  </div>
                  <div className="flex justify-end gap-2">
                    <Button variant="outline" onClick={() => setShowSimulationDialog(false)}>
                      취소
                    </Button>
                    <Button onClick={handleRunSimulation} disabled={isSimulating}>
                      {isSimulating ? '실행 중...' : '시뮬레이션 실행'}
                    </Button>
                  </div>
                </div>
              )}

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

                          {/* 입력/출력 토글 */}
                          <details className="group">
                            <summary className="flex items-center gap-1 cursor-pointer text-sm font-medium text-muted-foreground hover:text-foreground">
                              <ChevronRight className="w-4 h-4 group-open:hidden" />
                              <ChevronDown className="w-4 h-4 hidden group-open:block" />
                              입력/출력 보기
                            </summary>
                            <div className="mt-2 space-y-2">
                              {/* 입력 데이터 */}
                              <div>
                                <p className="text-sm font-medium mb-1">입력:</p>
                                <pre className="text-xs bg-muted p-2 rounded overflow-x-auto max-h-32 overflow-y-auto">
                                  {JSON.stringify(step.input, null, 2)}
                                </pre>
                              </div>

                              {/* 출력 데이터 */}
                              {step.output && (
                                <div>
                                  <p className="text-sm font-medium mb-1">출력:</p>
                                  <pre className="text-xs bg-muted p-2 rounded overflow-x-auto max-h-32 overflow-y-auto">
                                    {JSON.stringify(step.output, null, 2)}
                                  </pre>
                                </div>
                              )}
                            </div>
                          </details>

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
                    <details className="group">
                      <summary className="flex items-center gap-1 cursor-pointer font-semibold hover:text-primary">
                        <ChevronRight className="w-4 h-4 group-open:hidden" />
                        <ChevronDown className="w-4 h-4 hidden group-open:block" />
                        최종 결과 데이터
                      </summary>
                      <pre className="mt-2 text-xs bg-background p-3 rounded overflow-x-auto max-h-48 overflow-y-auto">
                        {JSON.stringify(simulationResult.final_result, null, 2)}
                      </pre>
                    </details>
                  </Card>
                </div>
              )}
            </>
          )}

          {/* Mock 데이터가 없는 경우 (템플릿 미선택) */}
          {mockData.length === 0 && !simulationResult && (
            <div className="text-center py-8">
              <p className="text-muted-foreground mb-4">
                시뮬레이션 Mock 데이터가 없습니다.<br />
                템플릿을 선택하면 자동으로 Mock 데이터가 생성됩니다.
              </p>
              <Button variant="outline" onClick={() => {
                setShowSimulationDialog(false)
                setShowTemplateDialog(true)
              }}>
                템플릿 선택하기
              </Button>
            </div>
          )}
        </DialogContent>
      </Dialog>

      {/* 템플릿 선택 Dialog */}
      <Dialog open={showTemplateDialog} onOpenChange={setShowTemplateDialog}>
        <DialogContent className="max-w-3xl max-h-[90vh] flex flex-col">
          <DialogHeader>
            <DialogTitle>워크플로우 템플릿</DialogTitle>
            <DialogDescription>
              템플릿을 적용한 후 상단의 시뮬레이션 버튼으로 가상 결과를 확인하세요.
            </DialogDescription>
          </DialogHeader>

          <div className="grid gap-4 py-4 overflow-y-auto flex-1 pr-2">
            {workflowTemplates.map((template) => (
              <Card key={template.id} className="hover:border-primary/50 transition-colors">
                <div className="p-4">
                  <div className="flex items-start justify-between gap-4">
                    <div className="flex-1 min-w-0">
                      <h3 className="font-semibold">{template.name}</h3>
                      <p className="text-sm text-muted-foreground mt-1 break-words">
                        {template.description}
                      </p>
                      <div className="flex flex-wrap items-center gap-2 mt-2">
                        <span className="text-xs bg-primary/10 text-primary px-2 py-0.5 rounded whitespace-nowrap">
                          {template.steps.length}개 스텝
                        </span>
                        <span className="text-xs text-muted-foreground break-all">
                          {template.steps.map(s => s.type).join(' → ')}
                        </span>
                      </div>
                    </div>
                    <Button
                      variant="default"
                      size="sm"
                      className="shrink-0"
                      onClick={() => handleLoadTemplate(template.id)}
                    >
                      적용
                    </Button>
                  </div>
                </div>
              </Card>
            ))}
          </div>
        </DialogContent>
      </Dialog>

    </div>
  )
}
