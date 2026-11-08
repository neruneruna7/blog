#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ArticleCompileWarning {
    pub(crate) url: String,
    pub(crate) reason: String,
}
