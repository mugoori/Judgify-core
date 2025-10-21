@echo off
REM Git Pre-Commit Hook: 브랜치 백업 자동 알림 (Windows)
REM 설치: copy scripts\git-hooks\pre-commit.bat .git\hooks\pre-commit.bat

setlocal enabledelayedexpansion

REM 백업 권장 임계값
set LINE_THRESHOLD=200
set ARCHITECTURE_FILES_THRESHOLD=3

REM 변경된 파일 목록
git diff --cached --name-only > temp_changed_files.txt

REM 백업 권장 플래그
set BACKUP_RECOMMENDED=0
set REASON_COUNT=0

REM 1. CLAUDE.md 200줄 이상 변경 체크
findstr /C:"CLAUDE.md" temp_changed_files.txt >nul 2>&1
if %errorlevel% equ 0 (
    for /f %%i in ('git diff --cached CLAUDE.md ^| findstr /R "^+ ^-" ^| find /c /v ""') do set CLAUDE_CHANGES=%%i
    if !CLAUDE_CHANGES! geq %LINE_THRESHOLD% (
        set BACKUP_RECOMMENDED=1
        set /a REASON_COUNT+=1
        set REASON_!REASON_COUNT!=CLAUDE.md !CLAUDE_CHANGES!줄 변경 (임계값: %LINE_THRESHOLD%줄^)
    )
)

REM 2. 핵심 컨텍스트 파일 변경 체크
for %%f in (
    "initial.md"
    "system-structure.md"
    "docs/architecture/system_overview.md"
    "docs/development/plan.md"
) do (
    findstr /C:"%%~f" temp_changed_files.txt >nul 2>&1
    if !errorlevel! equ 0 (
        for /f %%i in ('git diff --cached "%%~f" ^| findstr /R "^+ ^-" ^| find /c /v ""') do set FILE_CHANGES=%%i
        if !FILE_CHANGES! geq 50 (
            set BACKUP_RECOMMENDED=1
            set /a REASON_COUNT+=1
            set REASON_!REASON_COUNT!=%%~f !FILE_CHANGES!줄 변경 (중요 파일^)
        )
    )
)

REM 3. 아키텍처 파일 다수 변경 체크
for /f %%i in ('findstr /C:"docs/architecture/" temp_changed_files.txt ^| find /c /v ""') do set ARCHITECTURE_CHANGES=%%i
if !ARCHITECTURE_CHANGES! geq %ARCHITECTURE_FILES_THRESHOLD% (
    set BACKUP_RECOMMENDED=1
    set /a REASON_COUNT+=1
    set REASON_!REASON_COUNT!=아키텍처 파일 !ARCHITECTURE_CHANGES!개 변경 (임계값: %ARCHITECTURE_FILES_THRESHOLD%개^)
)

REM 4. 서비스 파일 다수 변경 체크
for /f %%i in ('findstr /C:"docs/services/" temp_changed_files.txt ^| find /c /v ""') do set SERVICE_CHANGES=%%i
if !SERVICE_CHANGES! geq 3 (
    set BACKUP_RECOMMENDED=1
    set /a REASON_COUNT+=1
    set REASON_!REASON_COUNT!=서비스 설계 파일 !SERVICE_CHANGES!개 변경 (임계값: 3개^)
)

REM 5. 의존성 파일 변경 체크
findstr /R "package\.json requirements\.txt Cargo\.toml go\.mod" temp_changed_files.txt >nul 2>&1
if %errorlevel% equ 0 (
    set BACKUP_RECOMMENDED=1
    set /a REASON_COUNT+=1
    set REASON_!REASON_COUNT!=의존성 파일 변경 감지
)

REM 6. Docker/Kubernetes 설정 변경 체크
for /f %%i in ('findstr /R "Dockerfile docker-compose k8s\\ \.yaml$ \.yml$" temp_changed_files.txt ^| find /c /v ""') do set INFRA_FILES=%%i
if !INFRA_FILES! geq 2 (
    set BACKUP_RECOMMENDED=1
    set /a REASON_COUNT+=1
    set REASON_!REASON_COUNT!=인프라 설정 파일 !INFRA_FILES!개 변경
)

REM 임시 파일 삭제
del temp_changed_files.txt

REM 백업 권장 알림 출력
if %BACKUP_RECOMMENDED% equ 1 (
    echo.
    echo ===============================================================
    echo       브랜치 백업 권장!
    echo ===============================================================
    echo.
    echo 다음 사유로 인해 브랜치 백업을 권장합니다:
    for /l %%i in (1,1,%REASON_COUNT%) do (
        echo   * !REASON_%%i!
    )
    echo.
    echo 권장 워크플로우:
    echo   1. 현재 커밋 취소: git reset HEAD~
    echo   2. 현재 상태 커밋 (백업): git add . ^&^& git commit -m "backup: 변경 전 상태"
    echo   3. 백업 브랜치 생성: git checkout -b {category}/{description}
    echo      예: git checkout -b docs/claude-md-major-update
    echo   4. 변경 작업 수행 및 커밋
    echo   5. 비교 보고서 작성 (템플릿: docs/templates/COMPARISON_TEMPLATE.md)
    echo.
    echo 상세 가이드: docs/development/git-branch-strategy.md
    echo ===============================================================
    echo.

    set /p CONTINUE="백업 없이 계속 진행하시겠습니까? (y/N): "
    if /i not "!CONTINUE!"=="y" (
        echo.
        echo 커밋이 취소되었습니다. 백업 브랜치를 생성한 후 다시 커밋하세요.
        exit /b 1
    )

    echo.
    echo 백업 없이 진행합니다. 나중에 롤백이 어려울 수 있습니다.
    echo.
)

REM 정상 진행
exit /b 0
