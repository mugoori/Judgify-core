# 자동 Rule 추출 알고리즘 (Ver2.0 Final) 🔥

## 1. 개요

### 1.1 알고리즘 목적
사용자 피드백 데이터를 분석하여 **자동으로 Rule을 추출**하는 3가지 전통적 알고리즘입니다. ML 모델 없이 해석 가능한 Rule로 직접 변환하여 즉시 워크플로우에 적용할 수 있습니다.

### 1.2 3가지 알고리즘 비교
| 알고리즘 | 처리 방식 | 장점 | 단점 | 적합한 상황 |
|----------|----------|------|------|------------|
| **빈도 분석** | 패턴 카운팅 + 임계값 | 단순 명확, 빠름 | 복잡한 패턴 놓침 | 명확한 반복 패턴 |
| **결정 트리** | sklearn 학습 + 변환 | 변수 중요도 제공 | 과적합 위험 | 다변수 조건 |
| **LLM 패턴** | 통계 분석 + LLM | 숨겨진 패턴 발견 | LLM 비용 | 복잡한 상관관계 |

### 1.3 알고리즘 선택 전략
```python
def select_algorithm(feedback_data: List[FeedbackData]) -> str:
    """
    데이터 특성에 따라 최적 알고리즘 선택

    선택 기준:
    1. 샘플 수 < 50: 알고리즘 실행 보류
    2. 변수 2개 이하 + 명확한 패턴: 빈도 분석
    3. 변수 3-5개 + 복잡한 조건: 결정 트리
    4. 변수 5개 이상 + 상관관계: LLM 패턴
    5. 불확실할 때: 3가지 모두 실행 후 통합
    """
    sample_count = len(feedback_data)
    variable_count = len(feedback_data[0].input_data.keys())

    if sample_count < 50:
        return "insufficient_data"

    if variable_count <= 2:
        return "frequency_analysis"
    elif variable_count <= 5:
        return "decision_tree"
    elif variable_count > 5:
        return "llm_pattern"
    else:
        return "all"  # 모든 알고리즘 실행 후 통합
```

---

## 2. 알고리즘 1: 빈도 분석 (Frequency Analysis)

### 2.1 알고리즘 원리
**핵심 아이디어**: 사용자 피드백에서 자주 발생하는 패턴(80% 이상)을 찾아 Rule로 변환

### 2.2 상세 처리 프로세스
```python
from collections import Counter
from typing import List, Dict

class FrequencyAnalyzer:
    def __init__(self, threshold: float = 0.80):
        """
        빈도 분석 Rule 추출기

        Args:
            threshold: Rule 후보로 선정할 최소 빈도 (기본: 80%)
        """
        self.threshold = threshold

    def extract_rules(self, feedback_data: List[FeedbackData]) -> List[Rule]:
        """
        빈도 분석 기반 Rule 추출

        처리 단계:
        1. 긍정 피드백만 필터링
        2. 조건 패턴 추출 및 카운팅
        3. 임계값 적용 (80% 이상)
        4. Rule 표현식 생성
        5. 신뢰도 계산
        """
        # 1. 긍정 피드백만 필터링
        positive_data = [d for d in feedback_data if d.feedback == "👍"]

        # 2. 조건 패턴 추출 및 카운팅
        pattern_counts = Counter()
        for data in positive_data:
            pattern = self._extract_pattern(data.input_data)
            pattern_counts[pattern] += 1

        # 3. 임계값 적용
        total_positive = len(positive_data)
        min_count = int(total_positive * self.threshold)

        # 4. Rule 생성
        extracted_rules = []
        for pattern, count in pattern_counts.items():
            if count >= min_count:
                rule = self._create_rule(
                    pattern=pattern,
                    count=count,
                    total=total_positive
                )
                extracted_rules.append(rule)

        # 5. 신뢰도 순 정렬
        extracted_rules.sort(key=lambda r: r.confidence, reverse=True)

        return extracted_rules

    def _extract_pattern(self, input_data: dict) -> str:
        """
        입력 데이터에서 조건 패턴 추출

        예시:
        {"temp": 88, "vib": 42} → "temp>85_vib>40"
        """
        conditions = []
        for key, value in sorted(input_data.items()):
            # 수치형 데이터는 범위로 변환
            if isinstance(value, (int, float)):
                threshold = self._find_threshold(key, value)
                conditions.append(f"{key}>{threshold}")
            # 문자열 데이터는 그대로
            else:
                conditions.append(f"{key}={value}")

        return "_".join(conditions)

    def _find_threshold(self, key: str, value: float) -> float:
        """
        변수별 임계값 자동 계산

        전략:
        - 5의 배수로 반올림 (예: 88 → 85)
        - 데이터 분포 고려 (중앙값 기준)
        """
        return round(value / 5) * 5 - 5

    def _create_rule(self, pattern: str, count: int, total: int) -> Rule:
        """
        패턴을 Rule 객체로 변환

        신뢰도 계산:
        confidence = (count / total) * 0.9
        → 0.9를 곱하는 이유: 과적합 방지
        """
        frequency = count / total
        confidence = frequency * 0.9  # 과적합 방지 보정

        # 패턴을 Rule 표현식으로 변환
        expression = pattern.replace("_", " AND ").replace("=", " == ")

        return Rule(
            expression=expression,
            frequency=frequency,
            confidence=confidence,
            sample_count=count,
            method="frequency_analysis"
        )
```

