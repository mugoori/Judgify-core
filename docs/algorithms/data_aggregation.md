# 데이터 집계 알고리즘 - LLM 할루시네이션 방지 (Ver2.0 Final) 🔥

## 1. 개요

### 1.1 알고리즘 목적
Ver2.0 Final에서는 **ALL 데이터를 영구 보관하면서 LLM에 전달할 데이터는 통계 집계**합니다. 이를 통해 LLM 할루시네이션을 방지하고 정확한 분석 결과를 제공합니다.

### 1.2 핵심 전략
| 전략 | 설명 | 효과 |
|------|------|------|
| **원본 데이터 영구 보관** | raw_data 테이블에 모든 원시 데이터 저장 | 감사 추적, 재분석 가능 |
| **통계 집계 전달** | LLM에는 집계 통계만 전달 (원시 데이터 X) | 할루시네이션 방지 |
| **3단계 집계 전략** | 통계 + 평가 + 트렌드 분석 | 정확성 향상 |
| **검증 메커니즘** | 집계 결과 교차 검증 | 신뢰도 보장 |

### 1.3 데이터 보관 구조
```sql
-- 1. raw_data: 원시 데이터 영구 보관 (절대 삭제 안 함!)
CREATE TABLE raw_data (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workflow_id UUID,
    input_data JSONB NOT NULL,         -- 원시 입력 데이터
    result JSONB,                      -- 원시 결과 데이터
    feedback VARCHAR(10),              -- 👍 / 👎
    created_at TIMESTAMP DEFAULT NOW(),
    CONSTRAINT never_delete CHECK (true)  -- 삭제 방지 제약
);

-- 2. judgment_executions: 최근 90일 데이터 (고속 조회)
CREATE TABLE judgment_executions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workflow_id UUID,
    input_data JSONB,
    result JSONB,
    feedback VARCHAR(10),
    created_at TIMESTAMP DEFAULT NOW()
);

-- 3. archived_judgments: 90일 이상 데이터 (집계 저장)
CREATE TABLE archived_judgments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workflow_id UUID,
    time_period VARCHAR(20),           -- "2025-01", "2025-Q1"
    aggregated_data JSONB NOT NULL,    -- 집계 통계
    sample_count INTEGER,
    created_at TIMESTAMP DEFAULT NOW()
);
```

---

## 2. 3단계 데이터 집계 전략

### 2.1 Stage 1: 통계적 집계 (Statistical Aggregation)
**목적**: 원시 데이터를 기술 통계로 요약

```python
class StatisticalAggregator:
    def aggregate_statistics(
        self,
        raw_data: List[Dict],
        variables: List[str]
    ) -> Dict:
        """
        통계적 집계 수행

        집계 항목:
        - 기본 통계: 평균, 표준편차, 최소/최대, 중앙값
        - 분포 통계: 사분위수, 백분위수
        - 이상치 탐지: IQR 기반 이상치 개수
        """
        aggregated = {
            "total_samples": len(raw_data),
            "variables": {}
        }

        for var in variables:
            values = [d[var] for d in raw_data if var in d]

            if not values:
                continue

            # 수치형 변수
            if isinstance(values[0], (int, float)):
                aggregated["variables"][var] = self._aggregate_numeric(values)

            # 범주형 변수
            else:
                aggregated["variables"][var] = self._aggregate_categorical(values)

        return aggregated

    def _aggregate_numeric(self, values: List[float]) -> Dict:
        """
        수치형 변수 통계 집계
        """
        import numpy as np

        return {
            # 기본 통계
            "mean": float(np.mean(values)),
            "std": float(np.std(values)),
            "min": float(np.min(values)),
            "max": float(np.max(values)),
            "median": float(np.median(values)),

            # 분포 통계
            "q25": float(np.percentile(values, 25)),
            "q50": float(np.percentile(values, 50)),
            "q75": float(np.percentile(values, 75)),

            # 이상치 탐지
            "outlier_count": self._count_outliers(values),
            "outlier_percentage": self._count_outliers(values) / len(values) * 100
        }

    def _aggregate_categorical(self, values: List[str]) -> Dict:
        """
        범주형 변수 통계 집계
        """
        from collections import Counter

        counts = Counter(values)
        total = len(values)

        return {
            "unique_count": len(counts),
            "most_common": counts.most_common(5),  # 상위 5개
            "frequency_distribution": {
                value: count / total for value, count in counts.items()
            }
        }

    def _count_outliers(self, values: List[float]) -> int:
        """
        IQR 기반 이상치 개수 계산
        """
        import numpy as np

        q1 = np.percentile(values, 25)
        q3 = np.percentile(values, 75)
        iqr = q3 - q1

        lower_bound = q1 - 1.5 * iqr
        upper_bound = q3 + 1.5 * iqr

        outliers = [v for v in values if v < lower_bound or v > upper_bound]

        return len(outliers)
```

