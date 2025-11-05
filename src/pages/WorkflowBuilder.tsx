import { useState, useCallback, useMemo } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import ReactFlow, {
  Node,
  Edge,
  addEdge,
  Background,
  Controls,
  Connection,
  useNodesState,
  useEdgesState,
  MiniMap,
} from 'reactflow';
import 'reactflow/dist/style.css';
import { getAllWorkflows, createWorkflow, updateWorkflow, executeJudgment, type JudgmentResult } from '@/lib/tauri-api';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Textarea } from '@/components/ui/textarea';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Label } from '@/components/ui/label';
import { Badge } from '@/components/ui/badge';
import { Save, Play, CheckCircle, XCircle, Sparkles, FileText, AlertCircle, Zap, RefreshCw, Workflow } from 'lucide-react';
import CustomNode from '@/components/workflow/CustomNode';
import EmptyState from '@/components/EmptyState';

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

  const onConnect = useCallback(
    (params: Connection) => setEdges((eds) => addEdge(params, eds)),
    [setEdges]
  );

  // Memoize node types to prevent re-renders
  const nodeTypes = useMemo(() => ({
    custom: CustomNode,
  }), []);

  const addNode = useCallback((type: string, label: string) => {
    const newNode: Node = {
      id: `${Date.now()}`,
      type: type === 'custom' ? 'custom' : type,
      data: { label },
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

  return (
    <div className="h-full flex gap-6">
      {/* Sidebar */}
      <div className="w-80 space-y-4 overflow-y-auto">
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
            <Button
              variant="outline"
              className="w-full justify-start"
              onClick={() => addNode('input', '📥 데이터 입력')}
            >
              <FileText className="w-4 h-4 mr-2 text-blue-500" />
              데이터 입력
            </Button>
            <Button
              variant="outline"
              className="w-full justify-start"
              onClick={() => addNode('default', '⚙️ 처리 단계')}
            >
              <Zap className="w-4 h-4 mr-2 text-yellow-500" />
              처리 단계
            </Button>
            <Button
              variant="outline"
              className="w-full justify-start"
              onClick={() => addNode('default', '✅ 판단 로직')}
            >
              <AlertCircle className="w-4 h-4 mr-2 text-purple-500" />
              판단 로직
            </Button>
            <Button
              variant="outline"
              className="w-full justify-start"
              onClick={() => addNode('output', '📤 결과 출력')}
            >
              <CheckCircle className="w-4 h-4 mr-2 text-green-500" />
              결과 출력
            </Button>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>저장된 워크플로우</CardTitle>
            <CardDescription>기존 워크플로우를 불러오세요.</CardDescription>
          </CardHeader>
          <CardContent>
            {workflows && workflows.length > 0 ? (
              <div className="space-y-2">
                {workflows.map((workflow) => (
                  <Button
                    key={workflow.id}
                    variant={selectedWorkflowId === workflow.id ? 'default' : 'outline'}
                    className="w-full justify-start"
                    onClick={() => loadWorkflow(workflow)}
                  >
                    {workflow.name}
                  </Button>
                ))}
              </div>
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
      <Card className="flex-1">
        <CardHeader>
          <CardTitle>워크플로우 캔버스</CardTitle>
          <CardDescription>
            노드를 드래그하여 연결하고 워크플로우를 구성하세요.
          </CardDescription>
        </CardHeader>
        <CardContent className="p-0">
          <div style={{ height: 'calc(100vh - 250px)' }}>
            <ReactFlow
              nodes={nodes}
              edges={edges}
              onNodesChange={onNodesChange}
              onEdgesChange={onEdgesChange}
              onConnect={onConnect}
              nodeTypes={nodeTypes}
              fitView
            >
              <Background />
              <Controls />
              <MiniMap />
            </ReactFlow>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
