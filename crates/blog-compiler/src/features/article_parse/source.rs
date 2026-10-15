use std::collections::HashMap;

use crate::commons::article::Slug;

use super::{article::Article, site::Site};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSource {
    pub articles: HashMap<Slug, Article>,
    pub site: Site,
}
