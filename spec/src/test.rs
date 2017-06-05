#![cfg(test)]

#[derive(Deserialize, Debug)]
pub struct RuntimeValue {
    #[serde(rename = "type")]
    pub value_type: String,
    pub value: String,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Action {
    #[serde(rename = "invoke")]
    Invoke { field: String, args: Vec<RuntimeValue> }
}

#[derive(Deserialize, Debug)]
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
    #[serde(rename = "assert_return_canonical_nan")]
    AssertReturnCanonicalNan {
        line: u64,
        action: Action,
    },
    #[serde(rename = "assert_return_arithmetic_nan")]
    AssertReturnArithmeticNan {
        line: u64,
        action: Action,
    },
    #[serde(rename = "assert_trap")]
    AssertTrap {
        line: u64,
        action: Action,
        text: String,
    },
    #[serde(rename = "assert_invalid")]
    AssertInvalid {
        line: u64,
        filename: String,
        text: String,
    },
    #[serde(rename = "action")]
    Action {
        line: u64,
        action: Action,
    }
}

#[derive(Deserialize, Debug)]
pub struct Spec {
    pub source_filename: String,
    pub commands: Vec<Command>,
}