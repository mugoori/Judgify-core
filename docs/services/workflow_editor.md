# 워크플로우 편집기 구현 명세서

**문서 버전**: v2.0  
**작성일**: 2024.08.05  
**대상**: 프론트엔드 개발자, UI/UX 디자이너, 제품 매니저  
**목적**: React Flow 기반 노코드 워크플로우 편집기의 상세 구현 사양

## 📋 1. 편집기 개요 및 요구사항

### 1.1 핵심 목표
- **직관적 UI**: 드래그 앤 드롭으로 워크플로우 생성
- **실시간 검증**: 워크플로우 작성 중 실시간 오류 검증
- **템플릿 지원**: 자주 사용되는 패턴의 템플릿 제공
- **협업 기능**: 워크플로우 공유 및 버전 관리
- **시뮬레이션**: 실제 실행 전 테스트 기능

### 1.2 기능 요구사항
```typescript
interface WorkflowEditorRequirements {
    // 편집 기능
    dragAndDrop: boolean;          // 노드 드래그 앤 드롭
    realTimeValidation: boolean;   // 실시간 유효성 검증
    autoLayout: boolean;           // 자동 레이아웃 정렬
    multiSelect: boolean;          // 다중 선택 및 일괄 편집
    
    // 노드 유형
    supportedNodeTypes: [
        'trigger',      // 트리거 노드
        'condition',    // 조건 판단 노드
        'action',       // 액션 실행 노드
        'decision',     // 의사결정 분기 노드
        'parallel',     // 병렬 실행 노드
        'delay',        // 지연 노드
        'end'          // 종료 노드
    ];
    
    // 저장 및 관리
    versionControl: boolean;       // 버전 관리
    templateSave: boolean;         // 템플릿으로 저장
    sharing: boolean;              // 워크플로우 공유
    simulation: boolean;           // 시뮬레이션 실행
}
```

## 🏗 2. 아키텍처 설계

### 2.1 컴포넌트 구조
```typescript
// 메인 편집기 컴포넌트 구조
// 확장된 워크플로우 편집기 컴포넌트 구조
WorkflowEditor/
├── components/
│   ├── Canvas/
│   │   ├── WorkflowCanvas.tsx          // 메인 캔버스
│   │   ├── NodeRenderer.tsx            // 노드 렌더링
│   │   ├── EdgeRenderer.tsx            // 연결선 렌더링
│   │   └── MiniMap.tsx                 // 미니맵
│   ├── Toolbar/
│   │   ├── NodePalette.tsx             // 노드 팔레트
│   │   ├── ActionButtons.tsx           // 액션 버튼들
│   │   └── ViewControls.tsx            // 뷰 컨트롤
│   ├── Properties/
│   │   ├── NodeProperties.tsx          // 노드 속성 패널
│   │   ├── RuleEditor.tsx              // 규칙 편집기
│   │   ├── ActionEditor.tsx            // 액션 편집기
│   │   └── DashboardProperties.tsx     // 🆕 대시보드 노드 속성 편집기
│   ├── Validation/
│   │   ├── ValidationPanel.tsx         // 검증 결과 패널
│   │   └── ErrorTooltip.tsx            // 오류 툴팁
│   ├── Simulation/
│   │   ├── SimulationRunner.tsx        // 시뮬레이션 실행기
│   │   └── ResultViewer.tsx            // 결과 뷰어
│   └── Dashboard/                      // 🆕 대시보드 관련 컴포넌트 폴더
│       ├── DashboardGenerator.tsx      // 🆕 자동 생성 메인 컴포넌트
│       ├── DashboardPreview.tsx        // 🆕 대시보드 미리보기
│       ├── NLPRequestInput.tsx         // 🆕 자연어 요청 입력
│       ├── ComponentLibrary.tsx        // 🆕 컴포넌트 라이브러리
│       ├── DashboardTemplates.tsx      // 🆕 대시보드 템플릿 갤러리
│       ├── RealTimePreview.tsx         // 🆕 실시간 데이터 미리보기
│       └── FeedbackPanel.tsx           // 🆕 사용자 피드백 패널
├── hooks/
│   ├── useWorkflowState.ts             // 워크플로우 상태 관리
│   ├── useValidation.ts                // 유효성 검증 훅
│   ├── useSimulation.ts                // 시뮬레이션 훅
│   ├── useDashboardGeneration.ts       // 🆕 대시보드 생성 훅
│   ├── useRealTimeData.ts              // 🆕 실시간 데이터 구독 훅
│   └── useLLMAnalysis.ts               // 🆕 LLM 분석 요청 훅
├── types/
│   ├── workflow.types.ts               // 워크플로우 타입 정의
│   ├── editor.types.ts                 // 편집기 타입 정의
│   └── dashboard.types.ts              // 🆕 대시보드 타입 정의
├── utils/
│   ├── workflowValidator.ts            // 워크플로우 검증 로직
│   ├── layoutEngine.ts                 // 자동 레이아웃 엔진
│   ├── exportImport.ts                 // 내보내기/가져오기
│   ├── dashboardGenerator.ts           // 🆕 대시보드 생성 유틸리티
│   ├── componentRenderer.ts            // 🆕 동적 컴포넌트 렌더러
│   └── nlpParser.ts                    // 🆕 자연어 요청 파싱 유틸리티
└── services/                           // 🆕 서비스 레이어 추가
    ├── dashboardAPI.ts                 // 🆕 대시보드 API 클라이언트
    ├── llmService.ts                   // 🆕 LLM 서비스 인터페이스
    └── websocketService.ts             // 🆕 실시간 데이터 웹소켓 서비스
```

### 2.2 상태 관리 구조
```typescript
// Zustand를 사용한 상태 관리
interface WorkflowEditorState {
    // 워크플로우 데이터
    workflow: {
        id: string;
        name: string;
        description: string;
        nodes: WorkflowNode[];
        edges: WorkflowEdge[];
        version: number;
        metadata: WorkflowMetadata;
    };
    
    // 편집기 상태
    editor: {
        selectedNodes: string[];
        selectedEdges: string[];
        clipboard: ClipboardData | null;
        viewMode: 'edit' | 'view' | 'simulate';
        zoom: number;
        pan: { x: number; y: number };
    };
    
    // 검증 상태
    validation: {
        isValid: boolean;
        errors: ValidationError[];
        warnings: ValidationWarning[];
    };
    
    // 시뮬레이션 상태
    simulation: {
        isRunning: boolean;
        currentStep: string | null;
        results: SimulationResult[];
        inputData: Record<string, any>;
    };
}
```

