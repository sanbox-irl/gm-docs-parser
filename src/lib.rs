use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Default)]
pub struct GmFunction {
    pub name: String,
    // pub signature: String, // do we even need this?
    pub parameters: Vec<GmParameter>,
    pub min_parameter: usize,
    pub max_parameter: usize,
    pub example: GmExample,
    pub description: String,
    pub returns: String,
    pub link: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Default)]
pub struct GmParameter {
    pub parameter: String,
    pub documentation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Default)]
pub struct GmExample {
    pub code: String,
    pub description: String,
}
