import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
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
    setSearchLoading(true);
    setSearchError('');
    try {
      const results = await invoke<CcpDocWithScore[]>('search_ccp_docs', {
        companyId: searchCompany,
        ccpId: searchCcp,
        query: searchQuery,
        topK: topK
      });
      setSearchResults(results);
    } catch (error) {
      setSearchError(`검색 실패: ${error}`);
      console.error('Search error:', error);
    } finally {
      setSearchLoading(false);
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
    <div className="ccp-demo-container">
      <header className="ccp-header">
        <h1>CCP 제조기업 RAG + 룰베이스 판단 데모</h1>
        <p>HACCP/ISO22000 품질 관리 시스템</p>
      </header>

      <div className="ccp-content">
        {/* Document Search Section */}
        <section className="search-section">
          <h2>📚 CCP 문서 검색 (RAG - BM25)</h2>

          <div className="search-form">
            <div className="form-row">
              <div className="form-group">
                <label>회사</label>
                <select
                  value={searchCompany}
                  onChange={(e) => setSearchCompany(e.target.value)}
                >
                  <option value="COMP_A">COMP_A</option>
                  <option value="COMP_B">COMP_B</option>
                </select>
              </div>

              <div className="form-group">
                <label>CCP 코드</label>
                <select
                  value={searchCcp || 'all'}
                  onChange={(e) => setSearchCcp(e.target.value === 'all' ? null : e.target.value)}
                >
                  <option value="all">전체</option>
                  <option value="CCP-01">CCP-01 (열처리)</option>
                  <option value="CCP-02">CCP-02 (냉각)</option>
                </select>
              </div>

              <div className="form-group">
                <label>검색 개수 (Top K)</label>
                <input
                  type="number"
                  min="1"
                  max="10"
                  value={topK}
                  onChange={(e) => setTopK(parseInt(e.target.value))}
                />
              </div>
            </div>

            <div className="form-row">
              <div className="form-group full-width">
                <label>검색어</label>
                <input
                  type="text"
                  placeholder="예: 관리 기준 시정조치"
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  onKeyPress={(e) => e.key === 'Enter' && handleSearch()}
                />
              </div>
            </div>

            <button
              className="btn-primary"
              onClick={handleSearch}
              disabled={searchLoading || !searchQuery}
            >
              {searchLoading ? '검색 중...' : '검색'}
            </button>
          </div>

          {searchError && (
            <div className="error-message">{searchError}</div>
          )}

          {searchResults.length > 0 && (
            <div className="search-results">
              <h3>검색 결과 ({searchResults.length}건)</h3>
              {searchResults.map((doc, index) => (
                <div key={doc.id} className="doc-card">
                  <div className="doc-header">
                    <span className="doc-rank">#{index + 1}</span>
                    <span className="doc-ccp">{doc.ccp_id}</span>
                    <span className="doc-score">BM25: {doc.score.toFixed(2)}</span>
                  </div>
                  <h4>{doc.title}</h4>
                  <p className="doc-section">{doc.section_type}</p>
                  <p className="doc-content">{doc.content}</p>
                </div>
              ))}
            </div>
          )}
        </section>

        {/* Judgment Section */}
        <section className="judgment-section">
          <h2>⚖️ CCP 상태 판단 (하이브리드)</h2>

          <div className="judgment-form">
            <div className="form-row">
              <div className="form-group">
                <label>회사</label>
                <select
                  value={judgmentCompany}
                  onChange={(e) => setJudgmentCompany(e.target.value)}
                >
                  <option value="COMP_A">COMP_A</option>
                  <option value="COMP_B">COMP_B</option>
                </select>
              </div>

              <div className="form-group">
                <label>CCP 코드</label>
                <select
                  value={judgmentCcp}
                  onChange={(e) => setJudgmentCcp(e.target.value)}
                >
                  <option value="CCP-01">CCP-01 (열처리)</option>
                  <option value="CCP-02">CCP-02 (냉각)</option>
                </select>
              </div>
            </div>

            <div className="form-row">
              <div className="form-group">
                <label>시작 날짜</label>
                <input
                  type="date"
                  value={dateFrom}
                  onChange={(e) => setDateFrom(e.target.value)}
                />
              </div>

              <div className="form-group">
                <label>종료 날짜</label>
                <input
                  type="date"
                  value={dateTo}
                  onChange={(e) => setDateTo(e.target.value)}
                />
              </div>
            </div>

            <button
              className="btn-primary"
              onClick={handleJudgment}
              disabled={judgmentLoading}
            >
              {judgmentLoading ? '판단 중...' : '판단 실행'}
            </button>
          </div>

          {judgmentError && (
            <div className="error-message">{judgmentError}</div>
          )}

          {judgmentResult && (
            <div className="judgment-results">
              {/* Statistics Cards */}
              <div className="stats-grid">
                <div className="stat-card">
                  <h4>총 점검 횟수</h4>
                  <p className="stat-value">{judgmentResult.stats.total_logs}회</p>
                </div>
                <div className="stat-card">
                  <h4>NG 발생</h4>
                  <p className="stat-value">{judgmentResult.stats.ng_count}회</p>
                </div>
                <div className="stat-card">
                  <h4>NG 비율</h4>
                  <p className="stat-value">{(judgmentResult.stats.ng_rate * 100).toFixed(1)}%</p>
                </div>
                <div className="stat-card">
                  <h4>평균 측정값</h4>
                  <p className="stat-value">{judgmentResult.stats.avg_value.toFixed(1)}</p>
                </div>
              </div>

              {/* Risk Level Badge */}
              <div className="risk-level" style={{ backgroundColor: getRiskColor(judgmentResult.risk_level) }}>
                <h3>위험도: {judgmentResult.risk_level}</h3>
                <p>{judgmentResult.rule_reason}</p>
              </div>

              {/* LLM Summary */}
              <div className="llm-summary">
                <h3>🤖 AI 요약</h3>
                <p>{judgmentResult.llm_summary}</p>
              </div>

              {/* Evidence Documents */}
              {judgmentResult.evidence_docs.length > 0 && (
                <div className="evidence-docs">
                  <h3>📚 참고 문서 ({judgmentResult.evidence_docs.length}건)</h3>
                  {judgmentResult.evidence_docs.map((doc, index) => (
                    <div key={doc.id} className="evidence-card">
                      <div className="evidence-header">
                        <span className="evidence-rank">#{index + 1}</span>
                        <span className="evidence-ccp">{doc.ccp_id}</span>
                        <span className="evidence-score">BM25: {doc.score.toFixed(2)}</span>
                      </div>
                      <h4>{doc.title}</h4>
                      <p className="evidence-section">{doc.section_type}</p>
                      <p className="evidence-content">{doc.content}</p>
                    </div>
                  ))}
                </div>
              )}

              {/* Judgment ID */}
              <div className="judgment-id">
                <small>판단 ID: {judgmentResult.judgment_id}</small>
              </div>
            </div>
          )}
        </section>
      </div>
    </div>
  );
};

export default CcpDemo;