## 🎨 3. UI 컴포넌트 상세 설계

### 3.1 메인 캔버스 (WorkflowCanvas)
```typescript
import ReactFlow, {
    Node,
    Edge,
    addEdge,
    Background,
    Controls,
    MiniMap,
    useNodesState,
    useEdgesState,
    MarkerType,
} from 'reactflow';

interface WorkflowCanvasProps {
    initialWorkflow?: Workflow;
    onWorkflowChange: (workflow: Workflow) => void;
    readOnly?: boolean;
}

const WorkflowCanvas: React.FC<WorkflowCanvasProps> = ({
    initialWorkflow,
    onWorkflowChange,
    readOnly = false
}) => {
    const [nodes, setNodes, onNodesChange] = useNodesState(initialWorkflow?.nodes || []);
    const [edges, setEdges, onEdgesChange] = useEdgesState(initialWorkflow?.edges || []);
    
    // 커스텀 노드 타입 정의
    const nodeTypes = useMemo(() => ({
        trigger: TriggerNode,
        condition: ConditionNode,
        action: ActionNode,
        decision: DecisionNode,
        parallel: ParallelNode,
        delay: DelayNode,
        end: EndNode,
    }), []);
    
    // 연결 생성 핸들러
    const onConnect = useCallback((params: Connection) => {
        const newEdge: Edge = {
            ...params,
            id: `edge-${Date.now()}`,
            type: 'smoothstep',
            markerEnd: {
                type: MarkerType.ArrowClosed,
                width: 20,
                height: 20,
                color: '#374151',
            },
            style: {
                strokeWidth: 2,
                stroke: '#374151',
            },
            data: {
                condition: null, // 조건부 연결의 경우 조건 설정
            }
        };
        
        setEdges((eds) => addEdge(newEdge, eds));
    }, []);
    
    // 드롭 핸들러 (팔레트에서 드래그)
    const onDrop = useCallback((event: React.DragEvent) => {
        event.preventDefault();
        
        const reactFlowBounds = event.currentTarget.getBoundingClientRect();
        const nodeType = event.dataTransfer.getData('application/reactflow');
        
        if (!nodeType) return;
        
        const position = {
            x: event.clientX - reactFlowBounds.left,
            y: event.clientY - reactFlowBounds.top,
        };
        
        const newNode: Node = {
            id: `${nodeType}-${Date.now()}`,
            type: nodeType,
            position,
            data: getDefaultNodeData(nodeType),
        };
        
        setNodes((nds) => nds.concat(newNode));
    }, []);
    
    return (
        <div className="workflow-canvas h-full w-full">
            <ReactFlow
                nodes={nodes}
                edges={edges}
                onNodesChange={onNodesChange}
                onEdgesChange={onEdgesChange}
                onConnect={onConnect}
                onDrop={onDrop}
                onDragOver={(event) => event.preventDefault()}
                nodeTypes={nodeTypes}
                fitView
                attributionPosition="bottom-left"
            >
                <Background color="#aaa" gap={16} />
                <Controls />
                <MiniMap 
                    nodeColor={(node) => getNodeColor(node.type)}
                    className="bg-white border border-gray-300 rounded-lg"
                />
            </ReactFlow>
        </div>
    );
};
```

### 3.2 노드 컴포넌트 (TriggerNode 예시)
```typescript
import { Handle, Position, NodeProps } from 'reactflow';
import { useState } from 'react';

interface TriggerNodeData {
    label: string;
    triggerType: 'sensor' | 'api' | 'schedule' | 'manual';
    config: {
        sensorId?: string;
        endpoint?: string;
        cronExpression?: string;
        dataSchema?: Record<string, any>;
    };
    isValid: boolean;
    errors: string[];
}

const TriggerNode: React.FC<NodeProps<TriggerNodeData>> = ({ 
    data, 
    selected,
    id 
}) => {
    const [isEditing, setIsEditing] = useState(false);
    
    const getTriggerIcon = (type: string) => {
        const icons = {
            sensor: '📊',
            api: '🔗',
            schedule: '⏰',
            manual: '👤'
        };
        return icons[type] || '❓';
    };
    
    const handleDoubleClick = () => {
        setIsEditing(true);
    };
    
    return (
        <div 
            className={`
                trigger-node bg-blue-50 border-2 border-blue-200 rounded-lg p-3 min-w-[150px]
                ${selected ? 'border-blue-500 shadow-lg' : ''}
                ${!data.isValid ? 'border-red-500 bg-red-50' : ''}
                hover:shadow-md transition-shadow cursor-pointer
            `}
            onDoubleClick={handleDoubleClick}
        >
            {/* 노드 헤더 */}
            <div className="flex items-center gap-2 mb-2">
                <span className="text-lg">{getTriggerIcon(data.triggerType)}</span>
                <span className="font-medium text-sm text-gray-800">
                    {data.label || 'Trigger'}
                </span>
            </div>
            
            {/* 트리거 타입 표시 */}
            <div className="text-xs text-gray-600 mb-2">
                {data.triggerType.toUpperCase()}
            </div>
            
            {/* 설정 요약 */}
            <div className="text-xs text-gray-500">
                {data.triggerType === 'sensor' && data.config.sensorId && (
                    <div>Sensor: {data.config.sensorId}</div>
                )}
                {data.triggerType === 'schedule' && data.config.cronExpression && (
                    <div>Cron: {data.config.cronExpression}</div>
                )}
            </div>
            
            {/* 오류 표시 */}
            {!data.isValid && data.errors.length > 0 && (
                <div className="text-xs text-red-600 mt-1">
                    ⚠️ {data.errors[0]}
                </div>
            )}
            
            {/* 연결 핸들 */}
            <Handle
                type="source"
                position={Position.Right}
                className="w-3 h-3 bg-blue-500 border-2 border-white"
            />
        </div>
    );
};
```

