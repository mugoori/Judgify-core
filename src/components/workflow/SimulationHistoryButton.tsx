/**
 * 시뮬레이션 히스토리 버튼 및 팝오버 (Week 6 Task 4)
 *
 * IndexedDB에 저장된 과거 시뮬레이션 기록을 표시하고,
 * 재실행 기능을 제공합니다.
 */

import { useState, useEffect } from 'react';
import { History, Clock, CheckCircle2, XCircle, Trash2, RotateCcw } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { ScrollArea } from '@/components/ui/scroll-area';
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@/components/ui/popover';
import { cn } from '@/lib/utils';
import {
  historyStorage,
  type SimulationHistory,
} from '@/lib/simulation-history-storage';

interface SimulationHistoryButtonProps {
  onLoadHistory: (history: SimulationHistory) => void;
}

export function SimulationHistoryButton({
  onLoadHistory,
}: SimulationHistoryButtonProps) {
  const [histories, setHistories] = useState<SimulationHistory[]>([]);
  const [loading, setLoading] = useState(false);
  const [isOpen, setIsOpen] = useState(false);

  // 히스토리 목록 로드
  const loadHistories = async () => {
    setLoading(true);
    try {
      const list = await historyStorage.getHistoryList(20);
      setHistories(list);
    } catch (error) {
      console.error('[HistoryButton] Failed to load histories:', error);
    } finally {
      setLoading(false);
    }
  };

  // 팝오버 열릴 때 히스토리 로드
  useEffect(() => {
    if (isOpen) {
      loadHistories();
    }
  }, [isOpen]);

  // 히스토리 삭제
  const handleDelete = async (id: string, event: React.MouseEvent) => {
    event.stopPropagation();

    try {
      await historyStorage.deleteHistory(id);
      await loadHistories(); // 목록 갱신
    } catch (error) {
      console.error('[HistoryButton] Failed to delete:', error);
    }
  };

  // 히스토리 재실행
  const handleLoad = (history: SimulationHistory) => {
    onLoadHistory(history);
    setIsOpen(false);
  };

  // 시간 포맷 (상대 시간)
  const formatTime = (timestamp: number): string => {
    const now = Date.now();
    const diff = now - timestamp;

    const minutes = Math.floor(diff / 60000);
    const hours = Math.floor(diff / 3600000);
    const days = Math.floor(diff / 86400000);

    if (minutes < 1) return '방금 전';
    if (minutes < 60) return `${minutes}분 전`;
    if (hours < 24) return `${hours}시간 전`;
    return `${days}일 전`;
  };

  // 실행 시간 포맷
  const formatDuration = (ms: number): string => {
    if (ms < 1000) return `${ms}ms`;
    return `${(ms / 1000).toFixed(1)}초`;
  };

  // 상태 아이콘
  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'success':
        return <CheckCircle2 className="w-3 h-3 text-green-600" />;
      case 'error':
        return <XCircle className="w-3 h-3 text-red-600" />;
      default:
        return <Clock className="w-3 h-3 text-yellow-600" />;
    }
  };

  // 상태 배지 색상
  const getStatusColor = (status: string): string => {
    switch (status) {
      case 'success':
        return 'bg-green-100 text-green-800';
      case 'error':
        return 'bg-red-100 text-red-800';
      default:
        return 'bg-yellow-100 text-yellow-800';
    }
  };

  return (
    <Popover open={isOpen} onOpenChange={setIsOpen}>
      <PopoverTrigger asChild>
        <Button variant="outline" size="sm" data-testid="history-button">
          <History className="w-4 h-4 mr-2" />
          히스토리
          {histories.length > 0 && (
            <Badge variant="secondary" className="ml-2 text-xs">
              {histories.length}
            </Badge>
          )}
        </Button>
      </PopoverTrigger>

      <PopoverContent className="w-[400px] p-0" align="start">
        <div className="flex items-center justify-between p-4 border-b">
          <div>
            <h3 className="font-semibold">시뮬레이션 히스토리</h3>
            <p className="text-xs text-muted-foreground mt-1">
              최근 20개 실행 기록
            </p>
          </div>
        </div>

        <ScrollArea className="h-[400px]">
          {loading ? (
            <div className="p-8 text-center text-sm text-muted-foreground">
              히스토리를 불러오는 중...
            </div>
          ) : histories.length === 0 ? (
            <div className="p-8 text-center">
              <History className="w-12 h-12 mx-auto text-muted-foreground/50 mb-3" />
              <p className="text-sm text-muted-foreground">
                저장된 히스토리가 없습니다
              </p>
              <p className="text-xs text-muted-foreground mt-1">
                시뮬레이션을 실행하면 자동으로 저장됩니다
              </p>
            </div>
          ) : (
            <div className="p-2 space-y-2">
              {histories.map((history) => (
                <div
                  key={history.id}
                  className="group relative p-3 rounded-lg border hover:bg-accent cursor-pointer transition-colors"
                  onClick={() => handleLoad(history)}
                  data-testid={`history-item-${history.id}`}
                >
                  <div className="flex items-start justify-between gap-2">
                    <div className="flex-1 min-w-0">
                      {/* 워크플로우 이름 */}
                      <div className="flex items-center gap-2 mb-1">
                        <span className="font-medium text-sm truncate">
                          {history.workflow_name}
                        </span>
                        <Badge
                          className={cn('text-xs', getStatusColor(history.status))}
                        >
                          <span className="flex items-center gap-1">
                            {getStatusIcon(history.status)}
                            {history.status === 'success' && '성공'}
                            {history.status === 'error' && '실패'}
                            {history.status === 'partial' && '부분'}
                          </span>
                        </Badge>
                      </div>

                      {/* 실행 정보 */}
                      <div className="flex items-center gap-3 text-xs text-muted-foreground">
                        <span className="flex items-center gap-1">
                          <Clock className="w-3 h-3" />
                          {formatTime(history.timestamp)}
                        </span>
                        <span>{formatDuration(history.duration_ms)}</span>
                        <span>{history.steps.length}단계</span>
                      </div>

                      {/* 초기 데이터 (간략) */}
                      {Object.keys(history.initial_data).length > 0 && (
                        <div className="mt-2 text-xs font-mono bg-muted/50 rounded p-1.5 truncate">
                          {Object.entries(history.initial_data)
                            .slice(0, 2)
                            .map(([key, value]) => `${key}: ${JSON.stringify(value)}`)
                            .join(', ')}
                          {Object.keys(history.initial_data).length > 2 && '...'}
                        </div>
                      )}
                    </div>

                    {/* 삭제 버튼 */}
                    <Button
                      variant="ghost"
                      size="icon"
                      className="opacity-0 group-hover:opacity-100 transition-opacity h-7 w-7"
                      onClick={(e) => handleDelete(history.id, e)}
                      data-testid={`delete-history-${history.id}`}
                    >
                      <Trash2 className="w-3 h-3 text-destructive" />
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </ScrollArea>

        {/* 전체 삭제 버튼 (히스토리가 있을 때만) */}
        {histories.length > 0 && (
          <div className="p-2 border-t">
            <Button
              variant="ghost"
              size="sm"
              className="w-full text-xs text-muted-foreground hover:text-destructive"
              onClick={async () => {
                if (confirm('모든 히스토리를 삭제하시겠습니까?')) {
                  await historyStorage.clearAll();
                  await loadHistories();
                }
              }}
            >
              <Trash2 className="w-3 h-3 mr-2" />
              전체 삭제
            </Button>
          </div>
        )}
      </PopoverContent>
    </Popover>
  );
}
