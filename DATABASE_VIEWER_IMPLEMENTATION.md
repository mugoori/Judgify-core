# Database Viewer 기능 구현 완료

## 📋 구현 내용

사용자 요청에 따라 Database Viewer 기능을 구현했습니다. 기존의 기능이 없는 "DB 연결", "LLM 활성" 상태 뱃지를 제거하고, 대신 데이터베이스 뷰어 버튼을 추가했습니다.

## 🚀 주요 기능

### 1. Database Viewer 버튼 (헤더)
- **위치**: 헤더 상단 좌측 (TriFlow AI Desktop 로고 옆)
- **디자인**: 데이터베이스 아이콘과 "데이터베이스 뷰어" 텍스트
- **동작**: 클릭 시 모달 창 열림

### 2. Database Viewer 모달
- **테이블 목록**: 좌측 사이드바에 모든 테이블 목록 표시
  - 각 테이블의 행 개수 표시
  - 클릭 시 테이블 확장하여 컬럼 정보 표시
  - 테이블 검색 기능

- **데이터 표시**: 우측 메인 영역
  - 선택한 테이블의 데이터를 표 형식으로 표시
  - 페이지네이션 (50개씩)
  - NULL 값 특별 표시
  - CSV 내보내기 기능

- **사용자 정의 쿼리**:
  - SELECT 쿼리 직접 실행 가능
  - 쿼리 결과를 표 형식으로 표시

### 3. 채팅 인터페이스 통합
- MES RAG 응답에 테이블 데이터 자동 표시
- 데이터가 포함된 응답은 인라인 테이블로 렌더링

## 📁 수정/생성된 파일

### Backend (Rust/Tauri)
- `src-tauri/src/commands/database.rs` - 새로 생성
  - `get_database_tables()`: 테이블 목록 조회
  - `query_table_data()`: 테이블 데이터 조회 (페이징 지원)
  - `execute_custom_query()`: 사용자 정의 쿼리 실행

- `src-tauri/src/commands/mod.rs` - 수정
  - database 모듈 추가

- `src-tauri/src/main.rs` - 수정
  - database 명령어 등록

### Frontend (React/TypeScript)
- `src/components/DatabaseViewer.tsx` - 새로 생성
  - 완전한 Database Viewer 컴포넌트
  - 테이블 목록, 데이터 표시, 쿼리 실행 기능

- `src/components/layout/Header.tsx` - 수정
  - 기존 상태 뱃지 제거
  - 데이터베이스 뷰어 버튼 추가

- `src/pages/ChatInterface.tsx` - 수정
  - Message 인터페이스에 tableData 필드 추가
  - MessageBubble 컴포넌트에 테이블 렌더링 기능 추가
  - MES RAG 응답 파싱하여 테이블 데이터 추출

## 🔒 보안 고려사항

### SQL Injection 방지
- 테이블명 화이트리스트 검증
- 사용자 정의 쿼리는 SELECT만 허용
- prepared statement 사용

### 데이터 접근 제어
- 허용된 테이블만 조회 가능
- 시스템 테이블 및 FTS 테이블 제외
- 읽기 전용 작업만 허용

## 📊 지원 테이블 목록

```
- ccp_docs
- ccp_sensor_logs
- mes_data_logs
- mes_data_fts
- chat_history
- workflows
- workflow_executions
- external_integrations
- judgment_history
- bi_dashboards
- bi_widgets
- bi_insights
- notifications
```

## 💡 사용 방법

1. **Database Viewer 열기**:
   - 헤더의 "데이터베이스 뷰어" 버튼 클릭

2. **테이블 데이터 보기**:
   - 좌측 테이블 목록에서 원하는 테이블 클릭
   - 데이터가 자동으로 우측에 표시됨
   - 페이지 네비게이션으로 추가 데이터 확인

3. **CSV 내보내기**:
   - 테이블 선택 후 "CSV 내보내기" 버튼 클릭
   - 자동으로 다운로드됨

4. **사용자 정의 쿼리**:
   - "사용자 정의 쿼리" 버튼 클릭
   - SELECT 쿼리 입력
   - "쿼리 실행" 버튼 클릭

## 🎯 향후 개선사항

- [ ] 실시간 데이터 업데이트 (WebSocket)
- [ ] 고급 필터링 기능
- [ ] 데이터 편집 기능 (관리자 권한)
- [ ] 쿼리 히스토리 저장
- [ ] 더 많은 데이터 내보내기 형식 (Excel, JSON)

## ✅ 완료 시간
2025-11-20 10:35 KST