### 3.3 노드 팔레트 (NodePalette)
```typescript
const NodePalette: React.FC = () => {
    const nodeTemplates = [
        {
            type: 'trigger',
            label: 'Trigger',
            icon: '🚀',
            description: 'Start workflow execution',
            category: 'input'
        },
        {
            type: 'condition',
            label: 'Condition',
            icon: '❓',
            description: 'Rule or AI-based decision',
            category: 'logic'
        },
        {
            type: 'action',
            label: 'Action',
            icon: '⚡',
            description: 'Execute external command',
            category: 'output'
        },
        {
            type: 'decision',
            label: 'Decision',
            icon: '🔄',
            description: 'Branch workflow path',
            category: 'logic'
        },
        {
            type: 'parallel',
            label: 'Parallel',
            icon: '⚡⚡',
            description: 'Execute multiple paths',
            category: 'logic'
        },
        {
            type: 'delay',
            label: 'Delay',
            icon: '⏳',
            description: 'Wait for specified time',
            category: 'utility'
        },
        {
            type: 'end',
            label: 'End',
            icon: '🏁',
            description: 'Terminate workflow',
            category: 'output'
        }
    ];
    
    const categories = ['input', 'logic', 'output', 'utility'];
    
    const onDragStart = (event: React.DragEvent, nodeType: string) => {
        event.dataTransfer.setData('application/reactflow', nodeType);
        event.dataTransfer.effectAllowed = 'move';
    };
    
    return (
        <div className="node-palette w-64 bg-white border-r border-gray-200 p-4">
            <h3 className="text-lg font-semibold mb-4">Components</h3>
            
            {categories.map(category => (
                <div key={category} className="mb-6">
                    <h4 className="text-sm font-medium text-gray-700 mb-2 uppercase">
                        {category}
                    </h4>
                    
                    <div className="space-y-2">
                        {nodeTemplates
                            .filter(template => template.category === category)
                            .map(template => (
                                <div
                                    key={template.type}
                                    className="
                                        node-template p-3 border border-gray-200 rounded-lg 
                                        cursor-grab hover:border-blue-300 hover:bg-blue-50
                                        transition-colors
                                    "
                                    draggable
                                    onDragStart={(e) => onDragStart(e, template.type)}
                                >
                                    <div className="flex items-center gap-2 mb-1">
                                        <span className="text-lg">{template.icon}</span>
                                        <span className="font-medium text-sm">
                                            {template.label}
                                        </span>
                                    </div>
                                    <div className="text-xs text-gray-500">
                                        {template.description}
                                    </div>
                                </div>
                            ))
                        }
                    </div>
                </div>
            ))}
        </div>
    );
};
```

### 3.4 속성 편집 패널 (NodeProperties)
```typescript
interface NodePropertiesProps {
    selectedNode: WorkflowNode | null;
    onNodeUpdate: (nodeId: string, updates: Partial<WorkflowNode>) => void;
}

const NodeProperties: React.FC<NodePropertiesProps> = ({
    selectedNode,
    onNodeUpdate
}) => {
    if (!selectedNode) {
        return (
            <div className="node-properties w-80 bg-white border-l border-gray-200 p-4">
                <div className="text-center text-gray-500 mt-8">
                    Select a node to edit properties
                </div>
            </div>
        );
    }
    
    const renderPropertiesForType = () => {
        switch (selectedNode.type) {
            case 'trigger':
                return <TriggerProperties node={selectedNode} onUpdate={onNodeUpdate} />;
            case 'condition':
                return <ConditionProperties node={selectedNode} onUpdate={onNodeUpdate} />;
            case 'action':
                return <ActionProperties node={selectedNode} onUpdate={onNodeUpdate} />;
            default:
                return <div>Properties for {selectedNode.type}</div>;
        }
    };
    
    return (
        <div className="node-properties w-80 bg-white border-l border-gray-200 p-4">
            <div className="mb-4">
                <h3 className="text-lg font-semibold flex items-center gap-2">
                    {getNodeIcon(selectedNode.type)}
                    {selectedNode.data.label || selectedNode.type}
                </h3>
                <div className="text-sm text-gray-500">
                    ID: {selectedNode.id}
                </div>
            </div>
            
            {/* 공통 속성 */}
            <div className="mb-6">
                <label className="block text-sm font-medium text-gray-700 mb-2">
                    Label
                </label>
                <input
                    type="text"
                    value={selectedNode.data.label || ''}
                    onChange={(e) => onNodeUpdate(selectedNode.id, {
                        data: { ...selectedNode.data, label: e.target.value }
                    })}
                    className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
            </div>
            
            {/* 타입별 전용 속성 */}
            {renderPropertiesForType()}
        </div>
    );
};

// 조건 노드 속성 편집기
const ConditionProperties: React.FC<{
    node: WorkflowNode;
    onUpdate: (nodeId: string, updates: Partial<WorkflowNode>) => void;
}> = ({ node, onUpdate }) => {
    const [judgmentMethod, setJudgmentMethod] = useState(
        node.data.judgmentMethod || 'hybrid'
    );
    const [ruleExpression, setRuleExpression] = useState(
        node.data.ruleExpression || ''
    );
    const [llmCriteria, setLlmCriteria] = useState(
        node.data.llmCriteria || ''
    );
    
    const handleMethodChange = (method: string) => {
        setJudgmentMethod(method);
        onUpdate(node.id, {
            data: { ...node.data, judgmentMethod: method }
        });
    };
    
    return (
        <div className="condition-properties">
            <div className="mb-4">
                <label className="block text-sm font-medium text-gray-700 mb-2">
                    Judgment Method
                </label>
                <select
                    value={judgmentMethod}
                    onChange={(e) => handleMethodChange(e.target.value)}
                    className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                >
                    <option value="rule">Rule-based</option>
                    <option value="llm">AI-based</option>
                    <option value="hybrid">Hybrid</option>
                </select>
            </div>
            
            {(judgmentMethod === 'rule' || judgmentMethod === 'hybrid') && (
                <div className="mb-4">
                    <label className="block text-sm font-medium text-gray-700 mb-2">
                        Rule Expression
                    </label>
                    <textarea
                        value={ruleExpression}
                        onChange={(e) => {
                            setRuleExpression(e.target.value);
                            onUpdate(node.id, {
                                data: { ...node.data, ruleExpression: e.target.value }
                            });
                        }}
                        placeholder="e.g., temperature > 85 && vibration > 40"
                        rows={3}
                        className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                    />
                    <div className="text-xs text-gray-500 mt-1">
                        Use variables from input data (e.g., temperature, pressure)
                    </div>
                </div>
            )}
            
            {(judgmentMethod === 'llm' || judgmentMethod === 'hybrid') && (
                <div className="mb-4">
                    <label className="block text-sm font-medium text-gray-700 mb-2">
                        AI Criteria
                    </label>
                    <textarea
                        value={llmCriteria}
                        onChange={(e) => {
                            setLlmCriteria(e.target.value);
                            onUpdate(node.id, {
                                data: { ...node.data, llmCriteria: e.target.value }
                            });
                        }}
                        placeholder="Describe the conditions for judgment decision..."
                        rows={4}
                        className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                    />
                </div>
            )}
            
            {/* 신뢰도 임계값 설정 */}
            <div className="mb-4">
                <label className="block text-sm font-medium text-gray-700 mb-2">
                    Confidence Threshold
                </label>
                <input
                    type="range"
                    min="0"
                    max="1"
                    step="0.1"
                    value={node.data.confidenceThreshold || 0.7}
                    onChange={(e) => onUpdate(node.id, {
                        data: { ...node.data, confidenceThreshold: parseFloat(e.target.value) }
                    })}
                    className="w-full"
                />
                <div className="flex justify-between text-xs text-gray-500">
                    <span>0.0</span>
                    <span>{node.data.confidenceThreshold || 0.7}</span>
                    <span>1.0</span>
                </div>
            </div>
        </div>
    );
};
```

