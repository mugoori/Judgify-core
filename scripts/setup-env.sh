#!/bin/bash

# Judgify-core 환경 설정 자동화 스크립트 (Mac/Linux)
# 새 PC에서 클론 후 필수 설정 파일 자동 생성

echo "🚀 Judgify-core Ver2.0 Final - 환경 설정 시작"
echo "================================================"
echo ""

# 프로젝트 루트 디렉토리로 이동
cd "$(dirname "$0")/.."

# 1. .env 파일 생성
if [ -f ".env" ]; then
    echo "✅ .env 파일이 이미 존재합니다. 건너뜁니다."
else
    if [ -f ".env.example" ]; then
        cp .env.example .env
        echo "✅ .env 파일 생성 완료 (.env.example에서 복사)"
    else
        echo "❌ 오류: .env.example 파일을 찾을 수 없습니다."
        exit 1
    fi
fi

# 2. .mcp.json 파일 생성
if [ -f ".mcp.json" ]; then
    echo "✅ .mcp.json 파일이 이미 존재합니다. 건너뜁니다."
else
    if [ -f ".mcp.template.json" ]; then
        cp .mcp.template.json .mcp.json
        echo "✅ .mcp.json 파일 생성 완료 (.mcp.template.json에서 복사)"
    else
        echo "❌ 오류: .mcp.template.json 파일을 찾을 수 없습니다."
        exit 1
    fi
fi

echo ""
echo "================================================"
echo "✅ 환경 설정 파일 생성 완료!"
echo ""
echo "📝 다음 단계:"
echo "1. .env 파일을 열고 다음 값을 입력하세요:"
echo "   - DATABASE_URL"
echo "   - REDIS_URL"
echo "   - OPENAI_API_KEY"
echo "   - JWT_SECRET"
echo ""
echo "2. .mcp.json 파일을 열고 다음 값을 입력하세요:"
echo "   - GITHUB_PERSONAL_ACCESS_TOKEN"
echo ""
echo "3. 상세 설정 가이드: SETUP.md 참조"
echo ""
echo "📂 생성된 파일:"
echo "   - .env (환경 변수)"
echo "   - .mcp.json (MCP 서버 설정)"
echo ""
echo "🔐 보안 주의: 이 파일들은 .gitignore에 포함되어 Git에 커밋되지 않습니다."
echo "================================================"
