import { useState, useEffect, useRef } from 'react';
import { useQuery, useMutation } from '@tanstack/react-query';
import { getSystemStatus, getDataDirectory, exportDatabase, testClaudeApi } from '@/lib/tauri-api-wrapper';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Label } from '@/components/ui/label';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Switch } from '@/components/ui/switch';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import {
  CheckCircle,
  XCircle,
  Database,
  Key,
  Download,
  Folder,
  Info,
  Zap,
  DollarSign,
  HelpCircle,
  RefreshCw,
  AlertCircle,
} from 'lucide-react';
import { invoke } from '@tauri-apps/api/tauri';
import { save } from '@tauri-apps/api/dialog';
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip';

interface MCPSettings {
  context7_enabled: boolean;
  complexity_threshold: 'simple' | 'medium' | 'complex';
  daily_token_limit: number;
  cache_ttl_minutes: number;
}

interface UpdateInfo {
  available: boolean;
  current_version: string;
  latest_version?: string;
  release_notes?: string;
}

export default function Settings() {
  const [claudeKey, setClaudeKey] = useState('');
  const originalApiKeyRef = useRef<string>(''); // 원본 API 키 보관용
  const [mcpSettings, setMcpSettings] = useState<MCPSettings>({
    context7_enabled: true,
    complexity_threshold: 'medium',
    daily_token_limit: 100000,
    cache_ttl_minutes: 30,
  });
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);

  const { data: status } = useQuery({
    queryKey: ['system-status'],
    queryFn: getSystemStatus,
    refetchInterval: 5000,
  });

  const { data: dataDir } = useQuery({
    queryKey: ['data-directory'],
    queryFn: getDataDirectory,
  });

  // Auto Update - 업데이트 체크
  const checkUpdateMutation = useMutation({
    mutationFn: async () => {
      return await invoke<UpdateInfo>('check_for_updates');
    },
    onSuccess: (data) => {
      setUpdateInfo(data);
    },
    onError: (error) => {
      console.error('업데이트 체크 실패:', error);
      alert('업데이트 확인 실패: ' + error);
    },
  });

  // Auto Update - 업데이트 설치
  const installUpdateMutation = useMutation({
    mutationFn: async () => {
      return await invoke<void>('install_update');
    },
    onSuccess: () => {
      alert('업데이트가 다운로드되었습니다. 앱을 재시작하면 적용됩니다.');
    },
    onError: (error) => {
      console.error('업데이트 설치 실패:', error);
      alert('업데이트 설치 실패: ' + error);
    },
  });

  // Claude API 키 테스트
  const testApiMutation = useMutation({
    mutationFn: async () => {
      return await testClaudeApi();
    },
    onSuccess: (data) => {
      alert('✅ ' + data);
    },
    onError: (error) => {
      alert('❌ API 키 테스트 실패: ' + error);
    },
  });

  useEffect(() => {
    // 시스템 keychain에서 Claude API 키 로드 (자동 복원)
    if (typeof window !== 'undefined') {
      loadApiKeyFromSystem();

      // MCP 설정 로드 (localStorage 유지)
      const savedMcpSettings = localStorage.getItem('mcp_settings');
      if (savedMcpSettings) {
        try {
          setMcpSettings(JSON.parse(savedMcpSettings));
        } catch (e) {
          console.error('Failed to parse MCP settings:', e);
        }
      }
    }
  }, []);

  const loadApiKeyFromSystem = async () => {
    try {
      const apiKey = await invoke<string>('load_api_key');
      if (apiKey) {
        // 원본 API 키 보관
        originalApiKeyRef.current = apiKey;
        // 마스킹 처리 (보안)
        const maskedKey = apiKey.substring(0, 10) + '...' + apiKey.substring(apiKey.length - 10);
        setClaudeKey(maskedKey);
        console.log('✅ API 키가 시스템 keychain에서 로드되었습니다.');
      }
    } catch (error) {
      console.log('ℹ️  저장된 API 키 없음:', error);
      // 에러는 무시 (첫 설치시 정상)
    }
  };

  const handleSaveApiKey = async () => {
    try {
      // 이미 마스킹된 값인지 확인
      const isAlreadyMasked = claudeKey.includes('...');
      const apiKeyToSave = isAlreadyMasked ? originalApiKeyRef.current : claudeKey;

      // 빈 값 체크
      if (!apiKeyToSave) {
        alert('❌ API 키를 입력해주세요.');
        return;
      }

      // 시스템 keychain에 영구 저장 (Windows Credential Manager / macOS Keychain)
      await invoke('save_api_key', { apiKey: apiKeyToSave });

      alert('✅ API 키가 성공적으로 저장되었습니다.\n앱 재시작 후에도 자동으로 복원됩니다!');

      // 원본 API 키 보관
      originalApiKeyRef.current = apiKeyToSave;
      // 저장 후 마스킹된 값으로 업데이트
      const maskedKey = apiKeyToSave.substring(0, 10) + '...' + apiKeyToSave.substring(apiKeyToSave.length - 10);
      setClaudeKey(maskedKey);
    } catch (error: any) {
      console.error('API 키 저장 실패:', error);
      alert(`❌ API 키 저장 실패: ${error}`);
    }
  };

  const handleSaveMcpSettings = () => {
    localStorage.setItem('mcp_settings', JSON.stringify(mcpSettings));
    alert('MCP 설정이 저장되었습니다.');
  };

  const handleExportDatabase = async () => {
    try {
      console.log('[DEBUG] 백업 다이얼로그 열기 시작...');
      const exportPath = await save({
        defaultPath: 'judgify-backup.db',
        filters: [{ name: 'SQLite Database', extensions: ['db'] }],
      });

      console.log('[DEBUG] save() 반환값:', exportPath);

      if (exportPath) {
        // 확장자가 없으면 .db 추가
        const finalPath = exportPath.endsWith('.db') ? exportPath : `${exportPath}.db`;
        console.log('[DEBUG] 최종 백업 경로:', finalPath);
        console.log('[DEBUG] IPC 호출 직전...');
        await exportDatabase(finalPath);
        console.log('[DEBUG] IPC 호출 성공!');
        alert('데이터베이스가 성공적으로 백업되었습니다.');
      } else {
        console.log('[DEBUG] 사용자가 다이얼로그를 취소했습니다.');
      }
    } catch (error) {
      console.error('[DEBUG] 에러 발생:', error);
      alert('백업 실패: ' + error);
    }
  };

  return (
    <TooltipProvider delayDuration={300}>
      <div className="space-y-6">
        {/* Header */}
        <div>
          <h1 className="text-3xl font-bold mb-2">설정</h1>
          <p className="text-muted-foreground">
            시스템 상태 확인 및 설정을 관리하세요.
          </p>
        </div>

      {/* System Status */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Info className="w-5 h-5" />
            시스템 상태
          </CardTitle>
          <CardDescription>현재 시스템 상태를 확인하세요.</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div className="flex items-center justify-between p-3 rounded-lg border">
              <div className="flex items-center gap-2">
                <Database className="w-4 h-4" />
                <span className="text-sm font-medium">데이터베이스</span>
              </div>
              {status?.database_connected ? (
                <Badge variant="default" className="gap-1">
                  <CheckCircle className="w-3 h-3" />
                  연결됨
                </Badge>
              ) : (
                <Badge variant="destructive" className="gap-1">
                  <XCircle className="w-3 h-3" />
                  연결 안됨
                </Badge>
              )}
            </div>

            <div className="flex items-center justify-between p-3 rounded-lg border">
              <div className="flex items-center gap-2">
                <Key className="w-4 h-4" />
                <span className="text-sm font-medium">Claude API</span>
              </div>
              {status?.claude_configured ? (
                <Badge variant="default" className="gap-1">
                  <CheckCircle className="w-3 h-3" />
                  설정됨
                </Badge>
              ) : (
                <Badge variant="secondary" className="gap-1">
                  <XCircle className="w-3 h-3" />
                  미설정
                </Badge>
              )}
            </div>
          </div>

          <div className="p-3 rounded-lg bg-muted space-y-1">
            <div className="flex justify-between text-sm">
              <span className="text-muted-foreground">버전</span>
              <span className="font-mono">{status?.version || 'N/A'}</span>
            </div>
            <div className="flex justify-between text-sm">
              <span className="text-muted-foreground">데이터 디렉토리</span>
              <span className="font-mono text-xs truncate max-w-xs">
                {status?.database_path || dataDir || 'N/A'}
              </span>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Auto Update */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <RefreshCw className="w-5 h-5" />
            자동 업데이트
          </CardTitle>
          <CardDescription>최신 버전 확인 및 업데이트</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          {/* Current Version */}
          <div className="p-3 rounded-lg bg-muted space-y-1">
            <div className="flex justify-between text-sm">
              <span className="text-muted-foreground">현재 버전</span>
              <span className="font-mono font-semibold">{status?.version || 'N/A'}</span>
            </div>
            {updateInfo && (
              <>
                <div className="flex justify-between text-sm">
                  <span className="text-muted-foreground">최신 버전</span>
                  <span className="font-mono font-semibold">
                    {updateInfo.latest_version || 'N/A'}
                  </span>
                </div>
                {updateInfo.available && (
                  <div className="mt-2 p-2 rounded bg-blue-50 dark:bg-blue-950 border border-blue-200 dark:border-blue-800">
                    <div className="flex items-center gap-2 text-sm text-blue-700 dark:text-blue-300">
                      <AlertCircle className="w-4 h-4" />
                      <span className="font-medium">새로운 업데이트가 있습니다!</span>
                    </div>
                    {updateInfo.release_notes && (
                      <p className="mt-1 text-xs text-blue-600 dark:text-blue-400">
                        {updateInfo.release_notes}
                      </p>
                    )}
                  </div>
                )}
              </>
            )}
          </div>

          {/* Update Actions */}
          <div className="flex gap-2">
            <Button
              variant="outline"
              onClick={() => checkUpdateMutation.mutate()}
              disabled={checkUpdateMutation.isPending}
            >
              <RefreshCw className={`w-4 h-4 mr-2 ${checkUpdateMutation.isPending ? 'animate-spin' : ''}`} />
              {checkUpdateMutation.isPending ? '확인 중...' : '업데이트 확인'}
            </Button>

            {updateInfo?.available && (
              <Button
                onClick={() => installUpdateMutation.mutate()}
                disabled={installUpdateMutation.isPending}
              >
                <Download className="w-4 h-4 mr-2" />
                {installUpdateMutation.isPending ? '다운로드 중...' : '업데이트 설치'}
              </Button>
            )}
          </div>

          <p className="text-xs text-muted-foreground">
            업데이트는 GitHub Releases에서 자동으로 확인됩니다. 설치 후 앱을 재시작하세요.
          </p>
        </CardContent>
      </Card>

      {/* Claude API Key */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Key className="w-5 h-5" />
            Claude API 설정
          </CardTitle>
          <CardDescription>
            LLM 기능을 사용하려면 Claude API 키를 설정하세요.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div>
            <div className="flex items-center gap-2 mb-2">
              <Label htmlFor="api-key">API 키</Label>
              <Tooltip>
                <TooltipTrigger asChild>
                  <HelpCircle className="w-4 h-4 text-muted-foreground cursor-help" />
                </TooltipTrigger>
                <TooltipContent>
                  <p className="max-w-xs">
                    Claude API 키를 입력하세요. <a href="https://console.anthropic.com/" target="_blank" rel="noopener noreferrer" className="underline">console.anthropic.com</a>에서 발급받을 수 있습니다.
                  </p>
                </TooltipContent>
              </Tooltip>
            </div>
            <Input
              id="api-key"
              type="password"
              value={claudeKey}
              onChange={(e) => {
                const newValue = e.target.value;
                setClaudeKey(newValue);
                // 새로 입력하는 값은 원본으로 간주
                if (newValue && !newValue.includes('...')) {
                  originalApiKeyRef.current = newValue;
                }
              }}
              onFocus={() => {
                // 포커스시 마스킹된 값이면 빈 문자열로 초기화 (새로 입력 유도)
                if (claudeKey.includes('...')) {
                  setClaudeKey('');
                }
              }}
              placeholder="sk-ant-..."
              className="font-mono"
            />
            <p className="text-xs text-muted-foreground mt-2">
              API 키는 로컬에 안전하게 저장됩니다. 외부로 전송되지 않습니다.
            </p>
          </div>

          <div className="flex gap-2">
            <Button onClick={handleSaveApiKey} disabled={!claudeKey.trim()}>
              API 키 저장
            </Button>
            <Button
              variant="outline"
              onClick={() => testApiMutation.mutate()}
              disabled={testApiMutation.isPending || !status?.claude_configured}
            >
              <Key className={`w-4 h-4 mr-2 ${testApiMutation.isPending ? 'animate-pulse' : ''}`} />
              {testApiMutation.isPending ? '테스트 중...' : 'API 키 테스트'}
            </Button>
          </div>
          {!status?.claude_configured && (
            <p className="text-xs text-amber-600">
              ⚠️ API 키를 저장하고 앱을 재시작한 후 테스트할 수 있습니다.
            </p>
          )}
        </CardContent>
      </Card>

      {/* Database Management */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Database className="w-5 h-5" />
            데이터베이스 관리
          </CardTitle>
          <CardDescription>데이터베이스 백업 및 관리 기능</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center gap-2">
            <Folder className="w-4 h-4 text-muted-foreground" />
            <span className="text-sm">데이터 경로:</span>
            <code className="text-xs bg-muted px-2 py-1 rounded flex-1 truncate">
              {dataDir || 'Loading...'}
            </code>
          </div>

          <Button variant="outline" onClick={handleExportDatabase}>
            <Download className="w-4 h-4 mr-2" />
            데이터베이스 백업
          </Button>
        </CardContent>
      </Card>

      {/* MCP Conditional Activation */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Zap className="w-5 h-5" />
            MCP Conditional Activation
          </CardTitle>
          <CardDescription>
            복잡도 기반 Context7 MCP 활성화로 토큰 비용 최적화 (예상 54% 절감)
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          {/* Context7 Enable Toggle */}
          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <div className="flex items-center gap-2">
                <Label htmlFor="context7-enabled">Context7 MCP 활성화</Label>
                <Tooltip>
                  <TooltipTrigger asChild>
                    <HelpCircle className="w-4 h-4 text-muted-foreground cursor-help" />
                  </TooltipTrigger>
                  <TooltipContent>
                    <p className="max-w-xs">
                      Context7 MCP는 라이브러리 문서를 자동으로 검색하여 판단 정확도를 높입니다. 복잡한 판단에만 활성화하여 비용을 절감할 수 있습니다.
                    </p>
                  </TooltipContent>
                </Tooltip>
              </div>
              <p className="text-xs text-muted-foreground">
                복잡한 판단에만 Context7 문서 검색 활성화
              </p>
            </div>
            <Switch
              id="context7-enabled"
              checked={mcpSettings.context7_enabled}
              onCheckedChange={(checked) =>
                setMcpSettings({ ...mcpSettings, context7_enabled: checked })
              }
            />
          </div>

          {/* Complexity Threshold Select */}
          <div className="space-y-2">
            <div className="flex items-center gap-2">
              <Label>활성화 임계값</Label>
              <Tooltip>
                <TooltipTrigger asChild>
                  <HelpCircle className="w-4 h-4 text-muted-foreground cursor-help" />
                </TooltipTrigger>
                <TooltipContent>
                  <p className="max-w-xs">
                    <strong>Simple:</strong> Rule만 사용 (MCP 없음, 가장 빠름)<br />
                    <strong>Medium:</strong> LLM 사용 (30% 케이스에 MCP)<br />
                    <strong>Complex:</strong> 모든 케이스에 MCP (가장 정확)
                  </p>
                </TooltipContent>
              </Tooltip>
            </div>
            <Select
              value={mcpSettings.complexity_threshold}
              onValueChange={(value: 'simple' | 'medium' | 'complex') =>
                setMcpSettings({ ...mcpSettings, complexity_threshold: value })
              }
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="simple">Simple (Rule만, 0% MCP)</SelectItem>
                <SelectItem value="medium">Medium (LLM, 30% MCP)</SelectItem>
                <SelectItem value="complex">Complex (Full MCP, 100%)</SelectItem>
              </SelectContent>
            </Select>
            <p className="text-xs text-muted-foreground">
              선택한 복잡도 이상일 때만 Context7 활성화
            </p>
          </div>

          {/* Daily Token Limit */}
          <div className="space-y-2">
            <Label htmlFor="token-limit">일일 토큰 한도</Label>
            <Input
              id="token-limit"
              type="number"
              value={mcpSettings.daily_token_limit}
              onChange={(e) =>
                setMcpSettings({
                  ...mcpSettings,
                  daily_token_limit: parseInt(e.target.value) || 0,
                })
              }
              className="font-mono"
            />
            <p className="text-xs text-muted-foreground">
              하루 최대 사용 가능한 토큰 수 (기본: 100,000)
            </p>
          </div>

          {/* Cache TTL */}
          <div className="space-y-2">
            <Label htmlFor="cache-ttl">캐시 TTL (분)</Label>
            <Input
              id="cache-ttl"
              type="number"
              value={mcpSettings.cache_ttl_minutes}
              onChange={(e) =>
                setMcpSettings({
                  ...mcpSettings,
                  cache_ttl_minutes: parseInt(e.target.value) || 30,
                })
              }
              className="font-mono"
            />
            <p className="text-xs text-muted-foreground">
              Redis 캐시 만료 시간 (기본: 30분, 권장 적중률 80%)
            </p>
          </div>

          {/* Cost Estimate */}
          <div className="p-4 rounded-lg bg-muted space-y-2">
            <div className="flex items-center gap-2 text-sm font-medium">
              <DollarSign className="w-4 h-4" />
              예상 비용 절감
            </div>
            <div className="text-2xl font-bold">54% ↓</div>
            <div className="text-xs text-muted-foreground">
              월 $300 → $138 (Simple: 0 tokens, Medium: 2K, Complex: 5K)
            </div>
          </div>

          {/* Save Button */}
          <Button onClick={handleSaveMcpSettings} className="w-full">
            MCP 설정 저장
          </Button>
        </CardContent>
      </Card>

      {/* About */}
      <Card>
        <CardHeader>
          <CardTitle>TriFlow Desktop</CardTitle>
          <CardDescription>Windows Desktop Application</CardDescription>
        </CardHeader>
        <CardContent className="space-y-2 text-sm text-muted-foreground">
          <p>버전: {status?.version || 'N/A'}</p>
          <p>Tauri + Rust + React 기반 하이브리드 AI 판단 시스템</p>
          <p>© 2024 TriFlow. All rights reserved.</p>
        </CardContent>
      </Card>
      </div>
    </TooltipProvider>
  );
}
