# Ver2.0 Final 구현 계획 통합 문서

> **통합 문서**: 요구사항 + MVP 로드맵 + 작업 분해 + 기술 분석
> **작성일**: 2025-01-21
> **버전**: Ver2.0 Final
> **상태**: 최종 확정

이 문서는 Judgify-core Ver2.0 Final의 전체 구현 계획을 통합하여 관리합니다.

---

## 📑 문서 구성

1. [요구사항 명확화](#1-요구사항-명확화) - 핵심 결정사항 및 구현 세부사항
2. [MVP 로드맵](#2-mvp-로드맵) - 10주 개발 일정 및 단계별 전략
3. [작업 분해](#3-작업-분해) - 약 105개 실행 가능한 태스크 목록
4. [기술 분석](#4-기술-분석) - 아키텍처/성능/보안/위험 평가

**빠른 참조**:
- 개발 우선순위: [섹션 1.7](#17-개발-우선순위-및-mvp-범위)
- 10주 일정: [섹션 2.2](#22-week-by-week-일정)
- 작업 통계: [섹션 3.1](#31-작업-통계)
- 위험 요소: [섹션 4.4](#44-위험-요소-및-대응-전략)

---

# 1. 요구사항 명확화

## 1.1 Learning Service (8009) 자동학습 시스템

### 1.1.1 Few-shot 샘플 개수 전략
**결정사항**: **동적 조정 방식**

실행 흐름:
1. 입력 데이터 복잡도 계산 (변수 개수, 타입 다양성, 중첩 깊이)
2. 복잡도 점수 기반 샘플 개수 결정:
   - 복잡도 < 0.3 → 10개 (단순 케이스)
   - 복잡도 0.3~0.7 → 15개 (보통 케이스)
   - 복잡도 > 0.7 → 20개 (복잡한 케이스)
3. pgvector 유사도 검색으로 최적 샘플 선택

**이점**:
- LLM 토큰 최적화 (단순한 케이스는 10개만 사용)
- 복잡한 케이스는 충분한 컨텍스트 제공 (20개)
- 정확도와 비용의 균형

---

### 1.1.2 자동 Rule 추출 알고리즘 전략
**결정사항**: **3개 알고리즘 동시 실행 후 최고 신뢰도 선택**

알고리즘:
1. **빈도 분석 (Frequency Analysis)**: 공통 패턴의 출현 빈도 기반
2. **결정 트리 학습 (Decision Tree)**: sklearn 기반 조건식 생성
3. **LLM 패턴 발견**: LLM이 데이터에서 패턴 추출

실행 방식:
- 3개 알고리즘 병렬 실행 (asyncio.gather)
- 각 알고리즘의 신뢰도 점수 비교
- 최고 신뢰도 Rule 자동 선택
- 모든 결과 로깅 (비교 분석용)

**비용 최적화**:
- Redis 캐싱: 동일 피드백 데이터 재추출 방지
- 배치 처리: 주기적 실행 (매일 1회)

---

### 1.1.3 사용자 피드백 수집 UI
**결정사항**: **판단 직후 팝업 + Chat Interface 메시지 옆**

옵션 1: 판단 결과 직후 팝업 (높은 응답률)
- 자동 표시, 3초 후 자동 닫힘
- 👍 정확해요 / 👎 틀렸어요 / 🤷 잘 모르겠어요 / 건너뛰기
- 선택적 코멘트 필드

옵션 2: Chat Interface 메시지 옆 (자연스러움)
- 각 판단 결과 메시지에 👍👎 버튼
- 중복 피드백 방지 (feedbackStatus)

**구현 우선순위**:
1. Phase 1: 판단 직후 팝업
2. Phase 2: Chat Interface 피드백 추가

---

## 1.2 데이터 집계 알고리즘 (할루시네이션 방지)

### 1.2.1 통계 집계 임계값 기준
**결정사항**: **워크플로우별 사용자 정의**

기본 임계값 (제조업 표준):
- **온도**: 정상 < 80°C, 경고 80-90°C, 위험 > 90°C
- **진동**: 정상 < 40Hz, 경고 40-50Hz, 위험 > 50Hz
- **불량률**: 정상 < 3%, 경고 3-5%, 위험 > 5%

사용자 커스터마이징:
- Workflow Editor에서 임계값 설정 UI 제공
- 워크플로우별 독립적인 임계값 설정
- workflows 테이블의 aggregation_thresholds 필드

---

### 1.2.2 데이터 집계 주기
**결정사항**: **1일 1회 (매일 자정) + 수동 트리거 옵션**

자동 실행:
- Celery beat 스케줄러: 매일 자정 (crontab: hour=0, minute=0)
- 모든 워크플로우 자동 집계

수동 트리거:
- API 엔드포인트: POST /api/v2/learning/aggregate-data
- 파라미터: workflow_id (선택), time_range (last_7_days, last_30_days)

집계 결과 저장:
- aggregated_data 테이블에 저장
- 90일 이후 판단 데이터는 집계 형태로만 유지
- 원본 데이터는 archived_judgments 테이블로 이동

---

## 1.3 MCP 통합 전략

### 1.3.1 PostgreSQL MCP 서버 설치 시점
**결정사항**: **지금 바로 설치**

필수 MCP 서버 설치 순서:
1. **PostgreSQL MCP** (최우선) - 데이터베이스 직접 연결
2. **GitHub MCP** (이미 설치됨) - 코드 관리
3. **Memory MCP** (Chat Interface용) - 컨텍스트 유지
4. **Filesystem MCP** (기본 제공) - 파일 관리

.mcp.json 설정:
```json
{
  "mcpServers": {
    "postgresql": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-postgres"],
      "env": {
        "DATABASE_URL": "postgresql://judgify:password@localhost:5432/judgify_core"
      }
    }
  }
}
```

---

### 1.3.2 Memory MCP 서버 컨텍스트 유지 기간
**결정사항**: **24시간 유지 + 중요 컨텍스트는 PostgreSQL에 별도 저장**

Memory MCP 설정:
- context_ttl: 86400 (24시간)
- max_contexts: 1000
- cleanup_interval: 3600 (1시간마다 정리)

중요 컨텍스트 영구 저장 (PostgreSQL):
- 워크플로우 생성 대화
- 복잡한 BI 분석 요청
- 사용자 선호도 설정
- expires_at: NULL (영구) 또는 +7일

---

## 1.4 Visual Workflow Builder

### 1.4.1 n8n 스타일 노드 타입 (7가지)

#### 1. Trigger 노드
트리거 유형:
- **REST API**: endpoint, method, headers
- **Schedule**: cron 표현식, timezone
- **Webhook**: url, secret
- **Sensor**: sensorId, pollingInterval

#### 2. Condition 노드
조건 유형:
- **IF-ELSE**: condition, trueOutput, falseOutput
- **Switch-Case**: variable, cases[], defaultOutput

#### 3. Judgment 노드
판단 방식:
- **rule_only**: ruleExpression
- **llm_only**: llmPrompt, fewShotEnabled
- **hybrid**: rulePriority, confidenceThreshold (0.7)

#### 4. Action 노드
액션 유형:
- **Slack**: channel, message
- **MCP**: system, command, parameters
- **Webhook**: url, method, body
- **Email**: to[], subject, body

#### 5. Data Transform 노드
변환 유형:
- **Map**: inputField, outputField, transformation
- **Filter**: condition
- **Aggregate**: groupBy[], aggregations[]

#### 6. Loop 노드
루프 유형:
- **For Each**: arrayField, iterateOutput, completeOutput
- **While**: condition, maxIterations
- **Until**: condition, maxIterations

#### 7. Merge 노드
병합 유형:
- **Wait All**: 모든 입력 대기 (timeout)
- **First**: 첫 번째 입력만 사용
- **Any**: 어떤 입력이든 도착하면 진행

---

## 1.5 Chat Interface Service

### 1.5.1 MCP 서버 상태 표시 방법
**결정사항**: **MCP ping 방식 (정확성 우선)**

상태 확인 프로세스:
1. MCP ping 명령 전송
2. 응답 시간 측정
3. 버전 정보 확인
4. 상태 반환 (connected / disconnected / error)

상태 정보:
- server_name: 서버명
- status: 연결 상태
- last_ping: 마지막 핑 시간
- response_time_ms: 응답 시간
- version: 서버 버전
- error_message: 오류 메시지 (있을 경우)

Settings UI 구현:
- 실시간 상태 아이콘 (🟢🔴🟡)
- 응답 시간 표시 ("45ms")
- 재연결 버튼
- 연결 테스트 버튼
- 로그 확인 버튼

---

## 1.6 기술 스택 및 아키텍처 결정사항

### 1.6.1 백엔드 기술 스택
- **Framework**: FastAPI (Python 3.11+)
- **Database**: PostgreSQL 15+ with pgvector
- **Cache**: Redis 7.0+
- **Message Queue**: Celery with Redis broker
- **API Gateway**: Kong/Nginx + JWT

### 1.6.2 프론트엔드 기술 스택
- **Framework**: Next.js 14 + TypeScript
- **Visual Editor**: React Flow (n8n 스타일)
- **UI Library**: Tailwind CSS + shadcn/ui

### 1.6.3 인프라 기술 스택
- **Containerization**: Docker
- **Orchestration**: Kubernetes
- **Monitoring**: Prometheus + Grafana
- **Logging**: ELK Stack
- **CI/CD**: GitHub Actions + ArgoCD

---

## 1.7 개발 우선순위 및 MVP 범위

### Phase 1 (MVP) - Week 1-6
**핵심 4개 서비스**:
1. ✅ **API Gateway (8000)** - 인증/라우팅
2. 🔥 **Judgment Service (8002)** - 하이브리드 판단 엔진 (최우선!)
3. ⭐ **Learning Service (8009)** - 자동학습 시스템 (혁신!)
4. 🎨 **BI Service (8007)** - MCP 컴포넌트 조립

**목표**:
- 하이브리드 판단 엔진 작동
- 자동학습 시스템 검증
- MCP 기반 BI 생성 데모

**성공 지표**:
- 판단 정확도 90% 이상
- Few-shot 학습 효과 +15%p 정확도 향상
- Rule 자동 추출 성공률 80% 이상

### Phase 2 (확장) - Week 7-8
**나머지 6개 서비스**:
5. **Workflow Service (8001)** - Visual Workflow Builder
6. **Chat Interface Service (8008)** - 통합 AI 어시스턴트
7. **Data Visualization Service (8006)** - 단순 대시보드
8. **Action Service (8003)** - 외부 시스템 연동
9. **Notification Service (8004)** - 알림 발송
10. **Logging Service (8005)** - 중앙 로그 관리

**목표**:
- 전체 9개 서비스 통합
- Visual Workflow Builder 사용성 검증
- Chat Interface 마스터 컨트롤러 작동

**성공 지표**:
- 9개 서비스 정상 작동
- E2E 테스트 통과율 95% 이상
- 사용자 만족도 4.5/5 이상

---

# 2. MVP 로드맵

## 2.1 전체 일정 개요

```
Week 1-2: 인프라 및 설계 (Phase 0)
Week 3-6: Phase 1 MVP (핵심 4개 서비스)
Week 7-8: Phase 2 확장 (나머지 6개 서비스)
Week 9-10: 통합 테스트 및 배포 (Phase 3)
```

---

## 2.2 Week-by-Week 일정

### Week 1-2: 인프라 및 핵심 설계 (Phase 0)

#### Week 1: 개발 환경 구축
**Day 1-2: 프로젝트 초기화**
- Git 저장소 초기화
- 프로젝트 디렉토리 구조 생성
- Docker Compose 개발 환경 설정
- PostgreSQL + pgvector 설치
- Redis 설치

**Day 3-4: MCP 서버 설치 및 테스트**
- PostgreSQL MCP 서버 설치
- Memory MCP 서버 설치
- GitHub MCP 서버 연결 확인
- MCP 서버 상태 테스트 스크립트

**Day 5: 데이터베이스 스키마 생성**
- PostgreSQL 스키마 생성 (initial.md 기반)
- pgvector extension 활성화
- 테이블 생성 스크립트
- 초기 데이터 마이그레이션

#### Week 2: 핵심 서비스 설계
**Day 1-2: API Gateway 구현**
- Kong/Nginx 선택 및 설정
- JWT 인증 미들웨어 구현
- Rate Limiting 설정
- 라우팅 규칙 정의

**Day 3-4: Judgment Service 설계**
- AST Rule Engine 설계
- LLM Judgment Engine 설계
- Hybrid 전략 구현
- Connector 패턴 설계

**Day 5: Learning Service 설계**
- Few-shot 관리 시스템 설계
- Rule 추출 알고리즘 설계
- 피드백 수집 시스템 설계

---

### Week 3-6: Phase 1 MVP (핵심 4개 서비스)

#### Week 3: Judgment Service 구현
- AST 기반 Rule Engine 구현
- LLM Judgment Engine 구현
- Hybrid 판단 로직 구현
- RAG 기반 설명 생성

#### Week 4: Learning Service 구현
- Few-shot 샘플 관리 구현
- 빈도 분석 알고리즘 구현
- 결정 트리 학습 구현
- LLM 패턴 발견 구현

#### Week 5: BI Service 구현
- MCP Component Library 연동
- LLM 기반 컴포넌트 선택
- React 컴포넌트 조립
- WebSocket 실시간 데이터

#### Week 6: Phase 1 통합 테스트
- 4개 서비스 통합 테스트
- 하이브리드 판단 정확도 검증
- Few-shot 학습 효과 측정
- 성능 벤치마크

---

### Week 7-8: Phase 2 확장 (나머지 6개 서비스)

#### Week 7: Workflow + Chat Interface
- Visual Workflow Builder 구현 (n8n 스타일)
- Chat Interface Service 구현
- MCP 서버 상태 표시
- 멀티턴 대화 컨텍스트

#### Week 8: Data Viz + Action + Notification + Logging
- Data Visualization Service 구현
- Action Service 구현
- Notification Service 구현
- Logging Service 구현

---

### Week 9-10: 통합 테스트 및 배포 (Phase 3)

#### Week 9: E2E 테스트
- 전체 서비스 통합 테스트
- E2E 테스트 시나리오 실행
- 성능 최적화
- 보안 취약점 점검

#### Week 10: 프로덕션 배포
- Staging 환경 배포
- Smoke 테스트
- Production 환경 배포
- 모니터링 대시보드 구성

---

## 2.3 성공 지표

### Phase 1 MVP (Week 6 종료 시점)
- ✅ 하이브리드 판단 정확도 90% 이상
- ✅ Few-shot 학습 효과 +15%p 정확도 향상
- ✅ Rule 자동 추출 성공률 80% 이상
- ✅ BI 컴포넌트 조립 성공률 90% 이상

### Phase 2 전체 (Week 8 종료 시점)
- ✅ 9개 서비스 정상 작동
- ✅ E2E 테스트 통과율 95% 이상
- ✅ API 응답 시간 < 2초 (95 percentile)
- ✅ Visual Workflow Builder 사용성 4.5/5 이상

### 프로덕션 배포 (Week 10 종료 시점)
- ✅ 서비스 가용성 99.5% 이상
- ✅ 배포 성공률 99.9%
- ✅ 모니터링 대시보드 100% 작동
- ✅ 문서화 완료 100%

---

# 3. 작업 분해

## 3.1 작업 통계

| Phase | 작업 개수 | 예상 기간 | 우선순위 |
|-------|----------|----------|----------|
| **Phase 0: 인프라** | 15개 | Week 1-2 | P0 |
| **Phase 1: MVP** | 45개 | Week 3-6 | P0 |
| **Phase 2: 확장** | 30개 | Week 7-8 | P1 |
| **Phase 3: 배포** | 15개 | Week 9-10 | P0 |
| **총합** | **105개** | **10주** | - |

---

## 3.2 Phase 0: 인프라 구축 (Week 1-2) - 15개 작업

### Week 1: 프로젝트 초기화 (8개 작업)

**프로젝트 구조 생성 (3개)**
- ☐ Task 1.1: Git 저장소 초기화 및 .gitignore 설정 (30분)
- ☐ Task 1.2: 프로젝트 디렉토리 구조 생성 (1시간)
- ☐ Task 1.3: Docker Compose 개발 환경 설정 (2시간)

**데이터베이스 설정 (3개)**
- ☐ Task 1.4: PostgreSQL 15 + pgvector 설치 (1시간)
- ☐ Task 1.5: Redis 7.0 설치 및 설정 (30분)
- ☐ Task 1.6: 데이터베이스 스키마 생성 스크립트 (3시간)

**MCP 서버 설정 (2개)**
- ☐ Task 1.7: PostgreSQL/Memory/GitHub MCP 서버 설치 (2시간)
- ☐ Task 1.8: MCP 서버 연결 테스트 (1시간)

### Week 2: 핵심 서비스 설계 (7개 작업)

**API Gateway (2개)**
- ☐ Task 2.1: Kong/Nginx 선택 및 설정 (2시간)
- ☐ Task 2.2: JWT 인증 미들웨어 구현 (4시간)

**Judgment Service 설계 (3개)**
- ☐ Task 2.3: AST Rule Engine 설계 (4시간)
- ☐ Task 2.4: LLM Judgment Engine 설계 (4시간)
- ☐ Task 2.5: Hybrid 전략 구현 (6시간)

**Learning Service 설계 (2개)**
- ☐ Task 2.6: Few-shot 관리 시스템 설계 (4시간)
- ☐ Task 2.7: Rule 추출 알고리즘 설계 (6시간)

---

## 3.3 Phase 1: MVP 구현 (Week 3-6) - 45개 작업

### Week 3: Judgment Service (12개 작업)
- AST Rule Engine 구현
- LLM Judgment Engine 구현
- Hybrid 판단 로직 구현
- RAG 기반 설명 생성
- Connector 패턴 구현
- 유닛 테스트 (90% 커버리지)

### Week 4: Learning Service (12개 작업)
- Few-shot 샘플 관리 구현
- 빈도 분석 알고리즘 구현
- 결정 트리 학습 구현
- LLM 패턴 발견 구현
- 피드백 수집 시스템
- 유닛 테스트

### Week 5: BI Service (12개 작업)
- MCP Component Library 연동
- LLM 컴포넌트 선택 로직
- React 컴포넌트 조립
- WebSocket 실시간 연동
- 대시보드 생성 API
- 유닛 테스트

### Week 6: Phase 1 통합 테스트 (9개 작업)
- 4개 서비스 통합 테스트
- 하이브리드 판단 정확도 검증
- Few-shot 학습 효과 측정
- 성능 벤치마크
- 문서화

---

## 3.4 Phase 2: 확장 구현 (Week 7-8) - 30개 작업

### Week 7: Workflow + Chat Interface (15개 작업)
- Visual Workflow Builder 구현
- 7가지 노드 타입 구현
- Chat Interface Service 구현
- MCP 서버 상태 표시
- 멀티턴 대화 컨텍스트

### Week 8: Data Viz + 지원 서비스 (15개 작업)
- Data Visualization Service
- Action Service
- Notification Service
- Logging Service
- 통합 테스트

---

## 3.5 Phase 3: 배포 및 검증 (Week 9-10) - 15개 작업

### Week 9: E2E 테스트 (8개 작업)
- E2E 테스트 시나리오 작성
- Playwright 테스트 자동화
- 성능 최적화
- 보안 취약점 점검

### Week 10: 프로덕션 배포 (7개 작업)
- Staging 환경 배포
- Smoke 테스트
- Production 환경 배포
- 모니터링 대시보드
- 문서화 완료

---

# 4. 기술 분석

## 4.1 아키텍처 분석

### 마이크로서비스 복잡도
**위험 수준**: 🟡 중간

**분석 내용**:
- 9개 마이크로서비스 간 통신 복잡도
- 서비스 간 의존성 관리 필요
- 데이터 일관성 유지 과제

**대응 전략**:
- 명확한 책임 분리 (Single Responsibility)
- API Gateway를 통한 중앙집중식 라우팅
- 서비스 간 비동기 통신 (Message Queue)
- Circuit Breaker 패턴 적용

---

## 4.2 성능 분석

### LLM 응답 시간
**위험 수준**: 🟡 중간

**분석 내용**:
- OpenAI API 응답 시간: 평균 1~3초
- Few-shot 학습시 토큰 증가 → 비용 증가
- 동시 요청 처리 능력 제한

**대응 전략**:
- Redis 캐싱: 자주 사용되는 판단 결과 (TTL 5분)
- 배치 처리: 비실시간 판단은 Celery 큐
- Rule Engine 우선: 간단한 케이스는 LLM 스킵
- 토큰 최적화: 데이터 집계 알고리즘

### pgvector 검색 성능
**위험 수준**: 🟢 낮음

**분석 내용**:
- 벡터 검색 인덱스 필요 (HNSW 알고리즘)
- 대규모 데이터시 검색 속도 저하 가능

**대응 전략**:
- 적절한 인덱스 설정 (CREATE INDEX ON training_samples USING hnsw)
- 파티셔닝: 워크플로우별 테이블 분리
- 정기적인 VACUUM ANALYZE

---

## 4.3 보안 분석

### AST 기반 Rule Engine
**위험 수준**: 🟢 낮음

**분석 내용**:
- JavaScript eval() 완전 제거
- AST 파싱으로 안전한 조건식 평가
- SQL Injection 방지

**대응 전략**:
- Python ast 모듈 사용
- 허용된 연산자만 평가 (>, <, ==, and, or)
- 입력 검증 강화 (Pydantic 모델)

### JWT 인증
**위험 수준**: 🟢 낮음

**대응 전략**:
- JWT 토큰 만료 시간: 1시간
- Refresh Token: 7일
- RBAC (Role-Based Access Control)
- API Rate Limiting (100 req/min per user)

---

## 4.4 위험 요소 및 대응 전략

### 일정 지연 위험
**위험 수준**: 🟡 중간

**위험 요소**:
- Visual Workflow Builder 복잡도 과소평가
- LLM 성능 불안정성
- MCP 서버 연동 문제

**대응 전략**:
- 단계적 출시 (Phase 1 MVP 우선)
- Visual Builder 간소화 (Phase 2로 연기 가능)
- MCP 대안 준비 (REST API fallback)

### 비용 증가 위험
**위험 수준**: 🟡 중간

**위험 요소**:
- LLM API 호출 비용 증가
- PostgreSQL 스토리지 비용
- Redis 메모리 비용

**대응 전략**:
- 데이터 집계 알고리즘으로 토큰 최적화
- 90일 아카이빙 정책
- Redis 캐시 TTL 최적화 (5분)
- Rule Engine 우선 전략 (LLM 호출 최소화)

---

## 4.5 성능 목표

| 지표 | 목표 | 측정 방법 |
|------|------|----------|
| **판단 응답 시간** | 평균 2초, 95% < 5초 | Prometheus 메트릭 |
| **API 가용성** | 99.5% 이상 | Uptime monitor |
| **판단 정확도** | 90% 이상 | User feedback |
| **Few-shot 효과** | +15%p 정확도 향상 | A/B 테스트 |
| **Rule 추출 성공률** | 80% 이상 | Logging 분석 |

---

## 📚 관련 문서

### 아키텍처 문서
- [system_overview.md](../architecture/system_overview.md): 상세 아키텍처 설계
- [database_design.md](../architecture/database_design.md): PostgreSQL + pgvector 스키마
- [system_structure.md](../architecture/system_structure.md): 시스템 구조 개요

### 서비스별 설계 문서
- [judgment_engine.md](../services/judgment_engine.md): 판단 엔진 구현 명세
- [learning_service.md](../services/learning_service.md): 자동학습 서비스 설계
- [workflow_editor.md](../services/workflow_editor.md): Visual Workflow Builder
- [dashboard_service.md](../services/dashboard_service.md): BI + Data Visualization

### 알고리즘 설계
- [auto_rule_extraction.md](../algorithms/auto_rule_extraction.md): 자동 Rule 추출 알고리즘
- [data_aggregation.md](../algorithms/data_aggregation.md): 데이터 집계 알고리즘

### 운영 및 모니터링
- [monitoring_guide.md](../operations/monitoring_guide.md): 모니터링 및 운영 가이드
- [deployment_guide.md](../operations/deployment_guide.md): 배포 전략 및 런북

---

**최종 업데이트**: 2025-01-21
**문서 버전**: Ver2.0 Final
**통합 작성자**: Claude Code
