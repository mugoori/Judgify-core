# MES RAG API 에러 디버깅 가이드

## 🔥 문제 상황
"판단 실행 실패: Claude API 에러 (400): 알 수 없는 오류"

## 📝 적용된 수정사항

### 1. LLM Engine 에러 처리 개선 (llm_engine.rs)
- API 에러 응답 본문을 읽어서 상세 정보 제공
- 콘솔에 에러 로그 상세 출력
- 400 에러시 실제 에러 메시지 표시

### 2. 프롬프트 최적화 (mes_data_service.rs)
- JSON 데이터에서 중요 필드만 추출 (온도, 습도, 진동, 압력, 판정, 설비)
- 컨텍스트 길이를 4000자로 제한
- 프롬프트 구조 간소화
- 프롬프트 길이를 콘솔에 로그 출력

## 🔍 디버깅 방법

### Step 1: 콘솔 로그 확인
터미널에서 다음과 같은 로그를 확인:

```
[MES RAG] 🔍 검색어: '온도 OR 90'
[MES RAG] 🔍 검색 결과: 5 행
[MES RAG] 📝 프롬프트 길이: 1234 문자
[LLM] ❌ API 에러 400: {"error": {"type": "invalid_request_error", "message": "..."}}
```

### Step 2: 가능한 400 에러 원인

1. **프롬프트 너무 김**
   - 현재 4000자로 제한했지만 여전히 길 수 있음
   - 해결: top_k 값을 3으로 줄이기

2. **잘못된 문자 포함**
   - 특수문자나 이스케이프 문자 문제
   - 해결: JSON 직렬화 확인

3. **API 버전 불일치**
   - anthropic-version 헤더 확인
   - 현재: "2023-06-01"

## 🧪 테스트 체크리스트

### 1. 간단한 질문으로 테스트
```
"온도 데이터 보여줘"  (짧은 질문)
```

### 2. topK 파라미터 조정
ChatInterface.tsx에서:
```typescript
const mesResult = await invoke<MesQueryResult>('query_mes_data', {
  sessionId: mesSessionId,
  question: input,
  topK: 3,  // 5에서 3으로 줄이기
});
```

### 3. API 키 재확인
Settings에서:
- API 키가 "sk-ant-"로 시작하는지 확인
- 공백이나 줄바꿈이 없는지 확인

## 📊 예상 로그 패턴

### 정상 작동시:
```
[MES RAG] 🔍 검색어: '온도'
[MES RAG] 🔍 검색 결과: 3 행
[MES RAG] 📝 프롬프트 길이: 800 문자
[MES RAG] ✅ LLM 답변 생성 완료
```

### 에러 발생시 (개선된 로그):
```
[MES RAG] 🔍 검색어: '온도'
[MES RAG] 🔍 검색 결과: 5 행
[MES RAG] 📝 프롬프트 길이: 5000 문자
[LLM] ❌ API 에러 400: {"error": {"type": "invalid_request_error", "message": "messages: text content blocks have a maximum size of 20000 characters"}}
```

## 🔧 추가 수정 필요시

### Option 1: topK 기본값 줄이기
mes.rs에서:
```rust
#[tauri::command]
pub async fn query_mes_data(
    session_id: String,
    question: String,
    top_k: Option<usize>,  // Option으로 변경
) -> Result<MesQueryResult, String> {
    let top_k = top_k.unwrap_or(3);  // 기본값 3
    // ...
}
```

### Option 2: 프롬프트 더 줄이기
mes_data_service.rs에서:
```rust
// 컨텍스트 길이 제한을 2000자로
let limited_context = if context.len() > 2000 {
    format!("{}...", &context[..2000])
} else {
    context.clone()
};
```

## 💡 즉시 시도해볼 것

1. **앱 재시작**
```bash
Ctrl+C (종료)
npm run tauri:dev (재시작)
```

2. **콘솔 로그 모니터링**
- 터미널에서 에러 상세 정보 확인
- 프롬프트 길이 확인

3. **간단한 질문부터 테스트**
- "데이터 보여줘"
- "온도 데이터"
- 점진적으로 복잡한 질문 시도

---
작성일: 2025-11-19