## 🧪 4. 시뮬레이션 기능

### 4.1 시뮬레이션 실행기
```typescript
interface SimulationRunner {
    workflow: Workflow;
    inputData: Record<string, any>;
    onStepUpdate: (step: SimulationStep) => void;
    onComplete: (result: SimulationResult) => void;
}

const useSimulationRunner = () => {
    const [isRunning, setIsRunning] = useState(false);
    const [currentStep, setCurrentStep] = useState<string | null>(null);
    const [steps, setSteps] = useState<SimulationStep[]>([]);
    
    const runSimulation = async (
        workflow: Workflow,
        inputData: Record<string, any>
    ) => {
        setIsRunning(true);
        setSteps([]);
        
        try {
            // 워크플로우 검증
            const validationResult = validateWorkflow(workflow);
            if (!validationResult.isValid) {
                throw new Error('Workflow validation failed');
            }
            
            // 시뮬레이션 실행
            const executor = new WorkflowSimulator(workflow);
            
            executor.onStepExecuted((step: SimulationStep) => {
                setCurrentStep(step.nodeId);
                setSteps(prev => [...prev, step]);
            });
            
            const result = await executor.execute(inputData);
            
            setCurrentStep(null);
            return result;
            
        } catch (error) {
            console.error('Simulation failed:', error);
            throw error;
        } finally {
            setIsRunning(false);
        }
    };
    
    return {
        isRunning,
        currentStep,
        steps,
        runSimulation
    };
};

// 워크플로우 시뮬레이터 클래스
class WorkflowSimulator {
    private workflow: Workflow;
    private stepCallbacks: ((step: SimulationStep) => void)[] = [];
    
    constructor(workflow: Workflow) {
        this.workflow = workflow;
    }
    
    onStepExecuted(callback: (step: SimulationStep) => void) {
        this.stepCallbacks.push(callback);
    }
    
    async execute(inputData: Record<string, any>): Promise<SimulationResult> {
        const context = { ...inputData };
        const executionPath: string[] = [];
        
        // 시작 노드 찾기
        const startNode = this.workflow.nodes.find(node => node.type === 'trigger');
        if (!startNode) {
            throw new Error('No trigger node found');
        }
        
        // 노드별 실행
        let currentNodeId = startNode.id;
        let maxSteps = 100; // 무한 루프 방지
        
        while (currentNodeId && maxSteps-- > 0) {
            const node = this.workflow.nodes.find(n => n.id === currentNodeId);
            if (!node) break;
            
            const stepResult = await this.executeNode(node, context);
            
            // 콜백 호출
            this.stepCallbacks.forEach(callback => {
                callback({
                    nodeId: currentNodeId,
                    nodeType: node.type,
                    input: context,
                    output: stepResult.output,
                    success: stepResult.success,
                    duration: stepResult.duration,
                    timestamp: new Date()
                });
            });
            
            executionPath.push(currentNodeId);
            
            // 다음 노드 결정
            if (stepResult.success && stepResult.nextNodeId) {
                currentNodeId = stepResult.nextNodeId;
            } else {
                break;
            }
        }
        
        return {
            success: true,
            executionPath,
            finalContext: context,
            totalSteps: executionPath.length,
            duration: Date.now() - startTime
        };
    }
    
    private async executeNode(
        node: WorkflowNode, 
        context: Record<string, any>
    ): Promise<NodeExecutionResult> {
        const startTime = Date.now();
        
        try {
            switch (node.type) {
                case 'trigger':
                    return this.executeTriggerNode(node, context);
                    
                case 'condition':
                    return await this.executeConditionNode(node, context);
                    
                case 'action':
                    return await this.executeActionNode(node, context);
                    
                case 'decision':
                    return this.executeDecisionNode(node, context);
                    
                case 'end':
                    return {
                        success: true,
                        output: context,
                        nextNodeId: null,
                        duration: Date.now() - startTime
                    };
                    
                default:
                    throw new Error(`Unknown node type: ${node.type}`);
            }
        } catch (error) {
            return {
                success: false,
                output: null,
                error: error.message,
                nextNodeId: null,
                duration: Date.now() - startTime
            };
        }
    }
    
    private async executeConditionNode(
        node: WorkflowNode, 
        context: Record<string, any>
    ): Promise<NodeExecutionResult> {
        const { judgmentMethod, ruleExpression, llmCriteria } = node.data;
        
        let result = false;
        let explanation = '';
        
        if (judgmentMethod === 'rule' && ruleExpression) {
            // 규칙 기반 판단 시뮬레이션
            result = this.evaluateRuleExpression(ruleExpression, context);
            explanation = `Rule "${ruleExpression}" evaluated to ${result}`;
            
        } else if (judgmentMethod === 'llm' && llmCriteria) {
            // LLM 판단 시뮬레이션 (Mock)
            result = this.simulateLLMJudgment(llmCriteria, context);
            explanation = `AI判定模拟结果: ${result}`;
            
        } else if (judgmentMethod === 'hybrid') {
            // 하이브리드 판단 시뮬레이션
            if (ruleExpression) {
                try {
                    result = this.evaluateRuleExpression(ruleExpression, context);
                    explanation = `Hybrid (Rule): ${result}`;
                } catch (error) {
                    result = this.simulateLLMJudgment(llmCriteria, context);
                    explanation = `Hybrid (AI fallback): ${result}`;
                }
            } else {
                result = this.simulateLLMJudgment(llmCriteria, context);
                explanation = `Hybrid (AI): ${result}`;
            }
        }
        
        // 다음 노드 결정 (true/false 경로)
        const nextNodeId = this.findNextNodeByCondition(node.id, result);
        
        return {
            success: true,
            output: { ...context, [`${node.id}_result`]: result, [`${node.id}_explanation`]: explanation },
            nextNodeId,
            duration: Math.random() * 1000 + 500 // 500-1500ms 시뮬레이션
        };
    }
    
    private evaluateRuleExpression(expression: string, context: Record<string, any>): boolean {
        // 안전한 규칙 평가 (실제로는 백엔드와 동일한 로직 사용)
        try {
            // 간단한 표현식 파싱 (실제 구현에서는 더 정교한 파서 필요)
            const sanitizedExpression = expression
                .replace(/\b(\w+)\b/g, (match) => {
                    if (context.hasOwnProperty(match)) {
                        const value = context[match];
                        return typeof value === 'string' ? `"${value}"` : String(value);
                    }
                    return match;
                });
            
            // 위험한 함수 호출 제거
            const safeExpression = sanitizedExpression
                .replace(/[^0-9a-zA-Z\s><=!&|()."'-]/g, '')
                .replace(/\b(eval|function|return|var|let|const)\b/g, '');
            
            return eval(safeExpression);
        } catch (error) {
            console.warn('Rule evaluation failed:', error);
            return false;
        }
    }
    
    private simulateLLMJudgment(criteria: string, context: Record<string, any>): boolean {
        // LLM 판단 시뮬레이션 (실제로는 API 호출)
        // 여기서는 간단한 휴리스틱 사용
        const contextString = JSON.stringify(context).toLowerCase();
        const criteriaLower = criteria.toLowerCase();
        
        const positiveKeywords = ['high', 'above', 'exceed', 'dangerous', 'alert', 'warning'];
        const negativeKeywords = ['low', 'below', 'normal', 'safe', 'ok', 'good'];
        
        let score = 0;
        
        positiveKeywords.forEach(keyword => {
            if (contextString.includes(keyword) || criteriaLower.includes(keyword)) {
                score += 1;
            }
        });
        
        negativeKeywords.forEach(keyword => {
            if (contextString.includes(keyword) || criteriaLower.includes(keyword)) {
                score -= 1;
            }
        });
        
        return score > 0;
    }
}
```

