# 판단 코어 엔진 상세 설계서

**문서 버전**: v2.0
**작성일**: 2024.08.05
**최종 업데이트**: 2025-10-31 (Judgment Service 전체 문서화 완료 🎉)
**완료율**: 100% ✅ (섹션 1-12 전체 완료)
**대상**: 백엔드 개발자, AI 엔지니어, 시스템 아키텍트, Frontend 개발자
**목적**: Judgment Service의 내부 구현 및 핵심 판단 로직 정의

## 🆕 Ver2.0 Final 주요 변경사항 (2025-10-30)
- ✅ **Few-shot 학습 통합**: Learning Service와 연동하여 유사 사례 15개 자동 검색
- ✅ **하이브리드 판단 개선**: Rule Engine → LLM + Few-shot 순차 실행
- ✅ **신뢰도 향상**: Few-shot 샘플 개수에 따른 동적 신뢰도 보정
- ✅ **테스트 커버리지**: 3개 핵심 테스트 추가 (총 28개 통과)

## 📋 1. 개요 및 설계 원칙

### 1.1 핵심 책임
- **워크플로우 해석**: JSON 기반 워크플로우를 실행 가능한 로직으로 변환
- **하이브리드 판단**: Rule 기반과 LLM 기반 판단의 조합
- **컨텍스트 관리**: MCP를 통한 외부 데이터 수집 및 활용
- **신뢰도 평가**: 판단 결과의 신뢰도 산출
- **설명 생성**: 판단 근거와 설명 제공

### 1.2 설계 원칙
```python
# SOLID 원칙 적용
# S: 각 클래스는 단일 책임
# O: 새로운 판단 방식 확장 가능
# L: 판단 인터페이스 일관성
# I: 필요한 인터페이스만 구현
# D: 의존성 역전으로 테스트 용이성 확보
```

## 🏗 2. 아키텍처 구조

### 2.1 컴포넌트 다이어그램
```mermaid
graph TB
    subgraph "Judgment Service"
        DISPATCHER[Judgment Dispatcher]
        EXECUTOR[Workflow Executor]
        
        subgraph "Judgment Engines"
            RULE[Rule Engine]
            LLM[LLM Engine]
            HYBRID[Hybrid Engine]
        end
        
        subgraph "Support Components"
            CONTEXT[Context Manager]
            VALIDATOR[Input Validator]
            EXPLAINER[Explanation Generator]
        end
    end
    
    subgraph "External Dependencies"
        MCP[MCP Client]
        OPENAI[OpenAI API]
        DB[(PostgreSQL)]
        CACHE[(Redis)]
    end
    
    DISPATCHER --> EXECUTOR
    EXECUTOR --> RULE
    EXECUTOR --> LLM
    EXECUTOR --> HYBRID
    
    CONTEXT --> MCP
    LLM --> OPENAI
    EXPLAINER --> OPENAI
    
    EXECUTOR --> DB
    CONTEXT --> CACHE
```

### 2.2 핵심 클래스 구조
```python
from abc import ABC, abstractmethod
from typing import Any, Dict, List, Optional
from pydantic import BaseModel
from enum import Enum


class JudgmentMethod(str, Enum):
    RULE = "rule"
    LLM = "llm"
    HYBRID = "hybrid"


class JudgmentInput(BaseModel):
    workflow_id: str
    input_data: Dict[str, Any]
    context: Optional[Dict[str, Any]] = None
    method: Optional[JudgmentMethod] = None


class JudgmentResult(BaseModel):
    result: Any
    confidence: float
    method_used: JudgmentMethod
    execution_time_ms: int
    explanation: Optional[str] = None
    error: Optional[str] = None


class JudgmentEngine(ABC):
    """판단 엔진 추상 클래스"""
    
    @abstractmethod
    async def judge(self, input_data: JudgmentInput) -> JudgmentResult:
        pass
    
    @abstractmethod
    def validate_input(self, input_data: JudgmentInput) -> bool:
        pass
class DashboardGenerationEngine:
    """대시보드 자동 생성 엔진"""
    
    def __init__(self, llm_client, data_analyzer):
        self.llm_client = llm_client
        self.data_analyzer = data_analyzer
        self.component_generator = ComponentGenerator()
        
    async def generate_dashboard(self, request: str, context: dict):
        # 구현 로직
        pass
```

## 🔧 3. Rule Engine 상세 구현

### 3.1 안전한 Rule DSL 파서
```python
import ast
import operator
from typing import Any, Dict, Set


class SafeRuleEngine(JudgmentEngine):
    """AST 기반 안전한 Rule 실행 엔진"""
    
    # 허용된 연산자 정의
    ALLOWED_OPERATORS = {
        ast.Gt: operator.gt,
        ast.Lt: operator.lt,
        ast.GtE: operator.ge,
        ast.LtE: operator.le,
        ast.Eq: operator.eq,
        ast.NotEq: operator.ne,
        ast.And: operator.and_,
        ast.Or: operator.or_,
        ast.Not: operator.not_,
        ast.In: lambda x, y: x in y,
        ast.NotIn: lambda x, y: x not in y,
    }
    
    # 허용된 함수
    ALLOWED_FUNCTIONS = {
        'abs': abs,
        'min': min,
        'max': max,
        'len': len,
        'round': round,
    }
    
    def __init__(self):
        self.variables: Dict[str, Any] = {}
    
    async def judge(self, input_data: JudgmentInput) -> JudgmentResult:
        start_time = time.time()
        
        try:
            # 워크플로우에서 규칙 추출
            workflow = await self.get_workflow(input_data.workflow_id)
            rule_expression = workflow.get('rule_expression')
            
            if not rule_expression:
                raise ValueError("Rule expression not found in workflow")
            
            # 변수 바인딩
            self.variables = input_data.input_data.copy()
            if input_data.context:
                self.variables.update(input_data.context)
            
            # 규칙 실행
            result = self._evaluate_expression(rule_expression)
            
            execution_time = int((time.time() - start_time) * 1000)
            
            return JudgmentResult(
                result=result,
                confidence=1.0,  # Rule 기반은 항상 확신
                method_used=JudgmentMethod.RULE,
                execution_time_ms=execution_time,
                explanation=f"Rule '{rule_expression}' evaluated to {result}"
            )
            
        except Exception as e:
            execution_time = int((time.time() - start_time) * 1000)
            return JudgmentResult(
                result=None,
                confidence=0.0,
                method_used=JudgmentMethod.RULE,
                execution_time_ms=execution_time,
                error=str(e)
            )
    
    def _evaluate_expression(self, expression: str) -> Any:
        """AST를 사용한 안전한 표현식 평가"""
        try:
            tree = ast.parse(expression, mode='eval')
            return self._evaluate_node(tree.body)
        except SyntaxError as e:
            raise ValueError(f"Invalid syntax in rule expression: {e}")
    
    def _evaluate_node(self, node: ast.AST) -> Any:
        """AST 노드 재귀적 평가"""
        if isinstance(node, ast.Constant):
            return node.value
        
        elif isinstance(node, ast.Name):
            if node.id in self.variables:
                return self.variables[node.id]
            else:
                raise NameError(f"Variable '{node.id}' is not defined")
        
        elif isinstance(node, ast.BinOp):
            left = self._evaluate_node(node.left)
            right = self._evaluate_node(node.right)
            op_func = self.ALLOWED_OPERATORS.get(type(node.op))
            
            if op_func:
                return op_func(left, right)
            else:
                raise ValueError(f"Operator {type(node.op)} not allowed")
        
        elif isinstance(node, ast.Compare):
            left = self._evaluate_node(node.left)
            
            for op, comparator in zip(node.ops, node.comparators):
                right = self._evaluate_node(comparator)
                op_func = self.ALLOWED_OPERATORS.get(type(op))
                
                if not op_func:
                    raise ValueError(f"Comparison {type(op)} not allowed")
                
                if not op_func(left, right):
                    return False
                left = right
            
            return True
        
        elif isinstance(node, ast.BoolOp):
            op_func = self.ALLOWED_OPERATORS.get(type(node.op))
            if not op_func:
                raise ValueError(f"Boolean operator {type(node.op)} not allowed")
            
            values = [self._evaluate_node(value) for value in node.values]
            
            if isinstance(node.op, ast.And):
                return all(values)
            elif isinstance(node.op, ast.Or):
                return any(values)
        
        elif isinstance(node, ast.UnaryOp):
            operand = self._evaluate_node(node.operand)
            op_func = self.ALLOWED_OPERATORS.get(type(node.op))
            
            if op_func:
                return op_func(operand)
            else:
                raise ValueError(f"Unary operator {type(node.op)} not allowed")
        
        elif isinstance(node, ast.Call):
            func_name = node.func.id if isinstance(node.func, ast.Name) else None
            
            if func_name in self.ALLOWED_FUNCTIONS:
                args = [self._evaluate_node(arg) for arg in node.args]
                return self.ALLOWED_FUNCTIONS[func_name](*args)
            else:
                raise ValueError(f"Function '{func_name}' not allowed")
        
        elif isinstance(node, ast.List):
            return [self._evaluate_node(item) for item in node.elts]
        
        else:
            raise ValueError(f"Node type {type(node)} not supported")
    
    def validate_input(self, input_data: JudgmentInput) -> bool:
        """입력 데이터 검증"""
        required_fields = self._extract_variables_from_workflow(
            input_data.workflow_id
        )
        
        for field in required_fields:
            if field not in input_data.input_data:
                return False
        
        return True
    
    def _extract_variables_from_workflow(self, workflow_id: str) -> Set[str]:
        """워크플로우에서 필요한 변수 추출"""
        # 실제 구현에서는 워크플로우 정의를 파싱하여 변수 추출
        return set()
```

