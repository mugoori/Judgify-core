@echo off
REM Judgify-core 환경 설정 자동화 스크립트 (Windows - Command Prompt)
REM 새 PC에서 클론 후 필수 설정 파일 자동 생성

echo 🚀 Judgify-core Ver2.0 Final - 환경 설정 시작
echo ================================================
echo.

REM 프로젝트 루트 디렉토리로 이동
cd /d "%~dp0\.."

REM 1. .env 파일 생성
if exist ".env" (
    echo ✅ .env 파일이 이미 존재합니다. 건너뜁니다.
) else (
    if exist ".env.example" (
        copy /Y ".env.example" ".env" >nul
        echo ✅ .env 파일 생성 완료 ^(.env.example에서 복사^)
    ) else (
        echo ❌ 오류: .env.example 파일을 찾을 수 없습니다.
        pause
        exit /b 1
    )
)

REM 2. .mcp.json 파일 생성
if exist ".mcp.json" (
    echo ✅ .mcp.json 파일이 이미 존재합니다. 건너뜁니다.
) else (
    if exist ".mcp.template.json" (
        copy /Y ".mcp.template.json" ".mcp.json" >nul
        echo ✅ .mcp.json 파일 생성 완료 ^(.mcp.template.json에서 복사^)
    ) else (
        echo ❌ 오류: .mcp.template.json 파일을 찾을 수 없습니다.
        pause
        exit /b 1
    )
)

echo.
echo ================================================
echo ✅ 환경 설정 파일 생성 완료!
echo.
echo 📝 다음 단계:
echo 1. .env 파일을 열고 다음 값을 입력하세요:
echo    - DATABASE_URL
echo    - REDIS_URL
echo    - OPENAI_API_KEY
echo    - JWT_SECRET
echo.
echo 2. .mcp.json 파일을 열고 다음 값을 입력하세요:
echo    - GITHUB_PERSONAL_ACCESS_TOKEN
echo.
echo 3. 상세 설정 가이드: SETUP.md 참조
echo.
echo 📂 생성된 파일:
echo    - .env ^(환경 변수^)
echo    - .mcp.json ^(MCP 서버 설정^)
echo.
echo 🔐 보안 주의: 이 파일들은 .gitignore에 포함되어 Git에 커밋되지 않습니다.
echo ================================================
echo.
pause
