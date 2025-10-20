# Judgify Desktop - Windows Application 개발 가이드

**현재 상태**: 프로젝트 기본 구조 완성 (30% 완료)
**다음 단계**: 백엔드 개발자가 Rust/React 코드 완성

---

## 🎯 프로젝트 현황

### ✅ 완료된 작업
1. **개발 계획서** (`docs/development-plan.md`) - 전체 8주 개발 계획
2. **프로젝트 설정 파일**
   - `package.json` - Frontend 의존성
   - `tsconfig.json` - TypeScript 설정
   - `vite.config.ts` - Vite 빌드 설정
   - `tailwind.config.js` - Tailwind CSS 설정
3. **Tauri 설정**
   - `src-tauri/Cargo.toml` - Rust 의존성
   - `src-tauri/tauri.conf.json` - Tauri 앱 설정
   - `src-tauri/src/main.rs` - Rust 엔트리포인트
4. **React 기본 구조**
   - `src/main.tsx` - React 엔트리포인트
   - `src/App.tsx` - 메인 앱 컴포넌트
   - `src/styles/globals.css` - 글로벌 스타일
5. **모듈 구조**
   - `src-tauri/src/commands/` - Tauri Command 레이어
   - `src-tauri/src/services/` - 비즈니스 로직 레이어

### ⚠️ 작업 필요
1. **Rust 백엔드 서비스 구현** (60% 작업량)
   - Judgment Engine (하이브리드 판단)
   - Learning Service (자동학습)
   - BI Service (LLM 기반 인사이트)
   - Database Layer (SQLite + FAISS)
2. **React Frontend 구현** (30% 작업량)
   - 5개 페이지 (Chat, Dashboard, Workflow, BI, Settings)
   - shadcn/ui 컴포넌트 통합
3. **테스트 및 배포** (10% 작업량)

---

## 🚀 빠른 시작

### 1. 환경 설정

#### 필수 도구 설치
```bash
# Rust 설치 (https://rustup.rs/)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Node.js 20+ 설치 (https://nodejs.org/)
# 설치 후 확인:
node --version  # v20.x.x
npm --version   # 10.x.x

# pnpm 설치
npm install -g pnpm

# Tauri CLI 설치
cargo install tauri-cli
```

#### Windows 추가 요구사항
```bash
# Visual Studio C++ Build Tools 설치
# https://visualstudio.microsoft.com/visual-cpp-build-tools/
# "Desktop development with C++" 워크로드 선택

# WebView2 Runtime (보통 Windows 11에 사전 설치됨)
# https://developer.microsoft.com/en-us/microsoft-edge/webview2/
```

### 2. 프로젝트 설정

```bash
# 의존성 설치
pnpm install

# Rust 의존성 빌드 (첫 실행시 시간 소요)
cd src-tauri
cargo build
cd ..
```

### 3. 개발 서버 실행

```bash
# 개발 모드 (Hot Reload)
pnpm tauri dev

# 또는
cargo tauri dev
```

처음 실행시 Rust 컴파일에 **5~10분** 소요됩니다.

---

## 📦 프로젝트 구조 상세