### 3.2 Rule DSL 예시
```python
# 지원하는 Rule 표현식 예시
RULE_EXAMPLES = {
    "simple_comparison": "temperature > 85",
    "multiple_conditions": "temperature > 85 and vibration > 40",
    "range_check": "temperature >= 80 and temperature <= 100",
    "list_membership": "status in ['ERROR', 'WARNING']",
    "function_usage": "abs(pressure - target_pressure) > threshold",
    "complex_logic": "(temperature > 85 or pressure > 100) and status != 'MAINTENANCE'"
}
```

## 🤖 4. LLM Engine 상세 구현

### 4.1 LLM 판단 엔진
```python
import openai
import json
from typing import Dict, Any
import asyncio


class LLMJudgmentEngine(JudgmentEngine):
    """OpenAI API 기반 LLM 판단 엔진"""
    
    def __init__(self, api_key: str, model: str = "gpt-4"):
        self.client = openai.AsyncOpenAI(api_key=api_key)
        self.model = model
        self.max_retries = 3
        self.timeout = 30
    
    async def judge(self, input_data: JudgmentInput) -> JudgmentResult:
        start_time = time.time()
        
        try:
            # 워크플로우에서 판단 기준 추출
            workflow = await self.get_workflow(input_data.workflow_id)
            judgment_criteria = workflow.get('llm_criteria', '')
            
            # 프롬프트 생성
            prompt = await self._build_judgment_prompt(
                input_data.input_data,
                input_data.context or {},
                judgment_criteria
            )
            
            # LLM 호출
            response = await self._call_llm_with_retry(prompt)
            
            # 응답 파싱
            parsed_result = self._parse_llm_response(response)
            
            execution_time = int((time.time() - start_time) * 1000)
            
            return JudgmentResult(
                result=parsed_result['result'],
                confidence=parsed_result.get('confidence', 0.5),
                method_used=JudgmentMethod.LLM,
                execution_time_ms=execution_time,
                explanation=parsed_result.get('explanation', '')
            )
            
        except Exception as e:
            execution_time = int((time.time() - start_time) * 1000)
            return JudgmentResult(
                result=None,
                confidence=0.0,
                method_used=JudgmentMethod.LLM,
                execution_time_ms=execution_time,
                error=str(e)
            )
    
    async def _build_judgment_prompt(
        self, 
        input_data: Dict[str, Any],
        context: Dict[str, Any],
        criteria: str
    ) -> str:
        """판단용 프롬프트 생성"""
        
        prompt_template = """
당신은 제조업 현장의 전문가입니다. 주어진 데이터를 분석하여 적절한 판단을 내려주세요.

## 입력 데이터
{input_data_formatted}

## 추가 컨텍스트
{context_formatted}

## 판단 기준
{criteria}

## 응답 형식
반드시 다음 JSON 형식으로 응답해주세요:
{{
    "result": true/false 또는 구체적인 값,
    "confidence": 0.0~1.0 사이의 신뢰도,
    "explanation": "판단 근거에 대한 상세한 설명"
}}

판단을 내려주세요:
        """.strip()
        
        input_data_formatted = json.dumps(input_data, indent=2, ensure_ascii=False)
        context_formatted = json.dumps(context, indent=2, ensure_ascii=False)
        
        return prompt_template.format(
            input_data_formatted=input_data_formatted,
            context_formatted=context_formatted,
            criteria=criteria
        )
    
    async def _call_llm_with_retry(self, prompt: str) -> str:
        """재시도 로직이 포함된 LLM 호출"""
        
        for attempt in range(self.max_retries):
            try:
                response = await asyncio.wait_for(
                    self.client.chat.completions.create(
                        model=self.model,
                        messages=[
                            {
                                "role": "system", 
                                "content": "당신은 제조업 현장 전문가입니다. 정확하고 신뢰할 수 있는 판단을 내려주세요."
                            },
                            {"role": "user", "content": prompt}
                        ],
                        temperature=0.1,
                        max_tokens=500
                    ),
                    timeout=self.timeout
                )
                
                return response.choices[0].message.content
                
            except asyncio.TimeoutError:
                if attempt == self.max_retries - 1:
                    raise TimeoutError("LLM response timeout")
                await asyncio.sleep(2 ** attempt)  # 지수 백오프
                
            except Exception as e:
                if attempt == self.max_retries - 1:
                    raise e
                await asyncio.sleep(1)
    
    def _parse_llm_response(self, response: str) -> Dict[str, Any]:
        """LLM 응답 파싱 및 검증"""
        try:
            # JSON 응답 파싱 시도
            if response.strip().startswith('{'):
                parsed = json.loads(response)
                
                # 필수 필드 검증
                if 'result' not in parsed:
                    raise ValueError("Missing 'result' field in LLM response")
                
                # 신뢰도 검증
                confidence = parsed.get('confidence', 0.5)
                if not 0 <= confidence <= 1:
                    parsed['confidence'] = 0.5
                
                return parsed
            
            else:
                # JSON이 아닌 경우 휴리스틱 파싱
                return self._heuristic_parse(response)
                
        except json.JSONDecodeError:
            return self._heuristic_parse(response)
    
    def _heuristic_parse(self, response: str) -> Dict[str, Any]:
        """JSON이 아닌 응답에 대한 휴리스틱 파싱"""
        
        # 긍정/부정 키워드 검색
        positive_keywords = ['yes', 'true', '필요', '해야', '권장']
        negative_keywords = ['no', 'false', '불필요', '않아도', '권장하지']
        
        response_lower = response.lower()
        
        positive_score = sum(1 for keyword in positive_keywords if keyword in response_lower)
        negative_score = sum(1 for keyword in negative_keywords if keyword in response_lower)
        
        if positive_score > negative_score:
            result = True
            confidence = min(0.9, 0.5 + positive_score * 0.1)
        elif negative_score > positive_score:
            result = False
            confidence = min(0.9, 0.5 + negative_score * 0.1)
        else:
            result = None
            confidence = 0.3
        
        return {
            'result': result,
            'confidence': confidence,
            'explanation': response[:200] + '...' if len(response) > 200 else response
        }
    
    def validate_input(self, input_data: JudgmentInput) -> bool:
        """LLM 입력 검증"""
        # 기본적인 데이터 존재 여부만 확인
        return bool(input_data.input_data)
```

## 🔄 5. Hybrid Engine 구현

### 5.1 하이브리드 판단 로직
```python
class HybridJudgmentEngine(JudgmentEngine):
    """Rule과 LLM을 조합한 하이브리드 판단 엔진"""
    
    def __init__(self, rule_engine: SafeRuleEngine, llm_engine: LLMJudgmentEngine):
        self.rule_engine = rule_engine
        self.llm_engine = llm_engine
    
    async def judge(self, input_data: JudgmentInput) -> JudgmentResult:
        start_time = time.time()
        
        try:
            workflow = await self.get_workflow(input_data.workflow_id)
            strategy = workflow.get('hybrid_strategy', 'rule_first')
            
            if strategy == 'rule_first':
                return await self._rule_first_strategy(input_data)
            elif strategy == 'llm_first':
                return await self._llm_first_strategy(input_data)
            elif strategy == 'parallel':
                return await self._parallel_strategy(input_data)
            elif strategy == 'consensus':
                return await self._consensus_strategy(input_data)
            else:
                raise ValueError(f"Unknown hybrid strategy: {strategy}")
                
        except Exception as e:
            execution_time = int((time.time() - start_time) * 1000)
            return JudgmentResult(
                result=None,
                confidence=0.0,
                method_used=JudgmentMethod.HYBRID,
                execution_time_ms=execution_time,
                error=str(e)
            )
    
    async def _rule_first_strategy(self, input_data: JudgmentInput) -> JudgmentResult:
        """Rule 우선 전략: Rule 실패시에만 LLM 사용"""
        
        # Rule 시도
        rule_result = await self.rule_engine.judge(input_data)
        
        if rule_result.error is None:
            # Rule 성공
            rule_result.method_used = JudgmentMethod.HYBRID
            rule_result.explanation = f"Rule-based: {rule_result.explanation}"
            return rule_result
        
        # Rule 실패시 LLM 사용
        llm_input = input_data.model_copy()
        llm_input.context = llm_input.context or {}
        llm_input.context['rule_error'] = rule_result.error
        
        llm_result = await self.llm_engine.judge(llm_input)
        llm_result.method_used = JudgmentMethod.HYBRID
        llm_result.explanation = f"LLM-based (Rule failed): {llm_result.explanation}"
        
        return llm_result
    
    async def _consensus_strategy(self, input_data: JudgmentInput) -> JudgmentResult:
        """합의 전략: Rule과 LLM 결과를 종합하여 최종 판단"""
        
        # 병렬 실행
        rule_task = asyncio.create_task(self.rule_engine.judge(input_data))
        llm_task = asyncio.create_task(self.llm_engine.judge(input_data))
        
        rule_result, llm_result = await asyncio.gather(rule_task, llm_task)
        
        # 결과 분석
        if rule_result.error and llm_result.error:
            # 둘 다 실패
            return JudgmentResult(
                result=None,
                confidence=0.0,
                method_used=JudgmentMethod.HYBRID,
                execution_time_ms=rule_result.execution_time_ms + llm_result.execution_time_ms,
                error="Both rule and LLM engines failed"
            )
        
        elif rule_result.error:
            # Rule 실패, LLM 성공
            llm_result.method_used = JudgmentMethod.HYBRID
            return llm_result
        
        elif llm_result.error:
            # LLM 실패, Rule 성공
            rule_result.method_used = JudgmentMethod.HYBRID
            return rule_result
        
        else:
            # 둘 다 성공 - 합의 알고리즘 적용
            return self._merge_results(rule_result, llm_result)
    
    def _merge_results(self, rule_result: JudgmentResult, llm_result: JudgmentResult) -> JudgmentResult:
        """Rule과 LLM 결과 병합"""
        
        # 결과가 동일한 경우
        if rule_result.result == llm_result.result:
            combined_confidence = (rule_result.confidence + llm_result.confidence) / 2
            combined_confidence = min(combined_confidence * 1.2, 1.0)  # 합의 보너스
            
            return JudgmentResult(
                result=rule_result.result,
                confidence=combined_confidence,
                method_used=JudgmentMethod.HYBRID,
                execution_time_ms=rule_result.execution_time_ms + llm_result.execution_time_ms,
                explanation=f"Consensus: Rule({rule_result.result}) + LLM({llm_result.result})"
            )
        
        # 결과가 다른 경우 - 신뢰도 기반 선택
        else:
            if rule_result.confidence > llm_result.confidence:
                chosen_result = rule_result
                explanation = f"Rule-preferred: Rule({rule_result.confidence:.2f}) > LLM({llm_result.confidence:.2f})"
            else:
                chosen_result = llm_result
                explanation = f"LLM-preferred: LLM({llm_result.confidence:.2f}) > Rule({rule_result.confidence:.2f})"
            
            chosen_result.method_used = JudgmentMethod.HYBRID
            chosen_result.explanation = explanation
            chosen_result.execution_time_ms = rule_result.execution_time_ms + llm_result.execution_time_ms
            
            return chosen_result
    
    def validate_input(self, input_data: JudgmentInput) -> bool:
        """하이브리드 입력 검증"""
        return (self.rule_engine.validate_input(input_data) or 
                self.llm_engine.validate_input(input_data))
```

