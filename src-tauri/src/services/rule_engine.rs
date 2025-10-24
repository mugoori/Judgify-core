use rhai::{Engine, Scope, Array, Map, Dynamic};
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::database::Database;
use crate::services::judgment_engine::{JudgmentInput, JudgmentResult};

// Rule 캐시 구조체
#[derive(Clone)]
struct CachedRule {
    expression: String,
    last_used: std::time::Instant,
}

pub struct RuleEngine {
    db: Database,
    // Rule 표현식 캐시 (성능 최적화)
    rule_cache: Arc<Mutex<HashMap<String, CachedRule>>>,
}

impl RuleEngine {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            db: Database::new()?,
            rule_cache: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    // Engine 생성 헬퍼 (매 호출시 새로 생성하여 Send 트레잇 문제 해결)
    fn create_engine() -> Engine {
        let mut engine = Engine::new();
        engine.set_max_operations(10000);

        // 사용자 정의 함수 등록 (Array/Object 헬퍼)
        engine.register_fn("contains", |arr: Array, val: Dynamic| -> bool {
            arr.into_iter().any(|v| {
                // Dynamic 값 비교 (Rhai의 내부 비교 로직 사용)
                if let (Some(v_i), Some(val_i)) = (v.as_int().ok(), val.as_int().ok()) {
                    v_i == val_i
                } else if let (Some(v_f), Some(val_f)) = (v.as_float().ok(), val.as_float().ok()) {
                    v_f == val_f
                } else if let (Some(v_s), Some(val_s)) = (v.clone().into_immutable_string().ok(), val.clone().into_immutable_string().ok()) {
                    v_s == val_s
                } else if let (Some(v_b), Some(val_b)) = (v.as_bool().ok(), val.as_bool().ok()) {
                    v_b == val_b
                } else {
                    false
                }
            })
        });

        engine.register_fn("len", |arr: Array| -> i64 {
            arr.len() as i64
        });

        engine.register_fn("has_key", |map: Map, key: &str| -> bool {
            map.contains_key(key)
        });

        engine
    }

