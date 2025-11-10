/**
 * Workflow Template Gallery Component
 * Phase 44: 사용자가 10개의 사전 정의된 워크플로우 템플릿을 선택할 수 있는 갤러리
 */

import { useState } from 'react';
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import {
  ALL_TEMPLATES,
  getTemplatesByCategory,
  searchTemplatesByTag,
  type WorkflowTemplate
} from '@/lib/workflow-templates';
import {
  Sparkles,
  Workflow,
  Zap,
  Search,
  CheckCircle2,
  Clock,
  Mail,
  FileUp,
  GitBranch,
  Repeat,
  Database,
  Webhook,
  ClipboardCheck
} from 'lucide-react';

interface TemplateGalleryProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSelectTemplate: (template: WorkflowTemplate) => void;
}

// 템플릿별 아이콘 매핑
const templateIcons: Record<string, React.ComponentType<{ className?: string }>> = {
  'quality-check': CheckCircle2,
  'api-integration': Zap,
  'data-transform': Database,
  'email-send': Mail,
  'file-upload': FileUp,
  'scheduling': Clock,
  'loop-processing': Repeat,
  'conditional-branching': GitBranch,
  'webhook-receiver': Webhook,
  'approval-workflow': ClipboardCheck,
};

// 카테고리별 색상
const categoryColors: Record<string, string> = {
  basic: 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-300',
  advanced: 'bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-300',
  integration: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300',
  automation: 'bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-300',
};

export default function TemplateGallery({ open, onOpenChange, onSelectTemplate }: TemplateGalleryProps) {
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedCategory, setSelectedCategory] = useState<string>('all');

  // 필터링된 템플릿 목록
  const getFilteredTemplates = (): WorkflowTemplate[] => {
    let templates = ALL_TEMPLATES;

    // 카테고리 필터
    if (selectedCategory !== 'all') {
      templates = getTemplatesByCategory(selectedCategory as any);
    }

    // 검색 필터
    if (searchQuery.trim()) {
      templates = templates.filter(
        (t) =>
          t.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
          t.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
          t.tags.some((tag) => tag.toLowerCase().includes(searchQuery.toLowerCase()))
      );
    }

    return templates;
  };

  const filteredTemplates = getFilteredTemplates();

  // 템플릿 선택 핸들러
  const handleSelectTemplate = (template: WorkflowTemplate) => {
    onSelectTemplate(template);
    onOpenChange(false);
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-4xl max-h-[90vh]">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2 text-2xl">
            <Sparkles className="h-6 w-6 text-primary" />
            워크플로우 템플릿 갤러리
          </DialogTitle>
          <DialogDescription>
            사전 정의된 10개의 템플릿 중에서 선택하여 빠르게 시작하세요
          </DialogDescription>
        </DialogHeader>

        {/* 검색 바 */}
        <div className="relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            placeholder="템플릿 검색... (예: 품질, API, 이메일)"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="pl-10"
          />
        </div>

        {/* 카테고리 탭 */}
        <Tabs value={selectedCategory} onValueChange={setSelectedCategory}>
          <TabsList className="grid grid-cols-5 w-full">
            <TabsTrigger value="all">
              전체 ({ALL_TEMPLATES.length})
            </TabsTrigger>
            <TabsTrigger value="basic">
              기본 ({getTemplatesByCategory('basic').length})
            </TabsTrigger>
            <TabsTrigger value="advanced">
              고급 ({getTemplatesByCategory('advanced').length})
            </TabsTrigger>
            <TabsTrigger value="integration">
              연동 ({getTemplatesByCategory('integration').length})
            </TabsTrigger>
            <TabsTrigger value="automation">
              자동화 ({getTemplatesByCategory('automation').length})
            </TabsTrigger>
          </TabsList>

          <TabsContent value={selectedCategory} className="mt-4">
            <ScrollArea className="h-[500px] pr-4">
              {filteredTemplates.length === 0 ? (
                <div className="text-center py-12">
                  <Workflow className="h-12 w-12 text-muted-foreground mx-auto mb-4" />
                  <p className="text-muted-foreground">
                    검색 결과가 없습니다. 다른 키워드를 시도해보세요.
                  </p>
                </div>
              ) : (
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  {filteredTemplates.map((template) => {
                    const Icon = templateIcons[template.id] || Workflow;

                    return (
                      <Card
                        key={template.id}
                        className="cursor-pointer transition-all hover:shadow-lg hover:border-primary"
                        onClick={() => handleSelectTemplate(template)}
                      >
                        <CardHeader>
                          <div className="flex items-start justify-between">
                            <div className="flex items-center gap-3">
                              <div className="p-2 rounded-lg bg-primary/10">
                                <Icon className="h-5 w-5 text-primary" />
                              </div>
                              <div>
                                <CardTitle className="text-lg">{template.name}</CardTitle>
                                <CardDescription className="text-sm mt-1">
                                  {template.description}
                                </CardDescription>
                              </div>
                            </div>
                          </div>
                        </CardHeader>
                        <CardContent>
                          <div className="space-y-3">
                            {/* 카테고리 배지 */}
                            <div className="flex items-center gap-2">
                              <Badge className={categoryColors[template.category]}>
                                {template.category === 'basic' && '기본'}
                                {template.category === 'advanced' && '고급'}
                                {template.category === 'integration' && '연동'}
                                {template.category === 'automation' && '자동화'}
                              </Badge>
                              <span className="text-sm text-muted-foreground">
                                노드 {template.nodes.length}개 · 연결 {template.edges.length}개
                              </span>
                            </div>

                            {/* 태그들 */}
                            <div className="flex flex-wrap gap-1">
                              {template.tags.map((tag) => (
                                <Badge key={tag} variant="outline" className="text-xs">
                                  {tag}
                                </Badge>
                              ))}
                            </div>

                            {/* 선택 버튼 */}
                            <Button className="w-full" size="sm">
                              이 템플릿 사용하기
                            </Button>
                          </div>
                        </CardContent>
                      </Card>
                    );
                  })}
                </div>
              )}
            </ScrollArea>
          </TabsContent>
        </Tabs>

        {/* 통계 표시 */}
        <div className="flex items-center justify-between text-sm text-muted-foreground border-t pt-4">
          <div className="flex items-center gap-4">
            <span>총 {ALL_TEMPLATES.length}개 템플릿</span>
            <span>·</span>
            <span>검색 결과: {filteredTemplates.length}개</span>
          </div>
          <Button variant="ghost" size="sm" onClick={() => onOpenChange(false)}>
            취소
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}