#### 집계 결과 예시
```json
{
  "total_samples": 100,
  "variables": {
    "temperature": {
      "mean": 86.5,
      "std": 3.2,
      "min": 78.0,
      "max": 95.0,
      "median": 87.0,
      "q25": 84.0,
      "q50": 87.0,
      "q75": 89.0,
      "outlier_count": 3,
      "outlier_percentage": 3.0
    },
    "vibration": {
      "mean": 42.1,
      "std": 2.5,
      "min": 35.0,
      "max": 48.0,
      "median": 42.0,
      "q25": 40.0,
      "q50": 42.0,
      "q75": 44.0,
      "outlier_count": 2,
      "outlier_percentage": 2.0
    }
  }
}
```

### 2.2 Stage 2: 평가적 집계 (Evaluative Aggregation)
**목적**: 피드백 및 성능 지표 집계

```python
class EvaluativeAggregator:
    def aggregate_evaluation(
        self,
        raw_data: List[Dict],
        feedback_key: str = "feedback"
    ) -> Dict:
        """
        평가 지표 집계

        집계 항목:
        - 피드백 분포: 👍 / 👎 비율
        - 정확도 지표: 성공률, 실패율
        - 신뢰도 분포: 고/중/저 신뢰도 비율
        """
        total = len(raw_data)

        # 피드백 분포
        positive_count = sum(1 for d in raw_data if d.get(feedback_key) == "👍")
        negative_count = sum(1 for d in raw_data if d.get(feedback_key) == "👎")
        no_feedback_count = total - positive_count - negative_count

        # 신뢰도 분포 (confidence 0.8 이상 = 고, 0.5-0.8 = 중, 0.5 미만 = 저)
        high_conf = sum(1 for d in raw_data if d.get("confidence", 0) >= 0.8)
        mid_conf = sum(1 for d in raw_data if 0.5 <= d.get("confidence", 0) < 0.8)
        low_conf = sum(1 for d in raw_data if d.get("confidence", 0) < 0.5)

        return {
            "feedback_distribution": {
                "positive": positive_count,
                "negative": negative_count,
                "no_feedback": no_feedback_count,
                "positive_rate": positive_count / total if total > 0 else 0,
                "negative_rate": negative_count / total if total > 0 else 0
            },
            "confidence_distribution": {
                "high": high_conf,
                "medium": mid_conf,
                "low": low_conf,
                "high_rate": high_conf / total if total > 0 else 0,
                "medium_rate": mid_conf / total if total > 0 else 0,
                "low_rate": low_conf / total if total > 0 else 0
            },
            "performance_metrics": {
                "success_rate": positive_count / (positive_count + negative_count) if (positive_count + negative_count) > 0 else 0,
                "avg_confidence": sum(d.get("confidence", 0) for d in raw_data) / total if total > 0 else 0
            }
        }
```

#### 집계 결과 예시
```json
{
  "feedback_distribution": {
    "positive": 85,
    "negative": 15,
    "no_feedback": 0,
    "positive_rate": 0.85,
    "negative_rate": 0.15
  },
  "confidence_distribution": {
    "high": 72,
    "medium": 25,
    "low": 3,
    "high_rate": 0.72,
    "medium_rate": 0.25,
    "low_rate": 0.03
  },
  "performance_metrics": {
    "success_rate": 0.85,
    "avg_confidence": 0.81
  }
}
```

### 2.3 Stage 3: 트렌드 집계 (Trend Aggregation)
**목적**: 시간별 변화 추이 분석

