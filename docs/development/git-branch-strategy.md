# Git 브랜치 백업 전략

## 🎯 기본 원칙

개발 방향에 큰 변화가 있을 경우, **브랜치를 나눠서 백업 후 비교 테스트 수행**

이를 통해:
- ✅ 변경 전 상태 안전하게 보존
- ✅ 변경 후 상태와 객관적 비교
- ✅ 데이터 기반 의사결정 가능
- ✅ 언제든 이전 상태로 복구 가능

---

## 📋 브랜치 백업이 필요한 경우

### 1. 아키텍처 변경
- 마이크로서비스 개수/구조 변경 (예: 9개 → 12개)
- 핵심 기술 스택 교체 (예: PostgreSQL → MongoDB)
- 서비스 간 통신 방식 변경 (예: REST → gRPC)
- 데이터베이스 스키마 대규모 변경

**예시**:
```bash
# Analytics, Cache, Search Service 추가 (9개 → 12개)
git checkout -b architecture/12-microservices-upgrade
```

### 2. 컨텍스트 문서 대규모 수정
- **CLAUDE.md 200줄 이상 변경 예상**
- 개발 철학/패턴 근본적 변경
- 섹션 재구성 (예: 14개 → 10개)
- 코드 예제 → 의사코드 대량 변환

**예시**:
```bash
# CLAUDE.md를 의사코드 기반으로 전환
git checkout -b docs/claude-md-pseudocode-conversion
```

### 3. 개발 전략 변경
- AI 에이전트 구성 변경 (예: 18개 → 25개)
- MCP 도구 우선순위 재정의
- 개발 우선순위 변경 (예: Judgment Service → BI Service 우선)
- Phase 전환 (예: Phase 2 → Phase 3)

**예시**:
```bash
# AI 에이전트 7개 추가 (monitoring, cost-optimizer 등)
git checkout -b strategy/ai-agents-phase4-expansion
```

### 4. 기술적 전환
- 프론트엔드 프레임워크 변경 (예: React → Svelte)
- 데이터베이스 마이그레이션 (PostgreSQL → MongoDB)
- 배포 전략 변경 (Docker → Kubernetes Operator)
- 언어 전환 (Python → Rust)

**예시**:
```bash
# GraphQL Federation 도입 테스트
git checkout -b tech/graphql-federation-experiment
```

---

## 🔧 브랜치 백업 워크플로우 (5단계)

### 단계 1: 현재 상태 커밋
```bash
# 작업 중인 파일 모두 커밋
git add .
git commit -m "backup: 변경 전 현재 상태 백업

변경 예정 사항:
- [구체적 변경 내용 설명]

현재 상태:
- [현재 주요 지표]

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"
```

**체크리스트**:
- [ ] 모든 변경사항 커밋됨
- [ ] 커밋 메시지에 변경 예정 사항 명시
- [ ] 현재 주요 지표 기록 (파일 크기, 서비스 개수 등)

### 단계 2: 백업 브랜치 생성
```bash
# 명명 규칙: {카테고리}/{변경-내용}
git checkout -b {category}/{description}

# 실제 예시:
git checkout -b architecture/12-microservices-upgrade
git checkout -b docs/claude-md-refactor
git checkout -b tech/migrate-to-mongodb
git checkout -b strategy/ai-agents-reorganization
```

