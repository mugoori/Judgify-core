import { Label } from '@/components/ui/label'
import { Input } from '@/components/ui/input'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Textarea } from '@/components/ui/textarea'

/**
 * QUERY 노드 설정 폼
 *
 * 제조업 데이터 조회 예시:
 * - 설비 센서 데이터 조회
 * - 불량률 통계 조회
 * - 재고 현황 조회
 * - 작업 이력 조회
 */

interface QueryFormProps {
  config: {
    dataSource?: 'database' | 'api' | 'sensor' | 'file'
    queryType?: 'sql' | 'rest' | 'graphql'
    query?: string
    parameters?: string
    resultMapping?: string
  }
  onChange: (config: any) => void
}

export default function QueryForm({ config, onChange }: QueryFormProps) {
  const updateConfig = (key: string, value: any) => {
    onChange({ ...config, [key]: value })
  }

  return (
    <div className="space-y-4">
      {/* 데이터 소스 선택 */}
      <div className="space-y-2">
        <Label htmlFor="dataSource">데이터 소스</Label>
        <Select
          value={config.dataSource || 'database'}
          onValueChange={(value) => updateConfig('dataSource', value)}
        >
          <SelectTrigger id="dataSource">
            <SelectValue placeholder="데이터 소스 선택" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="database">데이터베이스</SelectItem>
            <SelectItem value="api">외부 API</SelectItem>
            <SelectItem value="sensor">센서 데이터</SelectItem>
            <SelectItem value="file">파일 시스템</SelectItem>
          </SelectContent>
        </Select>
      </div>

      {/* 쿼리 타입 (database인 경우) */}
      {config.dataSource === 'database' && (
        <div className="space-y-2">
          <Label htmlFor="queryType">쿼리 타입</Label>
          <Select
            value={config.queryType || 'sql'}
            onValueChange={(value) => updateConfig('queryType', value)}
          >
            <SelectTrigger id="queryType">
              <SelectValue placeholder="쿼리 타입 선택" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="sql">SQL</SelectItem>
              <SelectItem value="rest">REST API</SelectItem>
              <SelectItem value="graphql">GraphQL</SelectItem>
            </SelectContent>
          </Select>
        </div>
      )}

      {/* 쿼리 */}
      <div className="space-y-2">
        <Label htmlFor="query">쿼리</Label>
        <Textarea
          id="query"
          placeholder={
            config.dataSource === 'database'
              ? "SELECT * FROM equipment WHERE temperature > {threshold}"
              : "https://api.example.com/sensors/{sensorId}/data"
          }
          value={config.query || ''}
          onChange={(e) => updateConfig('query', e.target.value)}
          className="min-h-[100px] font-mono text-sm"
        />
        <p className="text-xs text-muted-foreground">
          파라미터는 &#123;변수명&#125; 형식으로 사용
        </p>
      </div>

      {/* 파라미터 */}
      <div className="space-y-2">
        <Label htmlFor="parameters">파라미터 (JSON)</Label>
        <Textarea
          id="parameters"
          placeholder='{"threshold": 90, "sensorId": "TEMP-001"}'
          value={config.parameters || ''}
          onChange={(e) => updateConfig('parameters', e.target.value)}
          className="min-h-[80px] font-mono text-sm"
        />
      </div>

      {/* 결과 매핑 */}
      <div className="space-y-2">
        <Label htmlFor="resultMapping">결과 매핑 (선택사항)</Label>
        <Input
          id="resultMapping"
          placeholder="예: data.temperature"
          value={config.resultMapping || ''}
          onChange={(e) => updateConfig('resultMapping', e.target.value)}
        />
        <p className="text-xs text-muted-foreground">
          결과에서 특정 필드만 추출 (JSONPath 형식)
        </p>
      </div>
    </div>
  )
}
