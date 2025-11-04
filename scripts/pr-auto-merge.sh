#!/bin/bash
# PR 생성 및 자동 머지 스크립트 (Git Bash/Linux/Mac)
# 사용법: ./scripts/pr-auto-merge.sh "PR 제목"

set -e  # 에러 발생 시 즉시 중단

TITLE="$1"

# 사용법 체크
if [ -z "$TITLE" ]; then
  echo "❌ 사용법: ./scripts/pr-auto-merge.sh \"PR 제목\""
  echo ""
  echo "예시:"
  echo "  ./scripts/pr-auto-merge.sh \"feat: Add new feature\""
  echo "  ./scripts/pr-auto-merge.sh \"fix: Fix bug in chat interface\""
  exit 1
fi

# 현재 브랜치 확인
BRANCH=$(git branch --show-current)

if [ "$BRANCH" = "main" ] || [ "$BRANCH" = "develop" ]; then
  echo "❌ main 또는 develop 브랜치에서는 사용할 수 없습니다."
  echo "   feature/* 브랜치에서 실행해주세요."
  exit 1
fi

# GitHub CLI 설치 확인
if ! command -v gh &> /dev/null; then
  echo "❌ GitHub CLI (gh)가 설치되지 않았습니다."
  echo ""
  echo "설치 방법:"
  echo "  1. https://cli.github.com/ 접속"
  echo "  2. Windows 설치 파일 다운로드"
  echo "  3. 설치 후 터미널 재시작"
  echo "  4. gh auth login 실행"
  exit 1
fi

# GitHub 인증 확인
if ! gh auth status &> /dev/null; then
  echo "❌ GitHub 인증이 필요합니다."
  echo "   다음 명령어를 실행하세요: gh auth login"
  exit 1
fi

echo "🚀 PR 생성 중..."
echo "   브랜치: $BRANCH → main"
echo "   제목: $TITLE"
echo ""

# PR 생성 및 자동 머지 설정
gh pr create \
  --title "$TITLE" \
  --body "🤖 Auto-generated PR via GitHub CLI

## 변경 사항
<!-- PR에서 직접 수정 가능 -->

## 체크리스트
- [ ] 코드 자체 검토 완료
- [ ] 로컬 테스트 통과
- [ ] CI 통과 확인 (Lighthouse + Criterion)

---
Generated with [Claude Code](https://claude.com/claude-code)" \
  --base main \
  --head "$BRANCH"

# PR 번호 가져오기
PR_NUMBER=$(gh pr view --json number -q .number)

# 자동 머지 활성화 (CI 통과 후)
echo "⏳ 자동 머지 설정 중..."
gh pr merge "$PR_NUMBER" --auto --squash --delete-branch

echo ""
echo "✅ PR #$PR_NUMBER 생성 완료!"
echo "🔗 URL: $(gh pr view --json url -q .url)"
echo ""
echo "🔄 다음 단계:"
echo "   1. CI 실행 중 (Lighthouse + Criterion)"
echo "   2. CI 통과 시 자동 머지"
echo "   3. 브랜치 자동 삭제"
echo ""
echo "💡 진행 상황 확인: gh pr view $PR_NUMBER"