```
judgify-desktop/
├── docs/
│   └── development-plan.md         ← 전체 개발 계획서 (필독!)
│
├── src/                             ← React Frontend
│   ├── main.tsx                    ✅ 완료
│   ├── App.tsx                     ✅ 완료
│   ├── pages/                      ⚠️ 작업 필요
│   │   ├── ChatInterface.tsx       ⚠️ TODO
│   │   ├── Dashboard.tsx           ⚠️ TODO
│   │   ├── WorkflowBuilder.tsx     ⚠️ TODO
│   │   ├── BiInsights.tsx          ⚠️ TODO
│   │   └── Settings.tsx            ⚠️ TODO
│   ├── components/                  ⚠️ 작업 필요
│   │   ├── ui/                     ⚠️ shadcn/ui 컴포넌트
│   │   ├── charts/                 ⚠️ Recharts 차트
│   │   ├── workflow/               ⚠️ React Flow 노드
│   │   └── layout/                 ⚠️ Sidebar, Header
│   ├── lib/
│   │   └── tauri-api.ts            ⚠️ Tauri IPC 래퍼
│   ├── hooks/                      ⚠️ Custom Hooks
│   └── store/                      ⚠️ Zustand Store
│
├── src-tauri/                       ← Rust Backend
│   ├── src/
│   │   ├── main.rs                 ✅ 완료
│   │   ├── commands/               ✅ 구조만 완료
│   │   │   ├── mod.rs              ✅ 완료
│   │   │   ├── judgment.rs         ✅ 스켈레톤
│   │   │   ├── learning.rs         ⚠️ TODO
│   │   │   ├── bi.rs               ⚠️ TODO
│   │   │   ├── chat.rs             ⚠️ TODO
│   │   │   ├── workflow.rs         ⚠️ TODO
│   │   │   └── system.rs           ⚠️ TODO
│   │   ├── services/               ✅ 구조만 완료
│   │   │   ├── mod.rs              ✅ 완료
│   │   │   ├── judgment_engine.rs  ⚠️ TODO (핵심!)
│   │   │   ├── rule_engine.rs      ⚠️ TODO
│   │   │   ├── llm_engine.rs       ⚠️ TODO
│   │   │   ├── learning_service.rs ⚠️ TODO
│   │   │   ├── bi_service.rs       ⚠️ TODO
│   │   │   └── workflow_service.rs ⚠️ TODO
│   │   ├── database/               ⚠️ 전체 TODO
│   │   │   ├── mod.rs              ⚠️ TODO
│   │   │   ├── sqlite.rs           ⚠️ TODO
│   │   │   ├── faiss.rs            ⚠️ TODO (벡터 검색)
│   │   │   └── models.rs           ⚠️ TODO
│   │   └── utils/                  ⚠️ 전체 TODO
│   │       ├── mod.rs              ⚠️ TODO
│   │       ├── openai.rs           ⚠️ TODO (LLM 클라이언트)
│   │       └── embeddings.rs       ⚠️ TODO
│   ├── Cargo.toml                  ✅ 완료
│   └── tauri.conf.json             ✅ 완료
│
├── package.json                     ✅ 완료
├── tsconfig.json                    ✅ 완료
├── vite.config.ts                   ✅ 완료
├── tailwind.config.js               ✅ 완료
└── README-DESKTOP.md               ✅ 이 파일!
```

---

## 🔧 개발 워크플로우

### Phase 1: 데이터베이스 레이어 (Week 2)

**파일**: `src-tauri/src/database/`

#### 1.1 SQLite 연결 및 스키마
```rust
// src-tauri/src/database/sqlite.rs

use rusqlite::{Connection, Result};
use std::path::PathBuf;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new() -> Result<Self> {
        let db_path = Self::get_db_path()?;
        let conn = Connection::open(db_path)?;

        // 테이블 생성
        conn.execute_batch(include_str!("../../migrations/001_init.sql"))?;

        Ok(Self { conn })
    }

    fn get_db_path() -> Result<PathBuf> {
        let app_data = std::env::var("APPDATA")
            .or_else(|_| std::env::var("HOME"))?;
        let db_dir = PathBuf::from(app_data).join("Judgify");
        std::fs::create_dir_all(&db_dir)?;
        Ok(db_dir.join("judgify.db"))
    }

    // 판단 결과 저장
    pub fn save_judgment(&self, judgment: &JudgmentResult) -> Result<()> {
        self.conn.execute(
            "INSERT INTO judgments (id, workflow_id, result, confidence, method_used, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (
                &judgment.id,
                &judgment.workflow_id,
                &judgment.result,
                judgment.confidence,
                &judgment.method_used,
                chrono::Utc::now(),
            ),
        )?;
        Ok(())
    }
}
```