### 2.3 실제 적용 예시
```python
# 입력 데이터 예시 (제조업 온도/진동 모니터링)
feedback_data = [
    FeedbackData(input={"temp": 88, "vib": 42}, result=True, feedback="👍"),
    FeedbackData(input={"temp": 87, "vib": 43}, result=True, feedback="👍"),
    FeedbackData(input={"temp": 89, "vib": 41}, result=True, feedback="👍"),
    FeedbackData(input={"temp": 86, "vib": 44}, result=True, feedback="👍"),
    # ... 78개 더 (총 82개 유사 패턴)
    FeedbackData(input={"temp": 82, "vib": 38}, result=False, feedback="👍"),
    FeedbackData(input={"temp": 83, "vib": 37}, result=False, feedback="👍"),
    # ... 16개 더 (총 18개 다른 패턴)
]

# Rule 추출 실행
analyzer = FrequencyAnalyzer(threshold=0.80)
rules = analyzer.extract_rules(feedback_data)

# 추출된 Rule
print(rules[0])
# Output:
# Rule(
#   expression="temp > 85 AND vib > 40",
#   frequency=0.82,
#   confidence=0.74,  # 0.82 * 0.9 = 0.738
#   sample_count=82,
#   method="frequency_analysis"
# )
```

### 2.4 장단점 및 적용 상황
**장점**:
- ✅ 구현 단순, 실행 속도 빠름 (< 1초)
- ✅ 해석 용이, 결과 예측 가능
- ✅ 과적합 위험 낮음

**단점**:
- ❌ 복잡한 상호작용 패턴 놓칠 수 있음
- ❌ 변수 간 상관관계 고려 안 됨
- ❌ 희귀 패턴 발견 어려움

**적합한 상황**:
- 변수 2-3개, 명확한 임계값 존재
- 대부분 케이스가 유사한 패턴
- 빠른 Rule 추출 필요

---

## 3. 알고리즘 2: 결정 트리 변환 (Decision Tree Conversion)

### 3.1 알고리즘 원리
**핵심 아이디어**: sklearn DecisionTreeClassifier로 학습한 트리를 Rule 표현식으로 변환

