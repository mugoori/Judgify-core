import { useState, useEffect } from 'react';
import { Node } from 'reactflow';
import { X, Save, AlertCircle, CheckCircle2, Loader2 } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { cn } from '@/lib/utils';
import { useRuleValidation } from '@/hooks/useRuleValidation';

interface NodeEditPanelProps {
  node: Node;
  onUpdate: (nodeId: string, data: Partial<Node['data']>) => void;
  onClose: () => void;
}

export function NodeEditPanel({ node, onUpdate, onClose }: NodeEditPanelProps) {
  const [label, setLabel] = useState(node.data.label || '');
  const [description, setDescription] = useState(node.data.description || '');
  const [rule, setRule] = useState(node.data.rule || '');
  const [action, setAction] = useState(node.data.action || '');
  const [hasChanges, setHasChanges] = useState(false);

  // 실시간 Rule 검증 (Decision 노드만)
  const { isValid, errors, suggestions, isValidating } = useRuleValidation(rule, {
    enabled: node.data.type === 'decision',
    debounceMs: 500,
  });

  // 변경사항 추적
  useEffect(() => {
    const changed =
      label !== node.data.label ||
      description !== (node.data.description || '') ||
      rule !== (node.data.rule || '') ||
      action !== (node.data.action || '');
    setHasChanges(changed);
  }, [label, description, rule, action, node.data]);

  const handleSave = () => {
    onUpdate(node.id, {
      label,
      description,
      rule: node.data.type === 'decision' ? rule : undefined,
      action: node.data.type === 'action' ? action : undefined,
    });
    onClose();
  };

  const handleCancel = () => {
    if (hasChanges) {
      if (window.confirm('저장하지 않은 변경사항이 있습니다. 취소하시겠습니까?')) {
        onClose();
      }
    } else {
      onClose();
    }
  };

  // 노드 타입별 아이콘 색상 (미사용 - 직접 className에서 처리)

  return (
    <Card className="w-[400px] h-full border-l shadow-lg">
      <CardHeader className="border-b">
        <div className="flex items-center justify-between">
          <CardTitle className="text-lg flex items-center gap-2">
            <div
              className={cn(
                'w-3 h-3 rounded-full',
                node.data.type === 'input' && 'bg-blue-500',
                node.data.type === 'decision' && 'bg-purple-500',
                node.data.type === 'action' && 'bg-yellow-500',
                node.data.type === 'output' && 'bg-green-500'
              )}
            />
            노드 편집
          </CardTitle>
          <Button variant="ghost" size="icon" onClick={handleCancel}>
            <X className="h-4 w-4" />
          </Button>
        </div>
        <p className="text-sm text-muted-foreground">
          {node.data.type === 'input' && '데이터 입력 노드'}
          {node.data.type === 'decision' && '판단 로직 노드'}
          {node.data.type === 'action' && '액션 실행 노드'}
          {node.data.type === 'output' && '결과 출력 노드'}
        </p>
      </CardHeader>

      <CardContent className="space-y-4 p-6">
        {/* 노드 라벨 */}
        <div className="space-y-2">
          <Label htmlFor="label">노드 이름</Label>
          <Input
            id="label"
            value={label}
            onChange={(e) => setLabel(e.target.value)}
            placeholder="예: 재고 확인"
          />
        </div>

        {/* 노드 설명 */}
        <div className="space-y-2">
          <Label htmlFor="description">설명</Label>
          <Textarea
            id="description"
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            placeholder="이 노드가 수행하는 작업을 설명해주세요"
            rows={3}
          />
        </div>

        {/* Decision 노드: Rule 표현식 */}
        {node.data.type === 'decision' && (
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <Label htmlFor="rule">판단 규칙 (Rule Expression)</Label>
              {/* 검증 상태 표시 */}
              {rule.trim() && (
                <div className="flex items-center gap-1 text-xs">
                  {isValidating ? (
                    <>
                      <Loader2 className="w-3 h-3 animate-spin text-gray-500" />
                      <span className="text-gray-500">검증 중...</span>
                    </>
                  ) : isValid ? (
                    <>
                      <CheckCircle2 className="w-3 h-3 text-green-600" />
                      <span className="text-green-600">유효함</span>
                    </>
                  ) : (
                    <>
                      <AlertCircle className="w-3 h-3 text-red-600" />
                      <span className="text-red-600">오류</span>
                    </>
                  )}
                </div>
              )}
            </div>
            <Textarea
              id="rule"
              value={rule}
              onChange={(e) => setRule(e.target.value)}
              placeholder="예: 재고 < 10 && 상태 == '정상'"
              rows={4}
              className={cn(
                'font-mono text-sm',
                rule.trim() && !isValidating && !isValid && 'border-red-300 focus:border-red-500'
              )}
            />

            {/* 검증 에러 메시지 */}
            {rule.trim() && !isValidating && !isValid && errors.length > 0 && (
              <div className="flex flex-col gap-2 p-3 bg-red-50 rounded-md border border-red-200">
                <div className="flex items-start gap-2">
                  <AlertCircle className="w-4 h-4 text-red-600 mt-0.5 flex-shrink-0" />
                  <div className="flex-1">
                    <p className="text-xs font-semibold text-red-800">오류:</p>
                    {errors.map((error, idx) => (
                      <p key={idx} className="text-xs text-red-700 mt-1">
                        {error}
                      </p>
                    ))}
                  </div>
                </div>

                {/* 제안 사항 표시 */}
                {suggestions && suggestions.length > 0 && (
                  <div className="mt-2 pt-2 border-t border-red-200">
                    <p className="text-xs font-semibold text-red-800 mb-1">💡 제안:</p>
                    <ul className="list-disc list-inside space-y-1">
                      {suggestions.map((suggestion, idx) => (
                        <li key={idx} className="text-xs text-red-700">
                          {suggestion}
                        </li>
                      ))}
                    </ul>
                  </div>
                )}
              </div>
            )}

            {/* 사용 가능한 변수 안내 (에러 없을 때만) */}
            {(!rule.trim() || (isValid && !isValidating)) && (
              <div className="flex items-start gap-2 p-3 bg-blue-50 rounded-md border border-blue-200">
                <AlertCircle className="w-4 h-4 text-blue-600 mt-0.5 flex-shrink-0" />
                <div className="flex-1">
                  <p className="text-xs text-blue-700">
                    조건 표현식을 입력하세요. 사용 가능한 연산자: &lt;, &gt;, ==, !=,
                    &amp;&amp;, ||
                  </p>
                  <p className="text-xs text-blue-600 mt-1">
                    예시: temperature &gt; 90 &amp;&amp; vibration &lt; 50
                  </p>
                </div>
              </div>
            )}
          </div>
        )}

        {/* Action 노드: Action 설정 */}
        {node.data.type === 'action' && (
          <div className="space-y-2">
            <Label htmlFor="action">액션 설정</Label>
            <Textarea
              id="action"
              value={action}
              onChange={(e) => setAction(e.target.value)}
              placeholder="예: API 호출, 이메일 발송, Slack 알림"
              rows={4}
              className="font-mono text-sm"
            />
            <div className="flex items-start gap-2 p-3 bg-yellow-50 rounded-md border border-yellow-200">
              <AlertCircle className="w-4 h-4 text-yellow-600 mt-0.5 flex-shrink-0" />
              <p className="text-xs text-yellow-700">
                실행할 액션을 설정하세요. 외부 시스템 연동, 알림 발송 등
              </p>
            </div>
          </div>
        )}

        {/* 저장/취소 버튼 */}
        <div className="flex gap-2 pt-4">
          <Button
            onClick={handleSave}
            disabled={!hasChanges || (node.data.type === 'decision' && rule.trim() && !isValid)}
            className="flex-1"
          >
            <Save className="w-4 h-4 mr-2" />
            저장
          </Button>
          <Button onClick={handleCancel} variant="outline" className="flex-1">
            취소
          </Button>
        </div>

        {/* Rule 검증 실패시 저장 불가 안내 */}
        {node.data.type === 'decision' && rule.trim() && !isValid && (
          <div className="flex items-center gap-2 p-2 bg-red-50 rounded border border-red-200">
            <AlertCircle className="w-4 h-4 text-red-600" />
            <p className="text-xs text-red-700">Rule 표현식을 수정해야 저장할 수 있습니다</p>
          </div>
        )}

        {/* 변경사항 알림 */}
        {hasChanges && (
          <div className="flex items-center gap-2 p-2 bg-orange-50 rounded border border-orange-200">
            <AlertCircle className="w-4 h-4 text-orange-600" />
            <p className="text-xs text-orange-700">저장하지 않은 변경사항이 있습니다</p>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
