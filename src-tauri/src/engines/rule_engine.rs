use serde_json::Value;

/// AST 기반 안전한 Rule 평가 엔진 (eval() 금지!)
///
/// 지원하는 연산자:
/// - 비교: >, <, >=, <=, ==, !=
/// - 논리: &&, ||
/// - 괄호: (, )
///
/// 예시: "temperature > 80 && vibration < 50"
#[derive(Debug)]
pub struct RuleEngine {
    // 향후 확장: 함수 정의, 변수 캐싱 등
}

impl RuleEngine {
    pub fn new() -> Self {
        Self {}
    }

    /// Rule 표현식을 평가하여 true/false 반환
    ///
    /// # Arguments
    /// * `rule` - Rule 표현식 문자열 (예: "temperature > 80")
    /// * `data` - JSON 데이터 (예: {"temperature": 90})
    ///
    /// # Returns
    /// * `Ok(bool)` - 평가 결과 (true/false)
    /// * `Err(String)` - 파싱 또는 평가 오류
    pub fn evaluate(&self, rule: &str, data: &Value) -> Result<bool, String> {
        println!("🔍 [RuleEngine] Evaluating rule: {}", rule);

        // 1. Rule을 토큰으로 파싱
        let tokens = self.tokenize(rule)?;
        println!("   Tokens: {:?}", tokens);

        // 2. 토큰을 AST로 변환
        let ast = self.parse_tokens(&tokens)?;
        println!("   AST: {:?}", ast);

        // 3. AST를 평가
        let result = self.evaluate_ast(&ast, data)?;
        println!("   Result: {}", result);

        Ok(result)
    }

    /// Rule 문자열을 토큰으로 분리
    fn tokenize(&self, rule: &str) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        let mut current = String::new();

        let chars: Vec<char> = rule.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let ch = chars[i];

            match ch {
                // 공백 무시
                ' ' | '\t' | '\n' | '\r' => {
                    if !current.is_empty() {
                        tokens.push(self.parse_token(&current)?);
                        current.clear();
                    }
                }
                // 연산자
                '>' | '<' | '=' | '!' => {
                    if !current.is_empty() {
                        tokens.push(self.parse_token(&current)?);
                        current.clear();
                    }

                    // >=, <=, ==, != 처리
                    if i + 1 < chars.len() && chars[i + 1] == '=' {
                        let op = format!("{}{}", ch, chars[i + 1]);
                        tokens.push(Token::Operator(op));
                        i += 1;
                    } else if ch == '=' {
                        return Err("Single '=' not allowed. Use '==' for equality.".to_string());
                    } else if ch == '!' {
                        return Err("Single '!' not allowed. Use '!=' for inequality.".to_string());
                    } else {
                        tokens.push(Token::Operator(ch.to_string()));
                    }
                }
                // 논리 연산자
                '&' | '|' => {
                    if !current.is_empty() {
                        tokens.push(self.parse_token(&current)?);
                        current.clear();
                    }

                    // &&, || 처리
                    if i + 1 < chars.len() && chars[i + 1] == ch {
                        let op = format!("{}{}", ch, chars[i + 1]);
                        tokens.push(Token::Operator(op));
                        i += 1;
                    } else {
                        return Err(format!("Single '{}' not allowed. Use '&&' or '||'.", ch));
                    }
                }
                // 괄호
                '(' => {
                    if !current.is_empty() {
                        tokens.push(self.parse_token(&current)?);
                        current.clear();
                    }
                    tokens.push(Token::LParen);
                }
                ')' => {
                    if !current.is_empty() {
                        tokens.push(self.parse_token(&current)?);
                        current.clear();
                    }
                    tokens.push(Token::RParen);
                }
                // 기타 문자 (변수명, 숫자)
                _ => {
                    current.push(ch);
                }
            }