### 3.2 상세 처리 프로세스
```python
from sklearn.tree import DecisionTreeClassifier, _tree
import numpy as np

class DecisionTreeConverter:
    def __init__(self, max_depth: int = 3, min_samples_split: int = 10):
        """
        결정 트리 기반 Rule 추출기

        Args:
            max_depth: 트리 최대 깊이 (해석 가능성 유지)
            min_samples_split: 노드 분할 최소 샘플 수
        """
        self.max_depth = max_depth
        self.min_samples_split = min_samples_split

    def extract_rules(self, feedback_data: List[FeedbackData]) -> List[Rule]:
        """
        결정 트리 학습 → Rule 변환

        처리 단계:
        1. 데이터 준비 (X, y 분리)
        2. 결정 트리 학습 (sklearn)
        3. 리프 노드별 경로 추출
        4. 경로를 Rule 표현식으로 변환
        5. 신뢰도 및 Feature Importance 계산
        """
        # 1. 데이터 준비
        X, y, feature_names = self._prepare_data(feedback_data)

        # 2. 결정 트리 학습
        clf = DecisionTreeClassifier(
            max_depth=self.max_depth,
            min_samples_split=self.min_samples_split,
            random_state=42  # 재현 가능성
        )
        clf.fit(X, y)

        # 3. 트리 구조 분석
        tree = clf.tree_

        # 4. 리프 노드별 Rule 추출
        rules = []
        for leaf_id in self._get_leaf_nodes(tree):
            rule = self._extract_rule_from_path(
                clf, tree, leaf_id, feature_names
            )
            if rule.confidence >= 0.70:  # 신뢰도 필터
                rules.append(rule)

        return rules

    def _prepare_data(self, feedback_data: List[FeedbackData]):
        """
        피드백 데이터를 sklearn 형식으로 변환
        """
        feature_names = list(feedback_data[0].input_data.keys())

        X = []
        y = []
        for data in feedback_data:
            # 입력 특징
            features = [data.input_data[name] for name in feature_names]
            X.append(features)

            # 라벨 (👍=1, 👎=0)
            label = 1 if data.feedback == "👍" else 0
            y.append(label)

        return np.array(X), np.array(y), feature_names

    def _get_leaf_nodes(self, tree) -> List[int]:
        """
        트리에서 리프 노드 ID 목록 추출
        """
        leaf_nodes = []
        for node_id in range(tree.node_count):
            if tree.children_left[node_id] == _tree.TREE_LEAF:
                leaf_nodes.append(node_id)
        return leaf_nodes

    def _extract_rule_from_path(
        self,
        clf: DecisionTreeClassifier,
        tree,
        leaf_id: int,
        feature_names: List[str]
    ) -> Rule:
        """
        리프 노드까지의 경로를 Rule 표현식으로 변환

        예시:
        Node 0: temp <= 85.0 → left
        Node 2: vib <= 40.0 → right
        Node 4: class = True (leaf)

        → Rule: "temp > 85 AND vib > 40"
        """
        # 경로 추적
        path = []
        node_id = 0

        while node_id != leaf_id:
            feature_idx = tree.feature[node_id]
            threshold = tree.threshold[node_id]
            feature_name = feature_names[feature_idx]

            # 리프까지 어느 방향으로 갈지 결정
            if tree.children_left[node_id] == leaf_id or \
               self._is_in_subtree(tree, tree.children_left[node_id], leaf_id):
                # 왼쪽 (<=)
                path.append(f"{feature_name} <= {threshold:.1f}")
                node_id = tree.children_left[node_id]
            else:
                # 오른쪽 (>)
                path.append(f"{feature_name} > {threshold:.1f}")
                node_id = tree.children_right[node_id]

        # Rule 표현식 생성
        expression = " AND ".join(path)

        # 신뢰도 계산
        samples = tree.n_node_samples[leaf_id]
        class_counts = tree.value[leaf_id][0]
        confidence = max(class_counts) / sum(class_counts)

        # Feature Importance
        feature_importance = dict(zip(
            feature_names,
            clf.feature_importances_
        ))

        return Rule(
            expression=expression,
            confidence=confidence,
            sample_count=int(samples),
            method="decision_tree",
            tree_depth=clf.get_depth(),
            feature_importance=feature_importance
        )

    def _is_in_subtree(self, tree, parent: int, target: int) -> bool:
        """
        target 노드가 parent의 서브트리에 있는지 확인
        """
        if parent == target:
            return True

        left = tree.children_left[parent]
        right = tree.children_right[parent]

        if left != _tree.TREE_LEAF and self._is_in_subtree(tree, left, target):
            return True

        if right != _tree.TREE_LEAF and self._is_in_subtree(tree, right, target):
            return True

        return False
```