    pub fn evaluate(&self, input: &JudgmentInput) -> anyhow::Result<JudgmentResult> {
        // 주기적으로 캐시 정리 (캐시 크기 기반)
        {
            let cache = self.rule_cache.lock().unwrap();
            if cache.len() > 100 {  // 100개 이상일 때 정리
                drop(cache);  // lock 해제
                self.cleanup_cache();
            }
        }

        let workflow = self
            .db
            .get_workflow(&input.workflow_id)?
            .ok_or_else(|| anyhow::anyhow!("Workflow not found: {}", input.workflow_id))?;

        let rule_expression = workflow
            .rule_expression
            .ok_or_else(|| anyhow::anyhow!("No rule expression defined for workflow: {}", input.workflow_id))?;

        // Rule 캐시 확인 (성능 최적화)
        {
            let mut cache = self.rule_cache.lock().unwrap();
            if let Some(cached) = cache.get_mut(&input.workflow_id) {
                cached.last_used = std::time::Instant::now();
            } else {
                cache.insert(
                    input.workflow_id.clone(),
                    CachedRule {
                        expression: rule_expression.clone(),
                        last_used: std::time::Instant::now(),
                    },
                );
            }
        }

        let mut scope = Scope::new();
        let mut registered_vars = Vec::new();

        // Register input data as variables (Array/Object 지원 확장)
        if let Some(obj) = input.input_data.as_object() {
            for (key, value) in obj {
                match value {
                    serde_json::Value::Number(n) => {
                        if let Some(i) = n.as_i64() {
                            scope.push(key.clone(), i);
                            registered_vars.push(format!("{} = {}", key, i));
                        } else if let Some(f) = n.as_f64() {
                            scope.push(key.clone(), f);
                            registered_vars.push(format!("{} = {}", key, f));
                        }
                    }
                    serde_json::Value::String(s) => {
                        scope.push(key.clone(), s.clone());
                        registered_vars.push(format!("{} = \"{}\"", key, s));
                    }
                    serde_json::Value::Bool(b) => {
                        scope.push(key.clone(), *b);
                        registered_vars.push(format!("{} = {}", key, b));
                    }
                    serde_json::Value::Array(arr) => {
                        // Array를 Rhai Array로 변환
                        let rhai_array: Array = arr
                            .iter()
                            .filter_map(|v| self.json_to_dynamic(v))
                            .collect();
                        scope.push(key.clone(), rhai_array);
                        registered_vars.push(format!("{} = [array with {} items]", key, arr.len()));
                    }
                    serde_json::Value::Object(obj) => {
                        // Object를 Rhai Map으로 변환
                        let rhai_map: Map = obj
                            .iter()
                            .filter_map(|(k, v)| {
                                self.json_to_dynamic(v).map(|d| (k.clone().into(), d))
                            })
                            .collect();
                        scope.push(key.clone(), rhai_map);
                        registered_vars.push(format!("{} = {{object with {} keys}}", key, obj.len()));
                    }
                    serde_json::Value::Null => {
                        // Null은 스킵
                    }
                }
            }
        }

        // Execute rule with detailed error handling (매번 새 Engine 생성)
        let engine = Self::create_engine();
        let result: bool = engine
            .eval_with_scope(&mut scope, &rule_expression)
            .map_err(|e| {
                anyhow::anyhow!(
                    "Rule evaluation failed\n\nRule: {}\n\nVariables:\n{}\n\nError: {}",
                    rule_expression,
                    registered_vars.join("\n"),
                    e
                )
            })?;

        // 신뢰도 계산 (Rule 기반은 높은 신뢰도)
        let confidence = if registered_vars.len() >= 3 {
            0.95 // 충분한 데이터
        } else if registered_vars.len() >= 1 {
            0.85 // 일부 데이터
        } else {
            0.7 // 데이터 부족
        };

        Ok(JudgmentResult {
            id: Uuid::new_v4().to_string(),
            workflow_id: input.workflow_id.clone(),
            result,
            confidence,
            method_used: "rule".to_string(),
            explanation: format!(
                "Rule 기반 판단 완료\n\n📋 Rule: {}\n\n📊 입력 데이터:\n{}\n\n✅ 결과: {}\n💯 신뢰도: {:.1}%",
                rule_expression,
                registered_vars.join("\n"),
                if result { "합격 (통과)" } else { "불합격 (거부)" },
                confidence * 100.0
            ),
        })
    }

    // JSON Value를 Rhai Dynamic으로 변환하는 헬퍼 함수
    fn json_to_dynamic(&self, value: &serde_json::Value) -> Option<Dynamic> {
        match value {
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Some(Dynamic::from(i))
                } else if let Some(f) = n.as_f64() {
                    Some(Dynamic::from(f))
                } else {
                    None
                }
            }
            serde_json::Value::String(s) => Some(Dynamic::from(s.clone())),
            serde_json::Value::Bool(b) => Some(Dynamic::from(*b)),
            serde_json::Value::Array(arr) => {
                let rhai_array: Array = arr
                    .iter()
                    .filter_map(|v| self.json_to_dynamic(v))
                    .collect();
                Some(Dynamic::from(rhai_array))
            }
            serde_json::Value::Object(obj) => {
                let rhai_map: Map = obj
                    .iter()
                    .filter_map(|(k, v)| {
                        self.json_to_dynamic(v).map(|d| (k.clone().into(), d))
                    })
                    .collect();
                Some(Dynamic::from(rhai_map))
            }
            serde_json::Value::Null => None,
        }
    }

    // Rule 캐시 정리 (오래된 항목 제거)
    pub fn cleanup_cache(&self) {
        let mut cache = self.rule_cache.lock().unwrap();
        let now = std::time::Instant::now();
        cache.retain(|_, cached| {
            now.duration_since(cached.last_used).as_secs() < 3600 // 1시간
        });
    }
}
