import { useEffect } from 'react';
import { useToast } from './ui/use-toast';

export default function OfflineDetector() {
  const { toast } = useToast();

  useEffect(() => {
    const handleOnline = () => {
      toast({
        title: '다시 온라인 상태입니다',
        description: '네트워크 연결이 복구되었습니다.',
        duration: 3000,
        className: 'bg-green-50 border-green-200',
      });
    };

    const handleOffline = () => {
      toast({
        title: '오프라인 상태입니다',
        description: '네트워크 연결을 확인해주세요. 일부 기능이 제한됩니다.',
        duration: 5000,
        variant: 'destructive',
      });
    };

    // Check initial status
    if (!navigator.onLine) {
      handleOffline();
    }

    window.addEventListener('online', handleOnline);
    window.addEventListener('offline', handleOffline);

    return () => {
      window.removeEventListener('online', handleOnline);
      window.removeEventListener('offline', handleOffline);
    };
  }, [toast]);

  return null; // This component doesn't render anything
}
