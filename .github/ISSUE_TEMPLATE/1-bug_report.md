---
name: 🐛 Bug Report
about: 버그나 오류를 발견하셨나요? 문제를 해결할 수 있도록 도와주세요.
title: '[BUG] '
labels: ['type:bug', 'status:triage']
assignees: ''
---

## 🐛 버그 설명
버그에 대한 명확하고 간결한 설명을 작성해주세요.

## 📍 발생 위치
**서비스**: (예: Judgment Service, Learning Service, BI Service 등)
**포트**: (예: 8002, 8009 등)
**파일 경로**: (예: services/judgment/core/hybrid_engine.py)

## 🔄 재현 방법
버그를 재현하는 단계:
1. '...' 로 이동
2. '...' 클릭
3. '...' 까지 스크롤
4. 오류 발생

## ✅ 예상 동작
정상적으로 작동했을 때 어떤 결과가 나와야 하는지 설명해주세요.

## ❌ 실제 동작
실제로 어떤 결과가 나왔는지 설명해주세요.

## 📸 스크린샷
해당되는 경우 문제를 설명하는 스크린샷을 추가해주세요.

## 🖥 환경
- **OS**: (예: Windows 11, Ubuntu 22.04)
- **Python 버전**: (예: 3.11.5)
- **FastAPI 버전**: (예: 0.104.1)
- **데이터베이스**: (예: PostgreSQL 15.3)
- **브라우저** (프론트엔드 이슈인 경우): (예: Chrome 120)

## 📋 로그 출력
관련 에러 로그나 콘솔 출력을 붙여넣어주세요:
```
여기에 로그를 붙여넣으세요
```

## 🔍 추가 컨텍스트
문제에 대한 다른 컨텍스트를 여기에 추가해주세요.

## ⚡ 우선순위
- [ ] Critical - 시스템 중단
- [ ] High - 주요 기능 영향
- [ ] Medium - 기능 일부 영향
- [ ] Low - 사소한 문제

## 🏷 서비스 라벨
해당하는 서비스에 체크해주세요:
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