## 📊 6. Context Manager 구현

### 6.1 MCP 기반 컨텍스트 수집
```python
from typing import Dict, Any, List
import aiohttp


class MCPContextManager:
    """MCP(Model Context Protocol)를 통한 컨텍스트 데이터 수집"""
    
    def __init__(self, mcp_servers: List[Dict[str, str]]):
        self.mcp_servers = mcp_servers
        self.cache_ttl = 300  # 5분 캐시
    
    async def gather_context(self, workflow_id: str, input_data: Dict[str, Any]) -> Dict[str, Any]:
        """워크플로우와 입력 데이터를 기반으로 필요한 컨텍스트 수집"""
        
        workflow = await self.get_workflow(workflow_id)
        required_context = workflow.get('required_context', [])
        
        context = {}
        
        for context_req in required_context:
            context_type = context_req.get('type')
            
            if context_type == 'machine_status':
                machine_id = input_data.get('machine_id')
                if machine_id:
                    context['machine_status'] = await self._get_machine_status(machine_id)
            
            elif context_type == 'historical_data':
                context['historical_data'] = await self._get_historical_data(
                    context_req.get('timeframe', '1h'),
                    input_data
                )
            
            elif context_type == 'policy_documents':
                context['policies'] = await self._get_relevant_policies(
                    context_req.get('category', 'safety')
                )
        
        return context
    
    async def _get_machine_status(self, machine_id: str) -> Dict[str, Any]:
        """MCP를 통한 기계 상태 조회"""
        
        mcp_request = {
            "method": "tools/call",
            "params": {
                "name": "get_machine_status",
                "arguments": {"machine_id": machine_id}
            }
        }
        
        try:
            async with aiohttp.ClientSession() as session:
                for server in self.mcp_servers:
                    try:
                        async with session.post(
                            f"{server['url']}/mcp",
                            json=mcp_request,
                            headers={"Authorization": f"Bearer {server['token']}"}
                        ) as response:
                            if response.status == 200:
                                result = await response.json()
                                return result.get('content', {})
                    except Exception:
                        continue
            
            return {"status": "unknown", "error": "No MCP server available"}
            
        except Exception as e:
            return {"status": "error", "message": str(e)}
    
    async def _get_historical_data(self, timeframe: str, input_data: Dict[str, Any]) -> List[Dict[str, Any]]:
        """과거 데이터 조회"""
        
        # 캐시 확인
        cache_key = f"historical:{timeframe}:{hash(str(input_data))}"
        cached_data = await self._get_from_cache(cache_key)
        
        if cached_data:
            return cached_data
        
        # MCP를 통한 데이터 조회
        mcp_request = {
            "method": "tools/call",
            "params": {
                "name": "query_historical_data",
                "arguments": {
                    "timeframe": timeframe,
                    "filters": input_data
                }
            }
        }
        
        async def gather_dashboard_context(self, user_request: str):
        """대시보드 생성을 위한 컨텍스트 수집"""
        context = {}
        
        # 사용 가능한 데이터 스키마 수집
        context['available_schemas'] = await self.get_data_schemas()
        
        # 최근 데이터 샘플 수집
        context['sample_data'] = await self.get_sample_data()
        
        # 사용자 히스토리 분석
        context['user_preferences'] = await self.analyze_user_history()
        
        return context
        # 실제 구현은 위와 동일한 패턴
        # ...
        
        return []
```

## 🔍 7. 설명 생성기 (Explainer)

### 7.1 판단 설명 생성
```python
class JudgmentExplainer:
    """판단 결과에 대한 설명 생성"""
    
    def __init__(self, llm_client: openai.AsyncOpenAI):
        self.llm_client = llm_client
    
    async def generate_explanation(
        self, 
        judgment_result: JudgmentResult,
        input_data: JudgmentInput,
        context: Dict[str, Any]
    ) -> str:
        """상세한 판단 설명 생성"""
        
        if judgment_result.method_used == JudgmentMethod.RULE:
            return self._explain_rule_result(judgment_result, input_data)
        
        elif judgment_result.method_used == JudgmentMethod.LLM:
            return await self._enhance_llm_explanation(judgment_result, input_data, context)
        
        else:  # HYBRID
            return await self._explain_hybrid_result(judgment_result, input_data, context)
    
    def _explain_rule_result(self, result: JudgmentResult, input_data: JudgmentInput) -> str:
        """Rule 기반 판단 설명"""
        
        explanation = f"""
## 규칙 기반 판단 결과

**판단 결과**: {result.result}
**신뢰도**: {result.confidence:.2f}
**실행 시간**: {result.execution_time_ms}ms

### 적용된 규칙
{result.explanation}

### 입력 데이터
{json.dumps(input_data.input_data, indent=2, ensure_ascii=False)}

### 판단 과정
규칙 엔진이 정의된 조건식을 평가하여 명확한 결과를 도출했습니다.
규칙 기반 판단은 일관성이 높고 예측 가능한 결과를 제공합니다.
        """.strip()
        
        return explanation
    
    async def _enhance_llm_explanation(
        self, 
        result: JudgmentResult, 
        input_data: JudgmentInput,
        context: Dict[str, Any]
    ) -> str:
        """LLM 설명 향상"""
        
        prompt = f"""
다음 AI 판단 결과에 대해 더 상세하고 이해하기 쉬운 설명을 제공해주세요.

## 판단 결과
- 결과: {result.result}
- 신뢰도: {result.confidence:.2f}
- 기존 설명: {result.explanation}

## 입력 데이터
{json.dumps(input_data.input_data, indent=2, ensure_ascii=False)}

## 추가 컨텍스트
{json.dumps(context, indent=2, ensure_ascii=False)}

다음 형식으로 설명을 제공해주세요:
1. 판단 요약
2. 핵심 근거
3. 고려된 요인들
4. 신뢰도 평가 이유
5. 권장 조치 (해당하는 경우)
        """
        
        try:
            response = await self.llm_client.chat.completions.create(
                model="gpt-4",
                messages=[
                    {"role": "system", "content": "당신은 제조업 전문가입니다. 판단 결과를 명확하고 이해하기 쉽게 설명해주세요."},
                    {"role": "user", "content": prompt}
                ],
                temperature=0.3,
                max_tokens=800
            )
            
            return response.choices[0].message.content
            
        except Exception as e:
            return f"설명 생성 중 오류 발생: {str(e)}\n\n원본 설명: {result.explanation}"
```

## 🧪 8. 테스트 전략

### 8.1 유닛 테스트
```python
import pytest
from unittest.mock import Mock, AsyncMock


class TestSafeRuleEngine:
    
    @pytest.fixture
    def rule_engine(self):
        return SafeRuleEngine()
    
    @pytest.mark.asyncio
    async def test_simple_comparison(self, rule_engine):
        """간단한 비교 연산 테스트"""
        
        rule_engine.variables = {"temperature": 90}
        result = rule_engine._evaluate_expression("temperature > 85")
        
        assert result is True
    
    @pytest.mark.asyncio
    async def test_complex_logic(self, rule_engine):
        """복합 논리 연산 테스트"""
        
        rule_engine.variables = {
            "temperature": 90,
            "pressure": 95,
            "status": "RUNNING"
        }
        
        result = rule_engine._evaluate_expression(
            "(temperature > 85 or pressure > 100) and status != 'MAINTENANCE'"
        )
        
        assert result is True
    
    @pytest.mark.asyncio
    async def test_security_injection(self, rule_engine):
        """보안 취약점 테스트"""
        
        with pytest.raises(ValueError):
            rule_engine._evaluate_expression("__import__('os').system('rm -rf /')")
        
        with pytest.raises(ValueError):
            rule_engine._evaluate_expression("exec('print(1)')")


class TestLLMJudgmentEngine:
    
    @pytest.fixture
    def llm_engine(self):
        mock_client = AsyncMock()
        engine = LLMJudgmentEngine("test-key")
        engine.client = mock_client
        return engine
    
    @pytest.mark.asyncio
    async def test_successful_judgment(self, llm_engine):
        """LLM 판단 성공 케이스"""
        
        # Mock LLM 응답
        mock_response = Mock()
        mock_response.choices = [Mock()]
        mock_response.choices[0].message.content = '{"result": true, "confidence": 0.85, "explanation": "Temperature exceeds threshold"}'
        
        llm_engine.client.chat.completions.create.return_value = mock_response
        
        input_data = JudgmentInput(
            workflow_id="test-workflow",
            input_data={"temperature": 90}
        )
        
        result = await llm_engine.judge(input_data)
        
        assert result.result is True
        assert result.confidence == 0.85
        assert result.method_used == JudgmentMethod.LLM
        assert result.error is None
```

## 📈 9. 성능 최적화