## 🔍 5. 실시간 검증 시스템

### 5.1 워크플로우 검증기
```typescript
interface ValidationError {
    nodeId?: string;
    edgeId?: string;
    type: 'error' | 'warning';
    code: string;
    message: string;
    suggestion?: string;
}

class WorkflowValidator {
    private workflow: Workflow;
    private errors: ValidationError[] = [];
    
    constructor(workflow: Workflow) {
        this.workflow = workflow;
    }
    
    validate(): ValidationResult {
        this.errors = [];
        
        this.validateStructure();
        this.validateNodes();
        this.validateEdges();
        this.validateFlow();
        
        return {
            isValid: this.errors.filter(e => e.type === 'error').length === 0,
            errors: this.errors.filter(e => e.type === 'error'),
            warnings: this.errors.filter(e => e.type === 'warning'),
            score: this.calculateScore()
        };
    }
    
    private validateStructure() {
        // 트리거 노드 존재 확인
        const triggerNodes = this.workflow.nodes.filter(n => n.type === 'trigger');
        if (triggerNodes.length === 0) {
            this.errors.push({
                type: 'error',
                code: 'NO_TRIGGER',
                message: 'Workflow must have at least one trigger node',
                suggestion: 'Add a trigger node to start the workflow'
            });
        } else if (triggerNodes.length > 1) {
            this.errors.push({
                type: 'warning',
                code: 'MULTIPLE_TRIGGERS',
                message: 'Multiple trigger nodes found',
                suggestion: 'Consider using a single trigger with multiple conditions'
            });
        }
        
        // 고립된 노드 확인
        this.workflow.nodes.forEach(node => {
            const hasIncoming = this.workflow.edges.some(e => e.target === node.id);
            const hasOutgoing = this.workflow.edges.some(e => e.source === node.id);
            
            if (!hasIncoming && node.type !== 'trigger') {
                this.errors.push({
                    nodeId: node.id,
                    type: 'warning',
                    code: 'ISOLATED_NODE',
                    message: `Node "${node.data.label || node.id}" has no incoming connections`,
                    suggestion: 'Connect this node to the workflow'
                });
            }
            
            if (!hasOutgoing && node.type !== 'end') {
                this.errors.push({
                    nodeId: node.id,
                    type: 'warning',
                    code: 'DEAD_END',
                    message: `Node "${node.data.label || node.id}" has no outgoing connections`,
                    suggestion: 'Add connections or use an end node'
                });
            }
        });
    }
    
    private validateNodes() {
        this.workflow.nodes.forEach(node => {
            switch (node.type) {
                case 'condition':
                    this.validateConditionNode(node);
                    break;
                case 'action':
                    this.validateActionNode(node);
                    break;
                case 'decision':
                    this.validateDecisionNode(node);
                    break;
            }
        });
    }
    
    private validateConditionNode(node: WorkflowNode) {
        const { judgmentMethod, ruleExpression, llmCriteria, confidenceThreshold } = node.data;
        
        if (!judgmentMethod) {
            this.errors.push({
                nodeId: node.id,
                type: 'error',
                code: 'MISSING_JUDGMENT_METHOD',
                message: 'Judgment method is required',
                suggestion: 'Select rule-based, AI-based, or hybrid judgment'
            });
        }
        
        if ((judgmentMethod === 'rule' || judgmentMethod === 'hybrid') && !ruleExpression) {
            this.errors.push({
                nodeId: node.id,
                type: 'error',
                code: 'MISSING_RULE_EXPRESSION',
                message: 'Rule expression is required for rule-based judgment',
                suggestion: 'Enter a valid rule expression (e.g., temperature > 85)'
            });
        }
        
        if ((judgmentMethod === 'llm' || judgmentMethod === 'hybrid') && !llmCriteria) {
            this.errors.push({
                nodeId: node.id,
                type: 'error',
                code: 'MISSING_LLM_CRITERIA',
                message: 'AI criteria is required for AI-based judgment',
                suggestion: 'Describe the judgment criteria for AI'
            });
        }
        
        // 규칙 표현식 구문 검증
        if (ruleExpression) {
            try {
                this.validateRuleSyntax(ruleExpression);
            } catch (error) {
                this.errors.push({
                    nodeId: node.id,
                    type: 'error',
                    code: 'INVALID_RULE_SYNTAX',
                    message: `Invalid rule syntax: ${error.message}`,
                    suggestion: 'Check the rule expression syntax'
                });
            }
        }
        
        // 신뢰도 임계값 검증
        if (confidenceThreshold !== undefined && (confidenceThreshold < 0 || confidenceThreshold > 1)) {
            this.errors.push({
                nodeId: node.id,
                type: 'error',
                code: 'INVALID_CONFIDENCE_THRESHOLD',
                message: 'Confidence threshold must be between 0 and 1',
                suggestion: 'Set a value between 0.0 and 1.0'
            });
        }
    }
    
    private validateRuleSyntax(expression: string) {
        // 기본적인 구문 검증
        const allowedTokens = /^[a-zA-Z_][a-zA-Z0-9_]*|[0-9]+\.?[0-9]*|[><=!&|()"\s+-]$/;
        const dangerousPatterns = /\b(eval|function|return|import|require|process|global|window)\b/;
        
        if (dangerousPatterns.test(expression)) {
            throw new Error('Dangerous functions are not allowed');
        }
        
        // 괄호 균형 검사
        let parenthesesCount = 0;
        for (const char of expression) {
            if (char === '(') parenthesesCount++;
            if (char === ')') parenthesesCount--;
            if (parenthesesCount < 0) {
                throw new Error('Unmatched closing parenthesis');
            }
        }
        
        if (parenthesesCount !== 0) {
            throw new Error('Unmatched opening parenthesis');
        }
    }
    
    private calculateScore(): number {
        const totalNodes = this.workflow.nodes.length;
        const errorCount = this.errors.filter(e => e.type === 'error').length;
        const warningCount = this.errors.filter(e => e.type === 'warning').length;
        
        if (totalNodes === 0) return 0;
        
        let score = 100;
        score -= errorCount * 20; // 오류당 -20점
        score -= warningCount * 5; // 경고당 -5점
        
        return Math.max(0, score);
    }
}

// 실시간 검증 훅
const useWorkflowValidation = (workflow: Workflow) => {
    const [validation, setValidation] = useState<ValidationResult>({
        isValid: true,
        errors: [],
        warnings: [],
        score: 100
    });
    
    const [debounceTimer, setDebounceTimer] = useState<NodeJS.Timeout | null>(null);
    
    useEffect(() => {
        // 디바운스를 적용한 검증
        if (debounceTimer) {
            clearTimeout(debounceTimer);
        }
        
        const timer = setTimeout(() => {
            const validator = new WorkflowValidator(workflow);
            const result = validator.validate();
            setValidation(result);
        }, 500); // 500ms 디바운스
        
        setDebounceTimer(timer);
        
        return () => {
            if (timer) clearTimeout(timer);
        };
    }, [workflow]);
    
    return validation;
};
```

