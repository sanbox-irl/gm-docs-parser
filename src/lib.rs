use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct GmManual {
    pub functions: BTreeMap<String, GmManualFunction>,
    pub variables: BTreeMap<String, GmManualVariable>,
    pub constants: BTreeMap<String, GmManualConstant>,
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct GmManualFunction {
    /// The name of the function
    pub name: String,

    /// The parameters of the function.
    pub parameters: Vec<GmManualFunctionParameter>,

    /// The count of the number of required parameters.
    pub required_parameters: usize,

    /// By `variadic`, we mean if the final parameter can take "infinite" arguments. Examples
    /// are `ds_list_add`, where users can invoke it as `ds_list_add(list, index, 1, 2, 3, 4 /* etc */);`
    pub is_variadic: bool,

    /// The example given in the Manual.
    pub example: String,

    /// The description of what the function does.
    pub description: String,

    /// What the function returns.
    pub returns: String,

    /// The link to the webpage.
    pub link: Url,
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct GmManualVariable {
    /// The name of the variable
    pub name: String,

    /// The example given in the Manual.
    pub example: String,

    /// The description of what the variable does.
    pub description: String,

    /// The type of the variable.
    pub returns: String,

    /// The link to the webpage.
    pub link: Url,
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Default, Serialize, Deserialize)]
pub struct GmManualFunctionParameter {
    /// The name of the parameter.
    pub parameter: String,

    /// A description given of the parameter.
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct GmManualConstant {
    /// The name of the constant
    pub name: String,

    /// A description of the constant
    pub description: String,

    /// The link to the webpage.
    pub link: Url,

    /// Additional descriptors present. Most of the time, this will be None, but can
    /// have some Descriptors and Values present.
    pub secondary_description: Option<BTreeMap<String, String>>,
}