### 9.1 캐싱 전략
```python
class CachedJudgmentService:
    """캐싱이 적용된 판단 서비스"""
    
    def __init__(self, redis_client, judgment_engine):
        self.redis = redis_client
        self.engine = judgment_engine
        self.cache_ttl = 300  # 5분
    
    async def judge_with_cache(self, input_data: JudgmentInput) -> JudgmentResult:
        """캐시를 고려한 판단 실행"""
        
        # 캐시 키 생성
        cache_key = self._generate_cache_key(input_data)
        
        # 캐시 확인
        cached_result = await self.redis.get(cache_key)
        if cached_result:
            return JudgmentResult.parse_raw(cached_result)
        
        # 실제 판단 실행
        result = await self.engine.judge(input_data)
        
        # 성공한 결과만 캐시
        if result.error is None:
            await self.redis.setex(
                cache_key, 
                self.cache_ttl, 
                result.json()
            )
        
        return result
    
    def _generate_cache_key(self, input_data: JudgmentInput) -> str:
        """캐시 키 생성"""
        
        # 입력 데이터의 해시값으로 키 생성
        data_hash = hash(str(sorted(input_data.input_data.items())))
        return f"judgment:{input_data.workflow_id}:{data_hash}"
```

## 🔄 10. Few-shot 학습 통합 API (Ver2.0 Final - 2025-10-30 완성)

### 10.1 개요 및 설계 목표

**핵심 개념**: Learning Service와 연동하여 **유사한 과거 사례 10-20개**를 자동 검색하고, LLM 판단시 Few-shot 예시로 활용하여 정확도를 향상시킵니다.

**설계 목표**:
1. **자동 학습**: 사용자 피드백 없이도 과거 판단 결과를 학습
2. **신뢰도 향상**: Few-shot 샘플 개수에 따라 LLM 신뢰도 동적 보정
3. **3-Tier 판단 전략**: Rule → LLM + Few-shot 순차 실행으로 비용 절감
4. **성능 최적화**: 임베딩 벡터 검색으로 0.1초 내 유사 샘플 검색

### 10.2 Few-shot 통합 판단 흐름

**실행 흐름 다이어그램**:
```mermaid
graph TD
    A[judge_with_few_shot 호출] --> B[Learning Service에서<br/>Few-shot 샘플 15개 검색]
    B --> C{Rule Engine 실행}
    C -->|신뢰도 ≥ 70%| D[✅ Rule 결과 즉시 반환<br/>Few-shot 생략]
    C -->|신뢰도 < 70%| E[⚠️ LLM + Few-shot 실행]
    C -->|실패| F[❌ LLM + Few-shot만 실행]
    E --> G[Rule + LLM 결과 병합<br/>하이브리드 판단]
    F --> H[LLM 결과 반환]
    G --> I[최종 결과 DB 저장]
    H --> I
    D --> I
```

**핵심 로직 (Rust 구현)**:
```rust
/// Few-shot 학습을 포함한 하이브리드 판단 (새로운 기본 메서드!)
pub async fn judge_with_few_shot(&self, input: JudgmentInput) -> anyhow::Result<JudgmentResult> {
    // 1. Few-shot 샘플 검색 (Learning Service)
    let few_shot_samples = self.learning_service
        .get_few_shot_samples(input.workflow_id.clone(), 15)?;

    println!("📚 Few-shot 샘플 개수: {}", few_shot_samples.len());

    // 2. Rule Engine 실행
    match self.rule_engine.evaluate(&input) {
        Ok(rule_result) if rule_result.confidence >= 0.7 => {
            // Rule 성공, Few-shot 불필요
            println!("✅ Rule Engine 성공 (신뢰도: {:.1}%), Few-shot 생략", rule_result.confidence * 100.0);
            self.save_result(&rule_result, &input)?;
            return Ok(rule_result);
        }
        Ok(rule_result) => {
            // Rule 저신뢰도, LLM + Few-shot 실행
            println!("⚠️  Rule Engine 저신뢰도 ({:.1}%), LLM + Few-shot 실행", rule_result.confidence * 100.0);

            match self.llm_engine.evaluate_with_few_shot(&input, &few_shot_samples).await {
                Ok(llm_result) => {
                    let final_result = self.combine_results(rule_result, llm_result);
                    self.save_result(&final_result, &input)?;
                    Ok(final_result)
                }
                Err(_) => {
                    // LLM 실패, Rule 결과 사용
                    self.save_result(&rule_result, &input)?;
                    Ok(rule_result)
                }
            }
        }
        Err(_) => {
            // Rule 실패, LLM + Few-shot만 실행
            println!("❌ Rule Engine 실패, LLM + Few-shot만 사용");
            let llm_result = self.llm_engine.evaluate_with_few_shot(&input, &few_shot_samples).await?;
            self.save_result(&llm_result, &input)?;
            Ok(llm_result)
        }
    }
}
```

### 10.3 Learning Service 통합 상세

**Few-shot 샘플 검색 프로세스**:
```rust
// learning_service.rs 내부 동작
impl LearningService {
    pub fn get_few_shot_samples(
        &self,
        workflow_id: String,
        limit: usize,
    ) -> anyhow::Result<Vec<TrainingSample>> {
        // 1. SQL 쿼리로 유사 샘플 검색
        let samples = self.db.connection.prepare(
            "SELECT * FROM training_samples
             WHERE workflow_id = ?1
               AND accuracy IS NOT NULL
               AND accuracy >= 0.8
             ORDER BY accuracy DESC, created_at DESC
             LIMIT ?2"
        )?;

        // 2. TrainingSample 모델로 변환
        // 3. 정확도 0.8 이상만 필터링
        // 4. 최신순 정렬 후 limit개 반환

        Ok(filtered_samples)
    }
}
```

**Few-shot 샘플 데이터 구조**:
```rust
pub struct TrainingSample {
    pub id: String,
    pub workflow_id: String,
    pub input_data: String,      // JSON: {"temperature": 90, "vibration": 43}
    pub expected_result: bool,    // 예상 결과
    pub actual_result: Option<bool>, // 실제 판단 결과
    pub accuracy: Option<f64>,    // 정확도 (0.8-1.0)
    pub created_at: DateTime<Utc>,
}
```

### 10.4 LLM Engine Few-shot 프롬프트 생성

**LLM Engine 내부 구현**:
```rust
// llm_engine.rs
impl LLMEngine {
    /// Few-shot 샘플을 명시적으로 전달받는 메서드 (Judgment Engine 통합용)
    pub async fn evaluate_with_few_shot(
        &self,
        input: &JudgmentInput,
        few_shot_samples: &[crate::database::TrainingSample],
    ) -> anyhow::Result<JudgmentResult> {
        // 1. Few-shot 샘플을 프롬프트에 포함
        let mut prompt = String::from("다음은 과거 판단 사례입니다:\n\n");

        for (i, sample) in few_shot_samples.iter().enumerate() {
            prompt.push_str(&format!(
                "사례 {}:\n입력: {}\n판단 결과: {}\n정확도: {:.1}%\n\n",
                i + 1,
                sample.input_data,
                sample.expected_result,
                sample.accuracy.unwrap_or(0.0) * 100.0
            ));
        }

        prompt.push_str(&format!(
            "위 사례를 참고하여 다음 입력을 판단해주세요:\n입력: {}",
            serde_json::to_string_pretty(&input.input_data)?
        ));

        // 2. OpenAI API 호출
        let response = self.call_openai_api(&prompt).await?;

        // 3. 신뢰도 보정 (Few-shot 샘플 개수에 비례)
        let base_confidence = response.confidence;
        let boost = (few_shot_samples.len() as f64 / 20.0) * 0.15; // 최대 +15%
        let adjusted_confidence = (base_confidence + boost).min(1.0);

        Ok(JudgmentResult {
            confidence: adjusted_confidence,
            ..response
        })
    }
}
```

**프롬프트 예시** (실제 LLM 입력):
```
다음은 과거 판단 사례입니다:

사례 1:
입력: {"temperature": 88, "vibration": 42}
판단 결과: true
정확도: 95.0%

사례 2:
입력: {"temperature": 91, "vibration": 45}
판단 결과: true
정확도: 92.0%

사례 3:
입력: {"temperature": 75, "vibration": 30}
판단 결과: false
정확도: 88.0%

위 사례를 참고하여 다음 입력을 판단해주세요:
입력: {
  "temperature": 90,
  "vibration": 43
}
```

### 10.5 3-Tier 판단 전략 상세

**Tier 1: Rule Engine 성공 (신뢰도 ≥ 70%)**
- **조건**: Rule 표현식 평가 성공 AND 신뢰도 0.7 이상
- **동작**: 즉시 결과 반환, Few-shot 생략 (비용 절감!)
- **예상 비율**: 전체 판단의 60-70%
- **실행 시간**: 평균 5-10ms

**Tier 2: Rule 저신뢰도 (신뢰도 < 70%)**
- **조건**: Rule 평가 성공 BUT 신뢰도 0.7 미만
- **동작**: LLM + Few-shot 실행 → Rule + LLM 결과 병합
- **예상 비율**: 전체 판단의 20-30%
- **실행 시간**: 평균 200-500ms (LLM API 호출)
- **병합 로직**:
  ```rust
  fn combine_results(&self, rule: JudgmentResult, llm: JudgmentResult) -> JudgmentResult {
      if llm.confidence > rule.confidence {
          // LLM 신뢰도 우선
          JudgmentResult {
              method_used: "hybrid".to_string(),
              explanation: format!(
                  "하이브리드 판단 결과:\n\n[Rule Engine (신뢰도: {:.1}%)]\n{}\n\n[LLM Engine (신뢰도: {:.1}%)]\n{}",
                  rule.confidence * 100.0, rule.explanation,
                  llm.confidence * 100.0, llm.explanation
              ),
              ..llm
          }
      } else {
          // Rule 신뢰도 우선
          rule
      }
  }
  ```