### 3.3 실제 적용 예시
```python
# 입력 데이터 (100개 샘플)
feedback_data = [...]  # 동일 데이터

# Rule 추출 실행
converter = DecisionTreeConverter(max_depth=3, min_samples_split=10)
rules = converter.extract_rules(feedback_data)

# 추출된 Rule
print(rules[0])
# Output:
# Rule(
#   expression="temp > 85.0 AND vib > 40.0",
#   confidence=0.89,
#   sample_count=78,
#   method="decision_tree",
#   tree_depth=2,
#   feature_importance={
#     "temp": 0.62,  # 온도가 더 중요
#     "vib": 0.38
#   }
# )

# 트리 시각화
from sklearn.tree import export_text
tree_rules = export_text(converter.clf, feature_names=["temp", "vib"])
print(tree_rules)
# Output:
# |--- temp <= 85.0
# |   |--- class: False
# |--- temp > 85.0
# |   |--- vib <= 40.0
# |   |   |--- class: False
# |   |--- vib > 40.0
# |   |   |--- class: True
```

### 3.4 장단점 및 적용 상황
**장점**:
- ✅ 변수 중요도 제공 (Feature Importance)
- ✅ 다변수 조건 자동 발견
- ✅ 비선형 패턴 포착 가능

**단점**:
- ❌ 과적합 위험 (max_depth 조절 필요)
- ❌ 트리 깊이 증가시 해석 어려움
- ❌ 학습 데이터에 민감

**적합한 상황**:
- 변수 3-5개, 복잡한 조건
- 변수 간 상호작용 존재
- Feature Importance 분석 필요

---

## 4. 알고리즘 3: LLM 패턴 발견 (LLM Pattern Discovery)

### 4.1 알고리즘 원리
**핵심 아이디어**: 데이터 집계 통계를 LLM이 분석하여 숨겨진 상관관계 및 패턴 발견

