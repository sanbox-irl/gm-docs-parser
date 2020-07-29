use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Default)]
pub struct GmFunction {
    /// The name of the function
    pub name: String,

    /// The parameters of the function.
    pub parameters: Vec<GmParameter>,

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

    /// The link to the webpage. For now,
    pub link: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Default)]
pub struct GmParameter {
    pub parameter: String,
    pub description: String,
}
