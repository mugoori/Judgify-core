# TriFlow AI 자주 묻는 질문 (FAQ)

**버전**: 0.1.8
**최종 업데이트**: 2025-11-13

---

## 📑 목차

1. [설치 및 실행](#1-설치-및-실행)
2. [API 키 설정](#2-api-키-설정)
3. [Chat Interface](#3-chat-interface)
4. [Workflow Builder](#4-workflow-builder)
5. [Dashboard](#5-dashboard)
6. [백업 및 복구](#6-백업-및-복구)
7. [성능 및 최적화](#7-성능-및-최적화)
8. [보안 및 프라이버시](#8-보안-및-프라이버시)
9. [에러 코드](#9-에러-코드)
10. [기타](#10-기타)

---

## 1. 설치 및 실행

### Q1: MSI와 NSIS 설치 파일 중 어느 것을 선택해야 하나요?

**A**: 사용 환경에 따라 선택하세요:

| 환경 | 권장 설치 파일 | 이유 |
|------|----------------|------|
| 기업/조직 | **MSI** | GPO 배포, 자동 업데이트 지원 |
| 개인 사용자 | **NSIS** | 더 작은 크기, 빠른 설치 |

두 파일 모두 **동일한 기능**을 제공하며, 설치 방식만 다릅니다.

---

### Q2: Windows 7에서 실행할 수 있나요?

**A**: 아니요, TriFlow AI는 **Windows 10 (64-bit) 이상**에서만 작동합니다.

**이유**:
- Tauri 프레임워크가 Windows 10+ WebView2 필요
- Modern Windows API 사용

**대안**:
- Windows 10 또는 Windows 11로 업그레이드
- 가상 머신 (VM)에서 Windows 10 실행

---

### Q3: 설치 후 앱이 실행되지 않습니다.

**A**: 다음 단계를 시도하세요:

**1단계: WebView2 설치 확인**
- TriFlow AI는 Microsoft WebView2가 필요합니다.
- 자동 설치 실패시 수동 설치:
  - [WebView2 다운로드](https://developer.microsoft.com/microsoft-edge/webview2/#download-section)
  - "Evergreen Standalone Installer" 선택

**2단계: 관리자 권한으로 실행**
- 바로가기 → 우클릭 → "관리자 권한으로 실행"

**3단계: 안티바이러스 예외 처리**
- 일부 안티바이러스가 TriFlow AI를 차단할 수 있음
- Windows Defender 예외 추가:
  ```
  C:\Program Files\TriFlow AI\
  ```

**4단계: 로그 확인**
- 로그 파일 확인: `C:\Users\[사용자]\AppData\Local\TriFlow AI\logs\error.log`
- 에러 메시지를 GitHub Issues에 첨부

---

### Q4: "Windows Defender SmartScreen" 경고가 나타납니다.

**A**: 이는 정상적인 동작입니다. 새 앱은 Windows에 인증 기록이 없어 경고가 표시됩니다.

**안전하게 실행하는 방법**:
1. "추가 정보" 링크 클릭
2. "실행" 버튼 클릭

**보안**:
- TriFlow AI는 오픈소스 프로젝트입니다: [GitHub](https://github.com/mugoori/Judgify-core)
- 코드 검증 가능
- 악성 코드 없음 보장

---

## 2. API 키 설정

### Q5: Anthropic API 키는 어디서 발급받나요?

**A**: Anthropic Console에서 발급받을 수 있습니다.

**단계**:
1. [https://console.anthropic.com/](https://console.anthropic.com/) 접속
2. 계정 생성 (Google/Email)
3. "API Keys" 메뉴로 이동
4. "Create Key" 버튼 클릭
5. 키 이름 입력 (예: "TriFlow Desktop")
6. API 키 복사 (sk-ant-api03-으로 시작)

**비용**:
- 무료 체험: $5 크레딧 제공 (신규 가입시)
- 유료 플랜: 사용량 기반 과금

---

### Q6: API 키를 입력했는데 "Invalid API key" 오류가 나옵니다.

**A**: 다음 사항을 확인하세요:

**1. 키 형식 확인**:
- ✅ 올바른 형식: `sk-ant-api03-XXXXXXXXXXXXXXXXXXXXXXXXXXXXXX`
- ❌ 잘못된 형식: 공백, 줄바꿈 포함

**2. 키 복사 확인**:
- Ctrl+C로 정확히 복사했는지 확인
- 앞뒤 공백 제거

**3. 키 유효성 확인**:
- Anthropic Console에서 키가 활성 상태인지 확인
- 키를 삭제했거나 만료되지 않았는지 확인

**4. 네트워크 연결**:
- 인터넷 연결 확인
- 방화벽에서 TriFlow AI 허용

---

### Q7: API 키를 변경하려면 어떻게 해야 하나요?

**A**: Settings 페이지에서 간단히 변경할 수 있습니다.

**단계**:
1. Settings 페이지로 이동
2. "Anthropic API Key" 입력 필드에 새 키 입력
3. "Save Settings" 클릭
4. 앱 재시작 **불필요** (즉시 적용)

**주의**:
- 이전 키는 자동으로 덮어씌워집니다.
- 이전 키는 복구할 수 없으므로, 필요시 별도 보관하세요.

---

### Q8: API 사용량은 어떻게 확인하나요?

**A**: Anthropic Console에서 확인할 수 있습니다.

**단계**:
1. [https://console.anthropic.com/](https://console.anthropic.com/) 접속
2. "Usage" 메뉴로 이동
3. 일별/월별 사용량 확인

**TriFlow AI 내에서 확인 (향후 추가)**:
```
사용자: 이번 달 API 사용량 알려줘

AI: 📊 2025년 11월 API 사용량:
• 총 요청 횟수: 1,523회
• 총 토큰 사용: 450,000 토큰
• 예상 비용: $4.50
```

---

## 3. Chat Interface

### Q9: AI 응답이 느립니다. 어떻게 개선할 수 있나요?

**A**: 여러 요인이 응답 속도에 영향을 줍니다.

**원인 및 해결 방법**:

**1. 네트워크 속도**:
- 문제: 인터넷 연결이 느림
- 해결: 고속 인터넷 사용 권장 (5 Mbps 이상)

**2. LLM 모델 부하**:
- 문제: Anthropic 서버 부하
- 해결: 시간대를 바꿔서 재시도 (한국 시간 오전 시간대 권장)

**3. 복잡한 요청**:
- 문제: 매우 긴 텍스트 또는 복잡한 판단 요청
- 해결: 요청을 간단하게 분할

**4. Rule Engine 최적화**:
- Rule Engine 우선 사용: 간단한 판단은 LLM 없이 처리 (0.3초)
- LLM은 복잡한 케이스에만 사용

**평균 응답 시간**:
- Rule Engine only: 0.3~0.5초
- LLM 보완: 1.5~3초

---

### Q10: 대화 기록은 얼마나 오래 보관되나요?

**A**: **영구적**으로 보관됩니다.

**저장 위치**:
- 로컬 SQLite 데이터베이스
- `C:\Users\[사용자]\AppData\Local\TriFlow AI\data\judgify.db`

**보관 기간**:
- **무제한**: 사용자가 수동으로 삭제하지 않는 한 영구 보관
- **앱 재설치**: 앱을 제거해도 데이터는 남아 있음

**수동 삭제 방법**:
- `Ctrl + L`: 현재 세션 대화 기록 초기화 (DB에는 남음)
- 전체 삭제 (향후 추가): Settings > Clear All Data

---

### Q11: 여러 개의 대화 세션을 저장할 수 있나요? (향후 추가)

**A**: 현재는 **단일 세션**만 지원하지만, 향후 업데이트에서 **멀티 세션** 기능이 추가될 예정입니다.

**예정 기능**:
- 세션별 대화 기록 분리
- 세션 이름 지정 (예: "재고 분석", "품질 검사")
- 세션 간 전환 (탭 UI)

---

## 4. Workflow Builder

### Q12: n8n과 비교했을 때 어떤 점이 다른가요?

**A**: TriFlow AI Workflow Builder는 n8n에서 영감을 받았지만, **AI 판단에 특화**되어 있습니다.

**공통점**:
- ✅ 드래그앤드롭 노드 기반 에디터
- ✅ 비주얼 워크플로우 설계
- ✅ 외부 시스템 연동 (Slack, Email 등)

**차이점**:

| 기능 | n8n | TriFlow AI |
|------|-----|-----------|
| **AI 판단** | 수동 스크립트 필요 | **Rule Engine + LLM 자동 판단** |
| **자동 학습** | 없음 | **Few-shot 학습 지원** |
| **로컬 실행** | 셀프 호스팅 | **Desktop 앱 (설치 쉬움)** |
| **데이터 집계** | 없음 | **LLM 토큰 최적화 알고리즘** |
| **백업/복구** | 수동 | **자동 gzip 백업** |

**결론**: TriFlow AI는 **AI 판단 워크플로우**에 최적화되어 있습니다.

---

### Q13: 워크플로우 실행 중에 멈추면 어떻게 하나요?

**A**: 여러 원인이 있을 수 있습니다.

**1단계: 시뮬레이션으로 디버깅**
- 시뮬레이션 패널에서 단계별 실행
- 어느 노드에서 멈추는지 확인

**2단계: 에러 메시지 확인**
- 실패한 노드의 에러 메시지 읽기
- 일반적인 에러:
  - `Timeout`: 외부 API 응답 지연 → 타임아웃 증가
  - `Webhook URL not configured`: Slack/Email 설정 누락 → Settings에서 설정
  - `API key invalid`: API 키 만료 → 새 키 입력

**3단계: 워크플로우 재실행**
- 일시적 네트워크 오류일 수 있음
- "실행" 버튼 다시 클릭

**4단계: 로그 확인**
- `C:\Users\[사용자]\AppData\Local\TriFlow AI\logs\workflow.log`
- 상세한 에러 스택 트레이스 확인

---

### Q14: 복잡한 조건 분기(If/Else)를 만들 수 있나요?

**A**: 네, Logic 노드를 사용하여 복잡한 분기를 만들 수 있습니다.

**예시: If/Else 분기**
```
START
  ↓
Judgment (온도 체크)
  ↓
If 노드
  ├─ True → Send Email (정상 알림)
  └─ False → Send Slack (경고 알림)
```

**예시: Switch 분기** (향후 추가)
```
START
  ↓
Judgment (품질 등급)
  ↓
Switch 노드
  ├─ A등급 → Action 1
  ├─ B등급 → Action 2
  └─ C등급 → Action 3
```

---

### Q15: 워크플로우를 외부에서 트리거할 수 있나요? (향후 추가)

**A**: 향후 업데이트에서 **Webhook Trigger** 기능이 추가될 예정입니다.

**예정 기능**:
```
POST https://triflow-webhook.local/execute/[workflow-id]
Body: { "data": { "temperature": 85 } }
```

**사용 사례**:
- 외부 시스템에서 워크플로우 호출
- 센서 데이터 자동 전송
- CI/CD 파이프라인 통합

---

## 5. Dashboard

### Q16: Dashboard 데이터가 업데이트되지 않습니다.

**A**: WebSocket 연결 상태를 확인하세요.

**확인 방법**:
- Dashboard 우측 상단에 연결 상태 표시
  - 🟢 녹색 점: 연결됨 (정상)
  - 🔴 빨간 점: 연결 끊김 (문제)

**해결 방법**:

**1. 네트워크 확인**:
- 인터넷 연결 확인
- 방화벽에서 TriFlow AI 허용

**2. 앱 재시작**:
- TriFlow AI 종료 후 재실행
- WebSocket 자동 재연결

**3. 로그 확인**:
- `C:\Users\[사용자]\AppData\Local\TriFlow AI\logs\app.log`
- "WebSocket connection failed" 메시지 찾기

---

### Q17: 커스텀 차트를 만들 수 있나요? (향후 추가)

**A**: 향후 업데이트에서 **AI 기반 차트 생성** 기능이 추가될 예정입니다.

**예정 기능**:
```
사용자: 최근 7일간 워크플로우별 실행 횟수를 바 차트로 보여줘

AI: ✅ 차트가 생성되었습니다!
[바 차트 렌더링]
```

**지원 예정 차트**:
- 바 차트
- 라인 차트
- 파이 차트
- 히트맵
- 스캐터 플롯

---

## 6. 백업 및 복구

### Q18: 백업은 얼마나 자주 해야 하나요?

**A**: **자동 백업**이 권장되지만, 수동 백업도 가능합니다.

**권장 백업 주기**:
- **일반 사용**: 주 1회
- **중요 데이터**: 매일 (예: 프로덕션 환경)
- **대량 작업 전**: 수동 백업

**자동 백업 설정** (향후 추가):
```
사용자: 매일 오전 9시에 자동 백업 설정해줘

AI: ✅ 자동 백업이 설정되었습니다!
• 실행 시각: 매일 09:00 KST
• 보관 기간: 최근 10개 유지
```

**수동 백업**:
```
사용자: 지금 백업해줘

AI: ✅ 백업 완료!
• 파일: judgify_backup_20251113_143522.db.gz
```

---

### Q19: 백업 파일을 다른 PC로 옮길 수 있나요?

**A**: 네, 백업 파일은 **휴대 가능**하며, 다른 PC에서 복구할 수 있습니다.

**단계**:

**PC A (백업)**:
1. 백업 생성: "데이터베이스 백업해줘"
2. 백업 파일 위치: `C:\Users\[사용자]\AppData\Local\TriFlow AI\backups\`
3. 백업 파일을 USB 또는 클라우드로 복사

**PC B (복구)**:
1. TriFlow AI 설치
2. 백업 파일을 동일 경로에 복사:
   ```
   C:\Users\[사용자]\AppData\Local\TriFlow AI\backups\
   ```
3. Chat Interface에서 복구:
   ```
   사용자: judgify_backup_20251113_143522.db.gz 파일에서 복구해줘

   AI: ✅ 복구 완료!
   ```

---

### Q20: 복구 실패시 어떻게 하나요?

**A**: TriFlow AI는 **안전 백업**을 자동으로 생성하므로, 복구 실패시에도 데이터 손실이 없습니다.

**복구 실패 시나리오**:
```
사용자: 백업에서 복구해줘

AI: ⚠️ 복구 전 현재 DB를 안전 백업합니다...
✅ 안전 백업 완료: judgify.db.before_restore

🔄 복구 중...
❌ 복구 실패! 에러: 백업 파일이 손상되었습니다.

✅ 안전 백업에서 원상복구 중...
✅ 원상복구 완료! 데이터 손실 없음.
```

**수동 원상복구**:
1. 백업 디렉토리로 이동:
   ```
   C:\Users\[사용자]\AppData\Local\TriFlow AI\data\
   ```
2. `judgify.db.before_restore` 파일을 `judgify.db`로 이름 변경
3. TriFlow AI 재시작

---

## 7. 성능 및 최적화

### Q21: TriFlow AI가 느려졌습니다. 어떻게 해야 하나요?

**A**: 여러 최적화 방법이 있습니다.

**1. 캐시 정리**:
```
사용자: 캐시 정리해줘

AI: 🧹 캐시 정리 중...
✅ 완료!
• 삭제된 캐시: 1,523개
• 확보된 메모리: 120 MB
```

**2. 데이터베이스 최적화**:
```
사용자: 데이터베이스 최적화해줘

AI: 🔧 최적화 중...
✅ 완료!
• 이전 크기: 5.2 MB
• 최적화 후: 3.8 MB (27% 절감)
```

**3. 오래된 데이터 정리** (향후 추가):
```
사용자: 30일 이상 된 대화 기록 삭제해줘

AI: 🗑️ 정리 중...
✅ 완료!
• 삭제된 대화: 523개
• 확보된 공간: 15 MB
```

**4. 앱 재시작**:
- 가끔 메모리 누수로 느려질 수 있음
- 앱 종료 후 재실행

---

### Q22: CPU/메모리 사용량이 높습니다.

**A**: 정상적인 사용 범위를 확인하세요.

**정상 사용량**:
- **유휴 상태**: CPU 1~2%, RAM 150 MB
- **채팅 중**: CPU 5~10%, RAM 200 MB
- **워크플로우 실행**: CPU 20~30%, RAM 300 MB

**비정상 사용량 (해결 방법)**:

**1. CPU 사용량 > 50%**:
- 원인: 무한 루프, 복잡한 워크플로우
- 해결: 워크플로우 중지, 앱 재시작

**2. RAM 사용량 > 1GB**:
- 원인: 메모리 누수, 대량 데이터 처리
- 해결: 앱 재시작, 오래된 데이터 정리

**3. 지속적인 문제**:
- 로그 파일을 GitHub Issues에 첨부
- 개발팀이 메모리 프로파일링 수행

---

## 8. 보안 및 프라이버시

### Q23: 내 데이터는 어디에 저장되나요?

**A**: **100% 로컬 저장**입니다. 서버로 전송되지 않습니다.

**저장 위치**:
- **데이터베이스**: `C:\Users\[사용자]\AppData\Local\TriFlow AI\data\judgify.db`
- **백업**: `C:\Users\[사용자]\AppData\Local\TriFlow AI\backups\`
- **로그**: `C:\Users\[사용자]\AppData\Local\TriFlow AI\logs\`

**외부 전송 항목**:
- ✅ Anthropic API: LLM 판단 요청 (암호화됨)
- ❌ TriFlow 서버: 없음 (서버리스)

**프라이버시 보장**:
- 대화 기록, 판단 결과는 로컬에만 저장
- 개인정보 유출 위험 없음

---

### Q24: API 키는 안전하게 저장되나요?

**A**: 네, **암호화 저장**됩니다.

**보안 조치**:
- **암호화**: AES-256 암호화
- **접근 제한**: Windows 사용자 계정별 격리
- **메모리 보호**: 앱 종료시 메모리에서 제거

**권장 사항**:
- ⚠️ API 키를 공유하지 마세요
- ⚠️ 공개 저장소에 업로드 금지
- ✅ 정기적으로 키 교체 (3개월마다)

---

### Q25: 다른 사용자가 내 데이터를 볼 수 있나요?

**A**: 아니요, **Windows 사용자 계정별 격리**되어 있습니다.

**데이터 접근 권한**:
- **사용자 A**: `C:\Users\UserA\AppData\Local\TriFlow AI\`
- **사용자 B**: `C:\Users\UserB\AppData\Local\TriFlow AI\`

**관리자 계정**:
- Windows 관리자는 모든 사용자 데이터에 접근 가능
- **권장**: 관리자 계정 보안 철저히 관리

---

## 9. 에러 코드

### ERR_API_KEY_NOT_CONFIGURED

**증상**: Chat Interface에서 메시지 전송시 에러

**원인**: Anthropic API 키가 설정되지 않음

**해결**:
1. Settings 페이지로 이동
2. Anthropic API Key 입력
3. Save Settings 클릭

---

### ERR_WEBSOCKET_CONNECTION_FAILED

**증상**: Dashboard 데이터 업데이트 실패

**원인**:
- 네트워크 연결 끊김
- 방화벽 차단

**해결**:
1. 인터넷 연결 확인
2. 방화벽에서 TriFlow AI 허용
3. 앱 재시작

---

### ERR_WORKFLOW_EXECUTION_TIMEOUT

**증상**: 워크플로우 실행 중 타임아웃

**원인**:
- 외부 API 응답 지연
- LLM 요청 시간 초과

**해결**:
1. 워크플로우 재실행
2. Settings > Advanced > Timeout 설정 증가
3. 네트워크 연결 확인

---

### ERR_BACKUP_RESTORE_FAILED

**증상**: 백업 복구 실패

**원인**:
- 백업 파일 손상
- 잘못된 백업 파일 형식

**해결**:
1. 안전 백업에서 자동 복구됨 (데이터 손실 없음)
2. 다른 백업 파일 시도
3. 수동 원상복구 (Q20 참조)

---

### ERR_DATABASE_LOCKED

**증상**: 데이터베이스 작업 실패

**원인**:
- 다른 프로세스에서 DB 파일 사용 중
- 여러 TriFlow AI 인스턴스 실행

**해결**:
1. TriFlow AI 완전 종료
2. 다른 SQLite 도구 종료
3. 앱 재시작

---

## 10. 기타

### Q26: 오프라인에서 사용할 수 있나요?

**A**: **부분적으로** 가능합니다.

**오프라인 가능 기능**:
- ✅ Rule Engine 판단 (LLM 없이)
- ✅ 데이터베이스 작업 (로컬)
- ✅ 워크플로우 편집 (저장만)
- ✅ 백업/복구

**오프라인 불가능 기능**:
- ❌ LLM 판단 (Anthropic API 필요)
- ❌ 외부 연동 (Slack, Email 등)
- ❌ 자동 업데이트

---

### Q27: macOS/Linux 버전은 있나요?

**A**: 현재는 **Windows 전용**이지만, 향후 크로스 플랫폼 지원 예정입니다.

**로드맵**:
- ✅ Windows: 현재 지원 (v0.1.x)
- 🔜 macOS: 계획 중 (v0.2.x)
- 🔜 Linux: 계획 중 (v0.3.x)

**Tauri 프레임워크**:
- TriFlow AI는 Tauri 기반이므로, macOS/Linux 포팅이 용이합니다.
- GitHub Issues에서 요청이 많으면 우선순위 상향 조정

---

### Q28: 모바일 앱은 있나요?

**A**: 현재는 **Desktop 전용**이며, 모바일 앱은 장기 계획에 포함되어 있습니다.

**대안**:
- Web 버전 (향후 추가): 브라우저에서 접속
- Remote Desktop: Windows 원격 데스크톱으로 모바일에서 접속

---

### Q29: 언어를 영어로 변경할 수 있나요? (향후 추가)

**A**: 현재는 **한국어 전용**이지만, 향후 다국어 지원 예정입니다.

**계획 중인 언어**:
- 🇰🇷 한국어 (현재)
- 🇺🇸 영어 (우선순위 1)
- 🇯🇵 일본어 (우선순위 2)
- 🇨🇳 중국어 (우선순위 3)

---

### Q30: 기능 요청은 어떻게 하나요?

**A**: GitHub Issues에서 요청할 수 있습니다.

**단계**:
1. [GitHub Issues](https://github.com/mugoori/Judgify-core/issues) 접속
2. "New Issue" 클릭
3. "Feature Request" 템플릿 선택
4. 요청 내용 작성:
   - **제목**: 명확한 기능 요약
   - **설명**: 상세한 사용 사례
   - **스크린샷**: 예시 화면 (선택)
5. "Submit" 클릭

**우선순위**:
- 👍 반응이 많은 요청 우선 개발
- 💬 토론 참여로 요구사항 구체화
- 🚀 커뮤니티 투표로 로드맵 결정

---

## 📚 추가 리소스

- **사용자 가이드**: [USER_GUIDE.md](USER_GUIDE.md)
- **GitHub 저장소**: [https://github.com/mugoori/Judgify-core](https://github.com/mugoori/Judgify-core)
- **개발 문서**: [docs/development/](../development/)
- **API 레퍼런스**: [docs/architecture/api_specifications.md](../architecture/api_specifications.md)

---

**🙋 더 궁금한 사항이 있으신가요?**

위 FAQ에서 답을 찾지 못했다면:
1. [GitHub Issues](https://github.com/mugoori/Judgify-core/issues)에 질문 남기기
2. 로그 파일 첨부 (에러 관련시)
3. 스크린샷 첨부 (UI 관련시)

개발팀이 24시간 내에 답변해드립니다! 🚀
