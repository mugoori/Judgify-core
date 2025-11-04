# PR 생성 및 자동 머지 스크립트 (Windows PowerShell)
# 사용법: .\scripts\pr-auto-merge.ps1 -Title "PR 제목"

param(
    [Parameter(Mandatory=$true, HelpMessage="PR 제목을 입력하세요 (예: feat: Add new feature)")]
    [string]$Title
)

# 에러 발생 시 중단
$ErrorActionPreference = "Stop"

# 현재 브랜치 확인
$branch = git branch --show-current

if ($branch -eq "main" -or $branch -eq "develop") {
    Write-Host "❌ main 또는 develop 브랜치에서는 사용할 수 없습니다." -ForegroundColor Red
    Write-Host "   feature/* 브랜치에서 실행해주세요." -ForegroundColor Yellow
    exit 1
}

# GitHub CLI 설치 확인
try {
    $null = gh --version
} catch {
    Write-Host "❌ GitHub CLI (gh)가 설치되지 않았습니다." -ForegroundColor Red
    Write-Host ""
    Write-Host "설치 방법:" -ForegroundColor Yellow
    Write-Host "  1. https://cli.github.com/ 접속"
    Write-Host "  2. Windows 설치 파일 다운로드"
    Write-Host "  3. 설치 후 PowerShell 재시작"
    Write-Host "  4. gh auth login 실행"
    exit 1
}

# GitHub 인증 확인
$authStatus = gh auth status 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ GitHub 인증이 필요합니다." -ForegroundColor Red
    Write-Host "   다음 명령어를 실행하세요: gh auth login" -ForegroundColor Yellow
    exit 1
}

Write-Host "🚀 PR 생성 중..." -ForegroundColor Cyan
Write-Host "   브랜치: $branch → main" -ForegroundColor Gray
Write-Host "   제목: $Title" -ForegroundColor Gray
Write-Host ""

# PR Body 생성
$prBody = @"
🤖 Auto-generated PR via GitHub CLI

## 변경 사항
<!-- PR에서 직접 수정 가능 -->

## 체크리스트
- [ ] 코드 자체 검토 완료
- [ ] 로컬 테스트 통과
- [ ] CI 통과 확인 (Lighthouse + Criterion)

---
Generated with [Claude Code](https://claude.com/claude-code)
"@

# PR 생성
gh pr create `
    --title "$Title" `
    --body "$prBody" `
    --base main `
    --head "$branch"

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ PR 생성 실패" -ForegroundColor Red
    exit 1
}

# PR 번호 가져오기
$prNumber = gh pr view --json number -q .number

# 자동 머지 활성화
Write-Host "⏳ 자동 머지 설정 중..." -ForegroundColor Cyan
gh pr merge $prNumber --auto --squash --delete-branch

if ($LASTEXITCODE -ne 0) {
    Write-Host "⚠️ 자동 머지 설정 실패 (수동으로 머지하세요)" -ForegroundColor Yellow
} else {
    Write-Host ""
    Write-Host "✅ PR #$prNumber 생성 완료!" -ForegroundColor Green

    # PR URL 가져오기
    $prUrl = gh pr view --json url -q .url
    Write-Host "🔗 URL: $prUrl" -ForegroundColor Cyan

    Write-Host ""
    Write-Host "🔄 다음 단계:" -ForegroundColor Yellow
    Write-Host "   1. CI 실행 중 (Lighthouse + Criterion)"
    Write-Host "   2. CI 통과 시 자동 머지"
    Write-Host "   3. 브랜치 자동 삭제"
    Write-Host ""
    Write-Host "💡 진행 상황 확인: gh pr view $prNumber" -ForegroundColor Gray
}