**브랜치명 규칙**:
- 소문자 사용
- 하이픈(`-`)으로 단어 구분
- 카테고리 접두사 필수 ([브랜치 명명 규칙](#📂-브랜치-명명-규칙) 참조)

### 단계 3: 변경 작업 수행
```bash
# 변경 작업 실행
# - 코드 수정
# - 문서 업데이트
# - 설정 변경
# 등...

# 변경 완료 후 커밋
git add .
git commit -m "feat/refactor/docs: 변경 내용 구체적 설명

변경 사항:
- [변경 1]
- [변경 2]
- [변경 3]

영향 범위:
- [영향받는 서비스/파일]

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"
```

**체크리스트**:
- [ ] 변경 사항 명확히 기록
- [ ] 영향 범위 명시
- [ ] 테스트 완료 (가능한 경우)

### 단계 4: 비교 보고서 작성
```bash
# 비교 보고서 파일 생성
# 파일명: COMPARISON_{category}_{YYYYMMDD}.md
# 위치: 프로젝트 루트

# 예시: COMPARISON_docs_20250121.md
```

**비교 보고서 필수 포함 사항**:
1. **Executive Summary**: 핵심 비교 결과 (표)
2. **정량적 비교**: 파일 크기, 성능, 복잡도 등
3. **질적 비교**: 가독성, 유지보수성, 확장성 등
4. **구체적 예시**: 변경 전/후 코드/문서 비교
5. **장단점 분석**: 각 버전의 pros & cons
6. **최종 권장사항**: 데이터 기반 추천 + 이유
7. **대안 시나리오**: 다른 선택이 합리적인 상황
8. **실행 가이드**: git 명령어

**템플릿**:
```markdown
# {변경 내용} 비교 분석 보고서

## 🎯 Executive Summary
| 항목 | 현재 (main) | 변경 후 (branch) | 차이 |
|------|------------|-----------------|------|
| ... | ... | ... | ... |

## 📊 정량적 비교
...

## 🎨 질적 비교
...

## 🏆 최종 권장사항
...
```

### 단계 5: 사용자 선택
```bash
# 선택지 A: 변경 사항 채택
git checkout main
git merge {category}/{description}
git branch -D {category}/{description}

# 선택지 B: 현재 상태 유지
git checkout main
git branch -D {category}/{description}
```

**의사결정 기준**:
- 정량적 지표 (파일 크기, 성능 등)
- 질적 평가 (가독성, 유지보수성 등)
- 장기적 영향 (확장성, 팀 협업 등)
- 비용/효과 분석

---

## 📂 브랜치 명명 규칙

### 카테고리 접두사

| 카테고리 | 용도 | 예시 |
|----------|------|------|
| `architecture/` | 시스템 아키텍처 변경 | `architecture/12-microservices-upgrade` |
| `docs/` | 문서 대규모 수정 | `docs/claude-md-pseudocode-conversion` |
| `tech/` | 기술 스택 변경 | `tech/migrate-postgresql-to-mongodb` |
| `strategy/` | 개발 전략 변경 | `strategy/ai-agents-phase4-expansion` |
| `refactor/` | 코드 리팩토링 | `refactor/judgment-engine-optimization` |
| `experiment/` | 실험적 변경 | `experiment/graphql-federation-test` |

### 명명 규칙 상세

**형식**: `{category}/{description}`

**description 작성 규칙**:
- 소문자 사용
- 하이픈(`-`)으로 단어 구분
- 숫자 포함 가능 (예: `12-microservices`)
- 동사 사용 (예: `upgrade`, `migrate`, `optimize`)
- 명확하고 구체적으로 (예: ❌ `docs/refactor` → ✅ `docs/claude-md-pseudocode-conversion`)

**예시**:
```
✅ 좋은 예:
architecture/12-microservices-upgrade
docs/claude-md-pseudocode-conversion
tech/migrate-postgresql-to-mongodb
strategy/ai-agents-phase4-expansion

❌ 나쁜 예:
arch/upgrade (불명확)
DocRefactor (카멜케이스 사용)
tech/migration (구체성 부족)
test (카테고리 없음)
```

---

## 🚨 주의사항

### 1. 백업 브랜치 유지 기간
- **채택 후**: 즉시 삭제
- **미채택**: 30일 후 삭제 (태그로 기록 남김)

```bash
# 미채택 브랜치 태그 저장
git tag archive/{category}/{description} {category}/{description}
git branch -D {category}/{description}

# 태그 목록 확인
git tag -l "archive/*"
```

### 2. 비교 보고서 필수
- 모든 백업 브랜치는 **비교 보고서 작성 의무**
- 파일명: `COMPARISON_{category}_{YYYYMMDD}.md`
- 위치: 프로젝트 루트
- 최소 5개 섹션 포함 (Executive Summary, 정량적 비교, 질적 비교, 권장사항, 실행 가이드)

### 3. main 브랜치 보호
- main 브랜치에서 **직접 대규모 변경 금지**
- 항상 백업 브랜치에서 작업 후 병합
- main 브랜치는 항상 안정적 상태 유지

### 4. 커밋 메시지 규칙
```bash
# 백업 커밋
backup: {설명}

# 변경 커밋
feat: {기능 추가}
refactor: {리팩토링}
docs: {문서 수정}

# 병합 커밋
merge: {브랜치명} - {결정 이유}
```

---

## 📊 실전 예시: CLAUDE.md Phase 3 최적화

### 배경

**날짜**: 2025-01-21
**목적**: CLAUDE.md 최적화 (코드 예제 → 의사코드)
**예상 변경**: 코드 블록 ~44개 → ~30개, 200줄 이상 수정

### 실행 과정

#### Step 1: 현재 상태 백업 (Phase 2)
```bash
git add CLAUDE.md
git commit -m "docs: CLAUDE.md Phase 2 optimization complete

Phase 2 최적화 완료:
- ✅ 문서 구조 업데이트
- ✅ Ver1.0 용어 제거
- ✅ 중복 제거 (~100줄 감소)
- ✅ Quick Reference 추가

최종 상태:
- 파일 크기: 1,282줄
- 섹션 수: 14개
- 코드 블록: ~44개 (Python/TypeScript 예제)

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"
```

#### Step 2: 백업 브랜치 생성
```bash
git checkout -b docs/claude-md-phase3-test
```

#### Step 3: 변경 작업
**변경 내용**:
1. 코드 예제 → 의사코드 변환 (섹션 2.1-2.4, 4.1-4.3)
2. 섹션 14 → 섹션 0 통합

```bash
# 변경 완료 후 커밋
git add CLAUDE.md
git commit -m "docs: CLAUDE.md Phase 3 optimization test

Phase 3 최적화 테스트:
- ✅ 코드 예제 → 의사코드 변환 (주요 섹션)
- ✅ 섹션 14 → 섹션 0 통합

최종 상태:
- 파일 크기: 1,189줄 (93줄 감소)
- 섹션 수: 13개
- 코드 블록: ~30개 (의사코드)

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"
```

#### Step 4: 비교 보고서 작성
**파일**: `PHASE_COMPARISON.md` (3,000단어, 8개 섹션)

**주요 내용**:
1. Executive Summary: 핵심 비교 표
2. 파일 크기 비교: 1,282줄 vs 1,189줄
3. 코드 예제 vs 의사코드 비교 (섹션 2.1, 2.3 예시)
4. 문서 구조 비교 (섹션 14 독립 vs 섹션 0 통합)
5. 정량적 비교: 9개 지표
6. 질적 비교: 가독성, 실용성, 유지보수성
7. 최종 권장사항: Phase 3 채택 (이유 5가지)
8. 실행 가이드: git 명령어

#### Step 5: 사용자 선택 → Phase 3 채택
```bash
git checkout main
git merge docs/claude-md-phase3-test
git branch -D docs/claude-md-phase3-test
```

### 결과

**정량적 개선**:
- 파일 크기: 1,282줄 → 1,189줄 (7.3% 감소)
- 코드 블록: ~44개 → ~30개 (30% 감소)
- 섹션 수: 14개 → 13개
- 2,500줄 임계값 여유: 1,218줄 → 1,311줄 (+93줄)

**질적 개선**:
- ✅ 언어 독립적 (TypeScript/Rust 개발자도 이해 가능)
- ✅ 유지보수 간편 (실제 코드 변경과 문서 독립적)
- ✅ 간결성 (핵심 로직만 표현)
- ✅ 구조 개선 (아키텍처 변경 요약을 섹션 0에 배치)

**의사결정 근거**:
- 비교 보고서의 정량적/질적 데이터 기반
- Phase 3 장점 6개 vs Phase 2 장점 3개
- 장기적 유지보수성 우선

---

## 🎯 향후 적용 시나리오

### 시나리오 1: 마이크로서비스 12개로 확장

**배경**: Analytics, Cache, Search Service 추가

**워크플로우**:
```bash
# 1. 백업 브랜치 생성
git checkout -b architecture/12-microservices-upgrade

# 2. 변경 작업
# - 3개 서비스 추가 (포트 8010, 8011, 8012)
# - CLAUDE.md 섹션 1 업데이트 (9개 → 12개 서비스 테이블)
# - docs/services/ 디렉토리에 3개 설계 문서 추가
# - docs/architecture/system_overview.md 업데이트

# 3. 비교 보고서 작성
# COMPARISON_architecture_20250201.md
# - 9개 vs 12개 서비스 복잡도 비교
# - 성능 영향 분석
# - 운영 비용 증가율
# - 개발 팀 확장 필요성

# 4. 사용자 선택
# → 복잡도 증가 대비 가치 검증 후 결정
```

**예상 영향**:
- 복잡도: +30% (서비스 간 통신 증가)
- 성능: +20% (전문화된 서비스로 최적화)
- 운영 비용: +15% (컨테이너 3개 추가)

### 시나리오 2: AI 에이전트 25개로 확장

**배경**: Phase 4 에이전트 7개 추가

**워크플로우**:
```bash
# 1. 백업 브랜치 생성
git checkout -b strategy/ai-agents-phase4-expansion

# 2. 변경 작업
# - 에이전트 7개 추가 정의
#   - monitoring-specialist
#   - cost-optimizer
#   - security-auditor
#   - compliance-checker
#   - api-designer
#   - documentation-expert
#   - onboarding-specialist
# - CLAUDE.md 섹션 6 업데이트 (18개 → 25개)
# - 서비스별 에이전트 매핑 재조정

# 3. 비교 보고서 작성
# COMPARISON_strategy_20250215.md
# - 18개 vs 25개 에이전트 효율성 비교
# - 역할 중복 분석
# - 협업 복잡도 증가율
# - 비용 대비 효과 (ROI)

# 4. 사용자 선택
# → 에이전트 효율성 데이터 기반 결정
```

**예상 영향**:
- 개발 속도: +25% (전문화된 에이전트)
- 협업 복잡도: +20% (커뮤니케이션 오버헤드)
- 품질: +30% (전문성 향상)

### 시나리오 3: GraphQL Federation 도입

**배경**: REST API → GraphQL Gateway 전환 실험

**워크플로우**:
```bash
# 1. 백업 브랜치 생성
git checkout -b tech/graphql-federation-experiment

# 2. 변경 작업
# - Apollo Federation 서버 추가
# - 각 마이크로서비스에 GraphQL 서브그래프 구현
# - CLAUDE.md 섹션 4.1 업데이트 (API 패턴 변경)
# - docs/architecture/api_specifications.md 대규모 수정

# 3. 비교 보고서 작성
# COMPARISON_tech_20250301.md
# - REST vs GraphQL 성능 비교
# - 개발자 경험 (DX) 개선 정도
# - 마이그레이션 비용
# - 프론트엔드 개발 효율성 향상

# 4. 사용자 선택
# → 성능 벤치마크 + DX 개선 데이터 기반 결정
```

**예상 영향**:
- API 응답 시간: -20% (Over-fetching 제거)
- 프론트엔드 개발 속도: +40% (단일 엔드포인트)
- 백엔드 복잡도: +30% (Federation 관리)
- 마이그레이션 비용: 2주 개발 시간

---

## 📚 참고 자료

### 관련 문서
- [CLAUDE.md](../../CLAUDE.md): 컨텍스트 개발 가이드
- [docs/development/plan.md](plan.md): Windows Desktop App 개발 계획
- [docs/architecture/system_overview.md](../architecture/system_overview.md): 시스템 아키텍처

### Git 명령어 치트시트
```bash
# 브랜치 목록 확인
git branch

# 현재 브랜치 확인
git branch --show-current

# 브랜치 생성 및 전환
git checkout -b {branch-name}

# 브랜치 병합
git merge {branch-name}

# 브랜치 삭제
git branch -D {branch-name}

# 태그 생성
git tag {tag-name} {branch-name}

# 태그 목록
git tag -l "archive/*"
```

### 비교 보고서 템플릿
[PHASE_COMPARISON.md](../../PHASE_COMPARISON.md) 참조

---

**이 전략을 지속적으로 적용하여 안전하고 데이터 기반의 개발 결정을 내리세요! 🚀**