### 5.2 검증 결과 UI 컴포넌트
```typescript
const ValidationPanel: React.FC<{
    validation: ValidationResult;
    onNodeSelect: (nodeId: string) => void;
}> = ({ validation, onNodeSelect }) => {
    const getScoreColor = (score: number) => {
        if (score >= 80) return 'text-green-600';
        if (score >= 60) return 'text-yellow-600';
        return 'text-red-600';
    };
    
    const getScoreBadge = (score: number) => {
        if (score >= 80) return 'bg-green-100 text-green-800';
        if (score >= 60) return 'bg-yellow-100 text-yellow-800';
        return 'bg-red-100 text-red-800';
    };
    
    return (
        <div className="validation-panel bg-white border-t border-gray-200 p-4">
            <div className="flex items-center justify-between mb-4">
                <h3 className="text-lg font-semibold">Validation</h3>
                <div className={`px-3 py-1 rounded-full text-sm font-medium ${getScoreBadge(validation.score)}`}>
                    Score: {validation.score}/100
                </div>
            </div>
            
            {validation.isValid ? (
                <div className="flex items-center text-green-600">
                    <CheckCircleIcon className="w-5 h-5 mr-2" />
                    Workflow is valid and ready to deploy
                </div>
            ) : (
                <div className="space-y-3">
                    {validation.errors.map((error, index) => (
                        <div
                            key={index}
                            className="flex items-start p-3 bg-red-50 border border-red-200 rounded-lg cursor-pointer hover:bg-red-100"
                            onClick={() => error.nodeId && onNodeSelect(error.nodeId)}
                        >
                            <XCircleIcon className="w-5 h-5 text-red-500 mr-3 mt-0.5 flex-shrink-0" />
                            <div className="flex-1">
                                <div className="font-medium text-red-800">
                                    {error.message}
                                </div>
                                {error.suggestion && (
                                    <div className="text-sm text-red-600 mt-1">
                                        💡 {error.suggestion}
                                    </div>
                                )}
                                {error.nodeId && (
                                    <div className="text-xs text-red-500 mt-1">
                                        Node: {error.nodeId}
                                    </div>
                                )}
                            </div>
                        </div>
                    ))}
                    
                    {validation.warnings.map((warning, index) => (
                        <div
                            key={index}
                            className="flex items-start p-3 bg-yellow-50 border border-yellow-200 rounded-lg cursor-pointer hover:bg-yellow-100"
                            onClick={() => warning.nodeId && onNodeSelect(warning.nodeId)}
                        >
                            <ExclamationTriangleIcon className="w-5 h-5 text-yellow-500 mr-3 mt-0.5 flex-shrink-0" />
                            <div className="flex-1">
                                <div className="font-medium text-yellow-800">
                                    {warning.message}
                                </div>
                                {warning.suggestion && (
                                    <div className="text-sm text-yellow-600 mt-1">
                                        💡 {warning.suggestion}
                                    </div>
                                )}
                            </div>
                        </div>
                    ))}
                </div>
            )}
        </div>
    );
};
```

## 💾 6. 워크플로우 저장 및 버전 관리

### 6.1 워크플로우 저장 훅
```typescript
const useWorkflowSave = (workflowId?: string) => {
    const [isSaving, setIsSaving] = useState(false);
    const [lastSaved, setLastSaved] = useState<Date | null>(null);
    const [hasUnsavedChanges, setHasUnsavedChanges] = useState(false);
    
    const saveWorkflow = async (workflow: Workflow, options: SaveOptions = {}) => {
        setIsSaving(true);
        
        try {
            const payload = {
                ...workflow,
                version: workflow.version + (options.createNewVersion ? 1 : 0),
                changeSummary: options.changeSummary
            };
            
            let response;
            if (workflowId && !options.createNewVersion) {
                // 기존 워크플로우 업데이트
                response = await api.put(`/workflows/${workflowId}`, payload);
            } else {
                // 새 워크플로우 생성 또는 새 버전 생성
                response = await api.post('/workflows', payload);
            }
            
            setLastSaved(new Date());
            setHasUnsavedChanges(false);
            
            return response.data;
            
        } catch (error) {
            console.error('Failed to save workflow:', error);
            throw error;
        } finally {
            setIsSaving(false);
        }
    };
    
    const autoSave = useCallback(
        debounce(async (workflow: Workflow) => {
            if (hasUnsavedChanges && workflowId) {
                try {
                    await saveWorkflow(workflow, { autoSave: true });
                } catch (error) {
                    console.warn('Auto-save failed:', error);
                }
            }
        }, 30000), // 30초마다 자동 저장
        [hasUnsavedChanges, workflowId]
    );
    
    return {
        isSaving,
        lastSaved,
        hasUnsavedChanges,
        saveWorkflow,
        autoSave,
        setHasUnsavedChanges
    };
};

interface SaveOptions {
    createNewVersion?: boolean;
    changeSummary?: string;
    autoSave?: boolean;
}
```

