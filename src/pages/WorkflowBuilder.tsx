import { useState, useCallback, useMemo, useRef, useEffect } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import ReactFlow, {
  Node,
  Edge,
  addEdge,
  Background,
  Connection,
  useNodesState,
  useEdgesState,
  MiniMap,
  ConnectionLineType,
  ReactFlowInstance,
} from 'reactflow';
import 'reactflow/dist/style.css';
import { getAllWorkflows, createWorkflow, updateWorkflow, deleteWorkflow, executeJudgment, type JudgmentResult } from '@/lib/tauri-api';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Textarea } from '@/components/ui/textarea';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Label } from '@/components/ui/label';
import { Badge } from '@/components/ui/badge';
import { Save, Play, CheckCircle, XCircle, Sparkles, FileText, AlertCircle, Zap, RefreshCw, Workflow, Wand2, Trash2, AlertTriangle, ChevronDown, ChevronUp, Bug, Database, FileCode, Brain, Bell, BarChart, ZoomIn, ZoomOut, Maximize2, RotateCcw } from 'lucide-react';
import CustomNode from '@/components/workflow/CustomNode';
import { NodeType } from '@/types/workflow';
import { NodeEditPanel } from '@/components/workflow/NodeEditPanel';
import { SimulationPanel } from '@/components/workflow/SimulationPanel';
import EmptyState from '@/components/EmptyState';
import { WorkflowGenerator, testScenarios, type GenerationMode } from '@/lib/workflow-generator';
import { ClaudeProvider } from '@/lib/claude-provider';
import { useToast } from '@/components/ui/use-toast';
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group';
import { HelpCircle } from 'lucide-react';
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog';

const initialNodes: Node[] = [
  {
    id: '1',
    type: 'input',
    data: { label: '시작' },
    position: { x: 250, y: 0 },
  },
];

const initialEdges: Edge[] = [];