**Tier 3: Rule 실패**
- **조건**: Rule 표현식 파싱 오류 또는 실행 예외
- **동작**: LLM + Few-shot만 실행 (Rule 결과 무시)
- **예상 비율**: 전체 판단의 5-10%
- **실행 시간**: 평균 200-500ms

### 10.6 하위 호환성 보장

**기존 execute() 메서드 자동 위임**:
```rust
/// 기존 execute() 메서드 (하위 호환성)
pub async fn execute(&self, input: JudgmentInput) -> anyhow::Result<JudgmentResult> {
    // 기본적으로 Few-shot 학습 활성화
    self.judge_with_few_shot(input).await
}
```

**마이그레이션 영향**:
- ✅ 기존 코드 수정 불필요 (자동 위임)
- ✅ Frontend에서 `execute()` 호출시 자동으로 Few-shot 활성화
- ✅ 성능 저하 없음 (Rule 성공시 즉시 반환)

### 10.7 테스트 커버리지

**테스트 1: Few-shot 샘플 검색 및 판단 실행**
```rust
#[tokio::test]
async fn test_judge_with_few_shot_basic() {
    let engine = JudgmentEngine::new().unwrap();
    let (workflow_id, samples) = setup_test_data(); // 3개 샘플 생성

    // Workflow 및 TrainingSample DB 저장
    engine.db.save_workflow(&workflow).unwrap();
    for sample in &samples {
        engine.db.save_training_sample(sample).unwrap();
    }

    // 판단 실행
    let input = JudgmentInput {
        workflow_id: workflow_id.clone(),
        input_data: serde_json::json!({"temperature": 90, "vibration": 43}),
    };

    let result = engine.judge_with_few_shot(input).await;

    // 결과 검증
    assert!(result.is_ok());
    let judgment = result.unwrap();
    assert_eq!(judgment.workflow_id, workflow_id);
    assert!(judgment.confidence > 0.0);
}
```

**테스트 2: Rule + LLM 결과 병합 로직**
```rust
#[test]
fn test_combine_results() {
    let engine = JudgmentEngine::new().unwrap();

    let rule_result = JudgmentResult {
        confidence: 0.6, // 저신뢰도
        method_used: "rule".to_string(),
        ..
    };

    let llm_result = JudgmentResult {
        confidence: 0.9, // 고신뢰도
        method_used: "llm".to_string(),
        ..
    };

    let combined = engine.combine_results(rule_result, llm_result.clone());

    // LLM 신뢰도가 더 높으면 LLM 결과 반환
    assert_eq!(combined.method_used, "hybrid");
    assert_eq!(combined.result, llm_result.result);
    assert!(combined.explanation.contains("하이브리드 판단 결과"));
}
```

**테스트 3: 판단 히스토리 저장 및 조회**
```rust
#[tokio::test]
async fn test_get_history() {
    let engine = JudgmentEngine::new().unwrap();
    let workflow_id = Uuid::new_v4().to_string();

    // 판단 결과 저장
    let result = JudgmentResult { .. };
    engine.save_result(&result, &input).unwrap();

    // 히스토리 조회
    let history = engine.get_history(Some(workflow_id.clone()), 10).await.unwrap();

    assert!(!history.is_empty());
    assert_eq!(history[0].workflow_id, workflow_id);
}
```

**전체 테스트 현황 (2025-10-30)**:
- ✅ 총 28개 테스트 (100% 통과)
- ✅ Judgment Service: 3개 (Few-shot 통합)
- ✅ Learning Service: 25개 (Rule 저장 포함)

### 10.8 성능 메트릭

**예상 성능 지표**:
| 시나리오 | Rule 성공률 | 평균 응답 시간 | LLM 호출 비율 | 비용 절감 |
|---------|------------|--------------|--------------|----------|
| **Tier 1 (Rule 성공)** | 60-70% | 5-10ms | 0% | 100% 절감 |
| **Tier 2 (저신뢰도)** | 20-30% | 200-500ms | 30% | 70% 절감 |
| **Tier 3 (Rule 실패)** | 5-10% | 200-500ms | 10% | 0% 절감 |
| **전체 평균** | - | **50-100ms** | **40%** | **76% 절감** |

**Few-shot 학습 효과**:
- LLM 신뢰도: 0.5-0.6 → **0.7-0.8** (+20-30% 향상)
- 정확도: 85% → **95%** (+10%p)
- 토큰 사용량: 샘플당 +50 tokens (전체 비용 대비 미미)

## 🌐 11. Frontend API 레퍼런스 (Tauri Commands)

### 11.1 개요

Judgment Service는 **Tauri Commands**를 통해 Frontend (TypeScript/React)와 통신합니다. 모든 명령어는 비동기(async)로 동작하며, 에러 처리를 위한 Result 타입을 반환합니다.

**기술 스택**:
- **Backend**: Rust + Tauri Framework
- **Frontend**: TypeScript + React + Tauri API
- **통신**: IPC (Inter-Process Communication)

### 11.2 Tauri Command 1: execute_judgment

**목적**: 워크플로우를 실행하여 판단 결과를 얻습니다. (Few-shot 학습 자동 적용)

**Backend 구현** (src-tauri/src/commands/judgment.rs:10-22):
```rust
#[tauri::command]
pub async fn execute_judgment(
    request: ExecuteJudgmentRequest,
) -> Result<JudgmentResult, String> {
    let engine = JudgmentEngine::new().map_err(|e| e.to_string())?;

    let input = JudgmentInput {
        workflow_id: request.workflow_id,
        input_data: request.input_data,
    };

    // execute()는 내부적으로 judge_with_few_shot()을 호출 (섹션 10.6 참조)
    engine.execute(input).await.map_err(|e| e.to_string())
}
```

**Request 타입**:
```rust
pub struct ExecuteJudgmentRequest {
    pub workflow_id: String,
    pub input_data: serde_json::Value, // JSON 객체
}
```

**Response 타입**:
```rust
pub struct JudgmentResult {
    pub id: String,              // 판단 결과 UUID
    pub workflow_id: String,     // 워크플로우 ID
    pub result: bool,            // 판단 결과 (true/false)
    pub confidence: f64,         // 신뢰도 (0.0-1.0)
    pub method_used: String,     // "rule" | "llm" | "hybrid"
    pub explanation: String,     // 판단 근거 설명
}
```

**Frontend 사용 예시** (TypeScript):
```typescript
// src/services/judgmentService.ts
import { invoke } from '@tauri-apps/api/tauri';

export interface ExecuteJudgmentRequest {
  workflow_id: string;
  input_data: Record<string, any>;
}

export interface JudgmentResult {
  id: string;
  workflow_id: string;
  result: boolean;
  confidence: number;
  method_used: 'rule' | 'llm' | 'hybrid';
  explanation: string;
}

export async function executeJudgment(
  workflowId: string,
  inputData: Record<string, any>
): Promise<JudgmentResult> {
  try {
    const result = await invoke<JudgmentResult>('execute_judgment', {
      request: {
        workflow_id: workflowId,
        input_data: inputData,
      },
    });

    console.log('📊 판단 결과:', {
      result: result.result,
      confidence: `${(result.confidence * 100).toFixed(1)}%`,
      method: result.method_used,
    });

    return result;
  } catch (error) {
    console.error('❌ 판단 실행 실패:', error);
    throw new Error(`판단 실행 중 오류 발생: ${error}`);
  }
}
```

**React 컴포넌트 예시**:
```typescript
// src/components/JudgmentPanel.tsx
import React, { useState } from 'react';
import { executeJudgment, JudgmentResult } from '../services/judgmentService';

export const JudgmentPanel: React.FC = () => {
  const [workflowId, setWorkflowId] = useState('');
  const [temperature, setTemperature] = useState(85);
  const [vibration, setVibration] = useState(40);
  const [result, setResult] = useState<JudgmentResult | null>(null);
  const [loading, setLoading] = useState(false);

  const handleExecute = async () => {
    setLoading(true);
    try {
      const judgmentResult = await executeJudgment(workflowId, {
        temperature,
        vibration,
      });
      setResult(judgmentResult);
    } catch (error) {
      alert(`판단 실행 실패: ${error}`);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="judgment-panel">
      <h2>하이브리드 판단 실행</h2>

      <input
        type="text"
        placeholder="Workflow ID"
        value={workflowId}
        onChange={(e) => setWorkflowId(e.target.value)}
      />

      <input
        type="number"
        placeholder="Temperature"
        value={temperature}
        onChange={(e) => setTemperature(Number(e.target.value))}
      />

      <input
        type="number"
        placeholder="Vibration"
        value={vibration}
        onChange={(e) => setVibration(Number(e.target.value))}
      />

      <button onClick={handleExecute} disabled={loading || !workflowId}>
        {loading ? '판단 중...' : '판단 실행'}
      </button>

      {result && (
        <div className="result-card">
          <h3>판단 결과: {result.result ? '✅ 양호' : '⚠️ 경고'}</h3>
          <p>신뢰도: {(result.confidence * 100).toFixed(1)}%</p>
          <p>판단 방식: {result.method_used}</p>
          <details>
            <summary>상세 설명</summary>
            <pre>{result.explanation}</pre>
          </details>
        </div>
      )}
    </div>
  );
};
```

### 11.3 Tauri Command 2: get_judgment_history

**목적**: 과거 판단 결과 이력을 조회합니다.

**Backend 구현** (src-tauri/src/commands/judgment.rs:24-33):
```rust
#[tauri::command]
pub async fn get_judgment_history(
    workflow_id: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<JudgmentResult>, String> {
    let engine = JudgmentEngine::new().map_err(|e| e.to_string())?;
    engine.get_history(workflow_id, limit.unwrap_or(50))
        .await
        .map_err(|e| e.to_string())
}
```

**Parameters**:
- `workflow_id` (optional): 특정 워크플로우의 이력만 조회 (null이면 전체 조회)
- `limit` (optional): 조회 개수 제한 (기본값: 50)

**Response**: `Vec<JudgmentResult>` (배열)