### 4.2 상세 처리 프로세스
```python
import openai
from typing import Dict

class LLMPatternDiscoverer:
    def __init__(self, model: str = "gpt-4o", temperature: float = 0.3):
        """
        LLM 기반 패턴 발견 Rule 추출기

        Args:
            model: OpenAI 모델 (gpt-4o 권장)
            temperature: 일관성 있는 분석 위해 낮게 설정
        """
        self.model = model
        self.temperature = temperature

    async def extract_rules(self, feedback_data: List[FeedbackData]) -> List[Rule]:
        """
        LLM 기반 패턴 발견 Rule 추출

        처리 단계:
        1. 데이터 집계 (통계 요약)
        2. LLM Prompt 생성
        3. LLM 호출 (패턴 분석)
        4. Rule 파싱 및 검증
        """
        # 1. 데이터 집계
        summary = self._aggregate_data(feedback_data)

        # 2. LLM Prompt 생성
        prompt = self._create_prompt(summary)

        # 3. LLM 호출
        response = await openai.ChatCompletion.acreate(
            model=self.model,
            messages=[{"role": "user", "content": prompt}],
            temperature=self.temperature
        )

        # 4. Rule 파싱
        llm_output = response.choices[0].message.content
        rules = self._parse_llm_response(llm_output)

        return rules

    def _aggregate_data(self, feedback_data: List[FeedbackData]) -> Dict:
        """
        피드백 데이터를 통계 요약으로 집계

        집계 항목:
        - 전체 샘플 수
        - 긍정/부정 피드백 수
        - 변수별 평균 (긍정/부정 분리)
        - 변수 간 상관관계
        """
        positive_data = [d for d in feedback_data if d.feedback == "👍"]
        negative_data = [d for d in feedback_data if d.feedback == "👎"]

        # 변수 목록
        variables = list(feedback_data[0].input_data.keys())

        # 변수별 평균 계산
        stats = {}
        for var in variables:
            positive_values = [d.input_data[var] for d in positive_data]
            negative_values = [d.input_data[var] for d in negative_data]

            stats[f"{var}_avg_positive"] = np.mean(positive_values)
            stats[f"{var}_avg_negative"] = np.mean(negative_values)
            stats[f"{var}_std_positive"] = np.std(positive_values)
            stats[f"{var}_std_negative"] = np.std(negative_values)

        # 상관관계 계산 (긍정 피드백 데이터만)
        if len(variables) >= 2:
            correlations = self._calculate_correlations(positive_data, variables)
            stats["correlations"] = correlations

        return {
            "total_samples": len(feedback_data),
            "positive_feedback": len(positive_data),
            "negative_feedback": len(negative_data),
            "statistical_summary": stats
        }

    def _calculate_correlations(self, data: List[FeedbackData], variables: List[str]) -> Dict:
        """
        변수 간 상관관계 계산 (Pearson correlation)
        """
        import pandas as pd

        # DataFrame 생성
        df = pd.DataFrame([d.input_data for d in data])

        # 상관행렬 계산
        corr_matrix = df[variables].corr()

        # 유의미한 상관관계만 추출 (|r| > 0.5)
        correlations = {}
        for i, var1 in enumerate(variables):
            for var2 in variables[i+1:]:
                corr = corr_matrix.loc[var1, var2]
                if abs(corr) > 0.5:
                    correlations[f"{var1}_vs_{var2}"] = corr

        return correlations

    def _create_prompt(self, summary: Dict) -> str:
        """
        LLM에게 전달할 분석 Prompt 생성
        """
        return f"""
너는 제조업 데이터 패턴 발견 전문가야.
아래 통계 요약을 분석해서 숨겨진 Rule을 제안해줘.

## 데이터 집계 요약
{json.dumps(summary, indent=2)}

## 분석 프로세스
1. **통계 분석**: 긍정/부정 피드백 간 변수 차이 발견
2. **상관관계 분석**: 변수 간 관계 파악
3. **패턴 제안**: 발견한 패턴을 자연어로 설명
4. **Rule 생성**: 제안한 패턴을 조건식으로 변환

## 요구 응답 형식 (JSON)
```json
{{
  "analysis": "분석 결과 (1-2문장)",
  "pattern_description": "발견한 패턴 설명 (2-3문장)",
  "extracted_rules": [
    {{
      "rule_expression": "Rule 조건식 (예: temp > 85 AND vib > 40)",
      "reasoning": "이 Rule을 제안한 이유 (통계적 근거)",
      "confidence": 0.0-1.0,
      "method": "llm_pattern_discovery"
    }}
  ]
}}
```

## 주의사항
- 통계적으로 유의미한 차이만 Rule로 제안 (차이 > 5)
- 상관관계가 높은 변수들 (|r| > 0.5)을 함께 고려
- Rule 표현식은 반드시 실행 가능한 Python 조건식 형태로 작성
"""

    def _parse_llm_response(self, llm_output: str) -> List[Rule]:
        """
        LLM 응답 JSON 파싱 및 Rule 객체 변환
        """
        import json

        try:
            data = json.loads(llm_output)
            rules = []

            for rule_data in data["extracted_rules"]:
                rule = Rule(
                    expression=rule_data["rule_expression"],
                    reasoning=rule_data["reasoning"],
                    confidence=rule_data["confidence"],
                    method="llm_pattern_discovery"
                )
                rules.append(rule)

            return rules

        except json.JSONDecodeError:
            # JSON 파싱 실패시 빈 리스트 반환
            return []
```

### 4.3 실제 적용 예시
```python
# 입력 데이터 (100개 샘플)
feedback_data = [...]  # 동일 데이터

# Rule 추출 실행
discoverer = LLMPatternDiscoverer(model="gpt-4o", temperature=0.3)
rules = await discoverer.extract_rules(feedback_data)

# LLM 분석 결과
# {
#   "analysis": "긍정 피드백 케이스에서 온도와 진동이 모두 높은 경향",
#   "pattern_description": "긍정 케이스의 온도 평균 87.5도, 진동 평균 43.2로 부정 케이스보다 각각 5.2도, 4.5 높음. 온도와 진동의 상관관계 0.72로 강한 양의 상관관계 확인.",
#   "extracted_rules": [...]
# }

# 추출된 Rule
print(rules[0])
# Output:
# Rule(
#   expression="temp > 85 AND vib > 40",
#   reasoning="긍정 피드백 케이스에서 temp 평균 87.5, vib 평균 43.2로 부정 케이스보다 각각 5.2, 4.5 높음. 상관관계 0.72로 두 변수가 함께 움직임.",
#   confidence=0.83,
#   method="llm_pattern_discovery"
# )
```

