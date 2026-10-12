use crate::usecases::compile::article_compile;
use std::error::Error;

mod error;
mod usecases;

fn main() -> Result<(), Box<dyn Error>> {
    article_compile()
}
