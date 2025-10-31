import { useState, useEffect } from 'react';
import { useMutation } from '@tanstack/react-query';
import { sendChatMessage, type ChatMessageRequest, type ChatMessageResponse } from '@/lib/tauri-api';
import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';
import { Card } from '@/components/ui/card';
import { Send, Bot, User, Trash2 } from 'lucide-react';

interface Message {
  role: 'user' | 'assistant';
  content: string;
  intent?: string;
}

export default function ChatInterface() {
  const [messages, setMessages] = useState<Message[]>([
    {
      role: 'assistant',
      content: '안녕하세요! Judgify AI 어시스턴트입니다. 무엇을 도와드릴까요?',
    },
  ]);
  const [input, setInput] = useState('');
  const [sessionId, setSessionId] = useState<string | undefined>();

  // Load chat history from localStorage on mount
  useEffect(() => {
    const savedMessages = localStorage.getItem('chat-messages');
    const savedSessionId = localStorage.getItem('chat-session-id');

    if (savedMessages) {
      try {
        setMessages(JSON.parse(savedMessages));
      } catch (error) {
        console.error('Failed to parse saved messages:', error);
      }
    }
    if (savedSessionId) {
      setSessionId(savedSessionId);
    }
  }, []);

  // Save messages to localStorage whenever they change
  useEffect(() => {
    localStorage.setItem('chat-messages', JSON.stringify(messages));
  }, [messages]);

  // Save session ID to localStorage
  useEffect(() => {
    if (sessionId) {
      localStorage.setItem('chat-session-id', sessionId);
    }
  }, [sessionId]);

  const sendMessageMutation = useMutation({
    mutationFn: (request: ChatMessageRequest) => sendChatMessage(request),
    onSuccess: (response: ChatMessageResponse) => {
      setSessionId(response.session_id);
      setMessages((prev) => [
        ...prev,
        {
          role: 'assistant',
          content: response.response,
          intent: response.intent,
        },
      ]);
    },
    onError: (error: Error) => {
      console.error('Chat error:', error);
      setMessages((prev) => [
        ...prev,
        {
          role: 'assistant',
          content: `❌ 오류가 발생했습니다: ${error.message}\n\n설정 페이지에서 OpenAI API 키가 올바르게 설정되었는지 확인해주세요.`,
        },
      ]);
    },
  });

  const handleSend = () => {
    if (!input.trim()) return;

    const userMessage: Message = {
      role: 'user',
      content: input,
    };

    setMessages((prev) => [...prev, userMessage]);

    sendMessageMutation.mutate({
      message: input,
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
    if (confirm('채팅 내역을 모두 삭제하시겠습니까?')) {
      const initialMessage: Message = {
        role: 'assistant',
        content: '안녕하세요! Judgify AI 어시스턴트입니다. 무엇을 도와드릴까요?',
      };
      setMessages([initialMessage]);
      setSessionId(undefined);
      localStorage.removeItem('chat-messages');
      localStorage.removeItem('chat-session-id');
    }
  };

  return (
    <div className="flex flex-col h-full">
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

      {/* Messages */}
      <Card className="flex-1 overflow-y-auto p-6 mb-4 space-y-4">
        {messages.map((message, index) => (
          <div
            key={index}
            className={`flex gap-3 ${
              message.role === 'user' ? 'justify-end' : 'justify-start'
            }`}
          >
            {message.role === 'assistant' && (
              <div className="w-8 h-8 rounded-full bg-primary flex items-center justify-center flex-shrink-0">
                <Bot className="w-5 h-5 text-primary-foreground" />
              </div>
            )}

            <div
              className={`max-w-[70%] rounded-lg p-4 ${
                message.role === 'user'
                  ? 'bg-primary text-primary-foreground'
                  : 'bg-muted'
              }`}
            >
              <p className="whitespace-pre-wrap">{message.content}</p>
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
        ))}

        {sendMessageMutation.isPending && (
          <div className="flex gap-3 justify-start">
            <div className="w-8 h-8 rounded-full bg-primary flex items-center justify-center flex-shrink-0">
              <Bot className="w-5 h-5 text-primary-foreground animate-pulse" />
            </div>
            <div className="bg-muted rounded-lg p-4">
              <p className="text-sm text-muted-foreground">생각 중...</p>
            </div>
          </div>
        )}
      </Card>

      {/* Input */}
      <div className="flex gap-2">
        <Textarea
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyPress={handleKeyPress}
          placeholder="메시지를 입력하세요... (Shift+Enter로 줄바꿈)"
          className="min-h-[60px] resize-none"
        />
        <Button
          onClick={handleSend}
          disabled={!input.trim() || sendMessageMutation.isPending}
          size="icon"
          className="h-[60px] w-[60px]"
        >
          <Send className="w-5 h-5" />
        </Button>
      </div>
    </div>
  );
}
