import { AlertCircle } from 'lucide-react'
import { Alert, AlertDescription, AlertTitle } from './ui/alert'

interface NotImplementedProps {
  message?: string
}

export default function NotImplemented({ message = "이 기능은 준비 중입니다." }: NotImplementedProps) {
  return (
    <div className="flex items-center justify-center h-full p-8">
      <Alert className="max-w-md">
        <AlertCircle className="h-4 w-4" />
        <AlertTitle>기능 준비 중</AlertTitle>
        <AlertDescription>
          {message}
        </AlertDescription>
      </Alert>
    </div>
  )
}
