---
name: sync-docs
description: Synchronize API documentation (OpenAPI/Swagger) across services and update README files
---

Synchronize API documentation automatically across all microservices and update README files.

## 🎯 언제 사용하나요?

### ✅ 사용 조건
- API 엔드포인트를 추가/수정한 후
- OpenAPI 스펙을 최신 상태로 유지하고 싶을 때
- README 파일을 자동으로 업데이트하고 싶을 때
- 서비스 간 API 문서 일관성을 유지할 때

### ❌ 사용하지 말아야 할 경우
- 복잡한 기술 문서 작성 → **Task tool로 technical-writer Agent 사용**
- 아키텍처 설계 문서 작성 → **ai-engineer, graphql-architect Agent 사용**
- 사용자 매뉴얼 작성 → **customer-support Agent 사용**

---

## 📋 사용 방법

```bash
/sync-docs service-name
```

**예시:**
```bash
/sync-docs judgment-service
/sync-docs all  # 모든 서비스 동기화
```

---

## 🔧 동기화 작업 내용

### 1. OpenAPI 스펙 생성 (자동)

FastAPI의 자동 문서 생성 기능 활용:

```python
# app/main.py
from fastapi import FastAPI

app = FastAPI(
    title="Judgment Service API",
    version="2.0.0",
    description="Judgify-core Ver2.0 Judgment Service",
    openapi_tags=[
        {
            "name": "judgment",
            "description": "하이브리드 판단 엔진 (Rule + LLM)"
        },
        {
            "name": "learning",
            "description": "자동학습 및 Few-shot 관리"
        }
    ]
)

# OpenAPI JSON 파일 저장
import json
from pathlib import Path

@app.on_event("startup")
async def save_openapi_spec():
    openapi_spec = app.openapi()
    spec_path = Path("docs/openapi.json")
    spec_path.parent.mkdir(exist_ok=True)
    spec_path.write_text(json.dumps(openapi_spec, indent=2))
```

### 2. README.md 업데이트

서비스별 README 자동 생성:

```markdown
# {Service Name}

**포트**: {port}
**버전**: 2.0.0
**상태**: ✅ Active

## 📋 개요

{Service description}

## 🚀 API 엔드포인트

### {Category 1}

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| POST | /api/v2/{service}/execute | Execute judgment | JWT |
| GET | /api/v2/{service}/history | Get execution history | JWT |

### {Category 2}

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| ... | ... | ... | ... |

## 🔧 환경 변수

```env
DATABASE_URL=postgresql://...
REDIS_URL=redis://...
OPENAI_API_KEY=sk-...
LOG_LEVEL=INFO
```

## 📊 의존성

- FastAPI 0.104.1
- SQLAlchemy 2.0.23
- Redis 5.0.1
- OpenAI 1.3.0

## 🧪 테스트

```bash
pytest tests/ -v --cov
```

## 📖 API 문서

- Swagger UI: http://localhost:{port}/docs
- ReDoc: http://localhost:{port}/redoc
- OpenAPI JSON: http://localhost:{port}/openapi.json

## 🔗 관련 서비스

- API Gateway (8000)
- Workflow Service (8001)
- Learning Service (8009)
```

### 3. 중앙 API 문서 업데이트

`docs/architecture/api_specifications.md` 업데이트:

```markdown
## {Service Name} API (Port {port})

### POST /api/v2/{service}/{endpoint}

**요청:**
```json
{
  "field1": "value1",
  "field2": "value2"
}
```

**응답:** (201 Created)
```json
{
  "id": "uuid",
  "field1": "value1",
  "created_at": "2024-01-22T10:00:00Z"
}
```

**에러:**
- 400: 잘못된 요청
- 401: 인증 실패
- 422: 검증 실패
```

### 4. Postman 컬렉션 생성

```json
{
  "info": {
    "name": "Judgify-core Ver2.0 - {Service}",
    "schema": "https://schema.getpostman.com/json/collection/v2.1.0/collection.json"
  },
  "item": [
    {
      "name": "{Endpoint}",
      "request": {
        "method": "POST",
        "header": [
          {
            "key": "Authorization",
            "value": "Bearer {{jwt_token}}"
          }
        ],
        "url": "{{base_url}}/api/v2/{service}/{endpoint}"
      }
    }
  ]
}
```

---

## 📊 동기화 체크리스트

| 항목 | 자동화 | 수동 확인 |
|------|--------|----------|
| ✅ OpenAPI JSON 생성 | ✅ | - |
| ✅ Swagger UI 업데이트 | ✅ | - |
| ✅ 서비스 README.md | ✅ | 비즈니스 설명 |
| ✅ 중앙 API 문서 | ✅ | 예제 정확성 |
| ✅ Postman 컬렉션 | ✅ | 테스트 실행 |
| ⚠️ 아키텍처 다이어그램 | ❌ | 수동 업데이트 필요 |

---

## 🚀 실행 결과 예시

```bash
$ /sync-docs judgment-service

🔄 Synchronizing documentation for judgment-service...

✅ OpenAPI spec generated: services/judgment-service/docs/openapi.json
✅ README.md updated: services/judgment-service/README.md
✅ Central API docs updated: docs/architecture/api_specifications.md
✅ Postman collection created: postman/judgment-service.json

📊 Summary:
- API endpoints documented: 12
- New endpoints added: 3
- Updated endpoints: 2
- Deprecated endpoints: 1

🌐 View documentation:
- Swagger UI: http://localhost:8002/docs
- ReDoc: http://localhost:8002/redoc
```

---

## 🚀 다음 단계 추천

문서 동기화 후:

1. **문서 검증**: Swagger UI에서 "Try it out" 기능으로 API 테스트
2. **Postman 테스트**: 생성된 컬렉션으로 E2E 테스트
3. **아키텍처 검증**: `/validate-architecture` Skill 실행
4. **배포 준비**: 문서 확인 후 Docker 이미지 빌드
5. **팀 공유**: Postman 컬렉션을 팀원과 공유

---

## 💡 주의사항

- **자동 생성 제한**: 비즈니스 설명은 수동 작성 필요
- **버전 관리**: API 버전 변경시 OpenAPI spec 버전도 업데이트
- **보안**: Postman 컬렉션에 실제 API 키 포함 금지
- **예제 데이터**: 실제 프로덕션 데이터 사용 금지

---

## 🔗 관련 리소스

- **Agent 활용**: technical-writer (복잡한 문서), customer-support (사용자 가이드)
- **다음 Skill**: `/validate-architecture`, `/run-load-test`
- **문서**: [docs/architecture/api_specifications.md](../../docs/architecture/api_specifications.md)
- **도구**: Swagger UI, ReDoc, Postman
