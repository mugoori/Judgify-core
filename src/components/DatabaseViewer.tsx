import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { X, Database, RefreshCw, ChevronRight, ChevronDown, Search, Download, Eye, Calendar, Hash, Type, FileJson, ArrowUp, ArrowDown, ArrowUpDown } from 'lucide-react';
import { JsonViewerModal } from './JsonViewerModal';
import { format, parseISO } from 'date-fns';
import { ko } from 'date-fns/locale';

// 정렬 상태 타입
interface SortState {
  column: string | null;
  direction: 'ASC' | 'DESC';
}

interface TableInfo {
  name: string;
  display_name: string;  // 사용자에게 보여줄 한글 이름
  description?: string;  // 테이블 설명
  category: string;      // ERP, MES 등 분류
  row_count: number;
  columns: ColumnInfo[];
}

interface ColumnInfo {
  name: string;
  data_type: string;
  is_nullable: boolean;
}

interface QueryResult {
  columns: string[];
  rows: Record<string, any>[];
  total_count: number;
}

interface DatabaseViewerProps {
  isOpen: boolean;
  onClose: () => void;
}

// 카테고리 탭 정의
type CategoryTab = 'ALL' | 'ERP' | 'MES';

const CATEGORY_TABS: { key: CategoryTab; label: string; color: string }[] = [
  { key: 'ALL', label: '전체', color: 'bg-gray-600' },
  { key: 'ERP', label: 'ERP (계획/기준)', color: 'bg-blue-600' },
  { key: 'MES', label: 'MES (실행/품질)', color: 'bg-green-600' },
];