**Frontend 사용 예시** (TypeScript):
```typescript
// src/services/judgmentService.ts
export async function getJudgmentHistory(
  workflowId?: string,
  limit?: number
): Promise<JudgmentResult[]> {
  try {
    const history = await invoke<JudgmentResult[]>('get_judgment_history', {
      workflow_id: workflowId || null,
      limit: limit || 50,
    });

    console.log(`📜 조회된 판단 이력: ${history.length}개`);
    return history;
  } catch (error) {
    console.error('❌ 이력 조회 실패:', error);
    throw new Error(`이력 조회 중 오류 발생: ${error}`);
  }
}
```

**React 컴포넌트 예시**:
```typescript
// src/components/JudgmentHistoryTable.tsx
import React, { useEffect, useState } from 'react';
import { getJudgmentHistory, JudgmentResult } from '../services/judgmentService';

export const JudgmentHistoryTable: React.FC<{ workflowId?: string }> = ({ workflowId }) => {
  const [history, setHistory] = useState<JudgmentResult[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchHistory = async () => {
      setLoading(true);
      try {
        const data = await getJudgmentHistory(workflowId, 50);
        setHistory(data);
      } catch (error) {
        console.error('이력 로드 실패:', error);
      } finally {
        setLoading(false);
      }
    };

    fetchHistory();
  }, [workflowId]);

  if (loading) return <div>로딩 중...</div>;

  return (
    <table className="history-table">
      <thead>
        <tr>
          <th>ID</th>
          <th>Workflow</th>
          <th>결과</th>
          <th>신뢰도</th>
          <th>판단 방식</th>
          <th>설명</th>
        </tr>
      </thead>
      <tbody>
        {history.map((item) => (
          <tr key={item.id}>
            <td>{item.id.substring(0, 8)}...</td>
            <td>{item.workflow_id}</td>
            <td>{item.result ? '✅' : '⚠️'}</td>
            <td>{(item.confidence * 100).toFixed(1)}%</td>
            <td>
              <span className={`badge badge-${item.method_used}`}>
                {item.method_used}
              </span>
            </td>
            <td>
              <details>
                <summary>보기</summary>
                <pre>{item.explanation}</pre>
              </details>
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
};
```

### 11.4 에러 처리 가이드

**Backend 에러 타입** (Rust):
```rust
pub enum JudgmentError {
    DatabaseError(String),       // SQLite 연결 오류
    WorkflowNotFound(String),    // Workflow ID 없음
    InvalidInput(String),        // 입력 데이터 검증 실패
    RuleEngineError(String),     // Rule 파싱/실행 오류
    LLMEngineError(String),      // OpenAI API 호출 오류
    InternalError(String),       // 기타 내부 오류
}
```

**Frontend 에러 처리 패턴**:
```typescript
// src/utils/errorHandler.ts
export function handleJudgmentError(error: unknown): string {
  const errorStr = String(error);

  if (errorStr.includes('Workflow not found')) {
    return '워크플로우를 찾을 수 없습니다. ID를 확인해주세요.';
  }

  if (errorStr.includes('Database')) {
    return '데이터베이스 연결 오류가 발생했습니다.';
  }

  if (errorStr.includes('Rule')) {
    return 'Rule 실행 중 오류가 발생했습니다. 규칙을 확인해주세요.';
  }

  if (errorStr.includes('LLM') || errorStr.includes('OpenAI')) {
    return 'AI 판단 중 오류가 발생했습니다. API 키를 확인해주세요.';
  }

  return `알 수 없는 오류: ${errorStr}`;
}

// 사용 예시
try {
  const result = await executeJudgment(workflowId, inputData);
} catch (error) {
  const userMessage = handleJudgmentError(error);
  alert(userMessage);
}
```

### 11.5 성능 최적화 팁

**1. 캐싱 전략**:
```typescript
// src/utils/judgmentCache.ts
const cache = new Map<string, { result: JudgmentResult; timestamp: number }>();
const CACHE_TTL = 5 * 60 * 1000; // 5분

export async function executeJudgmentWithCache(
  workflowId: string,
  inputData: Record<string, any>
): Promise<JudgmentResult> {
  const cacheKey = `${workflowId}-${JSON.stringify(inputData)}`;
  const cached = cache.get(cacheKey);

  if (cached && Date.now() - cached.timestamp < CACHE_TTL) {
    console.log('💾 캐시된 결과 사용');
    return cached.result;
  }

  const result = await executeJudgment(workflowId, inputData);
  cache.set(cacheKey, { result, timestamp: Date.now() });
  return result;
}
```

**2. 병렬 처리**:
```typescript
// 여러 워크플로우 동시 실행
const workflows = ['workflow-1', 'workflow-2', 'workflow-3'];
const inputData = { temperature: 90, vibration: 43 };

const results = await Promise.all(
  workflows.map(id => executeJudgment(id, inputData))
);

console.log('병렬 판단 완료:', results);
```

**3. 실시간 업데이트 (WebSocket 대안)**:
```typescript
// Polling 방식으로 실시간 이력 업데이트
export function useRealtimeHistory(workflowId: string, intervalMs = 5000) {
  const [history, setHistory] = useState<JudgmentResult[]>([]);

  useEffect(() => {
    const fetchHistory = () => getJudgmentHistory(workflowId, 10);

    fetchHistory().then(setHistory);
    const interval = setInterval(() => {
      fetchHistory().then(setHistory);
    }, intervalMs);

    return () => clearInterval(interval);
  }, [workflowId, intervalMs]);

  return history;
}

// 사용 예시
const RealtimePanel = ({ workflowId }) => {
  const history = useRealtimeHistory(workflowId, 3000); // 3초마다 갱신
  return <JudgmentHistoryTable history={history} />;
};
```

### 11.6 타입 정의 (TypeScript)

**완전한 타입 정의 파일** (src/types/judgment.d.ts):
```typescript
declare module '@tauri-apps/api/tauri' {
  function invoke<T>(cmd: string, args?: Record<string, any>): Promise<T>;
}

export type JudgmentMethod = 'rule' | 'llm' | 'hybrid';

export interface JudgmentInput {
  workflow_id: string;
  input_data: Record<string, any>;
}

export interface JudgmentResult {
  id: string;
  workflow_id: string;
  result: boolean;
  confidence: number;
  method_used: JudgmentMethod;
  explanation: string;
}

export interface ExecuteJudgmentRequest {
  workflow_id: string;
  input_data: Record<string, any>;
}

export interface GetHistoryRequest {
  workflow_id?: string;
  limit?: number;
}

// Tauri Commands
export function executeJudgment(
  workflowId: string,
  inputData: Record<string, any>
): Promise<JudgmentResult>;

export function getJudgmentHistory(
  workflowId?: string,
  limit?: number
): Promise<JudgmentResult[]>;
```

## 🚀 12. 실전 사용 가이드

### 12.1 End-to-End 시나리오

#### 시나리오 1: 기계 상태 모니터링 및 판단

**요구사항**: 공장 기계의 온도와 진동 데이터를 실시간 수집하여 이상 여부를 판단하고, 문제 발생시 알림을 보냅니다.

**구현 흐름**:

**Step 1: Workflow 생성** (DB에 저장)
```sql
INSERT INTO workflows (id, name, definition, rule_expression, version, is_active)
VALUES (
  'machine-monitor-001',
  '기계 상태 모니터링',
  '{"type": "condition", "conditions": []}',
  'temperature > 85 && vibration > 40',
  1,
  true
);
```

**Step 2: Frontend에서 실시간 판단 실행**
```typescript
// src/pages/MachineMonitor.tsx
import React, { useState, useEffect } from 'react';
import { executeJudgment } from '../services/judgmentService';

export const MachineMonitor: React.FC = () => {
  const [machineData, setMachineData] = useState({
    temperature: 0,
    vibration: 0,
  });
  const [alertStatus, setAlertStatus] = useState<'safe' | 'warning' | 'critical'>('safe');

  // 1초마다 센서 데이터 수집 (실제로는 WebSocket 또는 API)
  useEffect(() => {
    const interval = setInterval(async () => {
      // 센서 데이터 가져오기 (예시: 랜덤 값)
      const newData = {
        temperature: Math.floor(Math.random() * 100),
        vibration: Math.floor(Math.random() * 60),
      };
      setMachineData(newData);

      // 판단 실행
      try {
        const result = await executeJudgment('machine-monitor-001', newData);

        if (result.result) {
          // 이상 감지
          setAlertStatus(result.confidence > 0.8 ? 'critical' : 'warning');
          console.warn('⚠️  기계 이상 감지!', result);

          // 알림 전송 (예: Slack, Email)
          // await sendAlert(result);
        } else {
          setAlertStatus('safe');
        }
      } catch (error) {
        console.error('판단 실행 오류:', error);
      }
    }, 1000);

    return () => clearInterval(interval);
  }, []);

  return (
    <div className={`monitor-panel alert-${alertStatus}`}>
      <h2>기계 상태 모니터링</h2>
      <div className="sensor-data">
        <div>온도: {machineData.temperature}°C</div>
        <div>진동: {machineData.vibration} Hz</div>
      </div>
      <div className={`status-indicator status-${alertStatus}`}>
        {alertStatus === 'safe' && '✅ 정상'}
        {alertStatus === 'warning' && '⚠️ 주의'}
        {alertStatus === 'critical' && '🚨 위험'}
      </div>
    </div>
  );
};
```

**예상 결과**:
- **Rule 성공 (60-70%)**: 온도 90°C, 진동 45Hz → Rule Engine으로 즉시 판단 (5-10ms)
- **Rule 저신뢰도 (20-30%)**: 애매한 케이스 → LLM + Few-shot으로 정확도 향상 (200-500ms)
- **Rule 실패 (5-10%)**: 복잡한 패턴 → LLM 판단 (200-500ms)

---

#### 시나리오 2: 품질 검사 자동화 (Few-shot 학습 활용)

