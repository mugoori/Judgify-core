import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Label } from '@/components/ui/label';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Badge } from '@/components/ui/badge';
import { Search, BarChart3, AlertTriangle, CheckCircle2, XCircle } from 'lucide-react';
import './CcpDemo.css';

// TypeScript interfaces matching Rust types
interface CcpDocWithScore {
  id: string;
  company_id: string;
  ccp_id: string;
  title: string;
  section_type: string;
  content: string;
  score: number;
}

interface CcpStats {
  total_logs: number;
  ng_count: number;
  ng_rate: number;
  avg_value: number;
  min_value: number;
  max_value: number;
}

interface CcpJudgmentRequest {
  company_id: string;
  ccp_id: string;
  period_from: string;
  period_to: string;
}

interface CcpJudgmentResponse {
  stats: CcpStats;
  risk_level: string;
  rule_reason: string;
  llm_summary: string;
  evidence_docs: CcpDocWithScore[];
  judgment_id: string;
}

const CcpDemo: React.FC = () => {
  // Document Search State
  const [searchCompany, setSearchCompany] = useState<string>('COMP_A');
  const [searchCcp, setSearchCcp] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState<string>('');
  const [topK, setTopK] = useState<number>(5);
  const [searchResults, setSearchResults] = useState<CcpDocWithScore[]>([]);
  const [searchLoading, setSearchLoading] = useState<boolean>(false);
  const [searchError, setSearchError] = useState<string>('');

  // Judgment State
  const [judgmentCompany, setJudgmentCompany] = useState<string>('COMP_A');
  const [judgmentCcp, setJudgmentCcp] = useState<string>('CCP-01');
  const [dateFrom, setDateFrom] = useState<string>('2025-11-01');
  const [dateTo, setDateTo] = useState<string>('2025-11-14');
  const [judgmentResult, setJudgmentResult] = useState<CcpJudgmentResponse | null>(null);
  const [judgmentLoading, setJudgmentLoading] = useState<boolean>(false);
  const [judgmentError, setJudgmentError] = useState<string>('');

  // Document Search Handler
  const handleSearch = async () => {
    console.log('🔵 [DEBUG] handleSearch START ===================================');
    console.log('[DEBUG] searchQuery:', searchQuery);
    console.log('[DEBUG] searchCompany:', searchCompany);
    console.log('[DEBUG] searchCcp:', searchCcp);
    console.log('[DEBUG] topK:', topK);

    // 필수 파라미터 검증
    if (!searchQuery || searchQuery.trim() === '') {
      console.error('❌ [DEBUG] searchQuery is empty!');
      setSearchError('검색어를 입력해주세요.');
      return;
    }
    if (!searchCompany) {
      console.error('❌ [DEBUG] searchCompany is empty!');
      setSearchError('회사를 선택해주세요.');
      return;
    }

    console.log('[DEBUG] Validation passed, calling invoke...');

    setSearchLoading(true);
    setSearchError('');
    try {
      console.log('[DEBUG] invoke("search_ccp_docs") called with params:', {
        companyId: searchCompany,
        ccpId: searchCcp,
        query: searchQuery,
        topK: topK
      });

      const results = await invoke<CcpDocWithScore[]>('search_ccp_docs', {
        companyId: searchCompany,
        ccpId: searchCcp,
        query: searchQuery,
        topK: topK
      });

      console.log('[DEBUG] ✅ invoke() SUCCESS!');
      console.log('[DEBUG] Search results:', results);
      console.log('[DEBUG] Results length:', results.length);

      setSearchResults(results);
    } catch (error) {
      console.error('❌ [DEBUG] invoke() FAILED!');
      console.error('[DEBUG] Error type:', typeof error);
      console.error('[DEBUG] Error value:', error);
      console.error('[DEBUG] Error stringified:', JSON.stringify(error, null, 2));

      setSearchError(`검색 실패: ${error}`);
    } finally {
      setSearchLoading(false);
      console.log('🔵 [DEBUG] handleSearch END =====================================');
    }
  };

  // Judgment Handler
  const handleJudgment = async () => {
    setJudgmentLoading(true);
    setJudgmentError('');
    try {
      const request: CcpJudgmentRequest = {
        company_id: judgmentCompany,
        ccp_id: judgmentCcp,
        period_from: dateFrom,
        period_to: dateTo
      };
      const result = await invoke<CcpJudgmentResponse>('judge_ccp_status', {
        request
      });
      setJudgmentResult(result);
    } catch (error) {
      setJudgmentError(`판단 실패: ${error}`);
      console.error('Judgment error:', error);
    } finally {
      setJudgmentLoading(false);
    }
  };

  // Risk level color helper
  const getRiskColor = (level: string): string => {
    switch (level) {
      case 'HIGH': return '#ef4444';
      case 'MEDIUM': return '#f59e0b';
      case 'LOW': return '#10b981';
      default: return '#6b7280';
    }
  };

  return (
    <div className="space-y-6 p-6">
      <div className="mb-6">
        <h1 className="text-3xl font-bold mb-2">CCP 품질 관리</h1>
        <p className="text-muted-foreground">
          HACCP/ISO22000 기반 CCP 문서 검색 및 하이브리드 판단
        </p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Document Search Section */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Search className="w-5 h-5" />
              CCP 문서 검색
            </CardTitle>
            <CardDescription>
              BM25 알고리즘 기반 RAG 검색
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid grid-cols-3 gap-4">
              <div>
                <Label htmlFor="search-company">회사</Label>
                <Select value={searchCompany} onValueChange={setSearchCompany}>
                  <SelectTrigger id="search-company">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="COMP_A">COMP_A</SelectItem>
                    <SelectItem value="COMP_B">COMP_B</SelectItem>
                  </SelectContent>
                </Select>
              </div>

              <div>
                <Label htmlFor="search-ccp">CCP 코드</Label>
                <Select value={searchCcp || 'all'} onValueChange={(v) => setSearchCcp(v === 'all' ? null : v)}>
                  <SelectTrigger id="search-ccp">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="all">전체</SelectItem>
                    <SelectItem value="CCP-01">CCP-01 (열처리)</SelectItem>
                    <SelectItem value="CCP-02">CCP-02 (냉각)</SelectItem>
                  </SelectContent>
                </Select>
              </div>

              <div>
                <Label htmlFor="top-k">검색 개수</Label>
                <Input
                  id="top-k"
                  type="number"
                  min="1"
                  max="10"
                  value={topK}
                  onChange={(e) => setTopK(parseInt(e.target.value))}
                />
              </div>
            </div>

            <div>
              <Label htmlFor="search-query">검색어</Label>
              <div className="flex gap-2">
                <Input
                  id="search-query"
                  placeholder="예: 관리 기준 시정조치"
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  onKeyPress={(e) => e.key === 'Enter' && handleSearch()}
                />
                <Button onClick={handleSearch} disabled={searchLoading || !searchQuery}>
                  {searchLoading ? '검색 중...' : <Search className="w-4 h-4" />}
                </Button>
              </div>
            </div>

            {searchError && (
              <div className="bg-destructive/15 text-destructive px-4 py-3 rounded-md text-sm">
                {searchError}
              </div>
            )}

            {searchResults.length > 0 && (
              <div className="space-y-3">
                <h3 className="text-sm font-semibold">검색 결과 ({searchResults.length}건)</h3>
                {searchResults.map((doc, index) => (
                  <Card key={doc.id} className="bg-muted/50">
                    <CardContent className="p-4 space-y-2">
                      <div className="flex items-center gap-2">
                        <Badge variant="secondary">#{index + 1}</Badge>
                        <Badge variant="outline">{doc.ccp_id}</Badge>
                        <Badge className="ml-auto">BM25: {doc.score.toFixed(2)}</Badge>
                      </div>
                      <h4 className="font-semibold">{doc.title}</h4>
                      <p className="text-xs text-muted-foreground">{doc.section_type}</p>
                      <p className="text-sm">{doc.content}</p>
                    </CardContent>
                  </Card>
                ))}
              </div>
            )}
          </CardContent>
        </Card>

        {/* Judgment Section */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <BarChart3 className="w-5 h-5" />
              CCP 상태 판단
            </CardTitle>
            <CardDescription>
              Rule-based + LLM 하이브리드 판단
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div>
                <Label htmlFor="judgment-company">회사</Label>
                <Select value={judgmentCompany} onValueChange={setJudgmentCompany}>
                  <SelectTrigger id="judgment-company">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="COMP_A">COMP_A</SelectItem>
                    <SelectItem value="COMP_B">COMP_B</SelectItem>
                  </SelectContent>
                </Select>
              </div>

              <div>
                <Label htmlFor="judgment-ccp">CCP 코드</Label>
                <Select value={judgmentCcp} onValueChange={setJudgmentCcp}>
                  <SelectTrigger id="judgment-ccp">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="CCP-01">CCP-01 (열처리)</SelectItem>
                    <SelectItem value="CCP-02">CCP-02 (냉각)</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div>
                <Label htmlFor="date-from">시작 날짜</Label>
                <Input
                  id="date-from"
                  type="date"
                  value={dateFrom}
                  onChange={(e) => setDateFrom(e.target.value)}
                />
              </div>

              <div>
                <Label htmlFor="date-to">종료 날짜</Label>
                <Input
                  id="date-to"
                  type="date"
                  value={dateTo}
                  onChange={(e) => setDateTo(e.target.value)}
                />
              </div>
            </div>

            <Button className="w-full" onClick={handleJudgment} disabled={judgmentLoading}>
              {judgmentLoading ? '판단 중...' : '판단 실행'}
            </Button>

            {judgmentError && (
              <div className="bg-destructive/15 text-destructive px-4 py-3 rounded-md text-sm">
                {judgmentError}
              </div>
            )}

            {judgmentResult && (
              <div className="space-y-4 mt-6">
                {/* Statistics Cards */}
                <div className="grid grid-cols-2 gap-3">
                  <Card className="bg-muted/50">
                    <CardContent className="p-4 text-center">
                      <p className="text-xs text-muted-foreground mb-1">총 점검 횟수</p>
                      <p className="text-2xl font-bold">{judgmentResult.stats.total_logs}회</p>
                    </CardContent>
                  </Card>
                  <Card className="bg-muted/50">
                    <CardContent className="p-4 text-center">
                      <p className="text-xs text-muted-foreground mb-1">NG 발생</p>
                      <p className="text-2xl font-bold">{judgmentResult.stats.ng_count}회</p>
                    </CardContent>
                  </Card>
                  <Card className="bg-muted/50">
                    <CardContent className="p-4 text-center">
                      <p className="text-xs text-muted-foreground mb-1">NG 비율</p>
                      <p className="text-2xl font-bold">{(judgmentResult.stats.ng_rate * 100).toFixed(1)}%</p>
                    </CardContent>
                  </Card>
                  <Card className="bg-muted/50">
                    <CardContent className="p-4 text-center">
                      <p className="text-xs text-muted-foreground mb-1">평균 측정값</p>
                      <p className="text-2xl font-bold">{judgmentResult.stats.avg_value.toFixed(1)}</p>
                    </CardContent>
                  </Card>
                </div>

                {/* Risk Level Badge */}
                <Card style={{ backgroundColor: getRiskColor(judgmentResult.risk_level), color: 'white' }}>
                  <CardContent className="p-4 text-center">
                    <div className="flex items-center justify-center gap-2 mb-2">
                      {judgmentResult.risk_level === 'HIGH' && <XCircle className="w-5 h-5" />}
                      {judgmentResult.risk_level === 'MEDIUM' && <AlertTriangle className="w-5 h-5" />}
                      {judgmentResult.risk_level === 'LOW' && <CheckCircle2 className="w-5 h-5" />}
                      <h3 className="text-lg font-bold">위험도: {judgmentResult.risk_level}</h3>
                    </div>
                    <p className="text-sm opacity-90">{judgmentResult.rule_reason}</p>
                  </CardContent>
                </Card>

                {/* LLM Summary */}
                <Card className="bg-amber-50 border-amber-200">
                  <CardHeader>
                    <CardTitle className="text-sm">AI 요약</CardTitle>
                  </CardHeader>
                  <CardContent>
                    <p className="text-sm whitespace-pre-wrap">{judgmentResult.llm_summary}</p>
                  </CardContent>
                </Card>

                {/* Evidence Documents */}
                {judgmentResult.evidence_docs.length > 0 && (
                  <div className="space-y-2">
                    <h3 className="text-sm font-semibold">참고 문서 ({judgmentResult.evidence_docs.length}건)</h3>
                    {judgmentResult.evidence_docs.map((doc, index) => (
                      <Card key={doc.id} className="bg-muted/30">
                        <CardContent className="p-3 space-y-2">
                          <div className="flex items-center gap-2">
                            <Badge variant="secondary" className="text-xs">#{index + 1}</Badge>
                            <Badge variant="outline" className="text-xs">{doc.ccp_id}</Badge>
                            <Badge className="ml-auto text-xs">BM25: {doc.score.toFixed(2)}</Badge>
                          </div>
                          <h4 className="font-semibold text-sm">{doc.title}</h4>
                          <p className="text-xs text-muted-foreground">{doc.section_type}</p>
                          <p className="text-xs">{doc.content}</p>
                        </CardContent>
                      </Card>
                    ))}
                  </div>
                )}

                {/* Judgment ID */}
                <p className="text-xs text-center text-muted-foreground font-mono">
                  판단 ID: {judgmentResult.judgment_id}
                </p>
              </div>
            )}
          </CardContent>
        </Card>
      </div>
    </div>
  );
};

export default CcpDemo;
