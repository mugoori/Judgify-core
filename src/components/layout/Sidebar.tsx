import { Link, useLocation } from 'react-router-dom';
import { cn } from '@/lib/utils';
import {
  MessageSquare,
  LayoutDashboard,
  GitBranch,
  Sparkles,
  Settings,
  ChevronLeft,
  ChevronRight,
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { motion, AnimatePresence } from 'framer-motion';

interface SidebarProps {
  isOpen: boolean;
  onToggle: () => void;
}

const navigation = [
  {
    name: 'AI 채팅',
    href: '/',
    icon: MessageSquare,
  },
  {
    name: '대시보드',
    href: '/dashboard',
    icon: LayoutDashboard,
  },
  {
    name: '워크플로우',
    href: '/workflow',
    icon: GitBranch,
  },
  {
    name: 'BI 인사이트',
    href: '/bi',
    icon: Sparkles,
  },
  {
    name: '설정',
    href: '/settings',
    icon: Settings,
  },
];

export default function Sidebar({ isOpen, onToggle }: SidebarProps) {
  const location = useLocation();

  return (
    <aside
      className={cn(
        'bg-card border-r transition-all duration-300 flex flex-col',
        isOpen ? 'w-64' : 'w-16'
      )}
      aria-label="메인 네비게이션"
    >
      {/* Logo */}
      <div className="h-16 flex items-center justify-between px-4 border-b">
        <AnimatePresence mode="wait">
          {isOpen && (
            <motion.div
              initial={{ opacity: 0, x: -20 }}
              animate={{ opacity: 1, x: 0 }}
              exit={{ opacity: 0, x: -20 }}
              transition={{ duration: 0.2 }}
              className="flex items-center gap-2"
            >
              <img
                src="/triflow-logo.png"
                alt="TriFlow AI Logo"
                className="w-8 h-8 rounded-lg object-contain"
              />
              <span className="font-bold text-lg">TriFlow AI</span>
            </motion.div>
          )}
        </AnimatePresence>
        <Button
          variant="ghost"
          size="icon"
          onClick={onToggle}
          className="ml-auto"
          aria-label={isOpen ? '사이드바 닫기' : '사이드바 열기'}
        >
          {isOpen ? (
            <ChevronLeft className="w-4 h-4" />
          ) : (
            <ChevronRight className="w-4 h-4" />
          )}
        </Button>
      </div>

      {/* Navigation */}
      <nav className="flex-1 p-2 space-y-1" aria-label="주요 메뉴">
        {navigation.map((item) => {
          const isActive = location.pathname === item.href;
          return (
            <Link
              key={item.name}
              to={item.href}
              aria-label={item.name}
              aria-current={isActive ? 'page' : undefined}
            >
              <div
                className={cn(
                  'flex items-center gap-3 px-3 py-2 rounded-lg transition-colors',
                  isActive
                    ? 'bg-primary text-primary-foreground'
                    : 'hover:bg-accent hover:text-accent-foreground'
                )}
              >
                <item.icon className="w-5 h-5 flex-shrink-0" aria-hidden="true" />
                <AnimatePresence mode="wait">
                  {isOpen && (
                    <motion.span
                      initial={{ opacity: 0, x: -10 }}
                      animate={{ opacity: 1, x: 0 }}
                      exit={{ opacity: 0, x: -10 }}
                      transition={{ duration: 0.2 }}
                      className="text-sm font-medium"
                    >
                      {item.name}
                    </motion.span>
                  )}
                </AnimatePresence>
              </div>
            </Link>
          );
        })}
      </nav>

      {/* Footer */}
      <AnimatePresence mode="wait">
        {isOpen && (
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: 20 }}
            transition={{ duration: 0.2 }}
            className="p-4 border-t text-xs text-muted-foreground"
          >
            <p>TriFlow AI Desktop v1.0.0</p>
            <p className="mt-1">Tauri + Rust + React</p>
          </motion.div>
        )}
      </AnimatePresence>
    </aside>
  );
}