#### 1.2 SQL 마이그레이션
```sql
-- src-tauri/migrations/001_init.sql

CREATE TABLE IF NOT EXISTS judgments (
    id TEXT PRIMARY KEY,
    workflow_id TEXT NOT NULL,
    input_data TEXT NOT NULL,
    result TEXT NOT NULL,
    confidence REAL NOT NULL,
    method_used TEXT NOT NULL,
    explanation TEXT,
    created_at DATETIME NOT NULL,
    INDEX idx_workflow_id (workflow_id),
    INDEX idx_created_at (created_at)
);

CREATE TABLE IF NOT EXISTS workflows (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    definition TEXT NOT NULL,
    rule_expression TEXT,
    version INTEGER DEFAULT 1,
    is_active BOOLEAN DEFAULT 1,
    created_at DATETIME NOT NULL
);

CREATE TABLE IF NOT EXISTS training_samples (
    id TEXT PRIMARY KEY,
    workflow_id TEXT NOT NULL,
    input_data TEXT NOT NULL,
    expected_result TEXT NOT NULL,
    actual_result TEXT,
    accuracy REAL,
    embedding BLOB,
    created_at DATETIME NOT NULL,
    FOREIGN KEY (workflow_id) REFERENCES workflows(id)
);
```

### Phase 2: Judgment Engine (Week 2-3)

**파일**: `src-tauri/src/services/judgment_engine.rs`

#### 2.1 하이브리드 판단 엔진
```rust
// src-tauri/src/services/judgment_engine.rs

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct JudgmentInput {
    pub workflow_id: String,
    pub input_data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JudgmentResult {
    pub id: String,
    pub workflow_id: String,
    pub result: bool,
    pub confidence: f64,
    pub method_used: String, // "rule" | "llm" | "hybrid"
    pub explanation: String,
}

pub struct JudgmentEngine {
    rule_engine: crate::services::rule_engine::RuleEngine,
    llm_engine: crate::services::llm_engine::LLMEngine,
    db: crate::database::sqlite::Database,
}

impl JudgmentEngine {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            rule_engine: crate::services::rule_engine::RuleEngine::new()?,
            llm_engine: crate::services::llm_engine::LLMEngine::new()?,
            db: crate::database::sqlite::Database::new()?,
        })
    }

    pub async fn execute(&self, input: JudgmentInput) -> anyhow::Result<JudgmentResult> {
        // 1. Rule Engine 시도
        let rule_result = self.rule_engine.evaluate(&input)?;

        if rule_result.confidence >= 0.7 {
            self.db.save_judgment(&rule_result)?;
            return Ok(rule_result);
        }

        // 2. LLM 보완
        let llm_result = self.llm_engine.evaluate(&input).await?;

        // 3. 결과 결합
        let final_result = self.combine_results(rule_result, llm_result);
        self.db.save_judgment(&final_result)?;

        Ok(final_result)
    }

    fn combine_results(
        &self,
        rule: JudgmentResult,
        llm: JudgmentResult,
    ) -> JudgmentResult {
        if llm.confidence > rule.confidence {
            JudgmentResult {
                id: Uuid::new_v4().to_string(),
                method_used: "hybrid".to_string(),
                explanation: format!(
                    "Rule 판단 (신뢰도 {:.1}%): {}\nLLM 판단 (신뢰도 {:.1}%): {}",
                    rule.confidence * 100.0,
                    rule.explanation,
                    llm.confidence * 100.0,
                    llm.explanation
                ),
                ..llm
            }
        } else {
            rule
        }
    }

    pub async fn get_history(
        &self,
        workflow_id: Option<String>,
        limit: u32,
    ) -> anyhow::Result<Vec<JudgmentResult>> {
        self.db.get_judgment_history(workflow_id, limit)
    }
}
```

