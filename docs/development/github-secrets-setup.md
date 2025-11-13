# GitHub Secrets 설정 가이드

**생성일**: 2025-11-05
**목적**: CI/CD 워크플로우에 필요한 GitHub Secrets 설정

---

## 📋 필수 Secrets 목록

### 1. Codecov Token

**Secret 이름**: `CODECOV_TOKEN`

**획득 방법**:
1. [Codecov](https://codecov.io) 로그인 (GitHub 계정 연동)
2. Repository 선택: `mugoori/Judgify-core`
3. Settings → Repository Upload Token 복사

**설정 경로**:
```
GitHub Repository → Settings → Secrets and variables → Actions → New repository secret
Name: CODECOV_TOKEN
Secret: [Codecov에서 복사한 토큰]
```

---

### 2. Tauri Signing Keys (선택 사항)

**Secret 이름**:
- `TAURI_PRIVATE_KEY`
- `TAURI_KEY_PASSWORD`

**획득 방법**:
```bash
# Tauri 키 생성 (필요시)
npm run tauri signer generate -- -w ~/.tauri/myapp.key
```

**설정 경로**:
```
GitHub Repository → Settings → Secrets and variables → Actions
Name: TAURI_PRIVATE_KEY
Secret: [생성된 private key 내용]

Name: TAURI_KEY_PASSWORD
Secret: [설정한 비밀번호]
```

**참고**: 현재 워크플로우에서는 `continue-on-error: true`로 설정되어 있어 키가 없어도 동작합니다.

---

## ✅ 설정 확인

### 1. Secrets 설정 확인
```
GitHub Repository → Settings → Secrets and variables → Actions
```

다음 Secret이 표시되어야 합니다:
- ✅ `CODECOV_TOKEN` (필수)
- ⏳ `TAURI_PRIVATE_KEY` (선택)
- ⏳ `TAURI_KEY_PASSWORD` (선택)

### 2. 워크플로우 실행 확인

**자동 트리거**:
- `git push` to `main` or `develop` 브랜치
- Pull Request 생성/업데이트

**수동 트리거**:
```
GitHub Actions 탭 → Test & Coverage 워크플로우 → Run workflow
```

### 3. 워크플로우 로그 확인

**성공 시**:
```
✅ Rust Tests & Coverage - 108 tests passed
✅ TypeScript Tests & Coverage - 0 tests (no unit tests yet)
✅ E2E Tests - 68 Playwright tests passed
✅ Coverage uploaded to Codecov
```

**실패 시 확인 사항**:
1. `CODECOV_TOKEN` Secret이 설정되었는지 확인
2. Codecov에서 Repository가 활성화되었는지 확인
3. 워크플로우 로그에서 상세 에러 메시지 확인

---

## 🔐 보안 권장사항

### 1. Secret 관리
- ❌ Secret을 코드에 하드코딩하지 말 것
- ❌ Secret을 로그에 출력하지 말 것
- ✅ GitHub Secrets 또는 환경 변수만 사용

### 2. Branch Protection
```
Settings → Branches → main → Add rule
- Require status checks to pass before merging
- Require branches to be up to date before merging
- Status checks required:
  ✅ Rust Tests & Coverage
  ✅ TypeScript Tests & Coverage
  ✅ E2E Tests
```

### 3. Codecov 설정
```yaml
# codecov.yml
coverage:
  status:
    project:
      default:
        threshold: 1%  # 1% 이상 감소시 실패
```

---

## 🚀 워크플로우 활용

### 1. PR 생성시 자동 테스트
```bash
git checkout -b feature/my-feature
git commit -m "feat: Add new feature"
git push origin feature/my-feature
gh pr create --title "feat: Add new feature"
```

→ GitHub Actions가 자동으로:
- Rust 테스트 (108 tests)
- TypeScript 테스트 (0 tests)
- E2E 테스트 (68 tests)
- 커버리지 측정 (Rust + TypeScript)
- Codecov 업로드
- PR 코멘트로 결과 게시

### 2. 커버리지 배지 확인
```markdown
README.md 상단:
[![Test & Coverage](badge...)](link)
[![codecov](badge...)](link)
[![Rust Coverage](48.31%)](link)
[![TypeScript Coverage](0%)](link)
```

### 3. Codecov 대시보드
- URL: https://codecov.io/gh/mugoori/Judgify-core
- 브랜치별 커버리지 추이
- 파일별 상세 커버리지
- PR별 커버리지 변화

---

## 📊 기준치 (Baseline)

**2025-11-05 기준**:
- **Rust**: 48.31% (1,402 / 2,902 lines)
- **TypeScript**: 0% (No unit tests, 68 E2E tests)

**목표 (Task 4.2)**:
- **Rust**: 75% (+26.69%p)
- **TypeScript**: 60% (+60%p)

---

## 🔧 트러블슈팅

### 문제 1: Codecov 업로드 실패
```
Error: Codecov token not found
```

**해결**:
1. `CODECOV_TOKEN` Secret 설정 확인
2. Codecov에서 Repository 활성화 확인

### 문제 2: E2E 테스트 타임아웃
```
Error: Timeout waiting for Tauri app
```

**해결**:
1. `timeout-minutes: 30` 증가 (워크플로우)
2. Playwright 브라우저 설치 확인
3. Tauri 빌드 캐시 확인

### 문제 3: 커버리지 threshold 실패
```
Error: Rust coverage decreased below baseline
```

**해결**:
1. 새 코드에 테스트 추가
2. 또는 `codecov.yml`에서 threshold 조정 (1% → 2%)

---

**관련 문서**:
- [GitHub Actions 워크플로우](.github/workflows/test.yml)
- [Codecov 설정](codecov.yml)
- [커버리지 베이스라인](docs/COVERAGE_BASELINE_2025-11-05.md)