export const DatabaseViewer: React.FC<DatabaseViewerProps> = ({ isOpen, onClose }) => {
  const [tables, setTables] = useState<TableInfo[]>([]);
  const [selectedTable, setSelectedTable] = useState<string | null>(null);
  const [tableData, setTableData] = useState<QueryResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [expandedTables, setExpandedTables] = useState<Set<string>>(new Set());
  const [currentPage, setCurrentPage] = useState(0);
  const [searchQuery, setSearchQuery] = useState('');
  const [customQuery, setCustomQuery] = useState('');
  const [isCustomQueryMode, setIsCustomQueryMode] = useState(false);
  const [jsonModalData, setJsonModalData] = useState<any>(null);
  const [jsonModalTitle, setJsonModalTitle] = useState<string>('');
  const [activeCategory, setActiveCategory] = useState<CategoryTab>('ALL');
  const [sortState, setSortState] = useState<SortState>({ column: null, direction: 'DESC' });

  const ROWS_PER_PAGE = 50;

  // 모달이 열릴 때 배경 스크롤 방지 (더 강력한 방식)
  useEffect(() => {
    if (isOpen) {
      // 현재 스크롤 위치 저장
      const scrollY = window.scrollY;
      const scrollX = window.scrollX;

      // HTML과 body 둘 다 스크롤 방지
      const originalHtmlOverflow = document.documentElement.style.overflow;
      const originalBodyOverflow = document.body.style.overflow;
      const originalBodyPosition = document.body.style.position;

      // 스크롤 완전 차단
      document.documentElement.style.overflow = 'hidden';
      document.body.style.overflow = 'hidden';
      document.body.style.position = 'fixed';
      document.body.style.top = `-${scrollY}px`;
      document.body.style.left = `-${scrollX}px`;
      document.body.style.width = '100%';
      document.body.style.height = '100%';

      // wheel 이벤트 차단
      const preventWheel = (e: WheelEvent) => {
        // 모달 내부 스크롤 영역이 아닌 경우에만 차단
        const target = e.target as HTMLElement;
        if (!target.closest('.table-scroll-container') && !target.closest('.overflow-y-auto')) {
          e.preventDefault();
          e.stopPropagation();
        }
      };

      // touch 이벤트 차단 (모바일)
      const preventTouch = (e: TouchEvent) => {
        const target = e.target as HTMLElement;
        if (!target.closest('.table-scroll-container') && !target.closest('.overflow-y-auto')) {
          e.preventDefault();
        }
      };

      document.addEventListener('wheel', preventWheel, { passive: false });
      document.addEventListener('touchmove', preventTouch, { passive: false });

      return () => {
        // 원래 상태로 복원
        document.documentElement.style.overflow = originalHtmlOverflow;
        document.body.style.overflow = originalBodyOverflow;
        document.body.style.position = originalBodyPosition;
        document.body.style.top = '';
        document.body.style.left = '';
        document.body.style.width = '';
        document.body.style.height = '';

        // 이벤트 리스너 제거
        document.removeEventListener('wheel', preventWheel);
        document.removeEventListener('touchmove', preventTouch);

        // 원래 스크롤 위치로 복원
        window.scrollTo(scrollX, scrollY);
      };
    }
  }, [isOpen]);

  // 테이블 목록 로드
  const loadTables = async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<TableInfo[]>('get_database_tables');
      setTables(result);
    } catch (err) {
      console.error('테이블 로드 실패:', err);
      setError('테이블 목록을 불러오는데 실패했습니다.');
    } finally {
      setLoading(false);
    }
  };

  // 테이블 데이터 로드 (정렬 옵션 지원)
  const loadTableData = async (
    tableName: string,
    page: number = 0,
    sort?: SortState
  ) => {
    setLoading(true);
    setError(null);
    try {
      const currentSort = sort || sortState;
      const result = await invoke<QueryResult>('query_table_data', {
        tableName,
        limit: ROWS_PER_PAGE,
        offset: page * ROWS_PER_PAGE,
        sortColumn: currentSort.column,
        sortDirection: currentSort.direction,
      });
      setTableData(result);
      setSelectedTable(tableName);
      setCurrentPage(page);
    } catch (err) {
      console.error('테이블 데이터 로드 실패:', err);
      setError('테이블 데이터를 불러오는데 실패했습니다.');
    } finally {
      setLoading(false);
    }
  };

  // 컬럼 헤더 클릭 시 정렬 변경
  const handleColumnSort = (columnName: string) => {
    if (!selectedTable) return;

    let newDirection: 'ASC' | 'DESC' = 'ASC';

    // 같은 컬럼을 다시 클릭하면 정렬 방향 토글
    if (sortState.column === columnName) {
      newDirection = sortState.direction === 'ASC' ? 'DESC' : 'ASC';
    }

    const newSortState: SortState = { column: columnName, direction: newDirection };
    setSortState(newSortState);
    loadTableData(selectedTable, 0, newSortState); // 첫 페이지로 리셋하고 새 정렬로 로드
  };

  // 정렬 아이콘 렌더링
  const renderSortIcon = (columnName: string) => {
    if (sortState.column !== columnName) {
      return <ArrowUpDown className="w-3 h-3 text-gray-600 opacity-50 group-hover:opacity-100 transition-opacity" />;
    }
    return sortState.direction === 'ASC'
      ? <ArrowUp className="w-3 h-3 text-blue-400" />
      : <ArrowDown className="w-3 h-3 text-blue-400" />;
  };

  // 사용자 정의 쿼리 실행
  const executeCustomQuery = async () => {
    if (!customQuery.trim()) return;

    setLoading(true);
    setError(null);
    try {
      const result = await invoke<QueryResult>('execute_custom_query', {
        query: customQuery,
      });
      setTableData(result);
      setSelectedTable(null);
    } catch (err) {
      console.error('쿼리 실행 실패:', err);
      setError(`쿼리 실행 실패: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  // CSV 내보내기
  const exportToCsv = () => {
    if (!tableData || tableData.rows.length === 0) return;

    const headers = tableData.columns.join(',');
    const rows = tableData.rows.map(row =>
      tableData.columns.map(col => {
        const value = row[col];
        // CSV 형식에 맞게 값 처리
        if (value === null || value === undefined) return '';
        if (typeof value === 'string' && value.includes(',')) {
          return `"${value.replace(/"/g, '""')}"`;
        }
        return value;
      }).join(',')
    );

    const csv = [headers, ...rows].join('\n');
    const blob = new Blob([csv], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = `${selectedTable || 'query_result'}_${new Date().toISOString().split('T')[0]}.csv`;
    link.click();
    URL.revokeObjectURL(url);
  };

  // 테이블 확장/축소 토글
  const toggleTableExpansion = (tableName: string) => {
    const newExpanded = new Set(expandedTables);
    if (newExpanded.has(tableName)) {
      newExpanded.delete(tableName);
    } else {
      newExpanded.add(tableName);
    }
    setExpandedTables(newExpanded);
  };

  // 데이터 타입 검사 함수들
  const isJSON = (value: any): boolean => {
    if (typeof value !== 'string') return false;
    try {
      const parsed = JSON.parse(value);
      return typeof parsed === 'object';
    } catch {
      return false;
    }
  };

  const isDateString = (value: any): boolean => {
    if (typeof value !== 'string') return false;
    // ISO 8601 형식 또는 일반적인 날짜 형식 감지
    const datePatterns = [
      /^\d{4}-\d{2}-\d{2}$/,  // YYYY-MM-DD
      /^\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}/,  // ISO 8601
      /^\d{2}\/\d{2}\/\d{4}$/,  // MM/DD/YYYY
    ];
    return datePatterns.some(pattern => pattern.test(value));
  };

  const formatDate = (value: string): string => {
    try {
      const date = parseISO(value);
      return format(date, 'yyyy-MM-dd HH:mm:ss', { locale: ko });
    } catch {
      return value;
    }
  };

  const truncateText = (text: string, maxLength: number = 50): string => {
    if (text.length <= maxLength) return text;
    return text.substring(0, maxLength) + '...';
  };

  // 셀 렌더링 함수
  const renderCell = (value: any, columnName: string) => {
    // NULL 처리
    if (value === null || value === undefined) {
      return <span className="text-gray-600 italic">NULL</span>;
    }

    // Boolean 처리
    if (typeof value === 'boolean') {
      return (
        <span className={`inline-block font-medium ${value ? 'text-green-400' : 'text-red-400'}`}>
          {value ? '✓ true' : '✗ false'}
        </span>
      );
    }

    // 숫자 처리
    if (typeof value === 'number') {
      return (
        <div className="flex items-center gap-1 max-w-full overflow-hidden">
          <Hash className="w-3 h-3 text-blue-400 flex-shrink-0" />
          <span className="font-mono text-blue-300 truncate">{value.toLocaleString()}</span>
        </div>
      );
    }

    // 문자열 처리
    if (typeof value === 'string') {
      // JSON 감지
      if (isJSON(value)) {
        const parsed = JSON.parse(value);
        const preview = JSON.stringify(parsed, null, 2);
        return (
          <button
            onClick={() => {
              setJsonModalData(parsed);
              setJsonModalTitle(`${columnName} 데이터`);
            }}
            className="flex items-center gap-1 px-2 py-1 bg-purple-900/30 hover:bg-purple-900/50 rounded text-purple-300 transition-colors text-left max-w-full overflow-hidden"
          >
            <FileJson className="w-3 h-3 flex-shrink-0" />
            <span className="text-xs font-mono truncate flex-1 min-w-0">
              {truncateText(preview.replace(/\s+/g, ' '), 40)}
            </span>
            <Eye className="w-3 h-3 flex-shrink-0" />
          </button>
        );
      }

      // 날짜 감지
      if (isDateString(value)) {
        return (
          <div className="flex items-center gap-1 max-w-full overflow-hidden">
            <Calendar className="w-3 h-3 text-amber-400 flex-shrink-0" />
            <span className="text-amber-300 truncate">{formatDate(value)}</span>
          </div>
        );
      }

      // 긴 텍스트
      if (value.length > 100) {
        return (
          <div className="group relative max-w-full overflow-hidden">
            <span className="block truncate pr-6">{truncateText(value, 80)}</span>
            <button
              onClick={() => {
                setJsonModalData({ text: value });
                setJsonModalTitle(`${columnName} 전체 텍스트`);
              }}
              className="absolute right-0 top-1/2 -translate-y-1/2 opacity-0 group-hover:opacity-100 transition-opacity"
            >
              <Eye className="w-3 h-3 text-gray-400 hover:text-white" />
            </button>
          </div>
        );
      }

      // 일반 텍스트
      return (
        <div className="flex items-center gap-1 max-w-full overflow-hidden whitespace-nowrap">
          <Type className="w-3 h-3 text-gray-500 flex-shrink-0" />
          <span className="truncate">{value}</span>
        </div>
      );
    }

    // 기타 타입
    return <span className="text-gray-400 block truncate">{String(value)}</span>;
  };

  useEffect(() => {
    if (isOpen) {
      loadTables();
    }
  }, [isOpen]);

  if (!isOpen) return null;

  // 카테고리 및 검색 필터링
  const filteredTables = tables.filter(table => {
    const matchesCategory = activeCategory === 'ALL' || table.category === activeCategory;
    const matchesSearch = table.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
                          table.display_name.toLowerCase().includes(searchQuery.toLowerCase());
    return matchesCategory && matchesSearch;
  });

  // 카테고리별 테이블 수
  const categoryCounts = {
    ALL: tables.length,
    ERP: tables.filter(t => t.category === 'ERP').length,
    MES: tables.filter(t => t.category === 'MES').length,
  };

  const totalPages = tableData ? Math.ceil(tableData.total_count / ROWS_PER_PAGE) : 0;

  return (
    <div
      className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4"
      style={{ isolation: 'isolate' }}
    >
      <div className="bg-gray-900 rounded-lg w-full max-w-[1920px] h-full max-h-[95vh] flex flex-col shadow-2xl" style={{ overflow: 'hidden', position: 'relative' }}>
        {/* 헤더 */}
        <div className="flex items-center justify-between p-4 border-b border-gray-700">
          <div className="flex items-center gap-2">
            <Database className="w-5 h-5 text-blue-500" />
            <h2 className="text-lg font-semibold text-white">데이터베이스 뷰어</h2>
          </div>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-white transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* 본문 */}
        <div className="flex flex-1" style={{ overflow: 'hidden', minHeight: 0 }}>
          {/* 좌측 사이드바 - 테이블 목록 */}
          <div className="w-56 border-r border-gray-700 p-3 overflow-y-auto flex-shrink-0" style={{ height: '100%' }}>
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-sm font-semibold text-gray-300">테이블 목록</h3>
              <button
                onClick={loadTables}
                className="text-gray-400 hover:text-white transition-colors"
                disabled={loading}
              >
                <RefreshCw className={`w-4 h-4 ${loading ? 'animate-spin' : ''}`} />
              </button>
            </div>

            {/* 카테고리 탭 */}
            <div className="flex gap-1 mb-3">
              {CATEGORY_TABS.map((tab) => (
                <button
                  key={tab.key}
                  onClick={() => setActiveCategory(tab.key)}
                  className={`flex-1 px-2 py-1.5 text-xs font-medium rounded transition-colors ${
                    activeCategory === tab.key
                      ? `${tab.color} text-white`
                      : 'bg-gray-800 text-gray-400 hover:bg-gray-700'
                  }`}
                >
                  <div>{tab.label.split(' ')[0]}</div>
                  <div className="text-[10px] opacity-75">{categoryCounts[tab.key]}개</div>
                </button>
              ))}
            </div>

            <div className="mb-3">
              <div className="relative">
                <Search className="absolute left-2 top-1/2 transform -translate-y-1/2 w-4 h-4 text-gray-500" />
                <input
                  type="text"
                  placeholder="테이블 검색..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className="w-full pl-8 pr-2 py-1.5 bg-gray-800 border border-gray-700 rounded text-sm text-white placeholder-gray-500 focus:outline-none focus:border-blue-500"
                />
              </div>
            </div>

            <div className="space-y-1">
              {filteredTables.map((table) => (
                <div key={table.name}>
                  <button
                    onClick={() => {
                      toggleTableExpansion(table.name);
                      // 테이블 변경 시 정렬 상태 초기화
                      const newSortState: SortState = { column: null, direction: 'DESC' };
                      setSortState(newSortState);
                      loadTableData(table.name, 0, newSortState);
                      setIsCustomQueryMode(false);
                    }}
                    className={`w-full flex items-center justify-between px-2 py-1.5 text-left text-sm rounded hover:bg-gray-800 transition-colors ${
                      selectedTable === table.name ? 'bg-gray-800 text-blue-400' : 'text-gray-300'
                    }`}
                  >
                    <div className="flex items-center gap-1 flex-1 min-w-0">
                      {expandedTables.has(table.name) ? (
                        <ChevronDown className="w-3 h-3 flex-shrink-0" />
                      ) : (
                        <ChevronRight className="w-3 h-3 flex-shrink-0" />
                      )}
                      <div className="flex flex-col min-w-0 flex-1">
                        <div className="flex items-center gap-1.5">
                          <span className={`text-[9px] px-1 py-0.5 rounded font-medium ${
                            table.category === 'ERP'
                              ? 'bg-blue-600/30 text-blue-400'
                              : 'bg-green-600/30 text-green-400'
                          }`}>
                            {table.category}
                          </span>
                          <span className="truncate">{table.display_name}</span>
                        </div>
                        {table.description && (
                          <span className="text-xs text-gray-500 mt-0.5 truncate">{table.name}</span>
                        )}
                      </div>
                    </div>
                    <span className="text-xs text-gray-500 flex-shrink-0 ml-1">{table.row_count}</span>
                  </button>

                  {expandedTables.has(table.name) && (
                    <div className="ml-6 mt-1 text-xs text-gray-500">
                      {table.columns.map((col) => (
                        <div key={col.name} className="py-0.5">
                          {col.name} <span className="text-gray-600">({col.data_type})</span>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              ))}
            </div>

            <div className="mt-6 pt-4 border-t border-gray-700">
              <button
                onClick={() => {
                  setIsCustomQueryMode(true);
                  setSelectedTable(null);
                  setTableData(null);
                }}
                className="w-full px-3 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm rounded transition-colors"
              >
                사용자 정의 쿼리
              </button>
            </div>
          </div>

          {/* 우측 메인 영역 - 테이블 데이터 */}
          <div className="flex-1 flex flex-col p-4" style={{ overflow: 'hidden', minHeight: 0 }}>
            {error && (
              <div className="mb-4 p-3 bg-red-900/50 border border-red-700 rounded text-red-300 text-sm">
                {error}
              </div>
            )}

            {isCustomQueryMode ? (
              <div className="mb-4">
                <div className="flex gap-2 mb-2">
                  <textarea
                    value={customQuery}
                    onChange={(e) => setCustomQuery(e.target.value)}
                    placeholder="SELECT * FROM table_name WHERE ..."
                    className="flex-1 p-2 bg-gray-800 border border-gray-700 rounded text-sm text-white placeholder-gray-500 focus:outline-none focus:border-blue-500 font-mono"
                    rows={4}
                  />
                </div>
                <button
                  onClick={executeCustomQuery}
                  disabled={loading || !customQuery.trim()}
                  className="px-4 py-2 bg-green-600 hover:bg-green-700 disabled:bg-gray-700 disabled:cursor-not-allowed text-white text-sm rounded transition-colors"
                >
                  쿼리 실행
                </button>
              </div>
            ) : selectedTable && (
              <div className="mb-3 flex items-center justify-between">
                <div>
                  <h3 className="text-lg font-semibold text-white">
                    {tables.find(t => t.name === selectedTable)?.display_name || selectedTable}
                  </h3>
                  {tables.find(t => t.name === selectedTable)?.description && (
                    <p className="text-xs text-gray-400 mt-1">
                      {tables.find(t => t.name === selectedTable)?.description}
                    </p>
                  )}
                </div>
                <button
                  onClick={exportToCsv}
                  disabled={!tableData || tableData.rows.length === 0}
                  className="flex items-center gap-1 px-3 py-1.5 bg-gray-700 hover:bg-gray-600 disabled:bg-gray-800 disabled:cursor-not-allowed text-white text-sm rounded transition-colors"
                >
                  <Download className="w-4 h-4" />
                  CSV 내보내기
                </button>
              </div>
            )}

            {tableData && (
              <>
                <div className="flex-1 bg-gray-800 rounded border-2 border-blue-500/30 relative" style={{ overflow: 'hidden', minHeight: 0 }}>
                  <style>{`
                    .table-scroll-container {
                      position: absolute;
                      top: 0;
                      left: 0;
                      right: 0;
                      bottom: 0;
                      overflow: auto !important;
                      scrollbar-width: thin;
                      scrollbar-color: #3b82f6 #1f2937;
                    }
                    .table-scroll-container::-webkit-scrollbar {
                      width: 14px;
                      height: 14px;
                    }
                    .table-scroll-container::-webkit-scrollbar-track {
                      background: #1f2937;
                      border-radius: 4px;
                    }
                    .table-scroll-container::-webkit-scrollbar-thumb {
                      background: #3b82f6;
                      border-radius: 4px;
                      border: 2px solid #1f2937;
                    }
                    .table-scroll-container::-webkit-scrollbar-thumb:hover {
                      background: #2563eb;
                    }
                    .table-scroll-container::-webkit-scrollbar-corner {
                      background: #1f2937;
                    }
                  `}</style>

                  {/* 스크롤 안내 메시지 */}
                  <div className="absolute top-2 right-2 bg-blue-600/20 text-blue-300 px-3 py-1 rounded text-xs z-20 animate-pulse pointer-events-none">
                    ↔ 좌우로 스크롤 가능
                  </div>

                  <div className="table-scroll-container">
                    <table className="text-sm" style={{ width: '3000px' }}>
                    <thead className="bg-gray-900 sticky top-0 z-10">
                      <tr>
                        {tableData.columns.map((col) => {
                          // 컬럼별 최소/최대 너비 설정 (더 넓게 설정)
                          let minWidth = '250px';
                          let maxWidth = '600px';

                          // 특정 컬럼명에 따른 너비 조정
                          if (col.toLowerCase().includes('id')) {
                            minWidth = '120px';
                            maxWidth = '180px';
                          } else if (col.toLowerCase().includes('type')) {
                            minWidth = '150px';
                            maxWidth = '200px';
                          } else if (col.toLowerCase().includes('data') || col.toLowerCase().includes('json')) {
                            minWidth = '400px';
                            maxWidth = '600px';
                          } else if (col.toLowerCase().includes('content') || col.toLowerCase().includes('text')) {
                            minWidth = '350px';
                            maxWidth = '500px';
                          } else if (col.toLowerCase().includes('time') || col.toLowerCase().includes('date') || col.toLowerCase().includes('created_at')) {
                            minWidth = '200px';
                            maxWidth = '250px';
                          }

                          const isSorted = sortState.column === col;

                          return (
                            <th
                              key={col}
                              style={{ minWidth, maxWidth }}
                              onClick={() => handleColumnSort(col)}
                              className={`px-3 py-2 text-left font-medium border-b border-gray-700 whitespace-nowrap cursor-pointer select-none group transition-colors ${
                                isSorted ? 'bg-blue-900/30 text-blue-300' : 'text-gray-300 hover:bg-gray-800'
                              }`}
                            >
                              <div className="flex items-center gap-1.5 truncate">
                                {col.toLowerCase().includes('id') && <Hash className="w-3 h-3 text-blue-400 flex-shrink-0" />}
                                {col.toLowerCase().includes('time') && <Calendar className="w-3 h-3 text-amber-400 flex-shrink-0" />}
                                {col.toLowerCase().includes('date') && <Calendar className="w-3 h-3 text-amber-400 flex-shrink-0" />}
                                {col.toLowerCase().includes('data') && <FileJson className="w-3 h-3 text-purple-400 flex-shrink-0" />}
                                <span className="truncate flex-1">{col}</span>
                                {renderSortIcon(col)}
                              </div>
                            </th>
                          );
                        })}
                      </tr>
                    </thead>
                    <tbody>
                      {tableData.rows.map((row, idx) => (
                        <tr
                          key={idx}
                          className="border-b border-gray-700 hover:bg-gray-700/50 transition-colors"
                        >
                          {tableData.columns.map((col) => {
                            // 컬럼별 최소/최대 너비 재사용 (동일하게 넓게)
                            let minWidth = '250px';
                            let maxWidth = '600px';

                            if (col.toLowerCase().includes('id')) {
                              minWidth = '120px';
                              maxWidth = '180px';
                            } else if (col.toLowerCase().includes('type')) {
                              minWidth = '150px';
                              maxWidth = '200px';
                            } else if (col.toLowerCase().includes('data') || col.toLowerCase().includes('json')) {
                              minWidth = '400px';
                              maxWidth = '600px';
                            } else if (col.toLowerCase().includes('content') || col.toLowerCase().includes('text')) {
                              minWidth = '350px';
                              maxWidth = '500px';
                            } else if (col.toLowerCase().includes('time') || col.toLowerCase().includes('date') || col.toLowerCase().includes('created_at')) {
                              minWidth = '200px';
                              maxWidth = '250px';
                            }

                            return (
                              <td
                                key={col}
                                style={{ minWidth, maxWidth }}
                                className="px-3 py-2 text-gray-300 whitespace-nowrap"
                              >
                                {renderCell(row[col], col)}
                              </td>
                            );
                          })}
                        </tr>
                      ))}
                    </tbody>
                  </table>
                  </div>
                </div>

                {/* 페이지네이션 */}
                {!isCustomQueryMode && totalPages > 1 && (
                  <div className="mt-3 flex items-center justify-center gap-2">
                    <button
                      onClick={() => loadTableData(selectedTable!, currentPage - 1)}
                      disabled={currentPage === 0 || loading}
                      className="px-3 py-1 bg-gray-700 hover:bg-gray-600 disabled:bg-gray-800 disabled:cursor-not-allowed text-white text-sm rounded transition-colors"
                    >
                      이전
                    </button>
                    <span className="text-sm text-gray-300">
                      {currentPage + 1} / {totalPages}
                    </span>
                    <button
                      onClick={() => loadTableData(selectedTable!, currentPage + 1)}
                      disabled={currentPage >= totalPages - 1 || loading}
                      className="px-3 py-1 bg-gray-700 hover:bg-gray-600 disabled:bg-gray-800 disabled:cursor-not-allowed text-white text-sm rounded transition-colors"
                    >
                      다음
                    </button>
                  </div>
                )}
              </>
            )}

            {!tableData && !isCustomQueryMode && !error && (
              <div className="flex-1 flex items-center justify-center text-gray-500">
                테이블을 선택하여 데이터를 확인하세요
              </div>
            )}
          </div>
        </div>
      </div>

      {/* JSON 뷰어 모달 */}
      <JsonViewerModal
        isOpen={jsonModalData !== null}
        onClose={() => {
          setJsonModalData(null);
          setJsonModalTitle('');
        }}
        data={jsonModalData}
        title={jsonModalTitle}
      />
    </div>
  );
};