#### 2.2 Rule Engine (rhai 기반)
```rust
// src-tauri/src/services/rule_engine.rs

use rhai::{Engine, Scope};

pub struct RuleEngine {
    engine: Engine,
}

impl RuleEngine {
    pub fn new() -> anyhow::Result<Self> {
        let mut engine = Engine::new();
        engine.set_max_operations(10000); // DOS 방지

        Ok(Self { engine })
    }

    pub fn evaluate(&self, input: &crate::services::judgment_engine::JudgmentInput)
        -> anyhow::Result<crate::services::judgment_engine::JudgmentResult> {

        let workflow = self.get_workflow(&input.workflow_id)?;

        let mut scope = Scope::new();

        // input_data를 rhai 변수로 등록
        if let Some(obj) = input.input_data.as_object() {
            for (key, value) in obj {
                scope.push(key.clone(), value.clone());
            }
        }

        // Rule 실행
        let result: bool = self.engine.eval_with_scope(
            &mut scope,
            &workflow.rule_expression,
        )?;

        Ok(crate::services::judgment_engine::JudgmentResult {
            id: uuid::Uuid::new_v4().to_string(),
            workflow_id: input.workflow_id.clone(),
            result,
            confidence: 0.9, // Rule Engine은 높은 신뢰도
            method_used: "rule".to_string(),
            explanation: format!("Rule: {} → {}", workflow.rule_expression, result),
        })
    }

    fn get_workflow(&self, workflow_id: &str) -> anyhow::Result<Workflow> {
        // DB에서 워크플로우 조회
        todo!("DB 조회 구현")
    }
}

struct Workflow {
    rule_expression: String,
}
```

#### 2.3 LLM Engine (OpenAI)
```rust
// src-tauri/src/services/llm_engine.rs

use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
}

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

pub struct LLMEngine {
    client: Client,
    api_key: String,
}

impl LLMEngine {
    pub fn new() -> anyhow::Result<Self> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .or_else(|_| Self::load_from_config())?;

        Ok(Self {
            client: Client::new(),
            api_key,
        })
    }

    pub async fn evaluate(
        &self,
        input: &crate::services::judgment_engine::JudgmentInput,
    ) -> anyhow::Result<crate::services::judgment_engine::JudgmentResult> {
        let prompt = self.build_prompt(input)?;

        let request = OpenAIRequest {
            model: "gpt-4".to_string(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: "당신은 제조 품질 판단 전문가입니다. 주어진 데이터를 분석하여 합격/불합격을 판단하고 상세한 이유를 설명하세요.".to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: prompt,
                },
            ],
            temperature: 0.3,
        };

        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?
            .json::<OpenAIResponse>()
            .await?;

        let llm_response = &response.choices[0].message.content;

        // LLM 응답 파싱 (예: "판단: 불합격\n이유: 온도가 임계값을 초과했습니다.")
        let (result, explanation) = self.parse_llm_response(llm_response)?;

        Ok(crate::services::judgment_engine::JudgmentResult {
            id: uuid::Uuid::new_v4().to_string(),
            workflow_id: input.workflow_id.clone(),
            result,
            confidence: 0.8, // LLM은 중간 신뢰도
            method_used: "llm".to_string(),
            explanation,
        })
    }

    fn build_prompt(&self, input: &crate::services::judgment_engine::JudgmentInput)
        -> anyhow::Result<String> {
        Ok(format!(
            "다음 데이터를 분석하여 합격/불합격을 판단하세요:\n\n{}",
            serde_json::to_string_pretty(&input.input_data)?
        ))
    }

    fn parse_llm_response(&self, response: &str) -> anyhow::Result<(bool, String)> {
        // 간단한 파싱 (실제로는 더 정교하게 구현)
        let result = response.contains("합격") && !response.contains("불합격");
        Ok((result, response.to_string()))
    }

    fn load_from_config() -> anyhow::Result<String> {
        // 설정 파일에서 API 키 로드
        todo!("설정 파일 구현")
    }
}
```

### Phase 3: React Frontend (Week 4-6)

**파일**: `src/pages/ChatInterface.tsx`