### 4.4 장단점 및 적용 상황
**장점**:
- ✅ 숨겨진 복잡한 패턴 발견
- ✅ 상관관계 및 인과관계 분석
- ✅ 자연어 설명 제공 (해석성)

**단점**:
- ❌ LLM API 비용 발생
- ❌ 응답 시간 느림 (1-2초)
- ❌ LLM 결과 검증 필요

**적합한 상황**:
- 변수 5개 이상, 복잡한 상관관계
- 기존 알고리즘으로 패턴 발견 실패
- 설명 가능성 중요

---

## 5. 3가지 알고리즘 통합 전략

### 5.1 통합 로직
```python
class RuleIntegrator:
    def integrate_rules(
        self,
        freq_rules: List[Rule],
        tree_rules: List[Rule],
        llm_rules: List[Rule]
    ) -> Rule:
        """
        3가지 알고리즘 결과 통합 → 최적 Rule 선택

        통합 전략:
        1. 동일 Rule 표현식 찾기 (일치율 계산)
        2. 신뢰도 가중 평균 계산
        3. 샘플 수 합산
        4. 최종 신뢰도 가장 높은 Rule 선택
        """
        all_rules = freq_rules + tree_rules + llm_rules

        # Rule 표현식 정규화 (공백, 대소문자 통일)
        normalized_rules = self._normalize_rules(all_rules)

        # Rule 그룹화
        rule_groups = {}
        for rule in normalized_rules:
            expr = rule.expression
            if expr not in rule_groups:
                rule_groups[expr] = []
            rule_groups[expr].append(rule)

        # 각 그룹별 통합 Rule 생성
        integrated_rules = []
        for expr, rules in rule_groups.items():
            integrated = self._create_integrated_rule(expr, rules)
            integrated_rules.append(integrated)

        # 최고 신뢰도 Rule 반환
        best_rule = max(integrated_rules, key=lambda r: r.confidence)

        return best_rule

    def _normalize_rules(self, rules: List[Rule]) -> List[Rule]:
        """
        Rule 표현식 정규화

        예시:
        "temp>85 AND vib>40"
        "temp > 85 and vib > 40"
        "temp > 85 AND vibration > 40"
        → "temp > 85 AND vib > 40" (통일)
        """
        normalized = []
        for rule in rules:
            expr = rule.expression.upper()  # 대문자 통일
            expr = re.sub(r'\s+', ' ', expr)  # 공백 정리
            expr = expr.replace(' AND ', ' AND ')  # AND 통일

            # 변수명 약어 통일 (vibration → vib)
            expr = self._standardize_variable_names(expr)

            rule.expression = expr
            normalized.append(rule)

        return normalized

    def _create_integrated_rule(self, expression: str, rules: List[Rule]) -> Rule:
        """
        동일 표현식의 Rule들을 통합

        신뢰도 계산:
        - 가중 평균 (샘플 수로 가중)
        - 일치율 보너스 (3개 알고리즘 모두 제안시 +0.05)
        """
        # 가중 평균 신뢰도
        total_samples = sum(r.sample_count for r in rules)
        weighted_confidence = sum(
            r.confidence * r.sample_count for r in rules
        ) / total_samples

        # 일치율 계산
        agreement_level = len(rules) / 3.0  # 3개 알고리즘 중 일치율

        # 일치율 보너스
        if agreement_level == 1.0:  # 3개 모두 일치
            weighted_confidence = min(weighted_confidence + 0.05, 1.0)

        # 통합 Rule 생성
        integrated_rule = Rule(
            expression=expression,
            confidence=weighted_confidence,
            sample_count=total_samples,
            method="integrated",
            methods_used=[r.method for r in rules],
            agreement_level=agreement_level
        )

        return integrated_rule
```