```python
class TrendAggregator:
    def aggregate_trend(
        self,
        raw_data: List[Dict],
        time_key: str = "created_at",
        time_unit: str = "hour"  # hour | day | week | month
    ) -> Dict:
        """
        시계열 트렌드 집계

        집계 항목:
        - 시간대별 샘플 수
        - 시간대별 평균 신뢰도
        - 시간대별 성공률
        - 변화율 (증가/감소 추세)
        """
        import pandas as pd

        # DataFrame 변환
        df = pd.DataFrame(raw_data)
        df[time_key] = pd.to_datetime(df[time_key])

        # 시간 단위별 그룹화
        if time_unit == "hour":
            df["time_group"] = df[time_key].dt.floor("h")
        elif time_unit == "day":
            df["time_group"] = df[time_key].dt.date
        elif time_unit == "week":
            df["time_group"] = df[time_key].dt.to_period("W")
        elif time_unit == "month":
            df["time_group"] = df[time_key].dt.to_period("M")

        # 시간대별 집계
        trend_data = []
        for time_group, group_df in df.groupby("time_group"):
            positive_count = (group_df["feedback"] == "👍").sum()
            total_count = len(group_df)

            trend_data.append({
                "time": str(time_group),
                "sample_count": total_count,
                "avg_confidence": group_df["confidence"].mean(),
                "success_rate": positive_count / total_count if total_count > 0 else 0
            })

        # 변화율 계산
        if len(trend_data) >= 2:
            first_success_rate = trend_data[0]["success_rate"]
            last_success_rate = trend_data[-1]["success_rate"]
            change_rate = (last_success_rate - first_success_rate) / first_success_rate if first_success_rate > 0 else 0
        else:
            change_rate = 0

        return {
            "time_unit": time_unit,
            "trend_data": trend_data,
            "summary": {
                "total_periods": len(trend_data),
                "change_rate": change_rate,
                "trend_direction": "increasing" if change_rate > 0.05 else "decreasing" if change_rate < -0.05 else "stable"
            }
        }
```

#### 집계 결과 예시
```json
{
  "time_unit": "day",
  "trend_data": [
    {
      "time": "2025-10-10",
      "sample_count": 25,
      "avg_confidence": 0.78,
      "success_rate": 0.80
    },
    {
      "time": "2025-10-11",
      "sample_count": 28,
      "avg_confidence": 0.82,
      "success_rate": 0.86
    },
    {
      "time": "2025-10-12",
      "sample_count": 30,
      "avg_confidence": 0.85,
      "success_rate": 0.90
    }
  ],
  "summary": {
    "total_periods": 3,
    "change_rate": 0.125,
    "trend_direction": "increasing"
  }
}
```

---

## 3. 통합 집계 파이프라인

