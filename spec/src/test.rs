#![cfg(test)]

#[derive(Deserialize)]
pub struct RuntimeValue {
    #[serde(rename = "type")]
    pub value_type: String,
    pub value: String,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum Action {
    #[serde(rename = "invoke")]
    Invoke { field: String, args: Vec<RuntimeValue> }
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum Command {
    #[serde(rename = "module")]
    Module { line: u64, filename: String },
    #[serde(rename = "assert_return")]
    AssertReturn { 
        line: u64, 
        action: Action,
        expected: Vec<RuntimeValue>,
    },
    #[serde(rename = "assert_trap")]
    AssertTrap {
        line: u64,
        action: Action,
        text: String,
    },
}

#[derive(Deserialize)]
pub struct Spec {
    pub source_filename: String,
    pub commands: Vec<Command>,
}