### 6.2 템플릿 관리 시스템
```typescript
const WorkflowTemplates: React.FC = () => {
    const [templates, setTemplates] = useState<WorkflowTemplate[]>([]);
    const [selectedCategory, setSelectedCategory] = useState<string>('all');
    
    const templateCategories = [
        { id: 'all', name: 'All Templates', icon: '📋' },
        { id: 'manufacturing', name: 'Manufacturing', icon: '🏭' },
        { id: 'quality', name: 'Quality Control', icon: '✅' },
        { id: 'safety', name: 'Safety', icon: '🛡️' },
        { id: 'maintenance', name: 'Maintenance', icon: '🔧' },
        { id: 'custom', name: 'My Templates', icon: '👤' }
    ];
    
    const predefinedTemplates: WorkflowTemplate[] = [
        {
            id: 'temp-monitor',
            name: 'Temperature Monitoring',
            description: 'Monitor machine temperature and alert when threshold exceeded',
            category: 'manufacturing',
            preview: '/templates/temp-monitor-preview.png',
            workflow: {
                nodes: [
                    {
                        id: 'trigger-1',
                        type: 'trigger',
                        data: {
                            label: 'Temperature Sensor',
                            triggerType: 'sensor',
                            config: { sensorId: 'temp_sensor_01' }
                        },
                        position: { x: 100, y: 100 }
                    },
                    {
                        id: 'condition-1',
                        type: 'condition',
                        data: {
                            label: 'Check Temperature',
                            judgmentMethod: 'rule',
                            ruleExpression: 'temperature > 85'
                        },
                        position: { x: 300, y: 100 }
                    },
                    {
                        id: 'action-1',
                        type: 'action',
                        data: {
                            label: 'Send Alert',
                            actionType: 'notification',
                            config: {
                                channels: ['slack', 'email'],
                                message: 'High temperature detected: {{temperature}}°C'
                            }
                        },
                        position: { x: 500, y: 100 }
                    }
                ],
                edges: [
                    { id: 'e1', source: 'trigger-1', target: 'condition-1' },
                    { id: 'e2', source: 'condition-1', target: 'action-1', data: { condition: 'true' } }
                ]
            },
            tags: ['temperature', 'alert', 'sensor'],
            usageCount: 156,
            rating: 4.8
        },
        // 더 많은 템플릿...
    ];
    
    const createWorkflowFromTemplate = async (template: WorkflowTemplate) => {
        const newWorkflow: Workflow = {
            id: generateId(),
            name: `${template.name} - Copy`,
            description: template.description,
            nodes: template.workflow.nodes.map(node => ({
                ...node,
                id: generateId() // 새로운 ID 생성
            })),
            edges: template.workflow.edges.map(edge => ({
                ...edge,
                id: generateId(),
                source: findNewNodeId(edge.source, template.workflow.nodes),
                target: findNewNodeId(edge.target, template.workflow.nodes)
            })),
            version: 1,
            status: 'draft',
            tags: template.tags,
            createdAt: new Date(),
            updatedAt: new Date()
        };
        
        // 편집기로 이동
        router.push(`/workflows/editor?template=${newWorkflow.id}`);
    };
    
    return (
        <div className="workflow-templates p-6">
            <div className="mb-6">
                <h2 className="text-2xl font-bold mb-2">Workflow Templates</h2>
                <p className="text-gray-600">
                    Start with a pre-built template or create your own from scratch
                </p>
            </div>
            
            {/* 카테고리 필터 */}
            <div className="flex gap-2 mb-6 overflow-x-auto">
                {templateCategories.map(category => (
                    <button
                        key={category.id}
                        onClick={() => setSelectedCategory(category.id)}
                        className={`
                            flex items-center gap-2 px-4 py-2 rounded-lg whitespace-nowrap
                            ${selectedCategory === category.id 
                                ? 'bg-blue-100 text-blue-700 border border-blue-300' 
                                : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
                            }
                        `}
                    >
                        <span>{category.icon}</span>
                        {category.name}
                    </button>
                ))}
            </div>
            
            {/* 템플릿 그리드 */}
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                {predefinedTemplates
                    .filter(template => 
                        selectedCategory === 'all' || template.category === selectedCategory
                    )
                    .map(template => (
                        <div
                            key={template.id}
                            className="template-card bg-white border border-gray-200 rounded-lg p-4 hover:shadow-md transition-shadow cursor-pointer"
                            onClick={() => createWorkflowFromTemplate(template)}
                        >
                            <div className="mb-3">
                                <img
                                    src={template.preview}
                                    alt={template.name}
                                    className="w-full h-32 object-cover rounded-md bg-gray-100"
                                />
                            </div>
                            
                            <h3 className="font-semibold text-lg mb-2">{template.name}</h3>
                            <p className="text-gray-600 text-sm mb-3 line-clamp-2">
                                {template.description}
                            </p>
                            
                            <div className="flex items-center justify-between text-sm text-gray-500 mb-3">
                                <span>Used {template.usageCount} times</span>
                                <div className="flex items-center gap-1">
                                    <StarIcon className="w-4 h-4 text-yellow-400" />
                                    {template.rating}
                                </div>
                            </div>
                            
                            <div className="flex flex-wrap gap-1">
                                {template.tags.slice(0, 3).map(tag => (
                                    <span
                                        key={tag}
                                        className="px-2 py-1 bg-gray-100 text-gray-600 text-xs rounded"
                                    >
                                        {tag}
                                    </span>
                                ))}
                                {template.tags.length > 3 && (
                                    <span className="px-2 py-1 bg-gray-100 text-gray-600 text-xs rounded">
                                        +{template.tags.length - 3}
                                    </span>
                                )}
                            </div>
                        </div>
                    ))
                }
            </div>
        </div>
    );
};
```

## 🚀 7. 성능 최적화

