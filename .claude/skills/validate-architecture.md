---
name: validate-architecture
description: Validate Judgify-core Ver2.0 architecture rules and microservices compliance
---

Validate that the codebase follows Judgify-core Ver2.0 architecture rules and best practices.

## 🎯 언제 사용하나요?

### ✅ 사용 조건
- 새로운 서비스나 기능을 추가한 후 검증할 때
- 아키텍처 규칙 준수 여부를 자동으로 체크할 때
- 코드 리뷰 전 사전 검증이 필요할 때
- 9개 마이크로서비스 구조를 유지하고 싶을 때

### ❌ 사용하지 말아야 할 경우
- 복잡한 아키텍처 설계 검토 → **Task tool로 ai-engineer Agent 사용**
- 보안 취약점 분석 → **Task tool로 security-engineer Agent 사용**
- 성능 병목 분석 → **performance-engineer Agent 사용**

---

## 📋 사용 방법

```bash
/validate-architecture
/validate-architecture service-name  # 특정 서비스만 검증
```

**예시:**
```bash
/validate-architecture
/validate-architecture judgment-service
```

---

## 🔧 검증 규칙 (Ver2.0 Final)

### 1. 마이크로서비스 구조 규칙

```yaml
✅ 필수 요구사항:
  - 9개 서비스 유지 (8000-8009 포트)
  - 각 서비스는 독립적으로 배포 가능
  - FastAPI 프레임워크 사용
  - PostgreSQL + pgvector 데이터베이스
  - Redis 캐싱

❌ 금지 사항:
  - 서비스 간 직접 데이터베이스 접근 (API Gateway 경유 필수)
  - eval() 함수 사용 (AST 기반 Rule Engine 필수)
  - 하드코딩된 비밀번호/API 키
```

### 2. 디렉토리 구조 검증

```
services/{service-name}/
├── app/
│   ├── main.py          ✅ 필수
│   ├── config.py        ✅ 필수
│   ├── dependencies.py  ✅ 필수
│   ├── models/          ✅ 필수
│   ├── routers/         ✅ 필수
│   ├── services/        ✅ 필수
│   └── utils/           ✅ 필수
├── tests/               ✅ 필수 (커버리지 90% 이상)
├── Dockerfile           ✅ 필수
├── requirements.txt     ✅ 필수
└── README.md            ✅ 필수
```

### 3. API 설계 규칙

```yaml
엔드포인트 패턴:
  ✅ /api/v2/{service}/{resource}
  ❌ /v2/api/{service}  # 잘못된 패턴

HTTP 메서드:
  ✅ GET, POST, PUT, DELETE (표준 CRUD)
  ❌ PATCH (일관성 유지를 위해 PUT 사용)

응답 코드:
  ✅ 201 (Created), 200 (OK), 204 (No Content), 404 (Not Found), 422 (Validation Error)
  ❌ 임의의 커스텀 코드

인증:
  ✅ JWT Bearer 토큰 (API Gateway)
  ❌ Basic Auth, API Key in URL
```

### 4. Judgment Service 특화 규칙

```yaml
하이브리드 판단 로직:
  ✅ Rule Engine 우선 실행
  ✅ 신뢰도 >= 0.7 체크
  ✅ LLM 보완 실행 (필요시)
  ❌ LLM만 단독 사용

Rule Engine:
  ✅ AST 기반 파싱
  ❌ eval() 사용 (보안 위협)
  ❌ exec() 사용 (보안 위협)
```

### 5. Learning Service 특화 규칙 (ML 대체)

```yaml
Few-shot 학습:
  ✅ pgvector 유사도 검색
  ✅ 10-20개 유사 샘플 사용
  ✅ 최소 정확도 0.8 샘플만 사용
  ❌ 모든 학습 데이터 무차별 사용

자동 Rule 추출:
  ✅ 3개 알고리즘 (빈도 분석 + 결정 트리 + LLM)
  ✅ 최적 알고리즘 자동 선택
  ❌ 단일 알고리즘만 사용
```