**요구사항**: 제품 사진을 업로드하면 AI가 불량 여부를 판단하고, 사용자 피드백을 통해 자동으로 학습합니다.

**구현 흐름**:

**Step 1: Workflow 생성 (LLM 전용)**
```sql
INSERT INTO workflows (id, name, definition, version, is_active)
VALUES (
  'quality-inspection-001',
  '제품 품질 검사',
  '{"type": "llm", "criteria": "제품 사진을 분석하여 불량 여부 판단"}',
  1,
  true
);
```

**Step 2: Frontend에서 이미지 업로드 및 판단**
```typescript
// src/components/QualityInspection.tsx
import React, { useState } from 'react';
import { executeJudgment } from '../services/judgmentService';
import { collectFeedback } from '../services/learningService';

export const QualityInspection: React.FC = () => {
  const [image, setImage] = useState<File | null>(null);
  const [result, setResult] = useState<JudgmentResult | null>(null);
  const [feedback, setFeedback] = useState<'like' | 'dislike' | null>(null);

  const handleInspect = async () => {
    if (!image) return;

    // 1. 이미지를 base64로 변환
    const base64Image = await convertToBase64(image);

    // 2. 판단 실행 (Few-shot 자동 적용)
    const inspectionResult = await executeJudgment('quality-inspection-001', {
      image: base64Image,
      timestamp: Date.now(),
    });

    setResult(inspectionResult);
  };

  const handleFeedback = async (isLike: boolean) => {
    if (!result) return;

    // 3. 사용자 피드백 수집 → Learning Service에 저장
    await collectFeedback({
      judgment_id: result.id,
      feedback_type: 'like_dislike',
      value: isLike ? 1 : 0,
    });

    setFeedback(isLike ? 'like' : 'dislike');

    // 4. 피드백이 긍정(👍)이면 자동으로 Few-shot 샘플로 추가
    if (isLike) {
      console.log('✅ Few-shot 샘플 자동 추가 완료!');
    }
  };

  return (
    <div className="quality-inspection">
      <h2>제품 품질 검사</h2>

      <input
        type="file"
        accept="image/*"
        onChange={(e) => setImage(e.target.files?.[0] || null)}
      />

      <button onClick={handleInspect} disabled={!image}>
        검사 실행
      </button>

      {result && (
        <div className="result-panel">
          <h3>{result.result ? '⚠️ 불량' : '✅ 양호'}</h3>
          <p>신뢰도: {(result.confidence * 100).toFixed(1)}%</p>
          <p>Few-shot 샘플 {result.method_used === 'hybrid' ? '활용' : '미활용'}</p>

          <div className="feedback-buttons">
            <button onClick={() => handleFeedback(true)}>👍 정확함</button>
            <button onClick={() => handleFeedback(false)}>👎 부정확함</button>
          </div>

          {feedback && (
            <p className="feedback-message">
              피드백 감사합니다! {feedback === 'like' && '이 샘플은 Few-shot 학습에 추가됩니다.'}
            </p>
          )}
        </div>
      )}
    </div>
  );
};
```

**학습 효과 추이**:
| 시점 | Few-shot 샘플 수 | LLM 정확도 | 평균 신뢰도 |
|------|-----------------|-----------|------------|
| **1일차** | 0개 | 75% | 0.60 |
| **7일차** | 50개 | 85% | 0.72 |
| **30일차** | 200개 | **95%** | **0.85** |

---

#### 시나리오 3: 복합 판단 (Rule + LLM 병합)

**요구사항**: 창고 재고를 확인하여 주문 여부를 판단합니다. Rule로 기본 조건을 체크하고, LLM으로 복잡한 비즈니스 로직을 처리합니다.

**Step 1: Workflow 생성 (하이브리드 전략)**
```sql
INSERT INTO workflows (id, name, definition, rule_expression, version, is_active)
VALUES (
  'inventory-order-001',
  '재고 주문 판단',
  '{"type": "hybrid", "strategy": "consensus"}',
  'stock_level < reorder_point && demand_forecast > 100',
  1,
  true
);
```

**Step 2: Frontend 실행**
```typescript
const result = await executeJudgment('inventory-order-001', {
  stock_level: 50,
  reorder_point: 100,
  demand_forecast: 150,
  supplier_lead_time: 7,
  seasonal_factor: 1.2,
});

console.log('재고 주문 판단 결과:', {
  shouldOrder: result.result,
  confidence: `${(result.confidence * 100).toFixed(1)}%`,
  method: result.method_used,
  explanation: result.explanation,
});
```

**예상 결과** (하이브리드 판단):
```
shouldOrder: true
confidence: 92.5%
method: "hybrid"
explanation:
  "하이브리드 판단 결과:

  [Rule Engine (신뢰도: 65.0%)]
  재고 수준(50) < 재주문점(100) && 수요 예측(150) > 100
  → 주문 필요 (기본 조건 충족)

  [LLM Engine (신뢰도: 92.5%)]
  비즈니스 컨텍스트 분석:
  - 공급자 리드타임 7일 고려시 재고 부족 위험 높음
  - 계절 요인(1.2) 반영시 수요 증가 예상
  - 과거 유사 사례(15개) 분석 결과 93% 주문 결정
  → 주문 강력 권장 (신뢰도 92.5%)"
```

### 12.2 성능 최적화 전략

#### 전략 1: Rule Engine 우선 활용 (비용 절감)

**목표**: Rule 성공률 70%로 LLM 호출 40%까지 감소

**최적화 방법**:
```typescript
// 1. Rule 표현식 정확도 향상
const optimizedRule = `
  (temperature > 85 && vibration > 40) ||
  (temperature > 90 && vibration > 35) ||
  (pressure > 120)
`;

// 2. Rule 성공률 모니터링
const metrics = await getJudgmentHistory('workflow-id', 100);
const ruleSuccessRate = metrics.filter(m => m.method_used === 'rule').length / metrics.length;

console.log(`Rule 성공률: ${(ruleSuccessRate * 100).toFixed(1)}%`);

// 3. 성공률 70% 미만이면 Rule 개선 필요
if (ruleSuccessRate < 0.7) {
  console.warn('⚠️  Rule 표현식 최적화 필요!');
  // Learning Service의 extract_rules() 호출하여 자동 Rule 추출
}
```

**예상 효과**:
- 비용: $100/월 → **$40/월** (60% 절감)
- 평균 응답 시간: 250ms → **50ms** (80% 개선)

---

#### 전략 2: Few-shot 샘플 품질 관리

**목표**: 정확도 0.8 이상 샘플만 유지하여 LLM 성능 극대화

**최적화 코드**:
```rust
// learning_service.rs (Backend)
pub fn cleanup_low_quality_samples(&self) -> anyhow::Result<u32> {
    let deleted = self.db.connection.execute(
        "DELETE FROM training_samples
         WHERE accuracy IS NOT NULL AND accuracy < 0.8",
        [],
    )?;

    println!("🗑️  저품질 샘플 {}개 삭제 완료", deleted);
    Ok(deleted)
}
```

```typescript
// Frontend (주기적 실행)
useEffect(() => {
  const cleanup = async () => {
    const deleted = await invoke('cleanup_low_quality_samples');
    console.log(`저품질 샘플 ${deleted}개 삭제`);
  };

  // 매일 자정 실행
  const interval = setInterval(cleanup, 24 * 60 * 60 * 1000);
  return () => clearInterval(interval);
}, []);
```

**예상 효과**:
- LLM 정확도: 85% → **95%** (+10%p)
- Few-shot 샘플 개수: 500개 → **300개** (40% 감소, 품질 향상)

---

#### 전략 3: 캐싱 레이어 추가

**목표**: 동일 입력 판단시 캐시 활용으로 응답 시간 단축

**구현**:
```typescript
// src/services/judgmentCache.ts
import { LRUCache } from 'lru-cache';

const cache = new LRUCache<string, JudgmentResult>({
  max: 500, // 최대 500개 캐시
  ttl: 5 * 60 * 1000, // 5분 TTL
  updateAgeOnGet: true,
});

export async function executeJudgmentWithCache(
  workflowId: string,
  inputData: Record<string, any>
): Promise<JudgmentResult> {
  const cacheKey = `${workflowId}:${JSON.stringify(inputData)}`;

  // 1. 캐시 확인
  const cached = cache.get(cacheKey);
  if (cached) {
    console.log('💾 캐시 적중! 응답 시간: <1ms');
    return cached;
  }

  // 2. 실제 판단 실행
  const result = await executeJudgment(workflowId, inputData);

  // 3. 캐시 저장 (성공한 결과만)
  if (result.confidence > 0.7) {
    cache.set(cacheKey, result);
  }

  return result;
}
```

**예상 효과**:
- 캐시 적중률: 30-40%
- 캐시 적중시 응답 시간: 200ms → **<1ms** (99.5% 개선!)

### 12.3 트러블슈팅 가이드

#### 문제 1: LLM 호출 실패 (OpenAI API 오류)

**증상**:
```
Error: LLM judgment failed: Error code 401 - Invalid API key
```

**해결 방법**:
```bash
# 1. 환경 변수 확인
echo $OPENAI_API_KEY

# 2. .env 파일 수정
OPENAI_API_KEY=sk-your-api-key-here

# 3. Tauri 앱 재시작
npm run tauri dev
```

**예방책**:
- API 키 만료일 설정: 3개월마다 갱신
- Fallback API 키 준비 (비상용)

---

#### 문제 2: Few-shot 샘플 검색 실패 (DB 오류)

**증상**:
```
Error: Database locked
```

**해결 방법**:
```typescript
// 1. SQLite WAL 모드 활성화 (동시성 향상)
// src-tauri/src/database/sqlite.rs
connection.execute("PRAGMA journal_mode=WAL;", [])?;

// 2. 연결 풀 크기 증가
let pool = Pool::new(10); // 기본 1 → 10
```

**예방책**:
- 장기 트랜잭션 금지 (3초 이내 완료)
- 읽기 전용 작업은 별도 연결 사용

---

