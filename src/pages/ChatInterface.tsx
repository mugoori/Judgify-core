import { useState, useEffect, useRef, memo } from 'react';
import { useMutation } from '@tanstack/react-query';
import { sendChatMessage, getChatHistory, getSystemStatus, type ChatMessageRequest, type ChatMessageResponse, type ChartResponse, type DataKeyConfig, type PieChartData } from '@/lib/tauri-api-wrapper';
import { useNavigate } from 'react-router-dom';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';
import { Card } from '@/components/ui/card';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog';
import { Send, User, Trash2, TrendingUp, Play, FileQuestion, Activity, Paperclip, FileText, X, ChevronDown, BarChart3, AlertTriangle, Settings } from 'lucide-react';
import { invoke } from '@tauri-apps/api/tauri';
import { toast } from '@/components/ui/use-toast';
import type { MesUploadResult, MesQueryResult } from '@/types/mes';
import {
  BarChart as RechartsBarChart,
  Bar,
  LineChart as RechartsLineChart,
  Line,
  PieChart as RechartsPieChart,
  Pie,
  Cell,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from 'recharts';

interface Message {
  role: 'user' | 'assistant';
  content: string;
  intent?: string;
  tableData?: {
    columns: string[];
    rows: any[];
  };
  chartData?: ChartResponse;
}

// Y축 숫자 압축 포맷터 (K: 천, M: 백만, B: 10억)
const formatYAxisValue = (value: number): string => {
  if (value >= 1000000000) {
    const formatted = (value / 1000000000).toFixed(1);
    return formatted.endsWith('.0') ? `${parseInt(formatted)}B` : `${formatted}B`;
  }
  if (value >= 1000000) {
    const formatted = (value / 1000000).toFixed(1);
    return formatted.endsWith('.0') ? `${parseInt(formatted)}M` : `${formatted}M`;
  }
  if (value >= 1000) {
    const formatted = (value / 1000).toFixed(1);
    return formatted.endsWith('.0') ? `${parseInt(formatted)}K` : `${formatted}K`;
  }
  return value.toString();
};

// Bar/Line 차트 데이터 평탄화 (values 객체를 최상위로 펼침)
// Backend: { name: "12-01", values: { "살균온도": 85.5, "냉각온도": 4.2 } }
// Recharts 필요: { name: "12-01", "살균온도": 85.5, "냉각온도": 4.2 }
const flattenChartData = (data: any[] | undefined): any[] => {
  if (!data) return [];
  // 디버그: 원본 데이터 확인
  console.log('[DEBUG] flattenChartData 입력:', JSON.stringify(data, null, 2));

  const result = data.map(item => {
    const { name, values, ...rest } = item;
    // values 객체가 있으면 펼치고, 없으면 그대로 사용
    if (values && typeof values === 'object') {
      console.log('[DEBUG] values 객체 발견 - 평탄화 진행');
      return { name, ...values, ...rest };
    }
    // 이미 평탄화된 데이터 (백엔드에서 serde_json::Value로 직접 생성)
    console.log('[DEBUG] 이미 평탄화된 데이터:', item);
    return item;
  });

  console.log('[DEBUG] flattenChartData 출력:', JSON.stringify(result, null, 2));
  return result;
};

// data_keys 자동 추출 (백엔드에서 제공하지 않을 경우 데이터에서 추출)
// 첫 번째 데이터 객체에서 'name' 키를 제외한 숫자 값을 가진 키들을 추출
const extractDataKeys = (data: any[] | undefined, providedDataKeys: DataKeyConfig[] | undefined): DataKeyConfig[] => {
  // 백엔드에서 data_keys를 제공한 경우 그대로 사용
  if (providedDataKeys && providedDataKeys.length > 0) {
    console.log('[DEBUG] 제공된 data_keys 사용:', providedDataKeys);
    return providedDataKeys;
  }

  // 데이터가 없으면 빈 배열 반환
  if (!data || data.length === 0) {
    console.log('[DEBUG] 데이터 없음 - 빈 data_keys 반환');
    return [];
  }

  // 첫 번째 데이터 객체에서 숫자 값을 가진 키들 추출
  const firstItem = data[0];
  const keys = Object.keys(firstItem).filter(key => {
    // 'name' 키는 x축 라벨이므로 제외
    if (key === 'name') return false;
    // 숫자 값을 가진 키만 포함
    const value = firstItem[key];
    return typeof value === 'number';
  });

  const extractedKeys: DataKeyConfig[] = keys.map((key, idx) => ({
    key,
    label: key,
    color: MODERN_COLORS[idx % MODERN_COLORS.length],
  }));

  console.log('[DEBUG] 자동 추출된 data_keys:', extractedKeys);
  return extractedKeys;
};

// 모던 색상 배열 (Bar/Line 차트용)
const MODERN_COLORS = [
  '#6366f1', // indigo
  '#10b981', // emerald
  '#f59e0b', // amber
  '#ec4899', // pink
  '#06b6d4', // cyan
  '#8b5cf6', // violet
  '#f97316', // orange
  '#14b8a6', // teal
];

// Pie 차트용 그라데이션 색상
const PIE_COLORS = [
  '#6366f1',
  '#10b981',
  '#f59e0b',
  '#ef4444',
  '#8b5cf6',
  '#06b6d4',
  '#ec4899',
  '#f97316',
];

// 커스텀 툴팁 스타일
const tooltipStyle = {
  backgroundColor: 'rgba(17, 24, 39, 0.95)',
  border: 'none',
  borderRadius: '12px',
  boxShadow: '0 25px 50px -12px rgba(0, 0, 0, 0.5)',
  padding: '12px 16px',
};

// 커스텀 Legend 스타일
const legendStyle = {
  paddingTop: '20px',
};

// Memoized MessageBubble component to prevent unnecessary re-renders
const MessageBubble = memo(({ message, index }: { message: Message; index: number }) => {
  return (
    <div
      key={index}
      className={`flex gap-3 ${
        message.role === 'user' ? 'justify-end' : 'justify-start'
      }`}
    >
      {message.role === 'assistant' && (
        <div className="w-8 h-8 rounded-full overflow-hidden flex-shrink-0">
          <img src="/chatbot_img.png" alt="AI" className="w-full h-full object-cover" />
        </div>
      )}

      <div
        className={`${
          message.tableData ? 'max-w-[90%]' : 'max-w-[70%]'
        } rounded-lg p-4 ${
          message.role === 'user'
            ? 'bg-primary text-primary-foreground'
            : 'bg-muted'
        }`}
      >
        <ReactMarkdown
          remarkPlugins={[remarkGfm]}
          components={{
            // 테이블 커스텀 스타일링
            table: ({ children }) => (
              <div className="overflow-x-auto my-4 rounded-lg border border-gray-300 dark:border-gray-600">
                <table className="min-w-full border-collapse">
                  {children}
                </table>
              </div>
            ),
            thead: ({ children }) => (
              <thead className="bg-slate-700 text-white">
                {children}
              </thead>
            ),
            th: ({ children }) => (
              <th className="border border-gray-300 dark:border-gray-600 px-4 py-2.5 text-left text-xs font-semibold uppercase tracking-wider whitespace-nowrap">
                {children}
              </th>
            ),
            tbody: ({ children }) => (
              <tbody className="divide-y divide-gray-200 dark:divide-gray-600">
                {children}
              </tbody>
            ),
            tr: ({ children }) => (
              <tr className="hover:bg-gray-50 dark:hover:bg-gray-800/50 transition-colors even:bg-gray-50/50 dark:even:bg-gray-800/30">
                {children}
              </tr>
            ),
            td: ({ children }) => (
              <td className="border border-gray-300 dark:border-gray-600 px-4 py-2 text-sm whitespace-nowrap">
                {children}
              </td>
            ),
            // 기존 텍스트 스타일 유지
            p: ({ children }) => (
              <p className="whitespace-pre-wrap mb-2 last:mb-0">{children}</p>
            ),
            // 코드 블록 스타일링
            code: ({ children, className }) => {
              const isInline = !className;
              return isInline ? (
                <code className="bg-gray-200 dark:bg-gray-700 px-1.5 py-0.5 rounded text-sm font-mono">
                  {children}
                </code>
              ) : (
                <code className="block bg-gray-100 dark:bg-gray-800 p-4 rounded-lg overflow-x-auto my-2 text-sm font-mono">
                  {children}
                </code>
              );
            },
            pre: ({ children }) => (
              <pre className="bg-gray-100 dark:bg-gray-800 rounded-lg overflow-x-auto my-2">
                {children}
              </pre>
            ),
            // 리스트 스타일링
            ul: ({ children }) => (
              <ul className="list-disc list-inside my-2 space-y-1">{children}</ul>
            ),
            ol: ({ children }) => (
              <ol className="list-decimal list-inside my-2 space-y-1">{children}</ol>
            ),
            li: ({ children }) => (
              <li className="ml-2">{children}</li>
            ),
            // 강조 스타일
            strong: ({ children }) => (
              <strong className="font-bold">{children}</strong>
            ),
            em: ({ children }) => (
              <em className="italic">{children}</em>
            ),
            // 링크 스타일
            a: ({ href, children }) => (
              <a href={href} className="text-blue-500 hover:text-blue-600 underline" target="_blank" rel="noopener noreferrer">
                {children}
              </a>
            ),
            // 헤딩 스타일
            h1: ({ children }) => (
              <h1 className="text-xl font-bold mt-4 mb-2">{children}</h1>
            ),
            h2: ({ children }) => (
              <h2 className="text-lg font-bold mt-3 mb-2">{children}</h2>
            ),
            h3: ({ children }) => (
              <h3 className="text-base font-bold mt-2 mb-1">{children}</h3>
            ),
            // 인용문 스타일
            blockquote: ({ children }) => (
              <blockquote className="border-l-4 border-gray-400 dark:border-gray-500 pl-4 my-2 italic text-gray-600 dark:text-gray-400">
                {children}
              </blockquote>
            ),
            // 수평선
            hr: () => (
              <hr className="my-4 border-gray-300 dark:border-gray-600" />
            ),
          }}
        >
          {message.content}
        </ReactMarkdown>

        {/* 테이블 데이터 표시 - 접을 수 있는 근거 자료 */}
        {message.tableData && (
          <details className="mt-4 border border-blue-300 rounded-lg overflow-hidden group shadow-sm">
            <summary className="px-3 py-2 bg-blue-50 border-b border-blue-200 text-xs text-blue-700 font-medium cursor-pointer hover:bg-blue-100 transition-colors flex items-center gap-2 list-none">
              <ChevronDown className="w-4 h-4 transition-transform group-open:rotate-180" />
              <span>📊 근거 데이터 보기 ({message.tableData.rows.length}건)</span>
            </summary>
            {/* 스크롤 가능한 테이블 컨테이너 */}
            <div className="max-h-[300px] overflow-auto bg-white">
              <table className="min-w-full border-collapse">
                <thead className="sticky top-0 bg-slate-700">
                  <tr className="border-b border-slate-600">
                    {message.tableData.columns.map((col, idx) => (
                      <th
                        key={idx}
                        className="px-3 py-2.5 text-left text-xs font-semibold text-white uppercase tracking-wider whitespace-nowrap"
                      >
                        {col}
                      </th>
                    ))}
                  </tr>
                </thead>
                <tbody className="divide-y divide-gray-200">
                  {message.tableData.rows.map((row, rowIdx) => (
                    <tr key={rowIdx} className={`${rowIdx % 2 === 0 ? 'bg-white' : 'bg-gray-50'} hover:bg-blue-50 transition-colors`}>
                      {message.tableData!.columns.map((col, colIdx) => (
                        <td key={colIdx} className="px-3 py-2 text-sm text-gray-800 whitespace-nowrap">
                          {row[col] === null || row[col] === undefined ? (
                            <span className="text-gray-400 italic">NULL</span>
                          ) : (
                            String(row[col])
                          )}
                        </td>
                      ))}
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
            {message.tableData.rows.length === 0 && (
              <p className="text-center text-gray-500 py-4 bg-white">데이터가 없습니다</p>
            )}
          </details>
        )}

        {/* 차트 렌더링 - 모던 글래스모피즘 디자인 */}
        {message.chartData && (
          <div className="mt-4 p-5 bg-gradient-to-br from-slate-900/90 via-slate-800/90 to-slate-900/90 backdrop-blur-xl rounded-2xl border border-slate-700/50 shadow-2xl">
            {/* 차트 헤더 */}
            <div className="flex items-center gap-3 mb-3">
              <div className="p-2 bg-gradient-to-br from-indigo-500 to-purple-600 rounded-xl shadow-lg shadow-indigo-500/25">
                <BarChart3 className="w-5 h-5 text-white" />
              </div>
              <div>
                <h4 className="font-bold text-base text-white tracking-tight">{message.chartData.title}</h4>
                <p className="text-xs text-slate-400">{message.chartData.description}</p>
              </div>
            </div>

            {/* 💡 AI 인사이트 표시 - 글로우 효과 */}
            {message.chartData.insight && (
              <div className="mb-5 p-4 bg-gradient-to-r from-indigo-500/10 via-purple-500/10 to-pink-500/10 border border-indigo-500/20 rounded-xl backdrop-blur-sm relative overflow-hidden">
                <div className="absolute inset-0 bg-gradient-to-r from-indigo-500/5 to-purple-500/5 animate-pulse" />
                <div className="flex items-start gap-3 relative">
                  <div className="p-1.5 bg-gradient-to-br from-amber-400 to-orange-500 rounded-lg shadow-lg shadow-amber-500/25">
                    <span className="text-sm">💡</span>
                  </div>
                  <p className="text-sm text-slate-200 leading-relaxed">{message.chartData.insight}</p>
                </div>
              </div>
            )}

            {/* Bar/Line 차트 - 모던 스타일 */}
            {(message.chartData.chart_type === 'bar' || message.chartData.chart_type === 'line') &&
              message.chartData.bar_line_data && (
                <div className="bg-slate-800/50 rounded-xl p-4 border border-slate-700/30">
                  {/* 디버그: data_keys 확인 */}
                  {console.log('[DEBUG] chart_type:', message.chartData.chart_type)}
                  {console.log('[DEBUG] data_keys:', JSON.stringify(message.chartData.data_keys, null, 2))}
                  {console.log('[DEBUG] x_axis_key:', message.chartData.x_axis_key)}
                  <ResponsiveContainer width="100%" height={320}>
                    {message.chartData.chart_type === 'bar' ? (
                      (() => {
                        const flattenedData = flattenChartData(message.chartData.bar_line_data);
                        const dataKeys = extractDataKeys(flattenedData, message.chartData.data_keys);
                        console.log('[DEBUG] Bar 차트 - flattenedData:', flattenedData);
                        console.log('[DEBUG] Bar 차트 - 사용할 dataKeys:', dataKeys);
                        return (
                      <RechartsBarChart data={flattenedData} margin={{ top: 20, right: 30, left: 20, bottom: 80 }}>
                        <defs>
                          {dataKeys.map((dk: DataKeyConfig, idx: number) => (
                            <linearGradient key={`gradient-${dk.key}`} id={`gradient-${dk.key}`} x1="0" y1="0" x2="0" y2="1">
                              <stop offset="0%" stopColor={MODERN_COLORS[idx % MODERN_COLORS.length]} stopOpacity={1} />
                              <stop offset="100%" stopColor={MODERN_COLORS[idx % MODERN_COLORS.length]} stopOpacity={0.6} />
                            </linearGradient>
                          ))}
                        </defs>
                        <CartesianGrid strokeDasharray="3 3" stroke="#334155" strokeOpacity={0.5} vertical={false} />
                        <XAxis
                          dataKey={message.chartData.x_axis_key || 'name'}
                          stroke="#64748b"
                          fontSize={10}
                          angle={-45}
                          textAnchor="end"
                          height={90}
                          interval="preserveStartEnd"
                          tick={{ fill: '#94a3b8' }}
                          axisLine={{ stroke: '#475569', strokeWidth: 1 }}
                          tickLine={{ stroke: '#475569' }}
                          tickFormatter={(value: string) => {
                            if (value && value.includes('-') && value.includes(':')) {
                              const parts = value.split(' ');
                              if (parts.length >= 2) {
                                const datePart = parts[0].split('-').slice(1).join('/');
                                const timePart = parts[1].substring(0, 5);
                                return `${datePart} ${timePart}`;
                              }
                            }
                            return value && value.length > 12 ? value.substring(0, 12) + '...' : value;
                          }}
                        />
                        <YAxis
                          stroke="#64748b"
                          fontSize={11}
                          tickFormatter={formatYAxisValue}
                          tick={{ fill: '#94a3b8' }}
                          axisLine={{ stroke: '#475569', strokeWidth: 1 }}
                          tickLine={{ stroke: '#475569' }}
                        />
                        <Tooltip
                          contentStyle={tooltipStyle}
                          labelStyle={{ color: '#f1f5f9', fontWeight: 600, marginBottom: '8px' }}
                          itemStyle={{ color: '#e2e8f0', padding: '2px 0' }}
                          cursor={{ fill: 'rgba(148, 163, 184, 0.1)' }}
                        />
                        <Legend
                          wrapperStyle={legendStyle}
                          iconType="circle"
                          iconSize={8}
                          formatter={(value) => <span className="text-slate-300 text-sm ml-1">{value}</span>}
                        />
                        {dataKeys.map((dk: DataKeyConfig, _idx: number) => (
                          <Bar
                            key={dk.key}
                            dataKey={dk.key}
                            fill={`url(#gradient-${dk.key})`}
                            name={dk.label}
                            radius={[6, 6, 0, 0]}
                            animationDuration={800}
                            animationEasing="ease-out"
                          />
                        ))}
                      </RechartsBarChart>
                        );
                      })()
                    ) : (
                      (() => {
                        const flattenedData = flattenChartData(message.chartData.bar_line_data);
                        const dataKeys = extractDataKeys(flattenedData, message.chartData.data_keys);
                        console.log('[DEBUG] Line 차트 - flattenedData:', flattenedData);
                        console.log('[DEBUG] Line 차트 - 사용할 dataKeys:', dataKeys);
                        return (
                      <RechartsLineChart data={flattenedData} margin={{ top: 20, right: 30, left: 20, bottom: 80 }}>
                        <defs>
                          {dataKeys.map((dk: DataKeyConfig, idx: number) => (
                            <linearGradient key={`area-gradient-${dk.key}`} id={`area-gradient-${dk.key}`} x1="0" y1="0" x2="0" y2="1">
                              <stop offset="0%" stopColor={MODERN_COLORS[idx % MODERN_COLORS.length]} stopOpacity={0.3} />
                              <stop offset="100%" stopColor={MODERN_COLORS[idx % MODERN_COLORS.length]} stopOpacity={0} />
                            </linearGradient>
                          ))}
                          <filter id="glow">
                            <feGaussianBlur stdDeviation="2" result="coloredBlur"/>
                            <feMerge>
                              <feMergeNode in="coloredBlur"/>
                              <feMergeNode in="SourceGraphic"/>
                            </feMerge>
                          </filter>
                        </defs>
                        <CartesianGrid strokeDasharray="3 3" stroke="#334155" strokeOpacity={0.5} vertical={false} />
                        <XAxis
                          dataKey={message.chartData.x_axis_key || 'name'}
                          stroke="#64748b"
                          fontSize={10}
                          angle={-45}
                          textAnchor="end"
                          height={90}
                          interval="preserveStartEnd"
                          tick={{ fill: '#94a3b8' }}
                          axisLine={{ stroke: '#475569', strokeWidth: 1 }}
                          tickLine={{ stroke: '#475569' }}
                          tickFormatter={(value: string) => {
                            if (value && value.includes('-') && value.includes(':')) {
                              const parts = value.split(' ');
                              if (parts.length >= 2) {
                                const datePart = parts[0].split('-').slice(1).join('/');
                                const timePart = parts[1].substring(0, 5);
                                return `${datePart} ${timePart}`;
                              }
                            }
                            return value && value.length > 12 ? value.substring(0, 12) + '...' : value;
                          }}
                        />
                        <YAxis
                          stroke="#64748b"
                          fontSize={11}
                          tickFormatter={formatYAxisValue}
                          tick={{ fill: '#94a3b8' }}
                          axisLine={{ stroke: '#475569', strokeWidth: 1 }}
                          tickLine={{ stroke: '#475569' }}
                        />
                        <Tooltip
                          contentStyle={tooltipStyle}
                          labelStyle={{ color: '#f1f5f9', fontWeight: 600, marginBottom: '8px' }}
                          itemStyle={{ color: '#e2e8f0', padding: '2px 0' }}
                          cursor={{ stroke: '#6366f1', strokeWidth: 1, strokeDasharray: '5 5' }}
                        />
                        <Legend
                          wrapperStyle={legendStyle}
                          iconType="circle"
                          iconSize={8}
                          formatter={(value) => <span className="text-slate-300 text-sm ml-1">{value}</span>}
                        />
                        {dataKeys.map((dk: DataKeyConfig, idx: number) => (
                          <Line
                            key={dk.key}
                            type="monotone"
                            dataKey={dk.key}
                            stroke={MODERN_COLORS[idx % MODERN_COLORS.length]}
                            name={dk.label}
                            strokeWidth={3}
                            dot={{ fill: MODERN_COLORS[idx % MODERN_COLORS.length], strokeWidth: 2, stroke: '#1e293b', r: 4 }}
                            activeDot={{ r: 6, stroke: '#1e293b', strokeWidth: 2, fill: MODERN_COLORS[idx % MODERN_COLORS.length] }}
                            animationDuration={1000}
                            animationEasing="ease-out"
                            filter="url(#glow)"
                          />
                        ))}
                      </RechartsLineChart>
                        );
                      })()
                    )}
                  </ResponsiveContainer>
                </div>
              )}

            {/* Pie 차트 - 모던 도넛 스타일 (항목이 많으면 기타로 그룹화) */}
            {message.chartData.chart_type === 'pie' && message.chartData.pie_data && (() => {
              // 항목이 6개 초과시 상위 5개만 표시하고 나머지는 "기타"로 그룹화
              const MAX_ITEMS = 6;
              const rawData = message.chartData.pie_data!;
              const sortedData = [...rawData].sort((a, b) => b.value - a.value);

              let displayData: PieChartData[];
              if (sortedData.length > MAX_ITEMS) {
                const topItems = sortedData.slice(0, MAX_ITEMS - 1);
                const otherItems = sortedData.slice(MAX_ITEMS - 1);
                const otherValue = otherItems.reduce((sum, item) => sum + item.value, 0);
                displayData = [...topItems, { name: `기타 (${otherItems.length}개)`, value: otherValue }];
              } else {
                displayData = sortedData;
              }

              const total = displayData.reduce((sum, item) => sum + item.value, 0);

              return (
              <div className="bg-slate-800/50 rounded-xl p-4 border border-slate-700/30">
                <ResponsiveContainer width="100%" height={320}>
                  <RechartsPieChart>
                    <defs>
                      {displayData.map((_entry: PieChartData, idx: number) => (
                        <linearGradient key={`pie-gradient-${idx}`} id={`pie-gradient-${idx}`} x1="0" y1="0" x2="1" y2="1">
                          <stop offset="0%" stopColor={PIE_COLORS[idx % PIE_COLORS.length]} stopOpacity={1} />
                          <stop offset="100%" stopColor={PIE_COLORS[idx % PIE_COLORS.length]} stopOpacity={0.7} />
                        </linearGradient>
                      ))}
                      <filter id="pie-shadow">
                        <feDropShadow dx="0" dy="4" stdDeviation="8" floodOpacity="0.3"/>
                      </filter>
                    </defs>
                    <Pie
                      data={displayData}
                      cx="50%"
                      cy="50%"
                      labelLine={false}
                      label={false}
                      outerRadius={100}
                      innerRadius={60}
                      fill="#8884d8"
                      dataKey="value"
                      nameKey="name"
                      paddingAngle={3}
                      animationDuration={800}
                      animationEasing="ease-out"
                      filter="url(#pie-shadow)"
                    >
                      {displayData.map((_entry: PieChartData, idx: number) => (
                        <Cell
                          key={`cell-${idx}`}
                          fill={`url(#pie-gradient-${idx})`}
                          stroke="#1e293b"
                          strokeWidth={2}
                        />
                      ))}
                    </Pie>
                    <Tooltip
                      contentStyle={tooltipStyle}
                      labelStyle={{ color: '#f1f5f9', fontWeight: 600 }}
                      itemStyle={{ color: '#e2e8f0' }}
                      formatter={(value: number) => [`${value.toLocaleString()} (${((value / total) * 100).toFixed(1)}%)`, '수량']}
                    />
                    <Legend
                      layout="horizontal"
                      align="center"
                      wrapperStyle={{ paddingTop: '20px' }}
                      iconType="circle"
                      iconSize={10}
                      formatter={(value) => <span className="text-slate-300 text-xs ml-1 mr-2">{value}</span>}
                    />
                  </RechartsPieChart>
                </ResponsiveContainer>
              </div>
              );
            })()}

            {/* Gauge 차트 - 모던 반원형 게이지 */}
            {message.chartData.chart_type === 'gauge' && message.chartData.gauge_data && (
              <div className="bg-slate-800/50 rounded-xl p-6 border border-slate-700/30">
                <div className="flex flex-col items-center">
                  {/* 메인 값 표시 */}
                  <div className="relative mb-4">
                    <div className="text-5xl font-bold bg-gradient-to-r from-indigo-400 via-purple-400 to-pink-400 bg-clip-text text-transparent">
                      {message.chartData.gauge_data.value.toFixed(1)}
                    </div>
                    <div className="text-lg text-slate-400 text-center mt-1">
                      {message.chartData.gauge_data.unit}
                    </div>
                  </div>

                  {/* 게이지 바 */}
                  <div className="w-full max-w-md">
                    <div className="relative h-4 bg-slate-700/50 rounded-full overflow-hidden shadow-inner">
                      {/* 배경 그라데이션 */}
                      <div className="absolute inset-0 bg-gradient-to-r from-slate-700 via-slate-600 to-slate-700 opacity-50" />

                      {/* 진행 바 */}
                      <div
                        className="h-full rounded-full transition-all duration-1000 ease-out relative overflow-hidden"
                        style={{
                          width: `${Math.min(100, Math.max(0, ((message.chartData.gauge_data.value - message.chartData.gauge_data.min) / (message.chartData.gauge_data.max - message.chartData.gauge_data.min)) * 100))}%`,
                          background: message.chartData.gauge_data.value > 80
                            ? 'linear-gradient(90deg, #10b981 0%, #34d399 50%, #6ee7b7 100%)'
                            : message.chartData.gauge_data.value > 50
                            ? 'linear-gradient(90deg, #f59e0b 0%, #fbbf24 50%, #fcd34d 100%)'
                            : 'linear-gradient(90deg, #ef4444 0%, #f87171 50%, #fca5a5 100%)',
                          boxShadow: message.chartData.gauge_data.value > 80
                            ? '0 0 20px rgba(16, 185, 129, 0.5)'
                            : message.chartData.gauge_data.value > 50
                            ? '0 0 20px rgba(245, 158, 11, 0.5)'
                            : '0 0 20px rgba(239, 68, 68, 0.5)',
                        }}
                      >
                        {/* 광택 효과 */}
                        <div className="absolute inset-0 bg-gradient-to-b from-white/20 to-transparent" />
                      </div>
                    </div>

                    {/* 라벨 */}
                    <div className="flex justify-between items-center mt-3 px-1">
                      <span className="text-sm font-medium text-slate-500">
                        {message.chartData.gauge_data.min}{message.chartData.gauge_data.unit}
                      </span>
                      <span className="text-sm font-semibold text-slate-300 bg-slate-700/50 px-3 py-1 rounded-full">
                        {message.chartData.gauge_data.label}
                      </span>
                      <span className="text-sm font-medium text-slate-500">
                        {message.chartData.gauge_data.max}{message.chartData.gauge_data.unit}
                      </span>
                    </div>
                  </div>
                </div>
              </div>
            )}
          </div>
        )}

        {message.intent && (
          <p className="text-xs mt-2 opacity-70">의도: {message.intent}</p>
        )}
      </div>

      {message.role === 'user' && (
        <div className="w-8 h-8 rounded-full bg-secondary flex items-center justify-center flex-shrink-0">
          <User className="w-5 h-5" />
        </div>
      )}
    </div>
  );
});

MessageBubble.displayName = 'MessageBubble';

export default function ChatInterface() {
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState('');
  const [sessionId, setSessionId] = useState<string | undefined>();
  const [, setClaudeApiKey] = useState<string>(''); // 🔧 API 키 상태 (읽기는 불필요, 설정만 사용)
  const [showClearDialog, setShowClearDialog] = useState(false); // ✅ AlertDialog 상태
  const [isApiKeyConfigured, setIsApiKeyConfigured] = useState<boolean | null>(null); // 🔑 API 키 설정 여부 (null = 확인 중)
  const messagesRef = useRef<Message[]>([]); // 🔧 최신 messages 추적용 ref
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const navigate = useNavigate();

  // MES RAG 상태
  const [mesSessionId] = useState<string>(() => crypto.randomUUID()); // MES 세션 ID (고정)
  const [uploadedFile, setUploadedFile] = useState<{ name: string; rowCount: number } | null>(null);
  const [isUploading, setIsUploading] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  // 🔑 API 키 설정 상태 확인 (앱 시작시)
  useEffect(() => {
    async function checkApiKeyStatus() {
      try {
        const status = await getSystemStatus();
        setIsApiKeyConfigured(status.claude_configured);
        console.log('[ChatInterface] API key configured:', status.claude_configured);
      } catch (error) {
        console.error('[ChatInterface] Failed to check API key status:', error);
        setIsApiKeyConfigured(false);
      }
    }
    checkApiKeyStatus();
  }, []);

  // 🔧 Phase 1 Security Fix: Load API key from Tauri IPC (프로덕션 빌드 호환)
  useEffect(() => {
    async function loadApiKey() {
      try {
        const { invoke } = await import('@tauri-apps/api/tauri');
        const apiKey = await invoke<string>('load_api_key');
        if (apiKey) {
          console.log('[ChatInterface] API key loaded from system keychain');
          setClaudeApiKey(apiKey);
          setIsApiKeyConfigured(true); // API 키 로드 성공시 상태 업데이트

          // Rust 환경변수에도 설정 (chat_service.rs가 사용)
          await invoke('save_api_key', { apiKey });
        }
      } catch (error) {
        console.error('[ChatInterface] Failed to load API key from keychain:', error);

        // Fallback: localStorage
        const localKey = localStorage.getItem('claude_api_key');
        if (localKey) {
          console.log('[ChatInterface] Fallback to localStorage API key');
          setClaudeApiKey(localKey);
          setIsApiKeyConfigured(true); // localStorage에서 로드 성공시 상태 업데이트

          try {
            const { invoke } = await import('@tauri-apps/api/tauri');
            await invoke('save_api_key', { apiKey: localKey });
          } catch (e) {
            console.error('[ChatInterface] Failed to save API key to Rust env:', e);
          }
        }
      }
    }
    loadApiKey();
  }, []);

  // 앱 시작시 초기 환영 메시지 설정 (sessionStorage 사용: 탭 이동시 유지, 앱 재시작시 초기화)
  useEffect(() => {
    // sessionStorage에서 현재 세션 대화 복원 시도
    const savedMessages = sessionStorage.getItem('chat-messages');
    const savedSessionId = sessionStorage.getItem('chat-session-id');

    if (savedMessages) {
      try {
        const parsedMessages = JSON.parse(savedMessages);
        setMessages(parsedMessages);
        console.log('📂 [ChatInterface] Restored messages from sessionStorage:', parsedMessages.length);
      } catch (error) {
        console.error('Failed to parse saved messages:', error);
        // 파싱 실패시 초기 메시지 설정
        const initialMessage: Message = {
          role: 'assistant',
          content: '안녕하세요! TriFlow AI 어시스턴트입니다. 무엇을 도와드릴까요?\n\n다음과 같은 작업을 도와드릴 수 있습니다:\n\n📊 "지난 주 불량률 트렌드 보여줘"\n⚙️ "품질 검사 워크플로우 실행해줘"\n📋 "워크플로우 생성 방법 알려줘"\n🔧 "시스템 상태 확인해줘"',
        };
        setMessages([initialMessage]);
      }
    } else {
      // 새 세션 (앱 재시작) - 초기 환영 메시지 설정
      console.log('🆕 [ChatInterface] New session - setting initial message');
      const initialMessage: Message = {
        role: 'assistant',
        content: '안녕하세요! TriFlow AI 어시스턴트입니다. 무엇을 도와드릴까요?\n\n다음과 같은 작업을 도와드릴 수 있습니다:\n\n📊 "지난 주 불량률 트렌드 보여줘"\n⚙️ "품질 검사 워크플로우 실행해줘"\n📋 "워크플로우 생성 방법 알려줘"\n🔧 "시스템 상태 확인해줘"',
      };
      setMessages([initialMessage]);
    }

    if (savedSessionId) {
      setSessionId(savedSessionId);
      console.log('📂 [ChatInterface] Restored session ID:', savedSessionId);
    }

    // 이전 localStorage 데이터 정리 (마이그레이션)
    localStorage.removeItem('chat-messages');
    localStorage.removeItem('chat-session-id');
    localStorage.removeItem('chat-pending-request');
    localStorage.removeItem('chat-pending-response');
  }, []);

  // 메시지 변경시 sessionStorage에 저장 (탭 이동시 유지)
  useEffect(() => {
    if (messages.length > 0) {
      sessionStorage.setItem('chat-messages', JSON.stringify(messages));
    }
  }, [messages]);

  // Session ID 변경시 sessionStorage에 저장
  useEffect(() => {
    if (sessionId) {
      sessionStorage.setItem('chat-session-id', sessionId);
    }
  }, [sessionId]);

  // 🔧 Track latest messages in ref for visibility handler (클로저 문제 해결)
  useEffect(() => {
    if (messages.length > 0) {  // ✅ Fix: empty array 체크 추가
      messagesRef.current = messages;
      console.log('📝 [messagesRef] Updated to', messages.length, 'messages');
    } else {
      console.log('⚠️ [messagesRef] Skipping update for empty messages array');
    }
  }, [messages]);

  // Session ID는 메모리에만 유지 (재시작시 새 세션 시작)

  // 🔄 Session ID 변경시 백엔드 히스토리 동기화 (새 메시지 응답 처리)
  useEffect(() => {
    const syncWithBackend = async () => {
      if (!sessionId) {
        return; // 세션 없으면 스킵
      }

      // 탭이 숨겨져 있으면 동기화 스킵 (visibilitychange에서 처리)
      if (document.hidden) {
        console.log('⏩ [SessionSync] Tab hidden - skipping sync');
        return;
      }

      console.log('🔄 [SessionSync] Syncing with backend...');
      console.log('   Session ID:', sessionId);
      console.log('   Current messages:', messages.length);

      try {
        const backendHistory = await getChatHistory(sessionId);
        console.log(`   Backend history: ${backendHistory.length} messages`);

        // 백엔드에 새 메시지가 있으면 동기화
        if (backendHistory.length > messages.length) {
          console.log(`✅ [SessionSync] Found ${backendHistory.length - messages.length} new messages!`);
          const newMessages: Message[] = backendHistory.map((msg: any) => ({
            role: msg.role,
            content: msg.content,
            intent: msg.intent,
          }));
          setMessages(newMessages);
        } else {
          console.log('ℹ️ [SessionSync] Already up to date');
        }
      } catch (error) {
        console.error('❌ [SessionSync] Failed:', error);
      }
    };

    // 약간의 지연을 주어 백엔드가 메시지를 저장할 시간 확보
    const timeoutId = setTimeout(syncWithBackend, 300);
    return () => clearTimeout(timeoutId);
  }, [sessionId, messages.length]); // sessionId 변경시 실행

  // ⌨️ Keyboard shortcuts
  useEffect(() => {
    const handleGlobalKeyPress = (e: KeyboardEvent) => {
      // Ctrl+/ to focus input
      if (e.key === '/' && e.ctrlKey) {
        e.preventDefault();
        textareaRef.current?.focus();
      }
    };

    document.addEventListener('keydown', handleGlobalKeyPress);
    return () => document.removeEventListener('keydown', handleGlobalKeyPress);
  }, []);

  // 🔄 Page Visibility API: 탭 복귀시 백엔드 히스토리와 무조건 동기화
  useEffect(() => {
    const handleVisibilityChange = async () => {
      console.log('👁️ [Visibility Change] Document visible:', !document.hidden);

      if (!document.hidden && sessionId) {
        // 탭이 다시 활성화됨 - 백엔드와 동기화
        console.log('🔄 [Tab Return] Syncing with backend...');
        console.log('   Session ID:', sessionId);
        console.log('   Current messages count (ref):', messagesRef.current.length);

        try {
          const backendHistory = await getChatHistory(sessionId);
          console.log(`   Backend history count: ${backendHistory.length}`);

          // ✅ 백그라운드 응답 플래그 확인 (탭 전환 시 누락된 응답 감지)
          const hasPendingResponse = localStorage.getItem('chat-pending-response');
          console.log(`   Pending response flag: ${hasPendingResponse ? 'YES' : 'NO'}`);

          // 백엔드에 더 많은 메시지가 있거나, 백그라운드 응답 플래그가 있으면 동기화
          if (backendHistory.length > messagesRef.current.length || hasPendingResponse) {
            console.log(`✅ [Tab Return] Syncing ${backendHistory.length} messages!`);
            if (hasPendingResponse) {
              console.log('   🔄 [Tab Return] Processing background response...');
            }
            const newMessages: Message[] = backendHistory.map((msg: any) => ({
              role: msg.role,
              content: msg.content,
              intent: msg.intent,
            }));
            setMessages(newMessages);
            console.log('   Sync complete - new total:', newMessages.length);
          } else {
            console.log('ℹ️ [Tab Return] Already up to date');
          }

          // 플래그 정리 (항상)
          console.log('🧹 [Tab Return] Cleaning up flags...');
          localStorage.removeItem('chat-pending-request');
          localStorage.removeItem('chat-pending-response'); // 백그라운드 응답 플래그 제거
        } catch (error) {
          console.error('❌ [Tab Return] Failed to sync:', error);
        }
      }
    };

    document.addEventListener('visibilitychange', handleVisibilityChange);

    return () => {
      document.removeEventListener('visibilitychange', handleVisibilityChange);
    };
  }, [sessionId]); // sessionId만 의존 - messagesRef.current로 최신 값 참조

  // 차트 관련 키워드 감지 함수 (MES/ERP 데이터 시각화)
  // 데이터 시각화가 필요한 요청을 차트 API로 라우팅
  const isChartRequest = (message: string): boolean => {
    // 차트 API를 사용하지 않을 키워드 (비시각화 요청)
    const nonChartKeywords = [
      '예측', '전망', '방법', '알려줘', '설명해',
      '청구', '결제', '정산', '주문', '배송',
    ];

    // MES/ERP 차트용 키워드 - 차트 API로 처리
    const chartKeywords = [
      // 시각화 직접 요청
      '차트', '그래프', '시각화', '분석', '통계', '현황', '추이', '트렌드',
      // MES 생산/품질 키워드
      '온도', '살균', 'ccp', 'CCP', '품질검사', '금속검출',
      '생산량', '가동률', '불량률', '수율', '공정',
      '라인별', '설비별', '공정별', '작업자별',
      '센서', '알람', '비가동', '다운타임',
      '배치', 'LOT', 'lot', '로트',
      // ERP 재고/창고 키워드 (DB에 데이터 있음)
      '재고', '창고', '창고별', '품목별', '제품별',
      '입고', '출고', '재고이동', '자재',
      // ERP 거래 키워드
      '매출', '판매', '수주', '발주', '구매',
      '고객별', '거래처별', '월별', '일별', '주별',
    ];

    const lowerMessage = message.toLowerCase();

    // 비차트 키워드가 있으면 일반 채팅으로
    const hasNonChartKeyword = nonChartKeywords.some(kw => lowerMessage.includes(kw.toLowerCase()));
    if (hasNonChartKeyword) {
      console.log('💬 [isChartRequest] Non-chart keyword detected, routing to chat:', message);
      return false;
    }

    // 차트 키워드가 있으면 차트 요청으로 처리
    const hasChartKeyword = chartKeywords.some(kw => lowerMessage.includes(kw.toLowerCase()));
    if (hasChartKeyword) {
      console.log('📊 [isChartRequest] Chart keyword detected:', message);
      return true;
    }

    return false;
  };

  const sendMessageMutation = useMutation({
    mutationFn: async (request: ChatMessageRequest) => {
      console.log('🚀 [Mutation] Starting chat request:', {
        message: request.message.substring(0, 50) + '...',
        session_id: request.session_id,
      });

      // 📝 답변 대기 플래그 저장 (탭 전환 대비)
      localStorage.setItem('chat-pending-request', 'true');
      console.log('🏁 [Mutation] Pending flag set:', localStorage.getItem('chat-pending-request'));
      console.log('🏁 [Mutation] Session ID:', request.session_id);

      // 📌 2024-12-08: 모든 요청을 sendChatMessage로 전송
      // 백엔드의 Intent::ChartAnalysis → generate_chart_response() → PromptRouter 사용
      // (기존 isChartRequest() 분기 제거 - generate_chart API는 PromptRouter를 우회함)
      return await sendChatMessage(request);
    },
    onSuccess: (response: ChatMessageResponse & { chart_data?: ChartResponse }) => {
      console.log('✅ [Mutation] onSuccess called!');
      console.log('   Session ID:', response.session_id);
      console.log('   Response:', response.response.substring(0, 50) + '...');
      console.log('   Document hidden:', document.hidden);
      console.log('   Has chart_data:', !!response.chart_data);

      // ✅ 답변 성공 - 플래그 제거
      localStorage.removeItem('chat-pending-request');

      // ✅ 핵심 수정: 탭 상태에 따라 처리 분기
      if (document.hidden) {
        // 🔄 탭이 백그라운드 → 플래그 설정 (기존 기능 유지)
        console.log('⏳ [Mutation] Tab is hidden - setting pending flag');
        localStorage.setItem('chat-pending-response', 'true');
      } else {
        // ✅ 탭이 활성 상태 → 즉시 메시지 추가 (새 기능!)
        console.log('✅ [Mutation] Tab is visible - adding message immediately');

        // table_data가 있는 경우 tableData 포맷으로 변환
        // 배열 형태의 rows를 객체 형태로 변환 (백엔드가 2D 배열로 전송)
        const tableData = response.table_data ? {
          columns: response.table_data.columns,
          rows: response.table_data.rows.map((row: any[]) => {
            // 각 row 배열을 column 이름을 키로 하는 객체로 변환
            const rowObj: Record<string, any> = {};
            response.table_data!.columns.forEach((col, idx) => {
              rowObj[col] = row[idx];
            });
            return rowObj;
          }),
          totalCount: response.table_data.total_count
        } : undefined;

        setMessages((prev) => [
          ...prev,
          {
            role: 'assistant',
            content: response.response,
            intent: response.intent,
            tableData: tableData,
            chartData: response.chart_data, // 📊 차트 데이터 추가
          },
        ]);
      }

      // Session ID 설정
      setSessionId(response.session_id);
    },
    onError: (error: Error) => {
      console.error('❌ [Mutation] onError called!');
      console.error('   Error:', error);
      console.error('   Error message:', error.message);
      console.error('   Error stack:', error.stack);

      // ❌ 답변 실패 - 플래그 제거
      console.log('🧹 [Cleanup] Removing pending flag (onError)');
      localStorage.removeItem('chat-pending-request');
      console.log('🧹 [Cleanup] Flag removed, current value:', localStorage.getItem('chat-pending-request'));

      console.error('Chat error:', error);

      // 에러 메시지 표시 (React Query가 언마운트 처리함)
      setMessages((prev) => [
        ...prev,
        {
          role: 'assistant',
          content: `❌ 오류가 발생했습니다: ${error instanceof Error ? error.message : '알 수 없는 오류'}\n\n설정 페이지에서 Claude API 키가 올바르게 설정되었는지 확인해주세요.`,
        },
      ]);
    },
  });

  // MES RAG 응답을 파싱하여 테이블 데이터 추출
  const parseTableFromMesResponse = (text: string): { columns: string[]; rows: any[] } | null => {
    try {
      const lines = text.split('\n');
      let dataLines: string[] = [];
      let isCollectingData = false;

      for (const line of lines) {
        // 데이터 행을 찾기 (번호로 시작하는 줄)
        if (/^\d+\./.test(line.trim())) {
          isCollectingData = true;
          dataLines.push(line);
        } else if (isCollectingData && line.trim() === '') {
          // 빈 줄이 나오면 데이터 수집 종료
          break;
        } else if (isCollectingData) {
          dataLines.push(line);
        }
      }

      if (dataLines.length === 0) return null;

      // 첫 줄에서 컬럼 추출 (콤마로 구분)
      const firstLine = dataLines[0].replace(/^\d+\.\s*/, '');
      const columns = firstLine.split(',').map(col => col.trim());

      // 나머지 줄에서 데이터 추출
      const rows: any[] = [];
      for (let i = 0; i < dataLines.length; i++) {
        const line = dataLines[i].replace(/^\d+\.\s*/, '');
        const values = line.split(',').map(val => val.trim());

        if (values.length === columns.length) {
          const row: any = {};
          columns.forEach((col, idx) => {
            row[col] = values[idx];
          });
          rows.push(row);
        }
      }

      return rows.length > 0 ? { columns, rows } : null;
    } catch (error) {
      console.error('테이블 파싱 실패:', error);
      return null;
    }
  };

  const handleSend = async () => {
    if (!input.trim()) return;

    const userMessage: Message = {
      role: 'user',
      content: input,
    };

    setMessages((prev) => [...prev, userMessage]);

    // Priority 1: MES RAG 쿼리 (파일 업로드되어 있으면)
    if (uploadedFile) {
      try {
        const mesResult = await invoke<MesQueryResult>('query_mes_data', {
          sessionId: mesSessionId,
          question: input,
          topK: 5,
        });

        // MES RAG 답변이 있고, "찾지 못했습니다" 메시지가 아닌 경우에만 사용
        if (mesResult.answer &&
            !mesResult.answer.includes('찾지 못했습니다') &&
            !mesResult.answer.includes('업로드된 데이터가 없습니다')) {

          // 테이블 데이터 파싱 시도
          const tableData = parseTableFromMesResponse(mesResult.answer);

          // MES RAG 답변 성공
          const assistantMessage: Message = {
            role: 'assistant',
            content: `📊 **데이터 조회 결과:**\n\n${mesResult.answer}`,
            tableData: tableData || undefined,
          };
          setMessages((prev) => [...prev, assistantMessage]);
          setInput('');
          return;
        } else {
          console.log('[MES RAG] No relevant data found, falling back to general Chat LLM');
        }
      } catch (error) {
        console.error('[MES RAG] Query error:', error);
        // MES RAG 실패시 일반 Chat LLM으로 fallback
      }
    }

    // Priority 2: 일반 Chat LLM (MES 데이터 컨텍스트 포함)
    // MES 컨텍스트가 있으면 메시지에 추가
    let enrichedMessage = input;
    if (uploadedFile) {
      enrichedMessage = `[MES 데이터 업로드됨: ${uploadedFile.name} (${uploadedFile.rowCount}건)]\n\n${input}`;
      console.log('[Chat LLM] Including MES context in request');
    }

    sendMessageMutation.mutate({
      message: enrichedMessage,
      session_id: sessionId,
    });

    setInput('');
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const handleClearHistory = () => {
    // ✅ AlertDialog 표시 (삭제하지 않음)
    setShowClearDialog(true);
  };

  const confirmClearHistory = () => {
    // ✅ 사용자 확인 후 실제 삭제 실행
    const initialMessage: Message = {
      role: 'assistant',
      content: '안녕하세요! 👋 TriFlow AI 어시스턴트입니다.\n\n판단 실행, 워크플로우 관리, 데이터 시각화, BI 인사이트 생성 등을 도와드릴 수 있어요. 무엇을 도와드릴까요?',
    };
    setMessages([initialMessage]);
    setSessionId(undefined);
    // sessionStorage도 정리
    sessionStorage.removeItem('chat-messages');
    sessionStorage.removeItem('chat-session-id');
    setShowClearDialog(false);
  };

  const handleQuickAction = (query: string) => {
    setInput(query);
    // 약간의 지연을 주어 입력창에 텍스트가 표시되도록 함
    setTimeout(() => {
      const userMessage: Message = {
        role: 'user',
        content: query,
      };
      setMessages((prev) => [...prev, userMessage]);
      sendMessageMutation.mutate({
        message: query,
        session_id: sessionId,
      });
      setInput('');
    }, 100);
  };

  // MES RAG: CSV 파일 업로드 핸들러
  const handleFileSelect = async (file: File) => {
    if (!file.name.endsWith('.csv')) {
      toast({
        variant: 'destructive',
        title: '지원하지 않는 파일 형식',
        description: 'CSV 파일만 업로드 가능합니다.',
      });
      return;
    }

    setIsUploading(true);
    try {
      // ArrayBuffer → Vec<u8> 변환
      const arrayBuffer = await file.arrayBuffer();
      const uint8Array = new Uint8Array(arrayBuffer);
      const fileContent = Array.from(uint8Array);

      // Tauri invoke
      const result = await invoke<MesUploadResult>('upload_mes_data', {
        sessionId: mesSessionId,
        fileName: file.name,
        fileContent,
      });

      setUploadedFile({ name: file.name, rowCount: result.row_count });
      toast({
        title: '파일 업로드 완료',
        description: `${result.row_count}건의 데이터가 업로드되었습니다.`,
      });

      // 업로드 안내 메시지 추가
      const assistantMessage: Message = {
        role: 'assistant',
        content: `✅ "${file.name}" 파일이 업로드되었습니다 (${result.row_count}건).\n\n이제 데이터에 대해 자연어로 질문할 수 있습니다. 예:\n- "온도가 90도 이상인 데이터는?"\n- "지난주 불량률 평균은?"\n- "재고 부족한 품목은?"`,
      };
      setMessages((prev) => [...prev, assistantMessage]);
    } catch (error) {
      console.error('[MES RAG] Upload error:', error);
      toast({
        variant: 'destructive',
        title: '업로드 실패',
        description: error instanceof Error ? error.message : '파일 업로드 중 오류가 발생했습니다',
      });
    } finally {
      setIsUploading(false);
    }
  };

  // MES RAG: 업로드된 파일 삭제
  const handleDeleteMesData = async () => {
    try {
      await invoke('delete_mes_session', { sessionId: mesSessionId });
      setUploadedFile(null);
      toast({
        title: '데이터 삭제 완료',
        description: '업로드된 MES/ERP 데이터가 삭제되었습니다.',
      });
    } catch (error) {
      console.error('[MES RAG] Delete error:', error);
      toast({
        variant: 'destructive',
        title: '삭제 실패',
        description: error instanceof Error ? error.message : '파일 삭제 중 오류가 발생했습니다',
      });
    }
  };

  return (
    <div className="flex flex-col h-full">
      {/* 🔑 API 키 미설정 경고 배너 */}
      {isApiKeyConfigured === false && (
        <div className="mb-4 p-4 bg-amber-50 dark:bg-amber-950/30 border border-amber-200 dark:border-amber-800 rounded-lg">
          <div className="flex items-start gap-3">
            <AlertTriangle className="w-5 h-5 text-amber-600 dark:text-amber-500 flex-shrink-0 mt-0.5" />
            <div className="flex-1">
              <h3 className="font-semibold text-amber-800 dark:text-amber-200 mb-1">
                Claude API 키가 설정되지 않았습니다
              </h3>
              <p className="text-sm text-amber-700 dark:text-amber-300 mb-3">
                AI 채팅 기능을 사용하려면 먼저 설정 페이지에서 Claude API 키를 등록해주세요.
              </p>
              <Button
                size="sm"
                onClick={() => navigate('/settings')}
                className="bg-amber-600 hover:bg-amber-700 text-white"
              >
                <Settings className="w-4 h-4 mr-2" />
                설정으로 이동
              </Button>
            </div>
          </div>
        </div>
      )}

      {/* Header */}
      <div className="mb-6 flex items-start justify-between">
        <div>
          <h1 className="text-3xl font-bold mb-2">AI 어시스턴트</h1>
          <p className="text-muted-foreground">
            자연어로 대화하며 판단 실행, 워크플로우 관리, 데이터 분석을 수행하세요.
          </p>
        </div>
        <Button
          variant="outline"
          size="sm"
          onClick={handleClearHistory}
          className="flex items-center gap-2"
        >
          <Trash2 className="w-4 h-4" />
          대화 초기화
        </Button>
      </div>

      {/* Quick Actions */}
      {messages.length === 1 && ( // 초기 환영 메시지만 있을 때 표시
        <div className="mb-4 grid grid-cols-2 gap-2">
          <Button
            variant="outline"
            className="justify-start h-auto py-3"
            onClick={() => handleQuickAction('지난 주 불량률 트렌드 보여줘')}
            disabled={sendMessageMutation.isPending || isApiKeyConfigured === false}
          >
            <TrendingUp className="w-4 h-4 mr-2 flex-shrink-0" />
            <span className="text-sm">지난 주 불량률 트렌드</span>
          </Button>
          <Button
            variant="outline"
            className="justify-start h-auto py-3"
            onClick={() => handleQuickAction('품질 검사 워크플로우 실행해줘')}
            disabled={sendMessageMutation.isPending || isApiKeyConfigured === false}
          >
            <Play className="w-4 h-4 mr-2 flex-shrink-0" />
            <span className="text-sm">워크플로우 실행</span>
          </Button>
          <Button
            variant="outline"
            className="justify-start h-auto py-3"
            onClick={() => handleQuickAction('워크플로우 생성 방법 알려줘')}
            disabled={sendMessageMutation.isPending || isApiKeyConfigured === false}
          >
            <FileQuestion className="w-4 h-4 mr-2 flex-shrink-0" />
            <span className="text-sm">워크플로우 생성 방법</span>
          </Button>
          <Button
            variant="outline"
            className="justify-start h-auto py-3"
            onClick={() => handleQuickAction('시스템 상태 확인해줘')}
            disabled={sendMessageMutation.isPending || isApiKeyConfigured === false}
          >
            <Activity className="w-4 h-4 mr-2 flex-shrink-0" />
            <span className="text-sm">시스템 상태 확인</span>
          </Button>
        </div>
      )}

      {/* Messages */}
      <Card className="flex-1 overflow-y-auto p-6 mb-4 space-y-4">
        {messages.map((message, index) => (
          <MessageBubble key={index} message={message} index={index} />
        ))}

        {sendMessageMutation.isPending && (
          <div className="flex gap-3 justify-start">
            <div className="w-8 h-8 rounded-full overflow-hidden flex-shrink-0 animate-pulse">
              <img src="/chatbot_img.png" alt="AI" className="w-full h-full object-cover" />
            </div>
            <div className="bg-muted rounded-lg p-4">
              <p className="text-sm text-muted-foreground">생각 중...</p>
            </div>
          </div>
        )}
      </Card>

      {/* Input */}
      <div className="space-y-2">
        {/* Uploaded File Indicator */}
        {uploadedFile && (
          <div className="flex items-center gap-2 px-3 py-2 bg-muted rounded-md text-sm">
            <FileText className="w-4 h-4 flex-shrink-0" />
            <span className="flex-1 truncate">
              {uploadedFile.name} ({uploadedFile.rowCount}건)
            </span>
            <Button
              variant="ghost"
              size="icon"
              className="h-6 w-6"
              onClick={handleDeleteMesData}
            >
              <X className="w-4 h-4" />
            </Button>
          </div>
        )}

        {/* Input Area */}
        <div className="flex gap-2">
          {/* File Upload Button */}
          <input
            type="file"
            accept=".csv"
            ref={fileInputRef}
            onChange={(e) => {
              const file = e.target.files?.[0];
              if (file) handleFileSelect(file);
            }}
            style={{ display: 'none' }}
          />
          <Button
            variant="outline"
            size="icon"
            className="h-[60px] w-[60px] flex-shrink-0"
            onClick={() => fileInputRef.current?.click()}
            disabled={isUploading}
          >
            {isUploading ? (
              <div className="w-5 h-5 border-2 border-primary border-t-transparent rounded-full animate-spin" />
            ) : (
              <Paperclip className="w-5 h-5" />
            )}
          </Button>

          <Textarea
            ref={textareaRef}
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyPress={handleKeyPress}
            placeholder={
              isApiKeyConfigured === false
                ? '⚠️ API 키를 먼저 설정해주세요 (설정 페이지)'
                : uploadedFile
                ? 'MES/ERP 데이터에 대해 질문하세요... (예: "온도가 90도 이상인 데이터는?")'
                : '메시지를 입력하세요... (Shift+Enter로 줄바꿈, Ctrl+/로 포커스)'
            }
            className="min-h-[60px] resize-none"
            disabled={isApiKeyConfigured === false}
          />
          <Button
            onClick={handleSend}
            disabled={!input.trim() || sendMessageMutation.isPending || isUploading || isApiKeyConfigured === false}
            size="icon"
            className="h-[60px] w-[60px] flex-shrink-0"
          >
            <Send className="w-5 h-5" />
          </Button>
        </div>
      </div>

      {/* ✅ 대화 초기화 확인 다이얼로그 */}
      <AlertDialog open={showClearDialog} onOpenChange={setShowClearDialog}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>대화 내역 삭제</AlertDialogTitle>
            <AlertDialogDescription>
              채팅 내역을 모두 삭제하시겠습니까? 이 작업은 되돌릴 수 없습니다.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>취소</AlertDialogCancel>
            <AlertDialogAction onClick={confirmClearHistory}>확인</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}