#### 3.1 Chat Interface
```typescript
// src/pages/ChatInterface.tsx

import { useState } from 'react'
import { invoke } from '@tauri-apps/api/tauri'
import { Send } from 'lucide-react'

interface Message {
  id: string
  role: 'user' | 'assistant'
  content: string
  timestamp: Date
}

export default function ChatInterface() {
  const [messages, setMessages] = useState<Message[]>([])
  const [input, setInput] = useState('')
  const [isLoading, setIsLoading] = useState(false)

  const sendMessage = async () => {
    if (!input.trim()) return

    const userMessage: Message = {
      id: Date.now().toString(),
      role: 'user',
      content: input,
      timestamp: new Date(),
    }

    setMessages((prev) => [...prev, userMessage])
    setInput('')
    setIsLoading(true)

    try {
      const response = await invoke<string>('send_chat_message', {
        message: input,
      })

      const assistantMessage: Message = {
        id: (Date.now() + 1).toString(),
        role: 'assistant',
        content: response,
        timestamp: new Date(),
      }

      setMessages((prev) => [...prev, assistantMessage])
    } catch (error) {
      console.error('Chat error:', error)
    } finally {
      setIsLoading(false)
    }
  }

  return (
    <div className="flex flex-col h-full">
      <div className="flex-1 overflow-auto p-4 space-y-4">
        {messages.map((message) => (
          <div
            key={message.id}
            className={`flex ${
              message.role === 'user' ? 'justify-end' : 'justify-start'
            }`}
          >
            <div
              className={`max-w-[70%] rounded-lg p-4 ${
                message.role === 'user'
                  ? 'bg-primary text-primary-foreground'
                  : 'bg-muted'
              }`}
            >
              <p className="text-sm">{message.content}</p>
            </div>
          </div>
        ))}
        {isLoading && (
          <div className="flex justify-start">
            <div className="bg-muted rounded-lg p-4">
              <div className="flex space-x-2">
                <div className="w-2 h-2 bg-gray-500 rounded-full animate-bounce" />
                <div className="w-2 h-2 bg-gray-500 rounded-full animate-bounce delay-75" />
                <div className="w-2 h-2 bg-gray-500 rounded-full animate-bounce delay-150" />
              </div>
            </div>
          </div>
        )}
      </div>

      <div className="border-t p-4">
        <div className="flex space-x-2">
          <input
            type="text"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyPress={(e) => e.key === 'Enter' && sendMessage()}
            placeholder="메시지를 입력하세요..."
            className="flex-1 px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-primary"
          />
          <button
            onClick={sendMessage}
            disabled={isLoading || !input.trim()}
            className="px-4 py-2 bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 disabled:opacity-50"
          >
            <Send className="w-5 h-5" />
          </button>
        </div>
      </div>
    </div>
  )
}
```

#### 3.2 Dashboard
```typescript
// src/pages/Dashboard.tsx

import { useQuery } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api/tauri'
import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from 'recharts'

interface DashboardStats {
  total_judgments: number
  success_rate: number
  avg_confidence: number
  recent_judgments: Array<{
    id: string
    workflow_id: string
    result: boolean
    confidence: number
    created_at: string
  }>
}

export default function Dashboard() {
  const { data, isLoading } = useQuery({
    queryKey: ['dashboard-stats'],
    queryFn: async () => {
      return await invoke<DashboardStats>('get_dashboard_stats')
    },
    refetchInterval: 30000, // 30초마다 갱신
  })

  if (isLoading) {
    return <div>Loading...</div>
  }

  return (
    <div className="space-y-6">
      <h1 className="text-3xl font-bold">Dashboard</h1>

      {/* KPI 카드 */}
      <div className="grid grid-cols-3 gap-4">
        <div className="bg-card p-6 rounded-lg border">
          <h3 className="text-sm font-medium text-muted-foreground">Total Judgments</h3>
          <p className="text-3xl font-bold mt-2">{data?.total_judgments}</p>
        </div>
        <div className="bg-card p-6 rounded-lg border">
          <h3 className="text-sm font-medium text-muted-foreground">Success Rate</h3>
          <p className="text-3xl font-bold mt-2">{data?.success_rate.toFixed(1)}%</p>
        </div>
        <div className="bg-card p-6 rounded-lg border">
          <h3 className="text-sm font-medium text-muted-foreground">Avg Confidence</h3>
          <p className="text-3xl font-bold mt-2">{data?.avg_confidence.toFixed(1)}%</p>
        </div>
      </div>

      {/* 차트 */}
      <div className="bg-card p-6 rounded-lg border">
        <h2 className="text-xl font-semibold mb-4">최근 판단 결과</h2>
        <ResponsiveContainer width="100%" height={300}>
          <BarChart data={data?.recent_judgments}>
            <CartesianGrid strokeDasharray="3 3" />
            <XAxis dataKey="created_at" />
            <YAxis />
            <Tooltip />
            <Bar dataKey="confidence" fill="#3b82f6" />
          </BarChart>
        </ResponsiveContainer>
      </div>
    </div>
  )
}
```