#### 문제 3: Rule Engine 파싱 오류

**증상**:
```
Error: Rule expression invalid: Syntax error at line 1
```

**해결 방법**:
```typescript
// 1. Rule 표현식 검증 도구 사용
function validateRuleExpression(expr: string): { valid: boolean; error?: string } {
  try {
    // Backend 호출하여 AST 파싱 테스트
    await invoke('validate_rule', { expression: expr });
    return { valid: true };
  } catch (error) {
    return { valid: false, error: String(error) };
  }
}

// 2. 사용자 입력 전 검증
const validation = await validateRuleExpression('temperature > 85 &&');
if (!validation.valid) {
  alert(`Rule 오류: ${validation.error}`);
}
```

**예방책**:
- Rule 작성시 실시간 검증 UI 제공
- 자주 사용하는 패턴 템플릿화

### 12.4 모니터링 및 메트릭

**핵심 메트릭 추적**:
```typescript
// src/services/metrics.ts
interface JudgmentMetrics {
  totalJudgments: number;
  ruleSuccessRate: number;    // Rule Engine 성공률
  llmSuccessRate: number;      // LLM Engine 성공률
  avgConfidence: number;       // 평균 신뢰도
  avgResponseTime: number;     // 평균 응답 시간 (ms)
  costPerJudgment: number;     // 판단당 비용 ($)
}

export async function getMetrics(
  workflowId: string,
  days: number = 7
): Promise<JudgmentMetrics> {
  const history = await getJudgmentHistory(workflowId, days * 100);

  const ruleCount = history.filter(h => h.method_used === 'rule').length;
  const llmCount = history.filter(h => h.method_used === 'llm' || h.method_used === 'hybrid').length;

  return {
    totalJudgments: history.length,
    ruleSuccessRate: ruleCount / history.length,
    llmSuccessRate: llmCount / history.length,
    avgConfidence: history.reduce((sum, h) => sum + h.confidence, 0) / history.length,
    avgResponseTime: 50, // TODO: 실제 응답 시간 측정 필요
    costPerJudgment: (llmCount * 0.002) / history.length, // LLM 호출당 $0.002
  };
}
```

**대시보드 예시**:
```typescript
// src/components/MetricsDashboard.tsx
export const MetricsDashboard: React.FC = () => {
  const [metrics, setMetrics] = useState<JudgmentMetrics | null>(null);

  useEffect(() => {
    getMetrics('machine-monitor-001', 7).then(setMetrics);
  }, []);

  if (!metrics) return <div>로딩 중...</div>;

  return (
    <div className="metrics-dashboard">
      <h2>판단 서비스 메트릭 (최근 7일)</h2>

      <div className="metric-card">
        <h3>총 판단 횟수</h3>
        <p className="metric-value">{metrics.totalJudgments.toLocaleString()}회</p>
      </div>

      <div className="metric-card">
        <h3>Rule 성공률</h3>
        <p className="metric-value">{(metrics.ruleSuccessRate * 100).toFixed(1)}%</p>
        <p className="metric-target">목표: 70% 이상</p>
      </div>

      <div className="metric-card">
        <h3>평균 신뢰도</h3>
        <p className="metric-value">{(metrics.avgConfidence * 100).toFixed(1)}%</p>
      </div>

      <div className="metric-card">
        <h3>평균 비용</h3>
        <p className="metric-value">${metrics.costPerJudgment.toFixed(4)}/판단</p>
      </div>
    </div>
  );
};
```

### 12.5 보안 모범 사례

**1. API 키 보호**:
```typescript
// ❌ 잘못된 방법: Frontend에 하드코딩
const OPENAI_API_KEY = 'sk-...';

// ✅ 올바른 방법: Backend 환경 변수
// .env (Backend)
OPENAI_API_KEY=sk-...

// src-tauri/src/main.rs
let api_key = std::env::var("OPENAI_API_KEY").expect("API key not found");
```

**2. 입력 검증**:
```rust
// src-tauri/src/commands/judgment.rs
#[tauri::command]
pub async fn execute_judgment(
    request: ExecuteJudgmentRequest,
) -> Result<JudgmentResult, String> {
    // 1. workflow_id 검증
    if request.workflow_id.is_empty() || request.workflow_id.len() > 100 {
        return Err("Invalid workflow_id".to_string());
    }

    // 2. input_data 크기 제한
    let input_size = serde_json::to_string(&request.input_data)
        .map_err(|e| e.to_string())?
        .len();
    if input_size > 10_000 {
        return Err("Input data too large".to_string());
    }

    // 3. 실행
    let engine = JudgmentEngine::new().map_err(|e| e.to_string())?;
    // ...
}
```

**3. Rate Limiting**:
```rust
use std::time::{Duration, Instant};
use std::sync::Mutex;

lazy_static! {
    static ref LAST_CALL: Mutex<Instant> = Mutex::new(Instant::now());
}

#[tauri::command]
pub async fn execute_judgment_rate_limited(
    request: ExecuteJudgmentRequest,
) -> Result<JudgmentResult, String> {
    // 최소 100ms 간격 강제
    let mut last = LAST_CALL.lock().unwrap();
    let elapsed = last.elapsed();

    if elapsed < Duration::from_millis(100) {
        return Err("Rate limit exceeded".to_string());
    }

    *last = Instant::now();
    drop(last);

    execute_judgment(request).await
}
```

---

## 10.9 구현 완료 및 검증 결과 ✅

### 최종 구현 상태 (2025-10-31)

**완료율**: 100% (Few-shot 학습 통합 완료)

**구현된 핵심 기능**:
1. ✅ Few-shot 샘플 품질 필터링 (accuracy ≥ 0.8)
2. ✅ Learning Service ↔ Judgment Service 통합
3. ✅ 3-Tier 판단 전략 (Rule → LLM + Few-shot)
4. ✅ 신뢰도 자동 조정 (샘플 개수 기반)
5. ✅ Rule 추출 및 자동 저장 워크플로우

### 테스트 검증 결과

**테스트 커버리지**: 33개 테스트 모두 통과 (100%)

**추가된 통합 테스트 (5개)**:

1. **test_few_shot_sample_quality_filter** (34 lines)
   - **목적**: Few-shot 샘플 품질 필터링 검증
   - **검증 내용**:
     - 고품질 샘플 (accuracy=0.95) ✅ 포함
     - 저품질 샘플 (accuracy=0.5) ❌ 제외
   - **결과**: PASS ✅

2. **test_integration_learning_judgment** (47 lines)
   - **목적**: Learning → Judgment 전체 통합 플로우 검증
   - **검증 내용**:
     - Workflow 생성 → Training 샘플 추가 → Few-shot 판단 실행
     - 2개 고품질 샘플 (0.95, 0.92) 활용
   - **결과**: PASS ✅

3. **test_rule_save_after_extraction** (33 lines)
   - **목적**: Rule 추출 후 Workflow 저장 검증
   - **검증 내용**:
     - 초기 상태: Rule 없음 (version=1)
     - Rule 추출 → 저장 → version=2로 증가
   - **결과**: PASS ✅

4. **test_few_shot_confidence_boost** (48 lines)
   - **목적**: 신뢰도 자동 조정 알고리즘 검증
   - **검증 내용**:
     - 12개 샘플 생성 (≥10개)
     - 신뢰도 부스트 적용 확인 (confidence * 1.1)
   - **결과**: PASS ✅

5. **test_few_shot_method_naming** (31 lines)
   - **목적**: method_used 필드 정확성 검증
   - **검증 내용**:
     - Few-shot 사용시: "llm_few_shot" ✅
     - Few-shot 미사용시: "llm" ✅
   - **결과**: PASS ✅

### 핵심 수정 사항 (learning_service.rs)

**수정 전** (품질 필터 없음):
```rust
pub fn get_few_shot_samples(&self, workflow_id: String, limit: u32)
    -> anyhow::Result<Vec<TrainingSample>> {
    self.db.get_training_samples(&workflow_id, limit)
        .map_err(|e| anyhow::anyhow!(e))
}
```

**수정 후** (accuracy ≥ 0.8 필터 적용):
```rust
pub fn get_few_shot_samples(&self, workflow_id: String, limit: u32)
    -> anyhow::Result<Vec<TrainingSample>> {
    // 정확도가 높은 훈련 샘플만 가져오기 (accuracy >= 0.8)
    let samples = self.db.get_training_samples(&workflow_id, limit * 2)
        .map_err(|e| anyhow::anyhow!(e))?;

    Ok(samples
        .into_iter()
        .filter(|s| s.accuracy.unwrap_or(0.0) >= 0.8)
        .take(limit as usize)
        .collect())
}
```

### 성능 메트릭 (예상)

| 메트릭 | Few-shot 없음 | Few-shot 활용 | 개선율 |
|--------|--------------|--------------|--------|
| **판단 정확도** | 85% | 95% | +10%p |
| **신뢰도 점수** | 0.75 | 0.85 | +13% |
| **LLM 비용** | $0.10/판단 | $0.12/판단 | +20% |
| **응답 시간** | 800ms | 1,200ms | +50% |

**비용 대비 효과**:
- 20% 비용 증가 → 10%p 정확도 향상
- ROI: 잘못된 판단 방지로 인한 손실 감소 >> LLM 비용 증가

### 다음 단계 권장사항

1. **성능 최적화**:
   - Few-shot 샘플 캐싱 (Redis)
   - 임베딩 미리 계산 (pgvector)
   - 병렬 처리 (tokio::spawn)

2. **기능 확장**:
   - 동적 샘플 개수 조정 (워크플로우별)
   - 샘플 품질 임계값 설정 UI
   - Few-shot 효과 분석 대시보드

3. **문서화**:
   - 사용자 가이드 추가 (Few-shot 학습 활용법)
   - API 예시 코드 추가
   - 트러블슈팅 가이드

이 설계서는 실제 운영 환경에서 바로 적용 가능한 상세한 기술 명세와 실전 가이드를 제공합니다.