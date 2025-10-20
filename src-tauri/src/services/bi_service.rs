use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BiInsight {
    pub title: String,
    pub insights: Vec<String>,
    pub component_code: String,
    pub recommendations: Vec<String>,
}

pub struct BiService;

impl BiService {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self)
    }

    pub async fn generate_insight(&self, user_request: String) -> anyhow::Result<BiInsight> {
        // Simplified BI insight generation
        Ok(BiInsight {
            title: "불량률 분석 결과".to_string(),
            insights: vec![
                "평균 불량률: 2.3%".to_string(),
                "전주 대비 0.5% 감소".to_string(),
                "주요 불량 원인: 온도 초과 (60%)".to_string(),
            ],
            component_code: r#"
<div className="grid grid-cols-3 gap-4">
  <MetricCard title="불량률" value="2.3%" trend="down" />
  <MetricCard title="합격률" value="97.7%" trend="up" />
  <MetricCard title="총 검사수" value="1,234" />
</div>
            "#.to_string(),
            recommendations: vec![
                "온도 임계값을 85도로 조정 권장".to_string(),
                "진동 센서 교정 필요".to_string(),
            ],
        })
    }
}
