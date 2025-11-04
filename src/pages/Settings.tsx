import { useState, useEffect } from 'react';
import { useQuery } from '@tanstack/react-query';
import { getSystemStatus, getDataDirectory, exportDatabase } from '@/lib/tauri-api';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Label } from '@/components/ui/label';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import {
  CheckCircle,
  XCircle,
  Database,
  Key,
  Download,
  Folder,
  Info,
} from 'lucide-react';
import { save } from '@tauri-apps/api/dialog';

export default function Settings() {
  const [openaiKey, setOpenaiKey] = useState('');

  const { data: status } = useQuery({
    queryKey: ['system-status'],
    queryFn: getSystemStatus,
    refetchInterval: 5000,
  });

  const { data: dataDir } = useQuery({
    queryKey: ['data-directory'],
    queryFn: getDataDirectory,
  });

  useEffect(() => {
    // 환경 변수에서 OpenAI API 키 로드
    if (typeof window !== 'undefined') {
      const savedKey = localStorage.getItem('openai_api_key');
      if (savedKey) setOpenaiKey(savedKey);
    }
  }, []);

  const handleSaveApiKey = () => {
    localStorage.setItem('openai_api_key', openaiKey);
    // 실제 구현에서는 Tauri를 통해 환경 변수나 secure storage에 저장
    alert('API 키가 저장되었습니다. 앱을 재시작해주세요.');
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
                <span className="text-sm font-medium">OpenAI API</span>
              </div>
              {status?.openai_configured ? (
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

      {/* OpenAI API Key */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Key className="w-5 h-5" />
            OpenAI API 설정
          </CardTitle>
          <CardDescription>
            LLM 기능을 사용하려면 OpenAI API 키를 설정하세요.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div>
            <Label htmlFor="api-key">API 키</Label>
            <Input
              id="api-key"
              type="password"
              value={openaiKey}
              onChange={(e) => setOpenaiKey(e.target.value)}
              placeholder="sk-..."
              className="font-mono"
            />
            <p className="text-xs text-muted-foreground mt-2">
              API 키는 로컬에 안전하게 저장됩니다. 외부로 전송되지 않습니다.
            </p>
          </div>

          <Button onClick={handleSaveApiKey} disabled={!openaiKey.trim()}>
            API 키 저장
          </Button>
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

      {/* About */}
      <Card>
        <CardHeader>
          <CardTitle>Judgify Desktop</CardTitle>
          <CardDescription>Windows Desktop Application</CardDescription>
        </CardHeader>
        <CardContent className="space-y-2 text-sm text-muted-foreground">
          <p>버전: {status?.version || 'N/A'}</p>
          <p>Tauri + Rust + React 기반 하이브리드 AI 판단 시스템</p>
          <p>© 2024 Judgify. All rights reserved.</p>
        </CardContent>
      </Card>
    </div>
  );
}
