use serde::Serialize;

use super::mid::CompiledArticle;

#[expect(dead_code, reason = "サイト出力段階はまだ実装していない")]
pub struct CompiledSiteBundle {
    pub articles: Vec<CompiledArticle>,
    pub site_config: SiteConfig,
}

/// 制約とかまだ特に考えていないので，Stringのままで．
#[derive(Debug, Clone, Serialize)]
#[expect(dead_code, reason = "サイト出力段階はまだ実装していない")]
pub struct SiteConfig {
    title: String,
    description: String,
    langurage: String,
}