export default function WorkflowBuilder() {
  const queryClient = useQueryClient();
  const { toast } = useToast();
  const [reactFlowInstance, setReactFlowInstance] = useState<ReactFlowInstance | null>(null);
  const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes);
  const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges);
  const [workflowName, setWorkflowName] = useState('새 워크플로우');
  const [ruleExpression, setRuleExpression] = useState('');
  const [selectedWorkflowId, setSelectedWorkflowId] = useState<string | null>(null);
  const [saveSuccess, setSaveSuccess] = useState(false);

  // Execute workflow state
  const [showExecutePanel, setShowExecutePanel] = useState(false);
  const [executeWorkflowId, setExecuteWorkflowId] = useState<string>('');
  const [inputData, setInputData] = useState('{\n  "temperature": 95,\n  "vibration": 45\n}');
  const [executionResult, setExecutionResult] = useState<JudgmentResult | null>(null);
  const executeSelectRef = useRef<HTMLSelectElement>(null);

  // AI workflow generation state
  const [showAIPanel, setShowAIPanel] = useState(false);
  const [aiDescription, setAiDescription] = useState('');
  const [isGenerating, setIsGenerating] = useState(false);
  const [generationMode, setGenerationMode] = useState<GenerationMode>('hybrid');
  const [claudeApiKey, setClaudeApiKey] = useState<string>(() => {
    // Phase 30: Check Vite env variable first, then localStorage
    // @ts-ignore - Vite env variables
    const viteKey = import.meta.env?.VITE_ANTHROPIC_API_KEY;
    if (viteKey) {
      console.log('[Phase 30] Using API key from VITE_ANTHROPIC_API_KEY');
      return viteKey;
    }
    // Load from localStorage
    return localStorage.getItem('claude_api_key') || '';
  });

  // Delete workflow state
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [workflowToDelete, setWorkflowToDelete] = useState<{ id: string | string[]; name: string } | null>(null);

  // Edit mode state for bulk delete
  const [isEditMode, setIsEditMode] = useState(false);
  const [selectedWorkflows, setSelectedWorkflows] = useState<Set<string>>(new Set());

  // Delete node state (for canvas nodes, not workflows)
  const [deleteNodeDialogOpen, setDeleteNodeDialogOpen] = useState(false);
  const [nodesToDelete, setNodesToDelete] = useState<Node[]>([]);

  // Node editing state
  const [selectedNode, setSelectedNode] = useState<Node | null>(null);

  // Simulation state
  const [showSimulationPanel, setShowSimulationPanel] = useState(false);
  const simulationInitialData: Record<string, any> = {
    temperature: 95,
    vibration: 45,
    status: 'normal',
    count: 10,
    pressure: 100.0,
  };

  // Focus on execute panel when opened
  useEffect(() => {
    if (showExecutePanel && executeSelectRef.current) {
      executeSelectRef.current.focus();
    }
  }, [showExecutePanel]);

  // Handle Delete key for node deletion
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      // Delete 또는 Backspace 키 감지
      if (event.key === 'Delete' || event.key === 'Backspace') {
        // 입력 필드에서는 작동 안 함
        const target = event.target as HTMLElement;
        if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') {
          return;
        }

        // 선택된 노드 필터링
        const selectedNodes = nodes.filter(node => node.selected);

        if (selectedNodes.length > 0) {
          event.preventDefault(); // 브라우저 뒤로가기 방지
          setNodesToDelete(selectedNodes);
          setDeleteNodeDialogOpen(true);
        }
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [nodes]);

  const { data: workflows } = useQuery({
    queryKey: ['workflows'],
    queryFn: getAllWorkflows,
  });

  const createMutation = useMutation({
    mutationFn: createWorkflow,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['workflows'] });
      setSaveSuccess(true);
      setTimeout(() => setSaveSuccess(false), 3000);
    },
  });

  const updateMutation = useMutation({
    mutationFn: updateWorkflow,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['workflows'] });
      setSaveSuccess(true);
      setTimeout(() => setSaveSuccess(false), 3000);
    },
  });

  const executeMutation = useMutation({
    mutationFn: (data: { workflow_id: string; input_data: any }) =>
      executeJudgment({ workflow_id: data.workflow_id, input_data: data.input_data }),
    onSuccess: (result) => {
      setExecutionResult(result);
      queryClient.invalidateQueries({ queryKey: ['recent-judgments'] });
    },
    onError: (error: Error) => {
      alert(`실행 오류: ${error.message}`);
    },
  });

  const deleteMutation = useMutation({
    mutationFn: deleteWorkflow,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['workflows'] });
      setDeleteDialogOpen(false);

      // 삭제한 워크플로우가 현재 선택된 것이라면 초기화
      if (typeof workflowToDelete?.id === 'string' && selectedWorkflowId === workflowToDelete?.id) {
        createNewWorkflow();
      }

      // 편집 모드 종료
      setIsEditMode(false);
      setSelectedWorkflows(new Set());
      setWorkflowToDelete(null);

      // Toast 알림 (alert 대신)
      toast({
        title: "워크플로우 삭제됨",
        description: "선택한 워크플로우가 삭제되었습니다.",
        duration: 3000,
      });
    },
    onError: (error: Error) => {
      toast({
        variant: "destructive",
        title: "삭제 실패",
        description: error.message,
        duration: 5000,
      });
    },
  });

  const onConnect = useCallback(
    (params: Connection) => setEdges((eds) => addEdge(params, eds)),
    [setEdges]
  );

  // Memoize node types to prevent re-renders
  const nodeTypes = useMemo(() => ({
    custom: CustomNode,
  }), []);

  const addNode = useCallback((
    nodeType: NodeType,
    label: string,
    description?: string,
    additionalData?: Record<string, any>
  ) => {
    const newNode: Node = {
      id: `${Date.now()}`,
      type: 'custom',
      data: {
        label,
        type: nodeType,
        description,
        ...additionalData, // rule, ruleExpression, prompt, service, channel 등
      },
      position: { x: 250 + Math.random() * 100, y: 100 + nodes.length * 80 },
    };
    setNodes((nds) => [...nds, newNode]);
  }, [nodes.length, setNodes]);

  const createNewWorkflow = useCallback(() => {
    setSelectedWorkflowId(null);
    setWorkflowName('새 워크플로우');
    setRuleExpression('');
    setNodes(initialNodes);
    setEdges(initialEdges);
    setExecutionResult(null);
  }, [setNodes, setEdges]);

  const handleSave = () => {
    const definition = {
      nodes,
      edges,
    };

    if (selectedWorkflowId) {
      updateMutation.mutate({
        id: selectedWorkflowId,
        name: workflowName,
        definition,
        rule_expression: ruleExpression || undefined,
      });
    } else {
      createMutation.mutate({
        name: workflowName,
        definition,
        rule_expression: ruleExpression || undefined,
      });
    }
  };

  const loadWorkflow = (workflow: any) => {
    setSelectedWorkflowId(workflow.id);
    setWorkflowName(workflow.name);
    setRuleExpression(workflow.rule_expression || '');
    if (workflow.definition.nodes) {
      setNodes(workflow.definition.nodes);
      setEdges(workflow.definition.edges || []);
    }
  };

  const handleExecute = () => {
    if (!executeWorkflowId) {
      alert('워크플로우를 선택해주세요.');
      return;
    }

    try {
      const parsedData = JSON.parse(inputData);
      executeMutation.mutate({
        workflow_id: executeWorkflowId,
        input_data: parsedData,
      });
    } catch (error) {
      alert('입력 데이터 JSON 형식이 올바르지 않습니다.');
    }
  };

  const useSampleData = () => {
    setInputData('{\n  "temperature": 95,\n  "vibration": 45\n}');
  };

  const handleGenerateAIWorkflow = async () => {
    if (!aiDescription.trim()) {
      toast({
        variant: 'destructive',
        title: '입력 필요',
        description: '워크플로우 설명을 입력해주세요.',
      });
      return;
    }

    // LLM 모드일 때 API 키 확인 (3-tier fallback)
    // Hybrid 모드는 API 키 없으면 Pattern만 사용
    let finalApiKey = '';
    if (generationMode === 'llm') {
      // 1순위: React state
      finalApiKey = claudeApiKey.trim();

      // 2순위: localStorage (Settings에서 저장한 값)
      if (!finalApiKey) {
        finalApiKey = localStorage.getItem('claude_api_key') || '';
      }

      // 3순위: DOM value 직접 읽기 (E2E 테스트 전용)
      if (!finalApiKey) {
        const inputElement = document.getElementById('claude-api-key') as HTMLInputElement;
        finalApiKey = inputElement?.value?.trim() || '';
      }

      if (!finalApiKey) {
        toast({
          variant: 'destructive',
          title: 'API 키 필요',
          description: 'LLM 모드는 Claude API 키가 필요합니다. Hybrid 모드를 사용하거나 API 키를 입력하세요.',
          action: (
            <button
              onClick={() => setGenerationMode('hybrid')}
              className="inline-flex h-8 shrink-0 items-center justify-center rounded-md border bg-transparent px-3 text-sm font-medium"
            >
              Hybrid 모드로 전환
            </button>
          ),
        });
        return;
      }

      // React state 동기화 (DOM에서 가져온 경우)
      if (finalApiKey !== claudeApiKey) {
        setClaudeApiKey(finalApiKey);
      }
    }

    setIsGenerating(true);
    const startTime = Date.now();

    try {
      // API 키 결정 (Hybrid 모드용)
      const apiKeyForGeneration = finalApiKey || claudeApiKey.trim();

      // WorkflowGenerator 초기화
      const llmProvider = (generationMode !== 'pattern' && apiKeyForGeneration)
        ? new ClaudeProvider()
        : undefined;

      const generator = new WorkflowGenerator(llmProvider);

      // 디버깅: 생성 시작 로그
      console.log('[AI Gen] Starting generation:', {
        mode: generationMode,
        hasLLM: !!llmProvider,
        hasApiKey: !!apiKeyForGeneration,
        description: aiDescription.substring(0, 50) + '...'
      });

      // 워크플로우 생성
      const result = await generator.generate(aiDescription, {
        mode: generationMode,
        llmConfig: apiKeyForGeneration ? {
          apiKey: apiKeyForGeneration,
          model: 'claude-3-5-sonnet-20241022',
          maxTokens: 4096,
          temperature: 0.7,
        } : undefined,
      });

      // React Flow 노드로 변환
      const flowNodes: Node[] = result.nodes.map((node: any, index) => ({
        id: node.id || `node-${index}`,
        type: 'custom',
        data: {
          label: node.label,
          ...(node.config || {}),  // API response config를 data로 변환
        },
        position: node.position || { x: 100 + index * 250, y: 100 },
        draggable: true,
        selectable: true,
      }));

      // 디버깅: 생성 성공 로그
      console.log('[AI Gen] Generation successful:', {
        nodeCount: flowNodes.length,
        edgeCount: result.edges.length,
        metadata: result.metadata
      });

      // 워크플로우 적용
      setWorkflowName(result.name);
      setNodes(flowNodes);
      setEdges(result.edges);

      // 패널 닫고 초기화
      setShowAIPanel(false);
      setAiDescription('');

      // 성공 Toast (메타데이터 포함)
      const generationTime = Date.now() - startTime;
      toast({
        title: '✨ 워크플로우 생성 완료',
        description: (
          <div className="space-y-1 text-sm">
            <p>• 이름: {result.name}</p>
            <p>• 모드: {result.metadata?.generationMode || generationMode}</p>
            <p>• LLM 사용: {result.metadata?.usedLLM ? '예' : '아니오'}</p>
            <p>• 생성 시간: {generationTime}ms</p>
            {result.metadata?.confidence && (
              <p>• 신뢰도: {Math.round(result.metadata.confidence * 100)}%</p>
            )}
          </div>
        ),
        duration: 5000,
      });

    } catch (error: any) {
      // 디버깅: 에러 발생 로그
      console.error('[AI Gen] CAUGHT ERROR:', {
        message: error.message,
        stack: error.stack,
        fullError: error
      });

      const errorMessage = error.message || '워크플로우 생성 중 오류가 발생했습니다.';

      // 에러 타입별 처리
      let description = errorMessage;
      let action = undefined;

      if (errorMessage.includes('Invalid Claude API key')) {
        description = 'API 키가 유효하지 않습니다. 확인 후 다시 시도해주세요.';
        action = (
          <button
            onClick={() => setClaudeApiKey('')}
            className="inline-flex h-8 shrink-0 items-center justify-center rounded-md border bg-transparent px-3 text-sm font-medium"
          >
            API 키 재입력
          </button>
        );
      } else if (errorMessage.includes('rate limit') || errorMessage.includes('429')) {
        description = 'API 사용량 한도 초과. 잠시 후 다시 시도해주세요.';
      } else if (errorMessage.includes('timeout') || errorMessage.includes('시간 초과')) {
        description = '생성 시간 초과 (30초). Pattern 모드로 재시도해보세요.';
        action = (
          <button
            onClick={() => setGenerationMode('pattern')}
            className="inline-flex h-8 shrink-0 items-center justify-center rounded-md border bg-transparent px-3 text-sm font-medium"
          >
            Pattern 재시도
          </button>
        );
      }

      toast({
        variant: 'destructive',
        title: '생성 실패',
        description,
        action,
        duration: 7000,
      });

      console.error('Workflow generation error:', error);
    } finally {
      setIsGenerating(false);
    }
  };

  const useSampleScenario = (scenario: string) => {
    setAiDescription(scenario);
  };

  const handleDeleteClick = (id: string, name: string) => {
    setWorkflowToDelete({ id, name });
    setDeleteDialogOpen(true);
  };

  const handleBulkDeleteClick = () => {
    const selectedNames = workflows
      ?.filter(w => selectedWorkflows.has(w.id))
      .map(w => w.name)
      .slice(0, 3)
      .join(', ') || '';

    const displayName = selectedWorkflows.size > 3
      ? `${selectedNames} 외 ${selectedWorkflows.size - 3}개`
      : selectedNames;

    setWorkflowToDelete({
      id: Array.from(selectedWorkflows),
      name: displayName,
    });
    setDeleteDialogOpen(true);
  };

  const handleConfirmDelete = async () => {
    if (!workflowToDelete) return;

    if (Array.isArray(workflowToDelete.id)) {
      // 다중 삭제
      try {
        await Promise.all(
          workflowToDelete.id.map(id => deleteMutation.mutateAsync(id))
        );
      } catch (error) {
        console.error('다중 삭제 오류:', error);
      }
    } else {
      // 단일 삭제
      deleteMutation.mutate(workflowToDelete.id);
    }
  };

  // Handle node deletion confirmation
  const handleConfirmNodeDelete = useCallback(() => {
    if (nodesToDelete.length === 0) return;

    // 삭제할 노드 ID 목록
    const nodeIdsToDelete = nodesToDelete.map(node => node.id);

    // 노드 필터링
    setNodes((nds) => nds.filter(node => !nodeIdsToDelete.includes(node.id)));

    // 연결된 엣지도 삭제
    setEdges((eds) => eds.filter(edge =>
      !nodeIdsToDelete.includes(edge.source) &&
      !nodeIdsToDelete.includes(edge.target)
    ));

    // Toast 알림
    toast({
      title: '노드 삭제 완료',
      description: `${nodesToDelete.length}개의 노드가 삭제되었습니다.`,
    });

    // 상태 초기화
    setDeleteNodeDialogOpen(false);
    setNodesToDelete([]);
  }, [nodesToDelete, setNodes, setEdges, toast]);

  // Handle node editing
  const handleNodeClick = useCallback((_event: React.MouseEvent, node: Node) => {
    setSelectedNode(node);
  }, []);

  const handleNodeUpdate = useCallback((nodeId: string, data: Partial<Node['data']>) => {
    setNodes((nds) =>
      nds.map((node) =>
        node.id === nodeId
          ? { ...node, data: { ...node.data, ...data } }
          : node
      )
    );

    toast({
      title: '노드 업데이트 완료',
      description: '노드 설정이 저장되었습니다.',
    });
  }, [setNodes, toast]);

  // Handle simulation step change
  const handleSimulationStepChange = useCallback((_stepIndex: number, nodeId: string) => {
    // Highlight the current node in the canvas
    setNodes((nds) =>
      nds.map((node) => ({
        ...node,
        data: {
          ...node.data,
          highlighted: node.id === nodeId,
        },
      }))
    );
  }, [setNodes]);

  return (
    <div className="h-screen max-h-screen flex gap-6 overflow-hidden">
      {/* Sidebar */}
      <div className="w-80 space-y-4 overflow-y-auto flex-shrink-0">
        <Card>
          <CardHeader>
            <CardTitle>워크플로우 정보</CardTitle>
            <CardDescription>워크플로우 기본 정보를 입력하세요.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <Button
              variant="outline"
              size="sm"
              onClick={createNewWorkflow}
              className="w-full"
            >
              <RefreshCw className="w-4 h-4 mr-2" />
              새로 만들기
            </Button>

            <div>
              <Label htmlFor="name">워크플로우 이름</Label>
              <Input
                id="name"
                value={workflowName}
                onChange={(e) => setWorkflowName(e.target.value)}
                placeholder="예: 품질 검사 워크플로우"
              />
            </div>

            <div>
              <Label htmlFor="rule">Rule 표현식 (선택)</Label>
              <Textarea
                id="rule"
                value={ruleExpression}
                onChange={(e) => setRuleExpression(e.target.value)}
                placeholder="예: temperature > 90 && vibration < 50"
                className="font-mono text-sm"
              />
            </div>

            {saveSuccess && (
              <div className="flex items-center gap-2 p-3 bg-green-50 text-green-700 rounded-md border border-green-200">
                <CheckCircle className="w-4 h-4" />
                <span className="text-sm font-medium">저장되었습니다!</span>
              </div>
            )}

            <div className="flex gap-2">
              <Button onClick={handleSave} className="flex-1" disabled={createMutation.isPending || updateMutation.isPending}>
                <Save className="w-4 h-4 mr-2" />
                {createMutation.isPending || updateMutation.isPending ? '저장 중...' : '저장'}
              </Button>
              <Button
                variant="outline"
                className="flex-1"
                onClick={() => setShowExecutePanel(!showExecutePanel)}
              >
                <Play className="w-4 h-4 mr-2" />
                실행
              </Button>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>노드 추가</CardTitle>
            <CardDescription>워크플로우에 노드를 추가하세요.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-2">
            {/* 기존 v1 노드 타입 (하위 호환성) */}
            <Button
              variant="outline"
              className="w-full justify-start"
              onClick={() => addNode(NodeType.INPUT, '데이터 입력', '사용자 입력 또는 외부 데이터 수집')}
            >
              <FileText className="w-4 h-4 mr-2 text-blue-500" />
              데이터 입력
            </Button>
            <Button
              variant="outline"
              className="w-full justify-start"
              onClick={() => addNode(NodeType.DECISION, '판단 로직', 'Rule 기반 조건 평가', { rule: 'temperature > 90' })}
            >
              <AlertCircle className="w-4 h-4 mr-2 text-purple-500" />
              판단 로직
            </Button>
            <Button
              variant="outline"
              className="w-full justify-start"
              onClick={() => addNode(NodeType.ACTION, '외부 연동', 'API 호출, 알림 전송 등')}
            >
              <Zap className="w-4 h-4 mr-2 text-yellow-500" />
              외부 연동
            </Button>
            <Button
              variant="outline"
              className="w-full justify-start"
              onClick={() => addNode(NodeType.OUTPUT, '결과 출력', '최종 판단 결과 저장')}
            >
              <CheckCircle className="w-4 h-4 mr-2 text-green-500" />
              결과 출력
            </Button>

            {/* 신규 v2 노드 타입 (Week 5) */}
            <div className="border-t pt-2 mt-2">
              <p className="text-xs text-muted-foreground mb-2 font-semibold">고급 노드 (v2)</p>
              <Button
                variant="outline"
                className="w-full justify-start mb-1"
                onClick={() => addNode(NodeType.DATA_INPUT, '데이터 소스', 'DB/API 연동', { source: 'database' })}
              >
                <Database className="w-4 h-4 mr-2 text-violet-500" />
                데이터 소스
              </Button>
              <Button
                variant="outline"
                className="w-full justify-start mb-1"
                onClick={() => addNode(NodeType.RULE_JUDGMENT, 'Rule 판단', 'AST 기반 규칙 평가', { ruleExpression: 'x > 0 && y < 100' })}
              >
                <FileCode className="w-4 h-4 mr-2 text-emerald-500" />
                Rule 판단
              </Button>
              <Button
                variant="outline"
                className="w-full justify-start mb-1"
                onClick={() => addNode(NodeType.LLM_JUDGMENT, 'AI 판단', 'LLM 기반 판단', { prompt: 'Analyze the data and provide judgment' })}
              >
                <Brain className="w-4 h-4 mr-2 text-cyan-500" />
                AI 판단
              </Button>
              <Button
                variant="outline"
                className="w-full justify-start mb-1"
                onClick={() => addNode(NodeType.ACTION_EXECUTION, '작업 실행', '외부 시스템 작업', { service: 'slack' })}
              >
                <Zap className="w-4 h-4 mr-2 text-orange-500" />
                작업 실행
              </Button>
              <Button
                variant="outline"
                className="w-full justify-start mb-1"
                onClick={() => addNode(NodeType.NOTIFICATION, '알림', 'Slack/Email 알림', { channel: 'slack' })}
              >
                <Bell className="w-4 h-4 mr-2 text-pink-500" />
                알림
              </Button>
              <Button
                variant="outline"
                className="w-full justify-start"
                onClick={() => addNode(NodeType.DATA_AGGREGATION, '데이터 집계', '통계 계산 및 변환', { aggregationType: 'sum' })}
              >
                <BarChart className="w-4 h-4 mr-2 text-teal-500" />
                데이터 집계
              </Button>
            </div>
          </CardContent>
        </Card>

        {/* AI Workflow Generation Panel */}
        <Card className="border-primary/50">
          <CardHeader>
            <div className="space-y-2">
              <CardTitle className="flex items-center gap-2">
                <Wand2 className="w-5 h-5 text-primary flex-shrink-0" />
                <span className="truncate">AI 워크플로우 생성</span>
              </CardTitle>
              <Button
                variant="outline"
                size="sm"
                onClick={() => setShowAIPanel(!showAIPanel)}
                className="w-full"
              >
                {showAIPanel ? (
                  <>
                    <ChevronUp className="w-4 h-4 mr-2" />
                    닫기
                  </>
                ) : (
                  <>
                    <ChevronDown className="w-4 h-4 mr-2" />
                    AI로 생성하기
                  </>
                )}
              </Button>
            </div>
            <CardDescription>
              자연어로 워크플로우를 설명하면 AI가 자동으로 생성합니다.
            </CardDescription>
          </CardHeader>
          {showAIPanel && (
            <CardContent className="space-y-4">
              <TooltipProvider>
              {/* Generation Mode Selection */}
              <div>
                <Label className="text-sm font-medium mb-3 block">생성 모드</Label>
                <div>
                  <RadioGroup value={generationMode} onValueChange={(value: string) => setGenerationMode(value as GenerationMode)}>
                    {/* Pattern Mode */}
                    <div className="flex items-center space-x-2 mb-2">
                      <RadioGroupItem value="pattern" id="mode-pattern" />
                      <Label htmlFor="mode-pattern" className="flex-1 cursor-pointer flex items-center gap-2">
                        <span>Pattern 모드</span>
                        <Tooltip>
                          <TooltipTrigger asChild>
                            <HelpCircle className="w-3.5 h-3.5 text-muted-foreground" />
                          </TooltipTrigger>
                          <TooltipContent>
                            <p className="text-xs max-w-xs">빠른 생성 (평균 0.5초). 간단한 조건문에 최적화.</p>
                          </TooltipContent>
                        </Tooltip>
                      </Label>
                    </div>

                    {/* LLM Mode */}
                    <div className="flex items-center space-x-2 mb-2">
                      <RadioGroupItem value="llm" id="mode-llm" />
                      <Label htmlFor="mode-llm" className="flex-1 cursor-pointer flex items-center gap-2">
                        <span>LLM 모드</span>
                        <Tooltip>
                          <TooltipTrigger asChild>
                            <HelpCircle className="w-3.5 h-3.5 text-muted-foreground" />
                          </TooltipTrigger>
                          <TooltipContent>
                            <p className="text-xs max-w-xs">지능형 생성 (평균 5초). 복잡한 비즈니스 로직 지원.</p>
                          </TooltipContent>
                        </Tooltip>
                      </Label>
                    </div>

                    {/* Hybrid Mode */}
                    <div className="flex items-center space-x-2">
                      <RadioGroupItem value="hybrid" id="mode-hybrid" />
                      <Label htmlFor="mode-hybrid" className="flex-1 cursor-pointer flex items-center gap-2">
                        <span>Hybrid 모드 (권장)</span>
                        <Tooltip>
                          <TooltipTrigger asChild>
                            <HelpCircle className="w-3.5 h-3.5 text-muted-foreground" />
                          </TooltipTrigger>
                          <TooltipContent>
                            <p className="text-xs max-w-xs">자동 선택 (권장). 간단하면 Pattern, 복잡하면 LLM 사용.</p>
                          </TooltipContent>
                        </Tooltip>
                      </Label>
                    </div>
                  </RadioGroup>
                </div>
              </div>

              {/* API Key Input (LLM/Hybrid 모드일 때만 표시) */}
              {(generationMode === 'llm' || generationMode === 'hybrid') && (
                <div>
                  <Label htmlFor="claude-api-key" className="flex items-center gap-2">
                    Claude API Key
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <HelpCircle className="w-3.5 h-3.5 text-muted-foreground" />
                      </TooltipTrigger>
                      <TooltipContent>
                        <p className="text-xs max-w-xs">Settings에서 저장하여 매번 입력하지 않도록 할 수 있습니다.</p>
                      </TooltipContent>
                    </Tooltip>
                  </Label>
                  <Input
                    id="claude-api-key"
                    type="password"
                    value={claudeApiKey}
                    onChange={(e) => setClaudeApiKey(e.target.value)}
                    placeholder="sk-ant-..."
                    className="font-mono text-xs"
                  />
                </div>
              )}

              {/* Sample Scenarios */}
              <div>
                <Label className="text-xs text-muted-foreground mb-2 block">
                  샘플 시나리오
                </Label>
                <div className="space-y-1">
                  {testScenarios.slice(0, 3).map((scenario, idx) => (
                    <Button
                      key={idx}
                      variant="ghost"
                      size="sm"
                      className="w-full justify-start text-xs h-auto py-2 text-left"
                      onClick={() => useSampleScenario(scenario)}
                    >
                      {scenario}
                    </Button>
                  ))}
                </div>
              </div>

              {/* Description Input */}
              <div>
                <Label htmlFor="ai-description">워크플로우 설명</Label>
                <Textarea
                  id="ai-description"
                  value={aiDescription}
                  onChange={(e) => setAiDescription(e.target.value)}
                  placeholder="예: 온도가 90도 이상이면 알림 보내기"
                  className="min-h-[100px] font-normal"
                />
              </div>

              {/* Generate Button */}
              <Button
                onClick={handleGenerateAIWorkflow}
                disabled={isGenerating || !aiDescription.trim()}
                className="w-full"
              >
                {isGenerating ? (
                  <>
                    <RefreshCw className="w-4 h-4 mr-2 animate-spin" />
                    생성 중...
                  </>
                ) : (
                  <>
                    <Sparkles className="w-4 h-4 mr-2" />
                    AI로 생성
                  </>
                )}
              </Button>
              </TooltipProvider>
            </CardContent>
          )}
        </Card>

        <Card>
          <CardHeader>
            <div className="flex items-center justify-between">
              <div>
                <CardTitle>저장된 워크플로우</CardTitle>
                <CardDescription>기존 워크플로우를 불러오거나 삭제하세요.</CardDescription>
              </div>
              {workflows && workflows.length > 0 && (
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => {
                    if (isEditMode) {
                      setIsEditMode(false);
                      setSelectedWorkflows(new Set());
                    } else {
                      setIsEditMode(true);
                    }
                  }}
                >
                  {isEditMode ? '완료' : '편집'}
                </Button>
              )}
            </div>
          </CardHeader>
          <CardContent>
            {workflows && workflows.length > 0 ? (
              <>
                {/* Bulk delete button (편집 모드일 때만 표시) */}
                {isEditMode && selectedWorkflows.size > 0 && (
                  <div className="mb-3">
                    <Button
                      variant="destructive"
                      size="sm"
                      onClick={handleBulkDeleteClick}
                      className="w-full"
                    >
                      <Trash2 className="w-4 h-4 mr-2" />
                      선택 삭제 ({selectedWorkflows.size}개)
                    </Button>
                  </div>
                )}

                <div className="space-y-2">
                  {workflows.map((workflow) => (
                    <div
                      key={workflow.id}
                      className={`flex items-center gap-2 p-2 rounded-md border transition-colors ${
                        selectedWorkflowId === workflow.id
                          ? 'bg-primary/10 border-primary'
                          : 'bg-background border-border hover:bg-muted'
                      }`}
                    >
                      {/* Checkbox (편집 모드일 때만 표시) */}
                      {isEditMode && (
                        <input
                          type="checkbox"
                          checked={selectedWorkflows.has(workflow.id)}
                          onChange={(e) => {
                            const newSelected = new Set(selectedWorkflows);
                            if (e.target.checked) {
                              newSelected.add(workflow.id);
                            } else {
                              newSelected.delete(workflow.id);
                            }
                            setSelectedWorkflows(newSelected);
                          }}
                          className="w-4 h-4"
                        />
                      )}

                      {/* 로드 버튼 (편집 모드가 아닐 때만 활성화) */}
                      <Button
                        variant="ghost"
                        className="flex-1 justify-start"
                        onClick={() => loadWorkflow(workflow)}
                        disabled={isEditMode}
                      >
                        <Workflow className="w-4 h-4 mr-2" />
                        {workflow.name}
                      </Button>

                      {/* 삭제 버튼 (편집 모드가 아닐 때만 표시) */}
                      {!isEditMode && (
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => handleDeleteClick(workflow.id, workflow.name)}
                          className="text-destructive hover:text-destructive hover:bg-destructive/10"
                        >
                          <Trash2 className="w-4 h-4" />
                        </Button>
                      )}
                    </div>
                  ))}
                </div>
              </>
            ) : (
              <EmptyState
                icon={Workflow}
                title="저장된 워크플로우 없음"
                description="저장 버튼을 눌러 현재 워크플로우를 저장하세요."
              />
            )}
          </CardContent>
        </Card>

        {/* Execute Panel */}
        {showExecutePanel && (
          <Card className="border-primary">
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Sparkles className="w-5 h-5" />
                워크플로우 실행
              </CardTitle>
              <CardDescription>테스트 데이터로 판단을 실행하세요.</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              {/* Workflow Selector */}
              <div>
                <Label htmlFor="execute-workflow">워크플로우 선택</Label>
                <select
                  ref={executeSelectRef}
                  id="execute-workflow"
                  value={executeWorkflowId}
                  onChange={(e) => setExecuteWorkflowId(e.target.value)}
                  className="w-full mt-1 p-2 border rounded-md bg-background"
                >
                  <option value="">선택하세요...</option>
                  {workflows?.map((workflow) => (
                    <option key={workflow.id} value={workflow.id}>
                      {workflow.name}
                    </option>
                  ))}
                </select>
              </div>

              {/* Input Data */}
              <div>
                <div className="flex items-center justify-between mb-1">
                  <Label htmlFor="input-data">입력 데이터 (JSON)</Label>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={useSampleData}
                    className="text-xs"
                  >
                    샘플 데이터 사용
                  </Button>
                </div>
                <Textarea
                  id="input-data"
                  value={inputData}
                  onChange={(e) => setInputData(e.target.value)}
                  placeholder='{"temperature": 95, "vibration": 45}'
                  className="font-mono text-sm min-h-[120px]"
                />
              </div>

              {/* Execute Button */}
              <Button
                onClick={handleExecute}
                disabled={!executeWorkflowId || executeMutation.isPending}
                className="w-full"
              >
                {executeMutation.isPending ? '실행 중...' : '판단 실행'}
              </Button>

              {/* Execution Result */}
              {executionResult && (
                <Card className="bg-muted/50">
                  <CardHeader className="pb-3">
                    <div className="flex items-center justify-between">
                      <CardTitle className="text-base">실행 결과</CardTitle>
                      <Badge
                        variant={executionResult.result ? 'default' : 'destructive'}
                        className="flex items-center gap-1"
                      >
                        {executionResult.result ? (
                          <>
                            <CheckCircle className="w-3 h-3" />
                            합격
                          </>
                        ) : (
                          <>
                            <XCircle className="w-3 h-3" />
                            불합격
                          </>
                        )}
                      </Badge>
                    </div>
                  </CardHeader>
                  <CardContent className="space-y-3 pt-0">
                    {/* Confidence Bar */}
                    <div>
                      <div className="flex items-center justify-between text-sm mb-1">
                        <span className="text-muted-foreground">신뢰도</span>
                        <span className="font-medium">
                          {(executionResult.confidence * 100).toFixed(1)}%
                        </span>
                      </div>
                      <div className="h-2 bg-background rounded-full overflow-hidden">
                        <div
                          className="h-full bg-primary transition-all"
                          style={{ width: `${executionResult.confidence * 100}%` }}
                        />
                      </div>
                    </div>

                    {/* Method Used */}
                    <div className="flex items-center justify-between text-sm">
                      <span className="text-muted-foreground">판단 방법</span>
                      <Badge variant="outline">{executionResult.method_used}</Badge>
                    </div>

                    {/* Explanation */}
                    <div>
                      <p className="text-sm text-muted-foreground mb-1">설명</p>
                      <p className="text-sm">{executionResult.explanation}</p>
                    </div>
                  </CardContent>
                </Card>
              )}
            </CardContent>
          </Card>
        )}
      </div>

      {/* Canvas */}
      <Card className="flex-1 flex flex-col min-h-0 overflow-hidden">
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle>워크플로우 캔버스</CardTitle>
              <CardDescription>
                노드를 드래그하여 연결하고 워크플로우를 구성하세요.
              </CardDescription>
            </div>
            <Button
              onClick={() => setShowSimulationPanel(true)}
              variant="outline"
              size="sm"
              disabled={nodes.length === 0}
            >
              <Bug className="w-4 h-4 mr-2" />
              시뮬레이션
            </Button>
          </div>
        </CardHeader>
        <CardContent className="p-0 flex-1 min-h-0 overflow-hidden">
          <div className="h-full relative">
            {/* Sticky 컨트롤 패널 (좌측 스크롤 따라다님) */}
            <div className="absolute left-4 z-10" style={{ position: 'sticky', top: '1rem' }}>
              <Card className="w-12 shadow-lg">
                <CardContent className="p-2 flex flex-col gap-2">
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => reactFlowInstance?.zoomIn()}
                    className="w-full h-10 p-0"
                    title="확대"
                    disabled={!reactFlowInstance}
                  >
                    <ZoomIn className="w-4 h-4" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => reactFlowInstance?.zoomOut()}
                    className="w-full h-10 p-0"
                    title="축소"
                    disabled={!reactFlowInstance}
                  >
                    <ZoomOut className="w-4 h-4" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => reactFlowInstance?.fitView()}
                    className="w-full h-10 p-0"
                    title="전체 보기"
                    disabled={!reactFlowInstance}
                  >
                    <Maximize2 className="w-4 h-4" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => {
                      setNodes(initialNodes);
                      setEdges(initialEdges);
                    }}
                    className="w-full h-10 p-0"
                    title="초기화"
                  >
                    <RotateCcw className="w-4 h-4" />
                  </Button>
                </CardContent>
              </Card>
            </div>

            {/* ReactFlow 캔버스를 absolute로 전체 공간 활용 */}
            <div className="absolute inset-0">
              <ReactFlow
                onInit={setReactFlowInstance}
                nodes={nodes}
                edges={edges}
                onNodesChange={onNodesChange}
                onEdgesChange={onEdgesChange}
                onConnect={onConnect}
                onNodeClick={handleNodeClick}
                nodeTypes={nodeTypes}
                fitView
                defaultEdgeOptions={{
                  type: 'smoothstep',
                  animated: true,
                  style: { stroke: 'hsl(var(--primary))', strokeWidth: 2 },
                }}
                connectionLineStyle={{ stroke: 'hsl(var(--primary))', strokeWidth: 2 }}
                connectionLineType={ConnectionLineType.SmoothStep}
                snapToGrid
                snapGrid={[15, 15]}
                // 성능 최적화 설정
                minZoom={0.1}
                maxZoom={4}
                onlyRenderVisibleElements={true}
                nodesDraggable={true}
                nodesConnectable={true}
                elementsSelectable={true}
                selectNodesOnDrag={false}
                panOnScroll={true}
                zoomOnScroll={true}
                zoomOnPinch={true}
                panOnDrag={true}
                preventScrolling={true}
              >
                <Background gap={15} />
                <MiniMap
                  nodeColor={(node) => {
                    const type = (node.data as any).type || 'default';
                    const colors = {
                      input: '#3b82f6',
                      decision: '#a855f7',
                      action: '#eab308',
                      output: '#22c55e',
                      default: '#6b7280',
                    };
                    return colors[type as keyof typeof colors] || colors.default;
                  }}
                  zoomable
                  pannable
                />
              </ReactFlow>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* 삭제 확인 다이얼로그 */}
      <AlertDialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle className="flex items-center gap-2">
              <AlertTriangle className="w-5 h-5 text-destructive" />
              워크플로우 삭제
            </AlertDialogTitle>
            <AlertDialogDescription>
              정말로 이 워크플로우를 삭제하시겠습니까?
              <br />
              <span className="font-semibold">
                "{workflowToDelete?.name}"
              </span>
              <br />
              <br />
              이 작업은 되돌릴 수 없습니다.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>취소</AlertDialogCancel>
            <AlertDialogAction
              onClick={handleConfirmDelete}
              className="bg-destructive hover:bg-destructive/90"
            >
              {deleteMutation.isPending ? '삭제 중...' : '삭제'}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* 노드 삭제 확인 다이얼로그 */}
      <AlertDialog open={deleteNodeDialogOpen} onOpenChange={setDeleteNodeDialogOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle className="flex items-center gap-2">
              <AlertTriangle className="w-5 h-5 text-destructive" />
              노드 삭제
            </AlertDialogTitle>
            <AlertDialogDescription>
              정말로 {nodesToDelete.length}개의 노드를 삭제하시겠습니까?
              <br />
              <br />
              연결된 엣지도 함께 삭제됩니다.
              <br />
              이 작업은 되돌릴 수 없습니다.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>취소</AlertDialogCancel>
            <AlertDialogAction
              onClick={handleConfirmNodeDelete}
              className="bg-destructive hover:bg-destructive/90"
            >
              삭제
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* 노드 편집 패널 */}
      {selectedNode && (
        <NodeEditPanel
          node={selectedNode}
          onUpdate={handleNodeUpdate}
          onClose={() => setSelectedNode(null)}
        />
      )}

      {/* 시뮬레이션 패널 */}
      {showSimulationPanel && (
        <SimulationPanel
          nodes={nodes}
          edges={edges}
          initialData={simulationInitialData}
          onStepChange={handleSimulationStepChange}
          onClose={() => setShowSimulationPanel(false)}
        />
      )}
    </div>
  );
}