            i += 1;
        }

        // 마지막 토큰 처리
        if !current.is_empty() {
            tokens.push(self.parse_token(&current)?);
        }

        Ok(tokens)
    }

    /// 문자열을 토큰으로 파싱 (변수 또는 숫자)
    fn parse_token(&self, s: &str) -> Result<Token, String> {
        // 숫자인지 확인
        if let Ok(num) = s.parse::<f64>() {
            return Ok(Token::Number(num));
        }

        // 변수로 처리
        Ok(Token::Variable(s.to_string()))
    }

    /// 토큰을 AST로 변환 (Shunting-yard 알고리즘 단순화 버전)
    fn parse_tokens(&self, tokens: &[Token]) -> Result<Expr, String> {
        let mut pos = 0;
        self.parse_or(tokens, &mut pos)
    }

    /// OR 표현식 파싱 (가장 낮은 우선순위)
    fn parse_or(&self, tokens: &[Token], pos: &mut usize) -> Result<Expr, String> {
        let mut left = self.parse_and(tokens, pos)?;

        while *pos < tokens.len() {
            if let Token::Operator(op) = &tokens[*pos] {
                if op == "||" {
                    *pos += 1;
                    let right = self.parse_and(tokens, pos)?;
                    left = Expr::BinaryOp {
                        left: Box::new(left),
                        op: "||".to_string(),
                        right: Box::new(right),
                    };
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        Ok(left)
    }

    /// AND 표현식 파싱 (중간 우선순위)
    fn parse_and(&self, tokens: &[Token], pos: &mut usize) -> Result<Expr, String> {
        let mut left = self.parse_comparison(tokens, pos)?;

        while *pos < tokens.len() {
            if let Token::Operator(op) = &tokens[*pos] {
                if op == "&&" {
                    *pos += 1;
                    let right = self.parse_comparison(tokens, pos)?;
                    left = Expr::BinaryOp {
                        left: Box::new(left),
                        op: "&&".to_string(),
                        right: Box::new(right),
                    };
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        Ok(left)
    }

    /// 비교 표현식 파싱 (높은 우선순위)
    fn parse_comparison(&self, tokens: &[Token], pos: &mut usize) -> Result<Expr, String> {
        let left = self.parse_primary(tokens, pos)?;

        if *pos < tokens.len() {
            if let Token::Operator(op) = &tokens[*pos] {
                if matches!(op.as_str(), ">" | "<" | ">=" | "<=" | "==" | "!=") {
                    let op_clone = op.clone();
                    *pos += 1;
                    let right = self.parse_primary(tokens, pos)?;
                    return Ok(Expr::BinaryOp {
                        left: Box::new(left),
                        op: op_clone,
                        right: Box::new(right),
                    });
                }
            }
        }

        Ok(left)
    }

    /// Primary 표현식 파싱 (변수, 숫자, 괄호)
    fn parse_primary(&self, tokens: &[Token], pos: &mut usize) -> Result<Expr, String> {
        if *pos >= tokens.len() {
            return Err("Unexpected end of expression".to_string());
        }

        match &tokens[*pos] {
            Token::Number(n) => {
                let num = *n;
                *pos += 1;
                Ok(Expr::Number(num))
            }
            Token::Variable(v) => {
                let var = v.clone();
                *pos += 1;
                Ok(Expr::Variable(var))
            }
            Token::LParen => {
                *pos += 1; // Skip '('
                let expr = self.parse_or(tokens, pos)?;
                if *pos >= tokens.len() || tokens[*pos] != Token::RParen {
                    return Err("Missing closing parenthesis".to_string());
                }
                *pos += 1; // Skip ')'
                Ok(expr)
            }
            _ => Err(format!("Unexpected token: {:?}", tokens[*pos])),
        }
    }

    /// AST를 평가하여 boolean 결과 반환
    fn evaluate_ast(&self, expr: &Expr, data: &Value) -> Result<bool, String> {
        match expr {
            Expr::Number(_) => Err("Cannot evaluate number as boolean".to_string()),
            Expr::Variable(name) => {
                // 변수 값 조회
                let value = data
                    .get(name)
                    .ok_or(format!("Variable '{}' not found in data", name))?;

                // Boolean으로 변환 (truthy 체크)
                Ok(self.is_truthy(value))
            }
            Expr::BinaryOp { left, op, right } => {
                match op.as_str() {
                    // 논리 연산자
                    "&&" => {
                        let left_result = self.evaluate_ast(left, data)?;
                        let right_result = self.evaluate_ast(right, data)?;
                        Ok(left_result && right_result)
                    }
                    "||" => {
                        let left_result = self.evaluate_ast(left, data)?;
                        let right_result = self.evaluate_ast(right, data)?;
                        Ok(left_result || right_result)
                    }
                    // 비교 연산자
                    ">" | "<" | ">=" | "<=" | "==" | "!=" => {
                        let left_val = self.evaluate_value(left, data)?;
                        let right_val = self.evaluate_value(right, data)?;

                        match op.as_str() {
                            ">" => Ok(left_val > right_val),
                            "<" => Ok(left_val < right_val),
                            ">=" => Ok(left_val >= right_val),
                            "<=" => Ok(left_val <= right_val),
                            "==" => Ok((left_val - right_val).abs() < f64::EPSILON),
                            "!=" => Ok((left_val - right_val).abs() >= f64::EPSILON),
                            _ => unreachable!(),
                        }
                    }
                    _ => Err(format!("Unsupported operator: {}", op)),
                }
            }
        }
    }

    /// AST를 평가하여 숫자 결과 반환
    fn evaluate_value(&self, expr: &Expr, data: &Value) -> Result<f64, String> {
        match expr {
            Expr::Number(n) => Ok(*n),
            Expr::Variable(name) => {
                let value = data
                    .get(name)
                    .ok_or(format!("Variable '{}' not found in data", name))?;

                // 숫자로 변환
                value
                    .as_f64()
                    .ok_or(format!("Variable '{}' is not a number", name))
            }
            Expr::BinaryOp { .. } => {
                Err("Cannot evaluate boolean expression as number".to_string())
            }
        }
    }

    /// Truthy 체크 (JavaScript 스타일)
    fn is_truthy(&self, value: &Value) -> bool {
        match value {
            Value::Null => false,
            Value::Bool(b) => *b,
            Value::Number(n) => n.as_f64().map(|f| f != 0.0).unwrap_or(false),
            Value::String(s) => !s.is_empty(),
            Value::Array(arr) => !arr.is_empty(),
            Value::Object(obj) => !obj.is_empty(),
        }
    }
}

/// 토큰 타입
#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f64),
    Variable(String),
    Operator(String), // >, <, >=, <=, ==, !=, &&, ||
    LParen,
    RParen,
}

/// AST 표현식
#[derive(Debug, Clone)]
enum Expr {
    Number(f64),
    Variable(String),
    BinaryOp {
        left: Box<Expr>,
        op: String,
        right: Box<Expr>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_simple_comparison() {
        let engine = RuleEngine::new();
        let data = json!({"temperature": 90});

        assert!(engine.evaluate("temperature > 80", &data).unwrap());
        assert!(!engine.evaluate("temperature < 80", &data).unwrap());
        assert!(engine.evaluate("temperature >= 90", &data).unwrap());
        assert!(engine.evaluate("temperature <= 90", &data).unwrap());
        assert!(engine.evaluate("temperature == 90", &data).unwrap());
        assert!(!engine.evaluate("temperature != 90", &data).unwrap());
    }

    #[test]
    fn test_logical_operators() {
        let engine = RuleEngine::new();
        let data = json!({"temperature": 90, "vibration": 45});

        assert!(engine
            .evaluate("temperature > 80 && vibration < 50", &data)
            .unwrap());
        assert!(!engine
            .evaluate("temperature > 80 && vibration > 50", &data)
            .unwrap());
        assert!(engine
            .evaluate("temperature > 80 || vibration > 50", &data)
            .unwrap());
    }

    #[test]
    fn test_parentheses() {
        let engine = RuleEngine::new();
        let data = json!({"a": 10, "b": 20, "c": 30});

        assert!(engine.evaluate("(a > 5 && b < 25) || c > 25", &data).unwrap());
        assert!(!engine.evaluate("a > 5 && (b < 15 || c < 25)", &data).unwrap());
    }

    #[test]
    fn test_variable_not_found() {
        let engine = RuleEngine::new();
        let data = json!({"temperature": 90});

        let result = engine.evaluate("pressure > 100", &data);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Variable 'pressure' not found"));
    }

    #[test]
    fn test_invalid_syntax() {
        let engine = RuleEngine::new();
        let data = json!({"temperature": 90});

        // Single '=' not allowed
        assert!(engine.evaluate("temperature = 90", &data).is_err());

        // Single '&' not allowed
        assert!(engine
            .evaluate("temperature > 80 & vibration < 50", &data)
            .is_err());
    }
}
