#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ArticleTemplate {
    pub(super) name: String,
    pub(super) contents: Vec<u8>,
}

impl ArticleTemplate {
    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn contents(&self) -> &[u8] {
        &self.contents
    }
}
