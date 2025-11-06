import { useQuery } from '@tanstack/react-query';
import { getSystemStatus } from '@/lib/tauri-api';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Bell, Moon, Sun } from 'lucide-react';
import { useTheme } from '@/components/theme-provider';

export default function Header() {
  const { theme, setTheme } = useTheme();

  const { data: status } = useQuery({
    queryKey: ['system-status'],
    queryFn: getSystemStatus,
    refetchInterval: 10000,
  });

  return (
    <header className="h-16 border-b bg-card px-6 flex items-center justify-between">
      <div className="flex items-center gap-4">
        <h2 className="text-lg font-semibold">Judgify Desktop</h2>

        <div className="flex items-center gap-2">
          {status?.database_connected && (
            <Badge variant="outline" className="text-xs">
              DB 연결
            </Badge>
          )}
          {status?.claude_configured && (
            <Badge variant="outline" className="text-xs">
              LLM 활성
            </Badge>
          )}
        </div>
      </div>

      <div className="flex items-center gap-2">
        <Button variant="ghost" size="icon">
          <Bell className="w-4 h-4" />
        </Button>

        <Button
          variant="ghost"
          size="icon"
          onClick={() => setTheme(theme === 'dark' ? 'light' : 'dark')}
        >
          {theme === 'dark' ? (
            <Sun className="w-4 h-4" />
          ) : (
            <Moon className="w-4 h-4" />
          )}
        </Button>
      </div>
    </header>
  );
}
