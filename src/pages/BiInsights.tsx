import { useState, useEffect } from 'react';
import { useMutation } from '@tanstack/react-query';
import { generateBiInsight, type BiInsightResponse } from '@/lib/tauri-api-wrapper';
import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Label } from '@/components/ui/label';
import { Sparkles, TrendingUp, AlertCircle } from 'lucide-react';

export default function BiInsights() {
  const [request, setRequest] = useState('');
  const [insight, setInsight] = useState<BiInsightResponse | null>(null);

  // 🔧 Phase 1 Security Fix: Load API key from Tauri IPC (프로덕션 빌드 호환)
  useEffect(() => {
    async function loadApiKey() {
      try {
        const { invoke } = await import('@tauri-apps/api/tauri');
        const apiKey = await invoke<string>('load_api_key');
        if (apiKey) {
          console.log('[BiInsights] API key loaded from system keychain');

          // Rust 환경변수에도 설정 (bi_service.rs가 사용)
          await invoke('save_api_key', { apiKey });
        }
      } catch (error) {
        console.error('[BiInsights] Failed to load API key from keychain:', error);

        // Fallback: localStorage
        const localKey = localStorage.getItem('claude_api_key');
        if (localKey) {
          console.log('[BiInsights] Fallback to localStorage API key');

          try {
            const { invoke } = await import('@tauri-apps/api/tauri');
            await invoke('save_api_key', { apiKey: localKey });
          } catch (e) {
            console.error('[BiInsights] Failed to save API key to Rust env:', e);
          }
        }
      }
    }
    loadApiKey();
  }, []);

  const generateMutation = useMutation({
    mutationFn: (userRequest: string) => generateBiInsight(userRequest),
    onSuccess: (data) => {
      setInsight(data);
    },
  });

  const handleGenerate = () => {
    if (!request.trim()) return;
    generateMutation.mutate(request);
  };

  // 예시 요청들
  const exampleRequests = [
    '지난 주 불량률 트렌드를 보여줘',
    '워크플로우별 성공률을 분석해줘',
    '평균 신뢰도가 낮은 판단들을 찾아줘',
    '시간대별 판단 실행 추이를 시각화해줘',
  ];

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold mb-2">BI 인사이트</h1>
        <p className="text-muted-foreground">
          AI가 데이터를 분석하고 자동으로 인사이트와 대시보드를 생성합니다.
        </p>
      </div>

      {/* Request Input */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Sparkles className="w-5 h-5" />
            요청 입력
          </CardTitle>
          <CardDescription>
            자연어로 분석 요청을 입력하면 AI가 자동으로 인사이트를 생성합니다.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div>
            <Label htmlFor="request">분석 요청</Label>
            <Textarea
              id="request"
              value={request}
              onChange={(e) => setRequest(e.target.value)}
              placeholder="예: 지난 주 워크플로우별 성공률을 보여줘"
              className="min-h-[100px]"
            />
          </div>

          <div className="flex flex-wrap gap-2">
            <span className="text-sm text-muted-foreground">예시:</span>
            {exampleRequests.map((example, index) => (
              <Button
                key={index}
                variant="outline"
                size="sm"
                onClick={() => setRequest(example)}
              >
                {example}
              </Button>
            ))}
          </div>

          <Button
            onClick={handleGenerate}
            disabled={!request.trim() || generateMutation.isPending}
            className="w-full"
          >
            {generateMutation.isPending ? '생성 중...' : 'AI 인사이트 생성'}
          </Button>
        </CardContent>
      </Card>

      {/* Generated Insights */}
      {insight && (
        <div className="space-y-6">
          {/* Title */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <TrendingUp className="w-5 h-5" />
                {insight.title}
              </CardTitle>
            </CardHeader>
          </Card>

          {/* Insights */}
          <Card>
            <CardHeader>
              <CardTitle>주요 인사이트</CardTitle>
              <CardDescription>데이터 분석 결과</CardDescription>
            </CardHeader>
            <CardContent>
              <ul className="space-y-3">
                {insight.insights.map((item, index) => (
                  <li key={index} className="flex items-start gap-2">
                    <div className="w-6 h-6 rounded-full bg-primary/10 flex items-center justify-center flex-shrink-0 mt-0.5">
                      <span className="text-xs font-medium text-primary">{index + 1}</span>
                    </div>
                    <p className="text-sm">{item}</p>
                  </li>
                ))}
              </ul>
            </CardContent>
          </Card>

          {/* Recommendations */}
          {insight.recommendations.length > 0 && (
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <AlertCircle className="w-5 h-5" />
                  권장사항
                </CardTitle>
                <CardDescription>개선을 위한 제안</CardDescription>
              </CardHeader>
              <CardContent>
                <ul className="space-y-3">
                  {insight.recommendations.map((item, index) => (
                    <li key={index} className="flex items-start gap-2">
                      <div className="w-6 h-6 rounded-full bg-orange-100 dark:bg-orange-900/20 flex items-center justify-center flex-shrink-0 mt-0.5">
                        <AlertCircle className="w-4 h-4 text-orange-600" />
                      </div>
                      <p className="text-sm">{item}</p>
                    </li>
                  ))}
                </ul>
              </CardContent>
            </Card>
          )}

          {/* Auto-generated Dashboard Component */}
          <Card>
            <CardHeader>
              <CardTitle>자동 생성된 대시보드</CardTitle>
              <CardDescription>AI가 생성한 시각화 컴포넌트</CardDescription>
            </CardHeader>
            <CardContent>
              <div
                className="p-4 rounded-lg border bg-muted/50"
                dangerouslySetInnerHTML={{ __html: insight.component_code }}
              />
              <details className="mt-4">
                <summary className="text-sm text-muted-foreground cursor-pointer">
                  생성된 코드 보기
                </summary>
                <pre className="mt-2 p-4 rounded-lg bg-muted text-xs overflow-x-auto">
                  <code>{insight.component_code}</code>
                </pre>
              </details>
            </CardContent>
          </Card>
        </div>
      )}

      {/* Empty State */}
      {!insight && !generateMutation.isPending && (
        <Card className="border-dashed">
          <CardContent className="flex flex-col items-center justify-center py-16">
            <Sparkles className="w-12 h-12 text-muted-foreground mb-4" />
            <p className="text-lg font-medium mb-2">AI 인사이트를 생성해보세요</p>
            <p className="text-sm text-muted-foreground text-center max-w-md">
              위에 분석 요청을 입력하면 AI가 자동으로 데이터를 분석하고 인사이트와
              대시보드를 생성합니다.
            </p>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
