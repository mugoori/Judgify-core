import { useQuery } from '@tanstack/react-query';
import { getSystemStats, getJudgmentHistory } from '@/lib/tauri-api';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, LineChart, Line, PieChart, Pie, Cell, Legend } from 'recharts';
import { Activity, CheckCircle, XCircle, TrendingUp } from 'lucide-react';

export default function Dashboard() {
  const { data: stats } = useQuery({
    queryKey: ['system-stats'],
    queryFn: getSystemStats,
    refetchInterval: 30000, // 30초마다 갱신
  });

  const { data: recentJudgments } = useQuery({
    queryKey: ['recent-judgments'],
    queryFn: () => getJudgmentHistory(undefined, 50),
    refetchInterval: 30000,
  });

  // 판단 방법별 통계
  const methodStats = recentJudgments?.reduce(
    (acc, j) => {
      if (j.method_used === 'rule') acc.rule++;
      else if (j.method_used === 'llm') acc.llm++;
      else acc.hybrid++;
      return acc;
    },
    { rule: 0, llm: 0, hybrid: 0 }
  );

  const methodChartData = methodStats
    ? [
        { name: 'Rule', count: methodStats.rule },
        { name: 'LLM', count: methodStats.llm },
        { name: 'Hybrid', count: methodStats.hybrid },
      ]
    : [];

  // 최근 판단 결과 트렌드
  const resultTrend = recentJudgments
    ?.slice(-20)
    .map((j, i) => ({
      index: i + 1,
      confidence: Math.round(j.confidence * 100),
      result: j.result ? 1 : 0,
    })) || [];

  // 7일 트렌드 데이터 생성
  const last7Days = Array.from({ length: 7 }, (_, i) => {
    const date = new Date();
    date.setDate(date.getDate() - (6 - i));
    return {
      date: date.toLocaleDateString('ko-KR', { month: 'short', day: 'numeric' }),
      fullDate: date.toDateString(),
    };
  });

  const dailyTrend = last7Days.map(({ date, fullDate }) => {
    const dayJudgments = recentJudgments?.filter((j) => {
      const jDate = new Date(j.created_at).toDateString();
      return jDate === fullDate;
    }) || [];

    const passCount = dayJudgments.filter((j) => j.result).length;
    const avgConfidence = dayJudgments.length > 0
      ? dayJudgments.reduce((sum, j) => sum + j.confidence, 0) / dayJudgments.length
      : 0;

    return {
      date,
      count: dayJudgments.length,
      passRate: dayJudgments.length > 0 ? (passCount / dayJudgments.length) * 100 : 0,
      avgConfidence: Math.round(avgConfidence * 100),
    };
  });

  // 합격률 파이 차트 데이터
  const passCount = recentJudgments?.filter((j) => j.result).length || 0;
  const failCount = (recentJudgments?.length || 0) - passCount;
  const passRateData = [
    { name: '합격', value: passCount, color: '#10b981' },
    { name: '불합격', value: failCount, color: '#ef4444' },
  ];

  // 워크플로우별 통계
  const workflowStats = recentJudgments?.reduce(
    (acc, j) => {
      const wfId = j.workflow_id;
      if (!acc[wfId]) {
        acc[wfId] = { id: wfId, count: 0, totalConf: 0 };
      }
      acc[wfId].count++;
      acc[wfId].totalConf += j.confidence;
      return acc;
    },
    {} as Record<string, { id: string; count: number; totalConf: number }>
  );

  const workflowChartData = workflowStats
    ? Object.values(workflowStats)
        .map((w) => ({
          name: w.id.split('-').slice(-2).join(' '),
          count: w.count,
          avgConfidence: Math.round((w.totalConf / w.count) * 100),
        }))
        .sort((a, b) => b.count - a.count)
        .slice(0, 5)
    : [];

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold mb-2">데이터 대시보드</h1>
        <p className="text-muted-foreground">실시간 시스템 상태 및 판단 통계를 확인하세요.</p>
      </div>

      {/* KPI Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">총 판단 횟수</CardTitle>
            <Activity className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats?.total_judgments || 0}</div>
            <p className="text-xs text-muted-foreground mt-1">누적 판단 실행</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">워크플로우</CardTitle>
            <CheckCircle className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats?.total_workflows || 0}</div>
            <p className="text-xs text-muted-foreground mt-1">활성 워크플로우</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">평균 신뢰도</CardTitle>
            <TrendingUp className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {stats?.average_confidence ? `${(stats.average_confidence * 100).toFixed(1)}%` : '0%'}
            </div>
            <p className="text-xs text-muted-foreground mt-1">판단 신뢰도</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">학습 샘플</CardTitle>
            <XCircle className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats?.total_training_samples || 0}</div>
            <p className="text-xs text-muted-foreground mt-1">학습용 데이터</p>
          </CardContent>
        </Card>
      </div>

      {/* 7일 트렌드 (전체 너비) */}
      <Card>
        <CardHeader>
          <CardTitle>7일 트렌드</CardTitle>
          <CardDescription>최근 7일간 판단 횟수 및 합격률 추이</CardDescription>
        </CardHeader>
        <CardContent>
          <ResponsiveContainer width="100%" height={300}>
            <LineChart data={dailyTrend}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis dataKey="date" />
              <YAxis yAxisId="left" />
              <YAxis yAxisId="right" orientation="right" />
              <Tooltip />
              <Legend />
              <Line
                yAxisId="left"
                type="monotone"
                dataKey="count"
                stroke="hsl(var(--primary))"
                strokeWidth={2}
                name="판단 횟수"
              />
              <Line
                yAxisId="right"
                type="monotone"
                dataKey="passRate"
                stroke="#10b981"
                strokeWidth={2}
                name="합격률 (%)"
              />
            </LineChart>
          </ResponsiveContainer>
        </CardContent>
      </Card>

      {/* Charts */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* 판단 방법별 분포 */}
        <Card>
          <CardHeader>
            <CardTitle>판단 방법별 분포</CardTitle>
            <CardDescription>최근 50개 판단의 방법별 통계</CardDescription>
          </CardHeader>
          <CardContent>
            <ResponsiveContainer width="100%" height={300}>
              <BarChart data={methodChartData}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="name" />
                <YAxis />
                <Tooltip />
                <Bar dataKey="count" fill="hsl(var(--primary))" />
              </BarChart>
            </ResponsiveContainer>
          </CardContent>
        </Card>

        {/* 신뢰도 트렌드 */}
        <Card>
          <CardHeader>
            <CardTitle>신뢰도 트렌드</CardTitle>
            <CardDescription>최근 20개 판단의 신뢰도 변화</CardDescription>
          </CardHeader>
          <CardContent>
            <ResponsiveContainer width="100%" height={300}>
              <LineChart data={resultTrend}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="index" />
                <YAxis domain={[0, 100]} />
                <Tooltip />
                <Line type="monotone" dataKey="confidence" stroke="hsl(var(--primary))" strokeWidth={2} />
              </LineChart>
            </ResponsiveContainer>
          </CardContent>
        </Card>

        {/* 합격률 파이 차트 */}
        <Card>
          <CardHeader>
            <CardTitle>합격률</CardTitle>
            <CardDescription>전체 판단 결과 비율</CardDescription>
          </CardHeader>
          <CardContent>
            <ResponsiveContainer width="100%" height={300}>
              <PieChart>
                <Pie
                  data={passRateData}
                  cx="50%"
                  cy="50%"
                  labelLine={false}
                  label={({ name, percent }) => `${name}: ${(percent * 100).toFixed(0)}%`}
                  outerRadius={80}
                  fill="#8884d8"
                  dataKey="value"
                >
                  {passRateData.map((entry, index) => (
                    <Cell key={`cell-${index}`} fill={entry.color} />
                  ))}
                </Pie>
                <Tooltip />
                <Legend />
              </PieChart>
            </ResponsiveContainer>
          </CardContent>
        </Card>
      </div>

      {/* 워크플로우별 통계 */}
      <Card>
        <CardHeader>
          <CardTitle>워크플로우별 실행 통계</CardTitle>
          <CardDescription>가장 많이 사용된 워크플로우 TOP 5</CardDescription>
        </CardHeader>
        <CardContent>
          <ResponsiveContainer width="100%" height={300}>
            <BarChart data={workflowChartData} layout="vertical">
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis type="number" />
              <YAxis dataKey="name" type="category" width={100} />
              <Tooltip />
              <Legend />
              <Bar dataKey="count" fill="hsl(var(--primary))" name="실행 횟수" />
              <Bar dataKey="avgConfidence" fill="#10b981" name="평균 신뢰도 (%)" />
            </BarChart>
          </ResponsiveContainer>
        </CardContent>
      </Card>

      {/* Recent Judgments Table */}
      <Card>
        <CardHeader>
          <CardTitle>최근 판단 이력</CardTitle>
          <CardDescription>최근 10개의 판단 결과</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-2">
            {recentJudgments?.slice(0, 10).map((judgment) => (
              <div
                key={judgment.id}
                className="flex items-center justify-between p-3 rounded-lg border"
              >
                <div className="flex-1">
                  <div className="flex items-center gap-2">
                    {judgment.result ? (
                      <CheckCircle className="w-4 h-4 text-green-600" />
                    ) : (
                      <XCircle className="w-4 h-4 text-red-600" />
                    )}
                    <span className="font-medium">
                      {judgment.result ? '합격' : '불합격'}
                    </span>
                  </div>
                  <p className="text-sm text-muted-foreground mt-1">
                    {judgment.explanation.slice(0, 100)}...
                  </p>
                </div>
                <div className="text-right">
                  <div className="text-sm font-medium">
                    {(judgment.confidence * 100).toFixed(1)}%
                  </div>
                  <div className="text-xs text-muted-foreground">{judgment.method_used}</div>
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