### 3.1 전체 집계 프로세스
```python
class DataAggregationPipeline:
    def __init__(self):
        self.statistical_aggregator = StatisticalAggregator()
        self.evaluative_aggregator = EvaluativeAggregator()
        self.trend_aggregator = TrendAggregator()

    async def aggregate_all(
        self,
        workflow_id: str,
        time_period: str = "last_90_days"
    ) -> Dict:
        """
        전체 데이터 집계 파이프라인

        입력: workflow_id + time_period
        출력: 3단계 집계 결과 통합
        """
        # 1. 원시 데이터 로드 (raw_data 테이블)
        raw_data = await self._load_raw_data(workflow_id, time_period)

        if len(raw_data) < 10:
            raise ValueError(f"샘플 부족: {len(raw_data)}개 (최소 10개)")

        # 2. 변수 목록 추출
        variables = self._extract_variables(raw_data)

        # 3. 3단계 집계 실행
        statistical_result = self.statistical_aggregator.aggregate_statistics(
            raw_data, variables
        )

        evaluative_result = self.evaluative_aggregator.aggregate_evaluation(
            raw_data
        )

        trend_result = self.trend_aggregator.aggregate_trend(
            raw_data, time_unit="day"
        )

        # 4. 통합 결과 생성
        aggregated_result = {
            "workflow_id": workflow_id,
            "time_period": time_period,
            "aggregation_timestamp": datetime.now().isoformat(),
            "sample_count": len(raw_data),
            "statistical_summary": statistical_result,
            "evaluative_summary": evaluative_result,
            "trend_summary": trend_result
        }

        # 5. 검증
        validation_result = self._validate_aggregation(aggregated_result)

        if not validation_result["valid"]:
            raise ValueError(f"집계 검증 실패: {validation_result['errors']}")

        return aggregated_result

    async def _load_raw_data(self, workflow_id: str, time_period: str) -> List[Dict]:
        """
        raw_data 테이블에서 원시 데이터 로드
        """
        if time_period == "last_90_days":
            query = f"""
            SELECT input_data, result, feedback, confidence, created_at
            FROM raw_data
            WHERE workflow_id = '{workflow_id}'
              AND created_at >= NOW() - INTERVAL '90 days'
            ORDER BY created_at DESC
            """
        # ... 기타 time_period 처리

        return await db.fetch_all(query)

    def _validate_aggregation(self, aggregated_result: Dict) -> Dict:
        """
        집계 결과 검증

        검증 항목:
        1. 샘플 수 일치 (통계/평가/트렌드 합산 = 전체)
        2. 비율 합산 = 1.0 (피드백 분포, 신뢰도 분포)
        3. 통계 범위 검증 (평균 < 최대, 평균 > 최소)
        4. 이상치 비율 < 10% (정상 데이터 보장)
        """
        errors = []

        # 1. 샘플 수 일치
        expected_count = aggregated_result["sample_count"]
        feedback_total = (
            aggregated_result["evaluative_summary"]["feedback_distribution"]["positive"] +
            aggregated_result["evaluative_summary"]["feedback_distribution"]["negative"] +
            aggregated_result["evaluative_summary"]["feedback_distribution"]["no_feedback"]
        )

        if expected_count != feedback_total:
            errors.append(f"샘플 수 불일치: {expected_count} != {feedback_total}")

        # 2. 비율 합산 검증
        feedback_rates = (
            aggregated_result["evaluative_summary"]["feedback_distribution"]["positive_rate"] +
            aggregated_result["evaluative_summary"]["feedback_distribution"]["negative_rate"]
        )

        if abs(feedback_rates - 1.0) > 0.01:  # 오차 허용 0.01
            errors.append(f"피드백 비율 합산 오류: {feedback_rates} != 1.0")

        # 3. 통계 범위 검증
        for var, stats in aggregated_result["statistical_summary"]["variables"].items():
            if "mean" in stats:
                if not (stats["min"] <= stats["mean"] <= stats["max"]):
                    errors.append(f"{var} 통계 범위 오류: min={stats['min']}, mean={stats['mean']}, max={stats['max']}")

        # 4. 이상치 비율 검증
        for var, stats in aggregated_result["statistical_summary"]["variables"].items():
            if "outlier_percentage" in stats:
                if stats["outlier_percentage"] > 10:
                    errors.append(f"{var} 이상치 비율 과다: {stats['outlier_percentage']}%")

        return {
            "valid": len(errors) == 0,
            "errors": errors
        }
```

---

## 4. LLM 할루시네이션 방지 메커니즘