### 7.1 가상화된 노드 렌더링
```typescript
// 대규모 워크플로우를 위한 가상화
const VirtualizedWorkflow: React.FC<{
    workflow: Workflow;
    viewportSize: { width: number; height: number };
}> = ({ workflow, viewportSize }) => {
    const [visibleNodes, setVisibleNodes] = useState<WorkflowNode[]>([]);
    const [visibleEdges, setVisibleEdges] = useState<WorkflowEdge[]>([]);
    
    const updateVisibleElements = useCallback((viewport: { x: number; y: number; zoom: number }) => {
        const buffer = 200; // 뷰포트 밖 여유 공간
        
        const visible = workflow.nodes.filter(node => {
            const nodeX = node.position.x * viewport.zoom + viewport.x;
            const nodeY = node.position.y * viewport.zoom + viewport.y;
            
            return (
                nodeX > -buffer &&
                nodeX < viewportSize.width + buffer &&
                nodeY > -buffer &&
                nodeY < viewportSize.height + buffer
            );
        });
        
        setVisibleNodes(visible);
        
        // 보이는 노드와 연결된 엣지만 렌더링
        const visibleNodeIds = new Set(visible.map(n => n.id));
        const visibleEdges = workflow.edges.filter(edge =>
            visibleNodeIds.has(edge.source) || visibleNodeIds.has(edge.target)
        );
        
        setVisibleEdges(visibleEdges);
    }, [workflow, viewportSize]);
    
    return (
        <ReactFlow
            nodes={visibleNodes}
            edges={visibleEdges}
            onViewportChange={updateVisibleElements}
            // ... 기타 props
        />
    );
};
```

### 7.2 메모이제이션 최적화
```typescript
// 노드 컴포넌트 메모이제이션
const MemoizedTriggerNode = React.memo<NodeProps<TriggerNodeData>>(
    TriggerNode,
    (prevProps, nextProps) => {
        // 깊은 비교 대신 필요한 속성만 비교
        return (
            prevProps.id === nextProps.id &&
            prevProps.selected === nextProps.selected &&
            JSON.stringify(prevProps.data) === JSON.stringify(nextProps.data) &&
            prevProps.xPos === nextProps.xPos &&
            prevProps.yPos === nextProps.yPos
        );
    }
);

// 상태 업데이트 최적화
const useOptimizedWorkflowState = () => {
    const [workflow, setWorkflow] = useState<Workflow | null>(null);
    
    const updateNode = useCallback((nodeId: string, updates: Partial<WorkflowNode>) => {
        setWorkflow(prev => {
            if (!prev) return null;
            
            return {
                ...prev,
                nodes: prev.nodes.map(node =>
                    node.id === nodeId
                        ? { ...node, ...updates }
                        : node
                ),
                updatedAt: new Date()
            };
        });
    }, []);
    
    const updateEdge = useCallback((edgeId: string, updates: Partial<WorkflowEdge>) => {
        setWorkflow(prev => {
            if (!prev) return null;
            
            return {
                ...prev,
                edges: prev.edges.map(edge =>
                    edge.id === edgeId
                        ? { ...edge, ...updates }
                        : edge
                ),
                updatedAt: new Date()
            };
        });
    }, []);
    
    return {
        workflow,
        setWorkflow,
        updateNode,
        updateEdge
    };
};
```

## 📱 8. 반응형 설계 및 모바일 지원

### 8.1 모바일 친화적 인터페이스
```typescript
const ResponsiveWorkflowEditor: React.FC = () => {
    const [isMobile, setIsMobile] = useState(false);
    const [sidebarCollapsed, setSidebarCollapsed] = useState(false);
    
    useEffect(() => {
        const checkMobile = () => {
            setIsMobile(window.innerWidth < 768);
            setSidebarCollapsed(window.innerWidth < 1024);
        };
        
        checkMobile();
        window.addEventListener('resize', checkMobile);
        
        return () => window.removeEventListener('resize', checkMobile);
    }, []);
    
    if (isMobile) {
        return <MobileWorkflowEditor />;
    }
    
    return (
        <div className="workflow-editor-desktop">
            {/* 데스크톱 레이아웃 */}
        </div>
    );
};

const MobileWorkflowEditor: React.FC = () => {
    const [activeTab, setActiveTab] = useState<'canvas' | 'palette' | 'properties'>('canvas');
    
    return (
        <div className="mobile-workflow-editor h-screen flex flex-col">
            {/* 모바일 헤더 */}
            <header className="bg-white border-b border-gray-200 p-4">
                <div className="flex items-center justify-between">
                    <h1 className="text-lg font-semibold">Workflow Editor</h1>
                    <button className="p-2 rounded-md hover:bg-gray-100">
                        <Bars3Icon className="w-5 h-5" />
                    </button>
                </div>
            </header>
            
            {/* 메인 콘텐츠 */}
            <main className="flex-1 overflow-hidden">
                {activeTab === 'canvas' && <MobileCanvas />}
                {activeTab === 'palette' && <MobileNodePalette />}
                {activeTab === 'properties' && <MobileProperties />}
            </main>
            
            {/* 하단 탭 네비게이션 */}
            <nav className="bg-white border-t border-gray-200">
                <div className="flex">
                    {[
                        { id: 'canvas', label: 'Canvas', icon: Square3Stack3DIcon },
                        { id: 'palette', label: 'Nodes', icon: Square2StackIcon },
                        { id: 'properties', label: 'Properties', icon: AdjustmentsHorizontalIcon }
                    ].map(tab => (
                        <button
                            key={tab.id}
                            onClick={() => setActiveTab(tab.id as any)}
                            className={`
                                flex-1 flex flex-col items-center justify-center p-3
                                ${activeTab === tab.id 
                                    ? 'text-blue-600 bg-blue-50' 
                                    : 'text-gray-600'
                                }
                            `}
                        >
                            <tab.icon className="w-5 h-5 mb-1" />
                            <span className="text-xs">{tab.label}</span>
                        </button>
                    ))}
                </div>
            </nav>
        </div>
    );
};
```

## 🔄 9. 다음 문서 연결

이 워크플로우 편집기 구현 명세서를 기반으로 다음 문서들이 작성됩니다:

1. **외부 시스템 연동 가이드**: MCP 및 산업제어시스템과의 실제 연동 방법
2. **모니터링 및 운영 가이드**: 시스템 운영, 성능 모니터링, 장애 대응 방안

각 문서는 이 편집기에서 생성된 워크플로우가 실제 시스템에서 어떻게 실행되고 관리되는지를 다룰 예정입니다.