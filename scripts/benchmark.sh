#!/bin/bash
# Benchmark Script: 5개 핵심 지표 자동 측정
# 사용법: ./scripts/benchmark.sh [output-file]

set -e

# 색상 정의
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 출력 파일 (선택사항)
OUTPUT_FILE="${1:-BENCHMARK_REPORT_$(date +%Y%m%d_%H%M%S).md}"

# 헤더 출력
print_header() {
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}   Judgify-core 성능 벤치마크 보고서${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
}

# 마크다운 보고서 시작
{
    echo "# 📊 성능 벤치마크 보고서"
    echo ""
    echo "> **측정 일시**: $(date '+%Y-%m-%d %H:%M:%S')"
    echo "> **브랜치**: $(git branch --show-current)"
    echo "> **커밋**: $(git log -1 --oneline)"
    echo ""
    echo "---"
    echo ""
} > "$OUTPUT_FILE"

print_header

# 1. API 응답 시간 측정
echo -e "${GREEN}1. API 응답 시간 측정...${NC}"
echo ""

{
    echo "## 1. API 응답 시간"
    echo ""
    echo "### 측정 방법"
    echo '```bash'
    echo "ab -n 1000 -c 10 http://localhost:8002/api/v2/judgment/execute"
    echo '```'
    echo ""
    echo "### 결과"
    echo '```'
} >> "$OUTPUT_FILE"

# API 서버 실행 확인
if curl -s http://localhost:8002/health > /dev/null 2>&1; then
    ab -n 1000 -c 10 http://localhost:8002/api/v2/judgment/execute 2>&1 | \
    grep -E "Time per request|Requests per second|Failed requests" | \
    tee -a "$OUTPUT_FILE"
else
    echo "⚠️  API 서버가 실행되지 않음 (localhost:8002)" | tee -a "$OUTPUT_FILE"
    echo "서버 실행 후 다시 측정하세요" | tee -a "$OUTPUT_FILE"
fi

{
    echo '```'
    echo ""
} >> "$OUTPUT_FILE"

echo ""

# 2. 메모리 사용량 측정
echo -e "${GREEN}2. 메모리 사용량 측정...${NC}"
echo ""

{
    echo "## 2. 메모리 사용량"
    echo ""
    echo "### 측정 방법"
    echo '```bash'
    echo "docker stats --no-stream"
    echo '```'
    echo ""
    echo "### 결과"
    echo '```'
} >> "$OUTPUT_FILE"

# Docker 실행 확인
if docker ps > /dev/null 2>&1; then
    docker stats --no-stream --format "table {{.Name}}\t{{.MemUsage}}\t{{.CPUPerc}}" | \
    grep -E "NAME|judgify" | \
    tee -a "$OUTPUT_FILE"
else
    echo "⚠️  Docker가 실행되지 않음" | tee -a "$OUTPUT_FILE"
fi

{
    echo '```'
    echo ""
} >> "$OUTPUT_FILE"

echo ""

# 3. 빌드 시간 측정
echo -e "${GREEN}3. 빌드 시간 측정...${NC}"
echo ""

{
    echo "## 3. 빌드 시간"
    echo ""
    echo "### 측정 방법"
    echo '```bash'
    echo "time npm run build"
    echo '```'
    echo ""
    echo "### 결과"
    echo '```'
} >> "$OUTPUT_FILE"

# package.json 존재 확인
if [ -f "package.json" ]; then
    { time npm run build 2>&1; } 2>&1 | grep -E "real|user|sys" | tee -a "$OUTPUT_FILE"
else
    echo "⚠️  package.json 없음 (JavaScript/TypeScript 프로젝트 아님)" | tee -a "$OUTPUT_FILE"
fi

{
    echo '```'
    echo ""
} >> "$OUTPUT_FILE"

echo ""

# 4. 파일 크기 측정
echo -e "${GREEN}4. 파일 크기 측정...${NC}"
echo ""

{
    echo "## 4. 파일 크기"
    echo ""
    echo "### 측정 항목"
    echo '```bash'
    echo "wc -l CLAUDE.md"
    echo "du -sh dist/"
    echo "du -sh services/"
    echo '```'
    echo ""
    echo "### 결과"
    echo '```'
} >> "$OUTPUT_FILE"

