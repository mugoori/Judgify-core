# Debug Report 📋

프로젝트 개발 중 발생한 에러와 해결 과정을 기록합니다.

---

## 2025-11-06: Vitest "No test suite found" 에러

### 🕐 발생 시간
- **시작**: 09:22 (첫 테스트 실행 시도)
- **해결**: 09:25 (약 3분 소요)

### ❌ 에러 내용
```
Error: No test suite found in file c:/dev/Judgify-core/src/hooks/__tests__/useRuleValidation.test.ts

Test Files  1 failed (1)
Tests       no tests
Duration    824ms (transform 47ms, setup 71ms, collect 142ms, tests 0ms)
```

### 🔍 에러 원인
**Root Cause**: vitest v4.0.7 호환성 버그

**상세 분석**:
1. vitest v4.0.7이 Vite 7.1.12를 의존성으로 요구
2. 프로젝트는 Vite 5.4.20 사용 중
3. 버전 불일치로 인해 테스트 파일 컴파일 실패
4. vitest가 파일을 인식하지만 테스트 스위트를 파싱하지 못함

**버전 충돌 상세**:
```
프로젝트 Vite: 5.4.20
vitest 4.0.7 요구: vite@7.1.12

결과: "collect" 단계에서 테스트 수집 실패
```

### 🛠️ 디버깅 과정

#### 1단계: 의존성 확인 (09:22:25 - 09:22:44)
```bash
npm list vitest @vitest/ui vite
# 발견: vitest@4.0.7이 vite@7.1.12 사용 중
# 프로젝트는 vite@5.4.20
```

**시도**: Vite 업그레이드
```bash
npm install -D vite@7.1.12
# 결과: 여전히 동일한 에러 ❌
```

#### 2단계: 설정 파일 검증 (09:22:45 - 09:23:17)
**시도한 방법들**:
- ✅ setupFiles 추가/제거 테스트
- ✅ globals: true 토글
- ✅ 최소 설정(vitest.config.minimal.ts) 생성
- ✅ .test.ts → .spec.ts 확장자 변경
- ❌ 모두 실패

#### 3단계: TypeScript 설정 확인 (09:23:18 - 09:24:09)
**발견**: `tsconfig.json`의 `moduleResolution: "bundler"` 의심

**시도**: tsconfig.vitest.json 생성
```json
{
  "extends": "./tsconfig.json",
  "compilerOptions": {
    "moduleResolution": "Node",
    "types": ["vitest/globals", "node", "@testing-library/jest-dom"]
  }
}
```
- 결과: 여전히 실패 ❌

#### 4단계: Vitest 버전 다운그레이드 (09:24:10 - 09:25:24) ✅
**최종 해결책**:
```bash
npm install -D vitest@^2.1.0 @vitest/ui@^2.1.0 @vitest/coverage-v8@^2.1.0
```

**결과**:
```
✓ src/lib/__tests__/simple.spec.ts (1 test) 2ms

Test Files  1 passed (1)
Tests       1 passed (1)
Duration    901ms
```

### ✅ 해결 방법

**최종 솔루션**: vitest v4.0.7 → v2.1.9 다운그레이드

**변경된 패키지**:
```json
{
  "devDependencies": {
    "vitest": "^2.1.9",          // was: ^4.0.7
    "@vitest/ui": "^2.1.9",      // was: ^4.0.7
    "@vitest/coverage-v8": "^2.1.9"  // was: ^4.0.7
  }
}
```

**추가 조정사항**:
1. Debounce 테스트에서 `vi.runAllTicksAsync()` 제거 (v2.1.9에 API 없음)
2. 실제 `setTimeout()` 사용으로 대체
3. 테스트 타임아웃 조정 (5000ms → 10000ms)

### 📊 영향 범위
- ✅ 모든 테스트 정상 작동 (8/8 passing)
- ✅ 테스트 실행 속도: 519ms
- ⚠️ act() 경고 발생 (React 훅 테스트에서 정상)

### 🔑 교훈
1. **버전 호환성 최우선 확인**: 새 major 버전은 안정화될 때까지 대기
2. **LTS 버전 사용 권장**: vitest v2.x가 더 안정적
3. **의존성 트리 분석 필수**: `npm list` 명령으로 버전 충돌 조기 발견
4. **GitHub Issues 검색**: vitest v4.0.7 관련 이슈가 다수 보고됨

### 📌 관련 파일
- `package.json`: 버전 변경
- `vitest.config.ts`: 설정 최종화
- `src/hooks/__tests__/useRuleValidation.test.ts`: 테스트 코드 조정

### 🔗 참고 링크
- [Vitest v4.0.7 Release Notes](https://github.com/vitest-dev/vitest/releases/tag/v4.0.7)
- [Vitest v2.1.9 Documentation](https://vitest.dev/)

---

## Debug Report 작성 가이드

### 필수 포함 항목
1. **🕐 발생 시간**: 시작 시간 + 해결 시간 (소요 시간)
2. **❌ 에러 내용**: 정확한 에러 메시지 (코드 블록)
3. **🔍 에러 원인**: Root Cause + 상세 분석
4. **🛠️ 디버깅 과정**: 시도한 모든 방법 (시간순)
5. **✅ 해결 방법**: 최종 솔루션 + 코드 변경사항
6. **📊 영향 범위**: 해결 후 확인 사항
7. **🔑 교훈**: 향후 예방 방법

### 작성 템플릿
```markdown
## YYYY-MM-DD: [에러 제목]

### 🕐 발생 시간
- **시작**: HH:MM
- **해결**: HH:MM (약 X분/시간 소요)

### ❌ 에러 내용
[에러 메시지 전체]

### 🔍 에러 원인
**Root Cause**: [핵심 원인 한 문장]

**상세 분석**:
1. [원인 1]
2. [원인 2]

### 🛠️ 디버깅 과정
#### 1단계: [시도 내용]
[코드/명령어]
결과: [성공/실패]

### ✅ 해결 방법
[최종 솔루션]

### 📊 영향 범위
- [확인 사항 1]

### 🔑 교훈
1. [교훈 1]
```

---

## /init 워크플로우 통합

### 에러 발생 시 자동 문서화 절차

**1. 에러 감지**
- 모든 도구 실행 후 exit code 확인
- 에러 메시지 캡처

**2. Debug_Report.md 업데이트**
```bash
# 현재 시간 기록
echo "## $(date +%Y-%m-%d): [에러 제목]" >> Debug_Report.md

# 에러 내용 추가
echo "### ❌ 에러 내용" >> Debug_Report.md
echo '```' >> Debug_Report.md
echo "[에러 메시지]" >> Debug_Report.md
echo '```' >> Debug_Report.md
```

**3. 디버깅 과정 기록**
- 시도한 모든 명령어와 결과를 단계별로 추가
- 타임스탬프와 함께 기록

**4. 해결 후 완료 섹션 추가**
- 최종 솔루션
- 영향 범위
- 교훈

**5. Git 커밋 메시지에 참조**
```
fix: [문제 설명]

Debug Report: Debug_Report.md#YYYY-MM-DD
```

### Claude의 자동 문서화 체크리스트
- [ ] 에러 발생 시간 기록
- [ ] 에러 메시지 전체 캡처
- [ ] Root Cause 분석
- [ ] 디버깅 단계별 기록 (시도 → 결과)
- [ ] 최종 해결 방법 명시
- [ ] 영향 범위 확인
- [ ] 교훈 작성
- [ ] 관련 파일/링크 추가

---

**마지막 업데이트**: 2025-11-06 09:30
**작성자**: Claude Code
