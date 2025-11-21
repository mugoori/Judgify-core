import { Label } from '@/components/ui/label'
import { Input } from '@/components/ui/input'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Textarea } from '@/components/ui/textarea'
import { Switch } from '@/components/ui/switch'

/**
 * ALERT 노드 설정 폼
 *
 * 제조업 알림 예시:
 * - 설비 이상 알림 (Slack, Email)
 * - 불량률 급증 알림
 * - 재고 부족 알림
 * - 생산 목표 달성 알림
 */

interface AlertFormProps {
  config: {
    channels?: string[]
    recipients?: string
    subject?: string
    messageTemplate?: string
    priority?: 'low' | 'medium' | 'high' | 'critical'
    includeData?: boolean
  }
  onChange: (config: any) => void
}

export default function AlertForm({ config, onChange }: AlertFormProps) {
  const updateConfig = (key: string, value: any) => {
    onChange({ ...config, [key]: value })
  }

  const toggleChannel = (channel: string) => {
    const currentChannels = config.channels || []
    const newChannels = currentChannels.includes(channel)
      ? currentChannels.filter(c => c !== channel)
      : [...currentChannels, channel]
    updateConfig('channels', newChannels)
  }

  return (
    <div className="space-y-4">
      {/* 알림 채널 선택 (다중 선택) */}
      <div className="space-y-2">
        <Label>알림 채널</Label>
        <div className="space-y-2">
          <div className="flex items-center space-x-2">
            <Switch
              id="channel-email"
              checked={(config.channels || []).includes('email')}
              onCheckedChange={() => toggleChannel('email')}
            />
            <Label htmlFor="channel-email" className="text-sm font-normal cursor-pointer">
              📧 이메일
            </Label>
          </div>
          <div className="flex items-center space-x-2">
            <Switch
              id="channel-slack"
              checked={(config.channels || []).includes('slack')}
              onCheckedChange={() => toggleChannel('slack')}
            />
            <Label htmlFor="channel-slack" className="text-sm font-normal cursor-pointer">
              💬 Slack
            </Label>
          </div>
          <div className="flex items-center space-x-2">
            <Switch
              id="channel-teams"
              checked={(config.channels || []).includes('teams')}
              onCheckedChange={() => toggleChannel('teams')}
            />
            <Label htmlFor="channel-teams" className="text-sm font-normal cursor-pointer">
              👥 Microsoft Teams
            </Label>
          </div>
          <div className="flex items-center space-x-2">
            <Switch
              id="channel-webhook"
              checked={(config.channels || []).includes('webhook')}
              onCheckedChange={() => toggleChannel('webhook')}
            />
            <Label htmlFor="channel-webhook" className="text-sm font-normal cursor-pointer">
              🔗 Webhook
            </Label>
          </div>
        </div>
      </div>

      {/* 수신자 목록 */}
      <div className="space-y-2">
        <Label htmlFor="recipients">수신자</Label>
        <Textarea
          id="recipients"
          placeholder="이메일 주소 또는 Slack 채널을 쉼표로 구분&#10;예: admin@company.com, #manufacturing-alerts"
          value={config.recipients || ''}
          onChange={(e) => updateConfig('recipients', e.target.value)}
          className="min-h-[80px]"
        />
      </div>

      {/* 우선순위 */}
      <div className="space-y-2">
        <Label htmlFor="priority">우선순위</Label>
        <Select
          value={config.priority || 'medium'}
          onValueChange={(value) => updateConfig('priority', value)}
        >
          <SelectTrigger id="priority">
            <SelectValue placeholder="우선순위 선택" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="low">🟢 낮음</SelectItem>
            <SelectItem value="medium">🟡 중간</SelectItem>
            <SelectItem value="high">🟠 높음</SelectItem>
            <SelectItem value="critical">🔴 긴급</SelectItem>
          </SelectContent>
        </Select>
      </div>

      {/* 제목 */}
      <div className="space-y-2">
        <Label htmlFor="subject">알림 제목</Label>
        <Input
          id="subject"
          placeholder="예: [긴급] 설비 온도 임계값 초과"
          value={config.subject || ''}
          onChange={(e) => updateConfig('subject', e.target.value)}
        />
      </div>

      {/* 메시지 템플릿 */}
      <div className="space-y-2">
        <Label htmlFor="messageTemplate">메시지 템플릿</Label>
        <Textarea
          id="messageTemplate"
          placeholder={"설비 {equipment_id}의 온도가 {temperature}°C로 임계값을 초과했습니다.\\n즉시 점검이 필요합니다."}
          value={config.messageTemplate || ''}
          onChange={(e) => updateConfig('messageTemplate', e.target.value)}
          className="min-h-[120px]"
        />
        <p className="text-xs text-muted-foreground">
          &#123;변수명&#125; 형식으로 데이터 삽입 가능
        </p>
      </div>

      {/* 옵션 */}
      <div className="flex items-center space-x-2">
        <Switch
          id="includeData"
          checked={config.includeData || false}
          onCheckedChange={(checked) => updateConfig('includeData', checked)}
        />
        <Label htmlFor="includeData" className="text-sm font-normal cursor-pointer">
          워크플로우 실행 데이터 전체 포함
        </Label>
      </div>
    </div>
  )
}