### 6. 데이터베이스 규칙

```yaml
테이블 설계:
  ✅ UUID 기본 키 사용
  ✅ created_at, updated_at 타임스탬프
  ✅ pgvector 임베딩 컬럼 (VECTOR(1536))
  ❌ Integer 자동 증가 ID (보안 취약)

쿼리 최적화:
  ✅ 인덱스 사용
  ✅ 페이지네이션 (limit/offset)
  ❌ SELECT * (필요한 컬럼만 조회)
```

---

## 🔍 검증 결과 예시

```bash
$ /validate-architecture

🔍 Validating Judgify-core Ver2.0 architecture...

✅ Microservices Structure
  ✅ 9 services found (8000-8009)
  ✅ All services use FastAPI
  ✅ PostgreSQL + Redis configured

✅ Directory Structure
  ✅ judgment-service: All required files present
  ✅ learning-service: All required files present
  ⚠️  workflow-service: Missing tests/test_integration.py

✅ API Design
  ✅ All endpoints follow /api/v2/{service}/{resource} pattern
  ✅ JWT authentication configured
  ❌ payment-service: Found PATCH method (use PUT instead)

✅ Security
  ✅ No eval() or exec() usage found
  ✅ No hardcoded secrets
  ✅ AST-based Rule Engine implemented

⚠️  Test Coverage
  ✅ judgment-service: 95% coverage
  ✅ learning-service: 92% coverage
  ❌ workflow-service: 78% coverage (target: 90%)

📊 Summary:
  ✅ Passed: 45 rules
  ⚠️  Warnings: 3 rules
  ❌ Failed: 2 rules

🔧 Recommendations:
  1. workflow-service: Add tests/test_integration.py
  2. payment-service: Change PATCH to PUT method
  3. workflow-service: Increase test coverage to 90%
```

---

## 📋 검증 체크리스트

| 카테고리 | 규칙 수 | 자동 검증 | 수동 확인 |
|----------|---------|----------|----------|
| **마이크로서비스 구조** | 12 | ✅ | - |
| **디렉토리 구조** | 8 | ✅ | - |
| **API 설계** | 15 | ✅ | 비즈니스 로직 |
| **보안** | 10 | ✅ | - |
| **데이터베이스** | 8 | ✅ | 쿼리 성능 |
| **테스트 커버리지** | 3 | ✅ | 테스트 품질 |
| **Judgment 특화** | 6 | ✅ | - |
| **Learning 특화** | 5 | ✅ | - |
| **문서화** | 3 | ✅ | 문서 정확성 |

**총 규칙**: 70개

---

## 🚀 다음 단계 추천

검증 완료 후:

1. **경고 수정**: ⚠️ 표시된 항목 우선 수정
2. **실패 항목 수정**: ❌ 표시된 항목 필수 수정
3. **테스트 실행**: `/generate-tests` Skill로 부족한 테스트 추가
4. **재검증**: 수정 후 다시 `/validate-architecture` 실행
5. **코드 리뷰**: Agent에게 전문 검토 요청 (ai-engineer, security-engineer)

---

## 💡 주의사항

- **자동 검증 한계**: 비즈니스 로직의 정확성은 수동 확인 필요
- **테스트 커버리지**: 90% 이상 목표 (단, 품질도 중요)
- **보안 검증**: 자동 검증 + security-engineer Agent 검토 병행
- **성능 최적화**: 별도로 `/run-load-test` Skill 사용 권장

---

## 🔗 관련 리소스

- **Agent 활용**: ai-engineer (아키텍처 설계), security-engineer (보안 검토)
- **다음 Skill**: `/generate-tests`, `/run-load-test`, `/sync-docs`
- **문서**:
  - [CLAUDE.md](../../CLAUDE.md) - 아키텍처 규칙
  - [docs/architecture/system_overview.md](../../docs/architecture/system_overview.md)
  - [docs/development/implementation_plan.md](../../docs/development/implementation_plan.md)
