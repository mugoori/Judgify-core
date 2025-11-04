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
import { getAllWorkflows, createWorkflow, updateWorkflow } from '@/lib/tauri-api';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Textarea } from '@/components/ui/textarea';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Label } from '@/components/ui/label';
import { Plus, Save, Play } from 'lucide-react';
import CustomNode from '@/components/workflow/CustomNode';

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

  const { data: workflows } = useQuery({
    queryKey: ['workflows'],
    queryFn: getAllWorkflows,
  });

  const createMutation = useMutation({
    mutationFn: createWorkflow,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['workflows'] });
      alert('워크플로우가 저장되었습니다.');
    },
  });

  const updateMutation = useMutation({
    mutationFn: updateWorkflow,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['workflows'] });
      alert('워크플로우가 업데이트되었습니다.');
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

  const addNode = useCallback((type: string) => {
    const newNode: Node = {
      id: `${nodes.length + 1}`,
      type,
      data: { label: `${type} ${nodes.length + 1}` },
      position: { x: Math.random() * 400, y: Math.random() * 400 },
    };
    setNodes((nds) => [...nds, newNode]);
  }, [nodes.length, setNodes]);

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

            <div className="flex gap-2">
              <Button onClick={handleSave} className="flex-1">
                <Save className="w-4 h-4 mr-2" />
                저장
              </Button>
              <Button variant="outline" className="flex-1">
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
              onClick={() => addNode('default')}
            >
              <Plus className="w-4 h-4 mr-2" />
              일반 노드
            </Button>
            <Button
              variant="outline"
              className="w-full justify-start"
              onClick={() => addNode('input')}
            >
              <Plus className="w-4 h-4 mr-2" />
              입력 노드
            </Button>
            <Button
              variant="outline"
              className="w-full justify-start"
              onClick={() => addNode('output')}
            >
              <Plus className="w-4 h-4 mr-2" />
              출력 노드
            </Button>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>저장된 워크플로우</CardTitle>
            <CardDescription>기존 워크플로우를 불러오세요.</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              {workflows?.map((workflow) => (
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
          </CardContent>
        </Card>
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
