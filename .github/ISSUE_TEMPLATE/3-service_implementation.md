---
name: 🏗 Service Implementation
about: 마이크로서비스 개발 및 구현 추적용 이슈
title: '[SERVICE] '
labels: ['type:service', 'status:planning']
assignees: ''
---

## 🏗 서비스 정보
**서비스명**:
**포트**:
**담당 에이전트**: (예: ai-engineer, mlops-engineer)
**의존 서비스**:

## 🎯 서비스 목표
이 서비스가 담당할 핵심 책임과 기능을 설명해주세요.

## 📋 구현 체크리스트

### Phase 1: 설계
- [ ] API 명세 작성 (OpenAPI/Swagger)
- [ ] 데이터베이스 스키마 설계
- [ ] 서비스 간 통신 인터페이스 정의
- [ ] 에러 처리 전략 수립
- [ ] 성능 목표 설정

### Phase 2: 핵심 기능 개발
- [ ] FastAPI 애플리케이션 구조 생성
- [ ] 데이터베이스 모델 구현 (SQLAlchemy)
- [ ] API 엔드포인트 구현
- [ ] 비즈니스 로직 구현
- [ ] 의존성 주입 설정

### Phase 3: 연동 및 테스트
- [ ] 다른 서비스와 통합 테스트
- [ ] 유닛 테스트 작성 (pytest)
- [ ] E2E 테스트 작성 (Playwright)
- [ ] 성능 테스트
- [ ] 보안 검증

### Phase 4: 배포 준비
- [ ] Dockerfile 작성
- [ ] docker-compose 설정
- [ ] Kubernetes 배포 설정 (deployment.yaml)
- [ ] 환경 변수 설정
- [ ] 헬스체크 엔드포인트 구현

### Phase 5: 모니터링 및 문서화
- [ ] Prometheus 메트릭 추가
- [ ] 로깅 구조화 (structured logging)
- [ ] API 문서 자동 생성
- [ ] 서비스 README 작성
- [ ] 운영 가이드 작성

## 🔧 기술 스택
```yaml
Backend:
  - Framework: FastAPI
  - Language: Python 3.11+
  - Database: PostgreSQL 15+ / pgvector
  - Cache: Redis 7.0+
  - Queue: Celery (해당시)

Testing:
  - pytest
  - pytest-asyncio
  - Playwright

Deployment:
  - Docker
  - Kubernetes
```

## 📊 API 엔드포인트 목록
| 메서드 | 경로 | 설명 | 상태 |
|--------|------|------|------|
| GET | /health | 헬스체크 | ⬜ TODO |
| POST | /api/v2/... | ... | ⬜ TODO |

## 🗄 데이터베이스 테이블
### 테이블 1: table_name
```sql
CREATE TABLE table_name (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- 컬럼 정의
);
```

## 🔗 서비스 의존성
**의존하는 서비스**:
- Service A (포트 8xxx): 용도 설명
- Service B (포트 8xxx): 용도 설명

**이 서비스를 사용하는 서비스**:
- Service C (포트 8xxx): 용도 설명

## 📈 성능 목표
- **응답 시간**: < XXX ms (95 percentile)
- **처리량**: XXX requests/sec
- **동시 연결**: XXX connections
- **에러율**: < 0.1%

## 🎨 주요 알고리즘 (해당시)
### 알고리즘 1
```python
def algorithm_name(input_data):
    """
    알고리즘 설명
    """
    pass
```

## 🔐 보안 고려사항
- [ ] JWT 인증 통합
- [ ] RBAC 권한 체크
- [ ] 입력 검증 (Pydantic)
- [ ] SQL Injection 방지
- [ ] Rate Limiting

## 📚 참고 문서
- 설계 문서: `docs/services/service_name.md`
- 아키텍처: `docs/architecture/system_overview.md`
- 알고리즘: `docs/algorithms/algorithm_name.md`

## 🎯 완료 기준 (Definition of Done)
- [ ] 모든 유닛 테스트 통과 (커버리지 > 80%)
- [ ] E2E 테스트 통과
- [ ] 코드 리뷰 완료
- [ ] API 문서 자동 생성 확인
- [ ] Docker 이미지 빌드 성공
- [ ] 로컬 환경 정상 작동 확인
- [ ] 성능 목표 달성

## 🏷 라벨
해당하는 항목에 체크해주세요:
- [ ] service:api-gateway (8000)
- [ ] service:workflow (8001)
- [ ] service:judgment (8002)
- [ ] service:action (8003)
- [ ] service:notification (8004)
- [ ] service:logging (8005)
- [ ] service:data-viz (8006)
- [ ] service:bi (8007)
- [ ] service:chat (8008)
- [ ] service:learning (8009)

## 🚀 마일스톤
- [ ] v2.0.0-alpha
- [ ] v2.0.0-beta
- [ ] v2.0.0-rc
- [ ] v2.0.0

## 📝 추가 노트
기타 구현시 유의사항이나 참고사항을 적어주세요.