# CLAUDE.md 크기
if [ -f "CLAUDE.md" ]; then
    echo "CLAUDE.md:" | tee -a "$OUTPUT_FILE"
    wc -l CLAUDE.md | tee -a "$OUTPUT_FILE"
else
    echo "⚠️  CLAUDE.md 없음" | tee -a "$OUTPUT_FILE"
fi

echo "" | tee -a "$OUTPUT_FILE"

# 빌드 결과 크기
if [ -d "dist" ]; then
    echo "빌드 결과 (dist/):" | tee -a "$OUTPUT_FILE"
    du -sh dist/ | tee -a "$OUTPUT_FILE"
fi

# 서비스 코드 크기
if [ -d "services" ]; then
    echo "서비스 코드 (services/):" | tee -a "$OUTPUT_FILE"
    du -sh services/ | tee -a "$OUTPUT_FILE"
fi

{
    echo '```'
    echo ""
} >> "$OUTPUT_FILE"

echo ""

# 5. 테스트 커버리지 측정
echo -e "${GREEN}5. 테스트 커버리지 측정...${NC}"
echo ""

{
    echo "## 5. 테스트 커버리지"
    echo ""
    echo "### 측정 방법"
    echo '```bash'
    echo "pytest --cov=. --cov-report=term"
    echo '```'
    echo ""
    echo "### 결과"
    echo '```'
} >> "$OUTPUT_FILE"

# Python 테스트 커버리지
if [ -f "requirements.txt" ] && command -v pytest &> /dev/null; then
    pytest --cov=. --cov-report=term 2>&1 | grep -E "TOTAL|Name|---" | tee -a "$OUTPUT_FILE"
else
    echo "⚠️  pytest 또는 requirements.txt 없음" | tee -a "$OUTPUT_FILE"
fi

{
    echo '```'
    echo ""
} >> "$OUTPUT_FILE"

# JavaScript/TypeScript 테스트 커버리지
if [ -f "package.json" ] && grep -q "test:coverage" package.json; then
    echo "JavaScript/TypeScript 커버리지:" | tee -a "$OUTPUT_FILE"
    npm run test:coverage 2>&1 | grep -E "Statements|Branches|Functions|Lines" | tee -a "$OUTPUT_FILE"
fi

echo ""

# 6. Git 통계 (추가)
echo -e "${GREEN}6. Git 통계...${NC}"
echo ""

{
    echo "## 6. Git 통계"
    echo ""
    echo "### 최근 커밋"
    echo '```'
} >> "$OUTPUT_FILE"

git log -5 --oneline | tee -a "$OUTPUT_FILE"

{
    echo '```'
    echo ""
    echo "### 변경된 파일 (지난 24시간)"
    echo '```'
} >> "$OUTPUT_FILE"

git log --since="24 hours ago" --name-only --pretty=format: | sort | uniq | grep -v "^$" | tee -a "$OUTPUT_FILE" || echo "변경사항 없음" | tee -a "$OUTPUT_FILE"

{
    echo '```'
    echo ""
} >> "$OUTPUT_FILE"

echo ""

# 종합 요약
{
    echo "---"
    echo ""
    echo "## 📋 종합 요약"
    echo ""
    echo "| 항목 | 측정값 | 상태 |"
    echo "|------|--------|------|"
    echo "| API 응답 시간 | {측정값 참조} | - |"
    echo "| 메모리 사용량 | {측정값 참조} | - |"
    echo "| 빌드 시간 | {측정값 참조} | - |"
    echo "| 파일 크기 | {측정값 참조} | - |"
    echo "| 테스트 커버리지 | {측정값 참조} | - |"
    echo ""
    echo "**참고**: 비교 분석을 위해 이 보고서를 보관하세요."
    echo ""
    echo "---"
    echo ""
    echo "**생성 명령어**: \`./scripts/benchmark.sh $OUTPUT_FILE\`"
} >> "$OUTPUT_FILE"

# 완료 메시지
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✅ 벤치마크 완료!${NC}"
echo -e "${YELLOW}보고서 저장 위치: $OUTPUT_FILE${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "다음 단계:"
echo "1. 비교 보고서에 이 데이터 복사"
echo "2. Before/After 버전 비교 분석"
echo "3. 개선 여부 판단 (docs/templates/COMPARISON_TEMPLATE.md 참조)"
echo ""
