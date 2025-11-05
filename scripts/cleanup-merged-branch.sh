#!/bin/bash
# 머지된 PR 브랜치 정리 스크립트
# 사용법: ./scripts/cleanup-merged-branch.sh <branch-name>
# 예시: ./scripts/cleanup-merged-branch.sh feature/my-feature

set -e  # 에러 발생 시 즉시 중단

BRANCH="$1"

# 사용법 체크
if [ -z "$BRANCH" ]; then
  echo "❌ 사용법: ./scripts/cleanup-merged-branch.sh <branch-name>"
  echo ""
  echo "예시:"
  echo "  ./scripts/cleanup-merged-branch.sh feature/my-feature"
  echo "  ./scripts/cleanup-merged-branch.sh fix/bug-fix"
  exit 1
fi

# main/develop 브랜치 보호
if [ "$BRANCH" = "main" ] || [ "$BRANCH" = "develop" ]; then
  echo "❌ main 또는 develop 브랜치는 삭제할 수 없습니다."
  exit 1
fi

# archive/backup 브랜치 보호
if [[ "$BRANCH" == archive/* ]] || [[ "$BRANCH" == backup/* ]]; then
  echo "❌ archive/* 또는 backup/* 브랜치는 삭제할 수 없습니다."
  exit 1
fi

echo "🗑️  브랜치 삭제 중..."
echo "   브랜치: $BRANCH"
echo ""

# 원격 브랜치 존재 확인
if git ls-remote --heads origin "$BRANCH" | grep -q "$BRANCH"; then
  echo "🌐 원격 브랜치 삭제 중..."
  git push origin --delete "$BRANCH"
  echo "   ✅ 원격 브랜치 삭제 완료"
else
  echo "   ℹ️  원격 브랜치가 이미 삭제되었거나 존재하지 않습니다"
fi

# 로컬 브랜치 존재 확인
if git show-ref --verify --quiet "refs/heads/$BRANCH"; then
  # 현재 브랜치 체크
  CURRENT_BRANCH=$(git branch --show-current)
  if [ "$CURRENT_BRANCH" = "$BRANCH" ]; then
    echo ""
    echo "💡 현재 체크아웃된 브랜치입니다. main으로 전환합니다..."
    git checkout main
  fi

  echo "💻 로컬 브랜치 삭제 중..."
  git branch -D "$BRANCH"
  echo "   ✅ 로컬 브랜치 삭제 완료"
else
  echo "   ℹ️  로컬 브랜치가 이미 삭제되었거나 존재하지 않습니다"
fi

echo ""
echo "✅ 브랜치 정리 완료!"
echo ""
echo "📊 남은 브랜치 확인:"
echo "   git branch -a | grep -v 'main\|develop\|archive\|backup'"