### 4.1 할루시네이션 방지 전략
```python
class LLMHallucinationPrevention:
    async def prepare_llm_context(
        self,
        aggregated_data: Dict
    ) -> str:
        """
        LLM에 전달할 안전한 컨텍스트 생성

        할루시네이션 방지:
        1. 원시 데이터 절대 전달 금지 → 집계 통계만 전달
        2. 명확한 제약 조건 명시
        3. 검증 가능한 근거 요구
        4. 불확실성 표현 허용
        """
        context = f"""
## 데이터 분석 컨텍스트

**중요**: 아래 데이터는 통계 집계 결과입니다. 원시 데이터가 아닙니다.
집계 통계에 기반한 분석만 수행하고, 구체적인 개별 사례는 언급하지 마세요.

### 전체 요약
- 총 샘플 수: {aggregated_data['sample_count']}개
- 분석 기간: {aggregated_data['time_period']}
- 집계 시간: {aggregated_data['aggregation_timestamp']}

### 통계적 요약
{json.dumps(aggregated_data['statistical_summary'], indent=2)}

### 평가적 요약
{json.dumps(aggregated_data['evaluative_summary'], indent=2)}

### 트렌드 요약
{json.dumps(aggregated_data['trend_summary'], indent=2)}

## 분석 지침
1. **근거 기반 분석**: 제공된 통계 수치에만 기반하여 분석
2. **불확실성 표현**: 확실하지 않으면 "통계에 따르면 ~인 것으로 보입니다" 표현 사용
3. **개별 사례 금지**: "특정 케이스에서 ~" 같은 표현 금지
4. **수치 인용**: 분석 결과에 반드시 통계 수치 인용

## 금지 사항
❌ 원시 데이터 언급 금지
❌ 구체적 개별 사례 언급 금지
❌ 집계 통계 벗어난 추측 금지
❌ "첫 번째 샘플은 ~", "마지막 케이스는 ~" 표현 금지
"""

        return context

    async def validate_llm_response(
        self,
        llm_response: str,
        aggregated_data: Dict
    ) -> Dict:
        """
        LLM 응답 검증 (할루시네이션 탐지)

        검증 항목:
        1. 제공되지 않은 수치 언급 여부
        2. 개별 사례 언급 여부
        3. 통계 범위 벗어난 주장 여부
        """
        issues = []

        # 1. 제공되지 않은 수치 언급 검증
        mentioned_numbers = self._extract_numbers(llm_response)
        provided_numbers = self._extract_all_numbers(aggregated_data)

        for num in mentioned_numbers:
            if num not in provided_numbers and abs(num - 100) > 0.01:  # 백분율 제외
                issues.append(f"제공되지 않은 수치 언급: {num}")

        # 2. 개별 사례 언급 금지 패턴
        forbidden_patterns = [
            r"첫\s*번째\s*샘플",
            r"마지막\s*케이스",
            r"특정\s*사례",
            r"개별\s*데이터",
            r"\d+번째\s*판단"
        ]

        import re
        for pattern in forbidden_patterns:
            if re.search(pattern, llm_response):
                issues.append(f"개별 사례 언급 패턴 탐지: {pattern}")

        # 3. 통계 범위 벗어난 주장
        for var, stats in aggregated_data["statistical_summary"]["variables"].items():
            if "mean" in stats:
                # LLM이 평균보다 훨씬 높거나 낮은 값을 언급했는지 확인
                # (예: 평균 86.5인데 "대부분 100 이상"이라고 주장)
                pass  # 구현 생략

        return {
            "valid": len(issues) == 0,
            "issues": issues,
            "hallucination_detected": len(issues) > 0
        }
```

### 4.2 안전한 LLM 호출 예시
```python
async def safe_llm_analysis(workflow_id: str) -> Dict:
    """
    할루시네이션 방지된 안전한 LLM 분석
    """
    # 1. 데이터 집계
    pipeline = DataAggregationPipeline()
    aggregated_data = await pipeline.aggregate_all(workflow_id, "last_90_days")

    # 2. 안전한 컨텍스트 생성
    prevention = LLMHallucinationPrevention()
    safe_context = await prevention.prepare_llm_context(aggregated_data)

    # 3. LLM 호출
    response = await openai.ChatCompletion.acreate(
        model="gpt-4o",
        messages=[
            {"role": "system", "content": "너는 통계 기반 데이터 분석가야. 제공된 집계 통계에만 기반하여 분석해."},
            {"role": "user", "content": safe_context}
        ],
        temperature=0.3  # 낮은 temperature = 일관성
    )

    llm_analysis = response.choices[0].message.content

    # 4. 응답 검증
    validation_result = await prevention.validate_llm_response(
        llm_analysis, aggregated_data
    )

    if validation_result["hallucination_detected"]:
        # 할루시네이션 탐지시 재시도 또는 경고
        logger.warning(f"LLM 할루시네이션 탐지: {validation_result['issues']}")

        # 재시도 (더 엄격한 프롬프트)
        safe_context += "\n\n**경고**: 이전 응답에서 제공되지 않은 정보를 언급했습니다. 통계 수치만 인용하세요."
        # ... 재시도 로직

    return {
        "llm_analysis": llm_analysis,
        "validation": validation_result,
        "aggregated_data": aggregated_data
    }
```

---

## 5. 데이터 아카이빙 전략

