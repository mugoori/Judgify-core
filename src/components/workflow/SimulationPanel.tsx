/**
 * Workflow Simulation Panel
 *
 * 워크플로우 Step-by-step 시뮬레이션 UI
 */

import { useState, useEffect } from 'react';
import { Node, Edge } from 'reactflow';
import {
  Play,
  Pause,
  StepForward,
  StepBack,
  RotateCcw,
  CheckCircle2,
  XCircle,
  Clock,
  AlertCircle,
  Loader2,
} from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { ScrollArea } from '@/components/ui/scroll-area';
import { cn } from '@/lib/utils';
import {
  WorkflowSimulator,
  SimulationState,
  NodeStatus,
} from '@/lib/workflow-simulator';

interface SimulationPanelProps {
  nodes: Node[];
  edges: Edge[];
  initialData: Record<string, any>;
  onStepChange?: (stepIndex: number, nodeId: string) => void;
  onClose: () => void;
}

export function SimulationPanel({
  nodes,
  edges,
  initialData,
  onStepChange,
  onClose,
}: SimulationPanelProps) {
  const [simulator] = useState(() => new WorkflowSimulator(nodes, edges, initialData));
  const [state, setState] = useState<SimulationState>(simulator.getState());
  const [autoPlay, setAutoPlay] = useState(false);

  // 자동 재생
  useEffect(() => {
    if (!autoPlay || !state.isRunning || state.isPaused) return;

    const timer = setTimeout(async () => {
      const newState = await simulator.stepForward();
      setState(newState);
    }, 1500);

    return () => clearTimeout(timer);
  }, [autoPlay, state, simulator]);

  // 현재 단계 변경 시 콜백 호출
  useEffect(() => {
    if (state.currentStepIndex >= 0 && state.steps[state.currentStepIndex]) {
      const currentStep = state.steps[state.currentStepIndex];
      onStepChange?.(state.currentStepIndex, currentStep.nodeId);
    }
  }, [state.currentStepIndex, state.steps, onStepChange]);

  const handleStart = async () => {
    const newState = await simulator.start();
    setState(newState);
  };

  const handleStepForward = async () => {
    const newState = await simulator.stepForward();
    setState(newState);
  };

  const handleStepBackward = () => {
    const newState = simulator.stepBackward();
    setState(newState);
  };

  const handlePause = () => {
    setAutoPlay(false);
    const newState = simulator.pause();
    setState(newState);
  };

  const handleResume = () => {
    setAutoPlay(true);
    const newState = simulator.resume();
    setState(newState);
  };

  const handleReset = () => {
    setAutoPlay(false);
    const newState = simulator.reset();
    setState(newState);
  };

  const getStatusIcon = (status: NodeStatus) => {
    switch (status) {
      case 'success':
        return <CheckCircle2 className="w-4 h-4 text-green-600" />;
      case 'error':
        return <XCircle className="w-4 h-4 text-red-600" />;
      case 'running':
        return <Loader2 className="w-4 h-4 text-blue-600 animate-spin" />;
      case 'pending':
        return <Clock className="w-4 h-4 text-gray-400" />;
      case 'skipped':
        return <AlertCircle className="w-4 h-4 text-yellow-600" />;
    }
  };

  const getStatusBadge = (status: NodeStatus) => {
    const variants: Record<NodeStatus, string> = {
      success: 'bg-green-100 text-green-800',
      error: 'bg-red-100 text-red-800',
      running: 'bg-blue-100 text-blue-800',
      pending: 'bg-gray-100 text-gray-600',
      skipped: 'bg-yellow-100 text-yellow-800',
    };

    return (
      <Badge className={cn('text-xs', variants[status])}>
        {status === 'success' && '성공'}
        {status === 'error' && '실패'}
        {status === 'running' && '실행 중'}
        {status === 'pending' && '대기'}
        {status === 'skipped' && '건너뜀'}
      </Badge>
    );
  };

  const currentStep = state.steps[state.currentStepIndex];

  return (
    <Card className="w-96 h-full border-r shadow-lg">
      <CardHeader className="border-b">
        <div className="flex items-center justify-between">
          <CardTitle className="text-lg">시뮬레이션</CardTitle>
          <Button variant="ghost" size="sm" onClick={onClose}>
            닫기
          </Button>
        </div>
      </CardHeader>

      <CardContent className="p-4 space-y-4">
        {/* 컨트롤 버튼 */}
        <div className="flex gap-2">
          {!state.isRunning ? (
            <Button onClick={handleStart} size="sm" className="flex-1">
              <Play className="w-4 h-4 mr-2" />
              시작
            </Button>
          ) : (
            <>
              {state.isPaused || !autoPlay ? (
                <Button onClick={handleResume} size="sm" variant="outline">
                  <Play className="w-4 h-4 mr-2" />
                  재생
                </Button>
              ) : (
                <Button onClick={handlePause} size="sm" variant="outline">
                  <Pause className="w-4 h-4 mr-2" />
                  일시정지
                </Button>
              )}
            </>
          )}

          <Button
            onClick={handleStepBackward}
            size="sm"
            variant="outline"
            disabled={state.currentStepIndex <= 0}
          >
            <StepBack className="w-4 h-4" />
          </Button>

          <Button
            onClick={handleStepForward}
            size="sm"
            variant="outline"
            disabled={!state.isRunning && state.currentStepIndex >= state.steps.length - 1}
          >
            <StepForward className="w-4 h-4" />
          </Button>

          <Button onClick={handleReset} size="sm" variant="outline">
            <RotateCcw className="w-4 h-4" />
          </Button>
        </div>

        {/* 진행 상황 */}
        <div className="flex items-center justify-between text-sm">
          <span className="text-muted-foreground">
            단계: {state.currentStepIndex + 1} / {state.steps.length}
          </span>
          {currentStep && (
            <span className="text-xs text-muted-foreground">
              {currentStep.executionTimeMs}ms
            </span>
          )}
        </div>

        {/* 현재 단계 상세 */}
        {currentStep && (
          <Card className="bg-muted/50">
            <CardHeader className="p-3">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  {getStatusIcon(currentStep.status)}
                  <span className="font-semibold text-sm">{currentStep.nodeName}</span>
                </div>
                {getStatusBadge(currentStep.status)}
              </div>
              <p className="text-xs text-muted-foreground">{currentStep.nodeType} 노드</p>
            </CardHeader>
            <CardContent className="p-3 space-y-2">
              {/* 입력 데이터 */}
              <div>
                <p className="text-xs font-semibold mb-1">입력:</p>
                <pre className="text-xs bg-background p-2 rounded border overflow-auto max-h-20">
                  {JSON.stringify(currentStep.input, null, 2)}
                </pre>
              </div>

              {/* 출력 데이터 */}
              {currentStep.output && (
                <div>
                  <p className="text-xs font-semibold mb-1">출력:</p>
                  <pre className="text-xs bg-background p-2 rounded border overflow-auto max-h-20">
                    {JSON.stringify(currentStep.output, null, 2)}
                  </pre>
                </div>
              )}

              {/* 에러 메시지 */}
              {currentStep.error && (
                <div className="p-2 bg-red-50 border border-red-200 rounded">
                  <p className="text-xs font-semibold text-red-800">에러:</p>
                  <p className="text-xs text-red-700">{currentStep.error}</p>
                </div>
              )}
            </CardContent>
          </Card>
        )}

        {/* 실행 이력 */}
        <div>
          <p className="text-sm font-semibold mb-2">실행 이력</p>
          <ScrollArea className="h-64 border rounded">
            <div className="p-2 space-y-2">
              {state.steps.map((step, index) => (
                <div
                  key={index}
                  className={cn(
                    'p-2 rounded border cursor-pointer transition-colors',
                    index === state.currentStepIndex
                      ? 'bg-primary/10 border-primary'
                      : 'hover:bg-muted'
                  )}
                  onClick={() => {
                    // 특정 단계로 이동
                    const diff = index - state.currentStepIndex;
                    if (diff > 0) {
                      for (let i = 0; i < diff; i++) {
                        handleStepForward();
                      }
                    } else if (diff < 0) {
                      for (let i = 0; i < Math.abs(diff); i++) {
                        handleStepBackward();
                      }
                    }
                  }}
                >
                  <div className="flex items-center justify-between mb-1">
                    <div className="flex items-center gap-2">
                      {getStatusIcon(step.status)}
                      <span className="text-xs font-semibold">{step.nodeName}</span>
                    </div>
                    {step.executionTimeMs && (
                      <span className="text-xs text-muted-foreground">
                        {step.executionTimeMs}ms
                      </span>
                    )}
                  </div>
                  <p className="text-xs text-muted-foreground">{step.nodeType}</p>
                </div>
              ))}

              {state.steps.length === 0 && (
                <p className="text-xs text-center text-muted-foreground py-4">
                  시뮬레이션을 시작하세요
                </p>
              )}
            </div>
          </ScrollArea>
        </div>

        {/* 전역 데이터 상태 */}
        <div>
          <p className="text-sm font-semibold mb-2">전역 데이터</p>
          <pre className="text-xs bg-background p-2 rounded border overflow-auto max-h-32">
            {JSON.stringify(state.globalData, null, 2)}
          </pre>
        </div>
      </CardContent>
    </Card>
  );
}
