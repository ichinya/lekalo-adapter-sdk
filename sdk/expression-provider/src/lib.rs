//! Wire types and bounded stdio transport for expression source generators.
use serde::Deserialize;
use serde_json::{json, Value};
use std::io::{self, Read};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub schema: String,
    pub identity: String,
    pub records: Vec<Record>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub id: String,
    pub params: Vec<Param>,
    pub result: ExprType,
    pub body: Code,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Param {
    pub scope: String,
    pub field: String,
    pub ty: ExprType,
    pub nullable: bool,
}

#[derive(Deserialize, Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ScalarType {
    /// A boolean.
    Bool,
    /// A bounded signed integer.
    Int,
    /// A bounded BMP-only UTF-8 string.
    Str,
    /// A canonical UTC timestamp.
    DateTime,
    /// An integer-seconds duration.
    Duration,
}

impl ScalarType {
    /// The exact wire spelling.
    pub const fn key(self) -> &'static str {
        match self {
            Self::Bool => "bool",
            Self::Int => "int",
            Self::Str => "string",
            Self::DateTime => "datetime",
            Self::Duration => "duration",
        }
    }

    /// Parse one wire spelling.
    pub fn parse(key: &str) -> Option<Self> {
        match key {
            "bool" => Some(Self::Bool),
            "int" => Some(Self::Int),
            "string" => Some(Self::Str),
            "datetime" => Some(Self::DateTime),
            "duration" => Some(Self::Duration),
            _ => None,
        }
    }

    /// Whether the scalar type participates in ordering operators.
    pub const fn ordered(self) -> bool {
        !matches!(self, Self::Bool)
    }
}

/// One expression type: a scalar or a homogeneous set of scalars.
#[derive(Deserialize, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ExprType {
    /// A scalar value.
    Scalar(ScalarType),
    /// A homogeneous, duplicate-free, sorted set.
    Set(ScalarType),
}

impl ExprType {
    /// Parse one wire spelling (`bool`, `set:int`, …).
    pub fn parse(key: &str) -> Option<Self> {
        if let Some(inner) = key.strip_prefix("set:") {
            return ScalarType::parse(inner).map(Self::Set);
        }
        ScalarType::parse(key).map(Self::Scalar)
    }

    /// The exact wire spelling.
    pub fn key(&self) -> String {
        match self {
            Self::Scalar(scalar) => scalar.key().to_owned(),
            Self::Set(scalar) => format!("set:{}", scalar.key()),
        }
    }

    /// The scalar type when this is a scalar.
    pub fn as_scalar(&self) -> Option<ScalarType> {
        match self {
            Self::Scalar(scalar) => Some(*scalar),
            Self::Set(_) => None,
        }
    }
}

/// The typed intermediate the per-target printers consume.
#[derive(serde::Deserialize, Clone, Debug)]
pub enum Code {
    /// A plain (untyped) integer literal; only the `lekCmpStr`
    /// zero comparison uses it, where a Node `BigInt` would never
    /// equal the helper's plain number.
    PlainInt(i64),
    /// An integer literal (an int, duration seconds, or datetime
    /// seconds — always in a statically typed position).
    Int(i64),
    /// A string literal.
    Str(String),
    /// A boolean literal.
    Bool(bool),
    /// A prelude call with one shared name.
    Call(String, Vec<Code>),
    /// An ordered comparison (never on booleans; string operands
    /// route through `lekCmpStr` at compile time).
    Compare(String, Box<Code>, Box<Code>),
    /// Strict equality (or inequality when negated).
    Equal(bool, Box<Code>, Box<Code>),
    /// Conjunction over two or more operands.
    And(Vec<Code>),
    /// Disjunction over two or more operands.
    Or(Vec<Code>),
    /// Negation.
    Not(Box<Code>),
    /// The branch without a loop; the branch type selects the typed
    /// Go helper.
    If(Box<Code>, Box<Code>, Box<Code>, BranchType),
}

/// The branch typing the Go printer needs. Every branch is a
/// concrete Go type: an `if` over sets closes over the element type
/// so the encoded result stays a typed slice, never an erased
/// `any`.
#[derive(serde::Deserialize, Clone, Copy, Debug)]
pub enum BranchType {
    /// An integer-valued branch (int, duration, or datetime).
    Int,
    /// A string-valued branch.
    Str,
    /// A boolean-valued branch.
    Bool,
    /// A set-valued branch with one concrete element type.
    Set(ScalarType),
}

pub fn serve(
    id: &str,
    version: &str,
    render: fn(&Document) -> String,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut bytes = Vec::new();
    io::stdin()
        .take(8 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 8 * 1024 * 1024 {
        return Err("input limit".into());
    }
    let mut request: Value = serde_json::from_slice(&bytes)?;
    if request["protocol"] != "lekalo/runtime/v1" {
        return Err("unsupported protocol".into());
    }
    let result = match request["operation"].as_str() {
        Some("describe") => {
            json!({"id":id, "version":version, "capabilities":[], "rules_api":[], "operations":["expressions.render"]})
        }
        Some("expressions.render") => {
            let doc: Document = serde_json::from_value(request["input"].take())?;
            if doc.schema != "lekalo/expression-projection/v1" {
                return Err("unsupported projection".into());
            }
            json!({"schema":"lekalo/expression-program/v1", "program":render(&doc)})
        }
        _ => return Err("unsupported operation".into()),
    };
    serde_json::to_writer(
        io::stdout().lock(),
        &json!({"protocol":request["protocol"],"request_id":request["request_id"],"result":result}),
    )?;
    Ok(())
}