### 5.2 통합 예시
```python
# 3가지 알고리즘 결과
freq_rules = [
    Rule(expression="temp > 85 AND vib > 40", confidence=0.74, sample_count=82)
]
tree_rules = [
    Rule(expression="temp > 85.0 AND vib > 40.0", confidence=0.89, sample_count=78)
]
llm_rules = [
    Rule(expression="temp > 85 AND vib > 40", confidence=0.83, sample_count=85)
]

# 통합 실행
integrator = RuleIntegrator()
best_rule = integrator.integrate_rules(freq_rules, tree_rules, llm_rules)

# 통합 결과
print(best_rule)
# Output:
# Rule(
#   expression="temp > 85 AND vib > 40",
#   confidence=0.87,  # (0.74*82 + 0.89*78 + 0.83*85) / (82+78+85) + 0.05(보너스)
#   sample_count=245,  # 82 + 78 + 85
#   method="integrated",
#   methods_used=["frequency_analysis", "decision_tree", "llm_pattern"],
#   agreement_level=1.0  # 3개 알고리즘 모두 일치!
# )
```

---

## 6. 성능 최적화 및 모니터링

### 6.1 성능 목표
```yaml
알고리즘별 실행 시간:
  - 빈도 분석: < 1초
  - 결정 트리: < 2초
  - LLM 패턴: < 3초 (LLM API 대기 포함)
  - 3가지 병렬 실행: < 3초 (병렬 처리)
  - 통합 로직: < 500ms

정확도 목표:
  - Rule 추출 정확도: 85% 이상
  - 알고리즘 일치율: 70% 이상 (2개 이상 일치)
  - 통합 Rule 신뢰도: 0.80 이상
```

### 6.2 에러 처리 및 폴백
```python
async def extract_rules_with_fallback(
    feedback_data: List[FeedbackData]
) -> List[Rule]:
    """
    에러 처리 및 폴백 전략

    우선순위:
    1. 3가지 모두 실행 (병렬) → 통합
    2. 1-2개 실패시 성공한 알고리즘만 사용
    3. 모두 실패시 빈도 분석 단독 실행 (가장 안전)
    """
    results = {
        "frequency": None,
        "decision_tree": None,
        "llm_pattern": None
    }

    # 1. 3가지 알고리즘 병렬 실행
    try:
        freq_task = frequency_analysis(feedback_data)
        tree_task = decision_tree_conversion(feedback_data)
        llm_task = llm_pattern_discovery(feedback_data)

        freq_rules, tree_rules, llm_rules = await asyncio.gather(
            freq_task, tree_task, llm_task,
            return_exceptions=True
        )

        # 성공한 결과만 저장
        if not isinstance(freq_rules, Exception):
            results["frequency"] = freq_rules
        if not isinstance(tree_rules, Exception):
            results["decision_tree"] = tree_rules
        if not isinstance(llm_rules, Exception):
            results["llm_pattern"] = llm_rules

    except Exception as e:
        logger.error(f"All algorithms failed: {e}")

    # 2. 통합 실행
    successful_results = [r for r in results.values() if r is not None]

    if len(successful_results) >= 2:
        # 2개 이상 성공 → 통합
        return integrate_rules(*successful_results)

    elif len(successful_results) == 1:
        # 1개만 성공 → 단독 사용
        return successful_results[0]

    else:
        # 모두 실패 → 빈도 분석 폴백
        logger.warning("All algorithms failed, fallback to frequency analysis")
        return await frequency_analysis(feedback_data)
```

---

## 7. 추가 참조 문서

- **`docs/services/learning_service.md`**: Learning Service 전체 아키텍처
- **`docs/algorithms/data_aggregation.md`**: 데이터 집계 알고리즘 (LLM 할루시네이션 방지)
- **`docs/architecture/database_design.md`**: feedback_data 테이블 스키마

---

**Ver2.0 Final 핵심 혁신**: ML 모델 없이 전통적 알고리즘 3가지로 해석 가능한 Rule 자동 추출! 🔥