### 5.1 자동 아카이빙 프로세스
```python
class DataArchiver:
    async def archive_old_data(self):
        """
        90일 이상 데이터 자동 아카이빙

        프로세스:
        1. judgment_executions에서 90일 이상 데이터 조회
        2. 데이터 집계 실행
        3. archived_judgments에 집계 결과 저장
        4. judgment_executions에서 원본 삭제
        5. raw_data는 절대 삭제 안 함!
        """
        # 1. 90일 이상 데이터 조회
        old_data = await db.fetch_all(f"""
            SELECT workflow_id, input_data, result, feedback, confidence, created_at
            FROM judgment_executions
            WHERE created_at < NOW() - INTERVAL '90 days'
        """)

        if not old_data:
            logger.info("아카이빙할 데이터 없음")
            return

        # 2. 워크플로우별로 그룹화
        workflow_groups = {}
        for data in old_data:
            wf_id = data["workflow_id"]
            if wf_id not in workflow_groups:
                workflow_groups[wf_id] = []
            workflow_groups[wf_id].append(data)

        # 3. 워크플로우별 집계 및 아카이빙
        for workflow_id, group_data in workflow_groups.items():
            # 집계 실행
            pipeline = DataAggregationPipeline()
            aggregated = await pipeline.aggregate_all(workflow_id, "archived")

            # 아카이브 저장
            await db.insert("archived_judgments", {
                "workflow_id": workflow_id,
                "time_period": self._get_time_period(group_data[0]["created_at"]),
                "aggregated_data": aggregated,
                "sample_count": len(group_data),
                "created_at": datetime.now()
            })

        # 4. judgment_executions에서 삭제
        await db.execute(f"""
            DELETE FROM judgment_executions
            WHERE created_at < NOW() - INTERVAL '90 days'
        """)

        logger.info(f"아카이빙 완료: {len(workflow_groups)} 워크플로우")

    def _get_time_period(self, timestamp: datetime) -> str:
        """
        타임스탬프를 기간 문자열로 변환

        예시:
        2025-10-16 → "2025-10"
        2025-01-15 → "2025-Q1"
        """
        return timestamp.strftime("%Y-%m")
```

### 5.2 아카이브 데이터 복원
```python
async def restore_archived_data(workflow_id: str, time_period: str) -> Dict:
    """
    아카이브된 집계 데이터 복원

    주의: 원시 데이터는 복원 불가 (집계 통계만 복원)
    """
    archived = await db.fetch_one(f"""
        SELECT aggregated_data
        FROM archived_judgments
        WHERE workflow_id = '{workflow_id}'
          AND time_period = '{time_period}'
    """)

    if not archived:
        raise ValueError(f"아카이브 데이터 없음: {workflow_id} / {time_period}")

    return archived["aggregated_data"]
```

---

## 6. 성능 최적화 및 모니터링

### 6.1 성능 목표
```yaml
집계 성능:
  - 통계 집계: < 1초 (10,000개 샘플)
  - 평가 집계: < 500ms
  - 트렌드 집계: < 1초
  - 전체 집계 파이프라인: < 3초

아카이빙 성능:
  - 일일 자동 아카이빙: < 5분
  - 워크플로우당 아카이빙: < 10초

검증 성능:
  - 집계 검증: < 200ms
  - LLM 응답 검증: < 500ms
```

### 6.2 모니터링 메트릭
```python
# Prometheus 메트릭

aggregation_duration = Histogram(
    'data_aggregation_duration_seconds',
    'Duration of data aggregation',
    ['stage']  # statistical | evaluative | trend
)

archived_records_total = Counter(
    'data_archived_records_total',
    'Total number of archived records',
    ['workflow_id']
)

hallucination_detected_total = Counter(
    'llm_hallucination_detected_total',
    'Total number of hallucination detections',
    ['validation_type']
)

raw_data_size_bytes = Gauge(
    'raw_data_size_bytes',
    'Total size of raw_data table'
)
```

---

## 7. 추가 참조 문서

- **`docs/services/learning_service.md`**: Learning Service 전체 아키텍처
- **`docs/algorithms/auto_rule_extraction.md`**: 자동 Rule 추출 알고리즘 (집계 데이터 활용)
- **`docs/architecture/database_design.md`**: raw_data, archived_judgments 테이블 스키마

---

**Ver2.0 Final 핵심 혁신**: ALL 데이터 영구 보관 + 집계 통계로 LLM 할루시네이션 완벽 방지! 🔥