---

## 🏃 다음 단계

### 1. 백엔드 개발자가 할 일 (우선순위 순서)

1. **데이터베이스 구현** (1일)
   - `src-tauri/src/database/sqlite.rs` 완성
   - `src-tauri/migrations/001_init.sql` 작성

2. **Judgment Engine** (2일)
   - `src-tauri/src/services/judgment_engine.rs` 구현
   - `src-tauri/src/services/rule_engine.rs` 구현
   - `src-tauri/src/services/llm_engine.rs` 구현

3. **나머지 Commands** (1일)
   - `learning.rs`, `bi.rs`, `chat.rs`, `workflow.rs`, `system.rs`

4. **나머지 Services** (2일)
   - Learning Service, BI Service, Workflow Service

### 2. 프론트엔드 개발자가 할 일

1. **shadcn/ui 컴포넌트 설치**
   ```bash
   npx shadcn-ui@latest init
   npx shadcn-ui@latest add button
   npx shadcn-ui@latest add card
   npx shadcn-ui@latest add input
   # ... 필요한 컴포넌트 추가
   ```

2. **페이지 구현**
   - `WorkflowBuilder.tsx` - React Flow 통합
   - `BiInsights.tsx` - 동적 차트 생성
   - `Settings.tsx` - 설정 관리

3. **컴포넌트 구현**
   - `Sidebar.tsx`, `Header.tsx` (레이아웃)
   - 차트 컴포넌트들

### 3. 빌드 및 배포

```bash
# Windows 실행 파일 빌드
pnpm tauri build

# 생성 파일:
# src-tauri/target/release/judgify-desktop.exe
# src-tauri/target/release/bundle/msi/Judgify_2.0.0_x64.msi
```

---

## 📚 참고 자료

### Tauri 문서
- https://tauri.app/v1/guides/
- https://tauri.app/v1/api/rust/

### Rust 학습
- https://doc.rust-lang.org/book/
- https://rust-lang.github.io/async-book/

### React + TypeScript
- https://react.dev/learn
- https://www.typescriptlang.org/docs/

### shadcn/ui
- https://ui.shadcn.com/docs

---

## 🐛 트러블슈팅

### 문제: Rust 컴파일 에러
```bash
# Cargo.lock 삭제 후 재빌드
rm src-tauri/Cargo.lock
cd src-tauri && cargo build
```

### 문제: Tauri 개발 서버 실행 안됨
```bash
# 포트 충돌 확인
netstat -ano | findstr :1420

# 프로세스 종료 후 재시작
pnpm tauri dev
```

### 문제: WebView2 관련 에러
```
WebView2 Runtime 설치:
https://developer.microsoft.com/en-us/microsoft-edge/webview2/
```

---

## ✅ 개발 체크리스트

- [x] 개발 계획서 작성
- [x] 프로젝트 기본 구조 생성
- [ ] 데이터베이스 레이어 구현
- [ ] Judgment Engine 구현
- [ ] Learning Service 구현
- [ ] BI Service 구현
- [ ] Chat Interface 구현
- [ ] Workflow Builder 구현
- [ ] Dashboard 구현
- [ ] Settings 구현
- [ ] 테스트 작성
- [ ] Windows Installer 생성
- [ ] 사용자 매뉴얼 작성

---

**마지막 업데이트**: 2025-01-16
**개발 진행률**: 30%
**예상 완성일**: 8주 후

**다음 우선순위**: Rust 백엔드 서비스 구현 → Frontend 페이지 구현 → 테스트 → 배포
