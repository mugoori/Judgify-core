import { useState, useEffect, useRef, memo } from 'react';
import { useMutation } from '@tanstack/react-query';
import { sendChatMessage, getChatHistory, type ChatMessageRequest, type ChatMessageResponse } from '@/lib/tauri-api-wrapper';
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
import { Send, Bot, User, Trash2, TrendingUp, Play, FileQuestion, Activity, Paperclip, FileText, X } from 'lucide-react';
import { invoke } from '@tauri-apps/api/tauri';
import { toast } from '@/components/ui/use-toast';
import type { MesUploadResult, MesQueryResult } from '@/types/mes';

interface Message {
  role: 'user' | 'assistant';
  content: string;
  intent?: string;
  tableData?: {
    columns: string[];
    rows: any[];
  };
}

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
        <div className="w-8 h-8 rounded-full bg-primary flex items-center justify-center flex-shrink-0">
          <Bot className="w-5 h-5 text-primary-foreground" />
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
        <p className="whitespace-pre-wrap">{message.content}</p>

        {/* 테이블 데이터 표시 */}
        {message.tableData && (
          <div className="mt-4 overflow-x-auto">
            <table className="min-w-full border-collapse">
              <thead>
                <tr className="border-b border-gray-600">
                  {message.tableData.columns.map((col, idx) => (
                    <th
                      key={idx}
                      className="px-3 py-2 text-left text-xs font-medium text-gray-300 uppercase tracking-wider"
                    >
                      {col}
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {message.tableData.rows.map((row, rowIdx) => (
                  <tr key={rowIdx} className="border-b border-gray-700 hover:bg-gray-800/50">
                    {message.tableData!.columns.map((col, colIdx) => (
                      <td key={colIdx} className="px-3 py-2 text-sm text-gray-300">
                        {row[col] === null || row[col] === undefined ? (
                          <span className="text-gray-500 italic">NULL</span>
                        ) : (
                          String(row[col])
                        )}
                      </td>
                    ))}
                  </tr>
                ))}
              </tbody>
            </table>
            {message.tableData.rows.length === 0 && (
              <p className="text-center text-gray-500 py-4">데이터가 없습니다</p>
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
  const [claudeApiKey, setClaudeApiKey] = useState<string>(''); // 🔧 API 키 상태
  const [showClearDialog, setShowClearDialog] = useState(false); // ✅ AlertDialog 상태
  const messagesRef = useRef<Message[]>([]); // 🔧 최신 messages 추적용 ref
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  // MES RAG 상태
  const [mesSessionId] = useState<string>(() => crypto.randomUUID()); // MES 세션 ID (고정)
  const [uploadedFile, setUploadedFile] = useState<{ name: string; rowCount: number } | null>(null);
  const [isUploading, setIsUploading] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  // 🔧 Phase 1 Security Fix: Load API key from Tauri IPC (프로덕션 빌드 호환)
  useEffect(() => {
    async function loadApiKey() {
      try {
        const { invoke } = await import('@tauri-apps/api/tauri');
        const apiKey = await invoke<string>('load_api_key');
        if (apiKey) {
          console.log('[ChatInterface] API key loaded from system keychain');
          setClaudeApiKey(apiKey);

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

  // Load chat history from localStorage on mount + recover pending responses
  useEffect(() => {
    const loadHistory = async () => {
      const savedMessages = localStorage.getItem('chat-messages');
      const savedSessionId = localStorage.getItem('chat-session-id');
      const pendingRequest = localStorage.getItem('chat-pending-request');

      // 파싱된 메시지를 저장할 변수 (복구 로직에서 재사용)
      let parsedMessages: Message[] = [];

      if (savedMessages) {
        try {
          parsedMessages = JSON.parse(savedMessages);

          // 🔄 마이그레이션: "Judgify AI" → "TriFlow AI" 자동 변환
          parsedMessages = parsedMessages.map((msg: Message) => ({
            ...msg,
            content: msg.content.replace(/Judgify AI/g, 'TriFlow AI')
          }));

          setMessages(parsedMessages);
        } catch (error) {
          console.error('Failed to parse saved messages:', error);
          // If parsing fails, set initial welcome message
          const initialMessage: Message = {
            role: 'assistant',
            content: '안녕하세요! TriFlow AI 어시스턴트입니다. 무엇을 도와드릴까요?\n\n다음과 같은 작업을 도와드릴 수 있습니다:\n\n📊 "지난 주 불량률 트렌드 보여줘"\n⚙️ "품질 검사 워크플로우 실행해줘"\n📋 "워크플로우 생성 방법 알려줘"\n🔧 "시스템 상태 확인해줘"',
          };
          parsedMessages = [initialMessage];
          setMessages(parsedMessages);
        }
      } else {
        // No saved messages, set initial welcome message
        const initialMessage: Message = {
          role: 'assistant',
          content: '안녕하세요! TriFlow AI 어시스턴트입니다. 무엇을 도와드릴까요?\n\n다음과 같은 작업을 도와드릴 수 있습니다:\n\n📊 "지난 주 불량률 트렌드 보여줘"\n⚙️ "품질 검사 워크플로우 실행해줘"\n📋 "워크플로우 생성 방법 알려줘"\n🔧 "시스템 상태 확인해줘"',
        };
        parsedMessages = [initialMessage];
        setMessages(parsedMessages);
      }

      if (savedSessionId) {
        setSessionId(savedSessionId);

        // 🔄 답변 대기 중이던 요청 복구
        if (pendingRequest) {
          console.log('⏳ Recovering pending chat response...');
          console.log(`   Session ID: ${savedSessionId}`);
          console.log(`   Current messages count: ${parsedMessages.length}`);

          try {
            const backendHistory = await getChatHistory(savedSessionId);
            console.log(`   Backend history count: ${backendHistory.length}`);
            console.log(`   Backend history:`, backendHistory);

            // 백엔드에 더 많은 메시지가 있으면 (답변이 와있음)
            if (backendHistory.length > parsedMessages.length) {
              console.log(`✅ Found new messages from backend! (${backendHistory.length} vs ${parsedMessages.length})`);
              const newMessages: Message[] = backendHistory.map((msg: any) => ({
                role: msg.role,
                content: msg.content,
                intent: msg.intent,
              }));
              console.log('   Setting messages:', newMessages);
              setMessages(newMessages);
              localStorage.removeItem('chat-pending-request');
            } else {
              console.log('⚠️ No new messages yet, clearing pending flag');
              localStorage.removeItem('chat-pending-request');
            }
          } catch (error) {
            console.error('❌ Failed to recover pending request:', error);
            localStorage.removeItem('chat-pending-request');
          }
        } else {
          console.log('ℹ️ No pending request found');
        }
      }
    };

    loadHistory();
  }, []);

  // Save messages to localStorage whenever they change (but not empty array)
  useEffect(() => {
    if (messages.length > 0) {
      localStorage.setItem('chat-messages', JSON.stringify(messages));
    }
  }, [messages]);

  // 🔧 Track latest messages in ref for visibility handler (클로저 문제 해결)
  useEffect(() => {
    if (messages.length > 0) {  // ✅ Fix: empty array 체크 추가
      messagesRef.current = messages;
      console.log('📝 [messagesRef] Updated to', messages.length, 'messages');
    } else {
      console.log('⚠️ [messagesRef] Skipping update for empty messages array');
    }
  }, [messages]);

  // Save session ID to localStorage
  useEffect(() => {
    if (sessionId) {
      localStorage.setItem('chat-session-id', sessionId);
    }
  }, [sessionId]);

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

      return await sendChatMessage(request);
    },
    onSuccess: (response: ChatMessageResponse) => {
      console.log('✅ [Mutation] onSuccess called!');
      console.log('   Session ID:', response.session_id);
      console.log('   Response:', response.response.substring(0, 50) + '...');
      console.log('   Document hidden:', document.hidden);

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
    localStorage.removeItem('chat-messages');
    localStorage.removeItem('chat-session-id');
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
            disabled={sendMessageMutation.isPending}
          >
            <TrendingUp className="w-4 h-4 mr-2 flex-shrink-0" />
            <span className="text-sm">지난 주 불량률 트렌드</span>
          </Button>
          <Button
            variant="outline"
            className="justify-start h-auto py-3"
            onClick={() => handleQuickAction('품질 검사 워크플로우 실행해줘')}
            disabled={sendMessageMutation.isPending}
          >
            <Play className="w-4 h-4 mr-2 flex-shrink-0" />
            <span className="text-sm">워크플로우 실행</span>
          </Button>
          <Button
            variant="outline"
            className="justify-start h-auto py-3"
            onClick={() => handleQuickAction('워크플로우 생성 방법 알려줘')}
            disabled={sendMessageMutation.isPending}
          >
            <FileQuestion className="w-4 h-4 mr-2 flex-shrink-0" />
            <span className="text-sm">워크플로우 생성 방법</span>
          </Button>
          <Button
            variant="outline"
            className="justify-start h-auto py-3"
            onClick={() => handleQuickAction('시스템 상태 확인해줘')}
            disabled={sendMessageMutation.isPending}
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
              uploadedFile
                ? 'MES/ERP 데이터에 대해 질문하세요... (예: "온도가 90도 이상인 데이터는?")'
                : '메시지를 입력하세요... (Shift+Enter로 줄바꿈, Ctrl+/로 포커스)'
            }
            className="min-h-[60px] resize-none"
          />
          <Button
            onClick={handleSend}
            disabled={!input.trim() || sendMessageMutation.isPending || isUploading}
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
