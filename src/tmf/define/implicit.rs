//! Implicit modeling extension.
//!
//! This module implements the 3MF Implicit Extension specification which enables
//! the definition of closed-form functions using implicit mathematical representations.
//! The implicit extension works in conjunction with the volumetric extension to provide
//! field-based geometric descriptions as an alternative to traditional mesh-based modeling.

use std::{collections::HashMap, fmt};
use std::sync::Arc;
use std::str::FromStr;
use super::error::{Error, Result};

/// Implicit function resource defining a node graph.
/// Owned defined namespace(xmlns attribute)
#[derive(Debug, Clone)]
pub struct ImplicitFunction {
    /// Unique resource ID.
    pub id: u32,
    /// Unique identifier for this implicit function.
    pub identifier: String,
    /// Optional display name.
    pub display_name: Option<String>,
    /// Input parameter, ref identifier prefix is "inputs".
    pub r#in: Input,
    /// Output parameter.
    pub out: Output,
    /// Nodes in the function graph.
    pub nodes: Vec<Node>,
}

impl ImplicitFunction {
    /// Create a new implicit function.
    pub fn new(id: u32, identifier: impl Into<String>) -> Self {
        Self {
            id,
            identifier: identifier.into(),
            display_name: None,
            r#in: Input::default(),
            out: Output::default(),
            nodes: Vec::new(),
        }
    }
    /// Set the input parameter.
    pub fn set_input(&mut self, input: Input) -> &mut Self {
        self.r#in = input;
        self
    }

    /// Set the output parameter.
    pub fn set_output(&mut self, output: Output) -> &mut Self {
        self.out = output;
        self
    }

    /// Add a node.
    pub fn add_node(&mut self, node: Node) -> &mut Self {
        self.nodes.push(node);
        self
    }

    /// Get a node by identifier.
    pub fn get_node(&self, identifier: &str) -> Option<&Node> {
        self.nodes.iter().find(|n| n.identifier == identifier)
    }
}

/// Function input parameter.
#[derive(Debug, Clone, Default)]
pub struct Input {
    /// Values of the input.
    pub values: Vec<Value>,
}

/// Function output parameter.
#[derive(Debug, Default, Clone)]
pub struct Output {
    /// Values of the output.
    pub values: Vec<Value>,
}

/// Function input or output parameter value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Value {
    /// Unique identifier.
    pub identifier: String,
    /// Optional display name.
    pub display_name: Option<String>,
    /// Value type.
    pub r#type: ValueType,
}

/// Value type for function parameters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueType {
    Data(ValueData),
    Ref(ValueRef),
}

/// Value data type for function parameters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueData {
    /// Scalar value.
    Scalar,
    /// 3D vector.
    Vector,
    /// 4x4 matrix.
    Matrix,
    /// Resource ID.
    ResourceId,
}

impl fmt::Display for ValueData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueData::Scalar => write!(f, "scalar"),
            ValueData::Vector => write!(f, "vector"),
            ValueData::Matrix => write!(f, "matrix"),
            ValueData::ResourceId => write!(f, "resourceid"),
        }
    }
}

impl FromStr for ValueData {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "scalar" => Ok(ValueData::Scalar),
            "vector" => Ok(ValueData::Vector),
            "matrix" => Ok(ValueData::Matrix),
            "resourceid" => Ok(ValueData::ResourceId),
            _ => Err(Error::InvalidStructure(format!("unknown value data type: {}", s))),
        }
    }
}

/// Data type for function parameters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueRef {
    /// Scalar value reference.
    ScalarRef(String),
    /// 3D vector reference.
    VectorRef(String),
    /// 4x4 matrix reference.
    MatrixRef(String),
    /// Resource ID reference.
    ResourceRef(String),
}

impl fmt::Display for ValueRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueRef::ScalarRef(r) => write!(f, "scalarref={}", r),
            ValueRef::VectorRef(r) => write!(f, "vectorref={}", r),
            ValueRef::MatrixRef(r) => write!(f, "matrixref={}", r),
            ValueRef::ResourceRef(r) => write!(f, "resourceref={}", r),
        }
    }
}

impl FromStr for ValueRef {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let vs = s.split('=').collect::<Vec<_>>();
        if vs.len() == 2 {
            match vs[0].to_lowercase().as_str() {
                "scalarref" => Ok(ValueRef::ScalarRef(vs[1].to_string())),
                "vectorref" => Ok(ValueRef::VectorRef(vs[1].to_string())),
                "matrixref" => Ok(ValueRef::MatrixRef(vs[1].to_string())),
                "resourceref" => Ok(ValueRef::ResourceRef(vs[1].to_string())),
                _ => Err(Error::InvalidStructure(format!("unknown value ref type: {}", s))),
            }
        } else {
            Err(Error::InvalidStructure(format!("invalid value ref format: {}", s)))
        }
    }
}

/// A node in the implicit function graph.
#[derive(Debug, Clone)]
pub struct Node {
    /// Unique identifier for this node.
    pub identifier: String,
    /// Optional display name.
    pub display_name: Option<String>,
    /// Optional tag for categorization.
    pub tag: Option<String>,
    /// Node type.
    pub r#type: Arc<Box<dyn NodeType>>,
    /// Input connections.
    pub r#in: Input,
    /// Output definitions.
    pub out: Output,
}

impl Node {
    /// Create a new node.
    pub fn new<T: NodeType + 'static>(identifier: impl Into<String>, r#type: T) -> Self {
        Self {
            identifier: identifier.into(),
            display_name: None,
            tag: None,
            r#type: Arc::new(Box::new(r#type)),
            r#in: Input::default(),
            out: Output::default(),
        }
    }

    /// Set the display name.
    pub fn set_display_name(&mut self, name: impl Into<String>) -> &mut Self {
        self.display_name = Some(name.into());
        self
    }

    /// Set the tag.
    pub fn set_tag(&mut self, tag: impl Into<String>) -> &mut Self {
        self.tag = Some(tag.into());
        self
    }

    /// Set the input connections.
    pub fn set_input(&mut self, input: Input) -> &mut Self {
        self.r#in = input;
        self
    }

    /// Set the output definitions.
    pub fn set_output(&mut self, output: Output) -> &mut Self {
        self.out = output;
        self
    }
}

/// Native node types supported by the implicit extension.
pub trait NodeType: fmt::Display + fmt::Debug {
    fn name(&self) -> String {
        self.to_string()
    }
}

/// Addition node, 
/// The inputs: two(MUST identifier "A" and "B") of [scalarref|vectorref|matrixref],
/// The output: one(MUST identifier "result") of [scalar|vector|matrix], two-to-one type correspondence.
#[derive(Debug, Clone)]
pub struct Addition;

impl NodeType for Addition {}

impl fmt::Display for Addition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "addition")
    }
}

/// Subtraction node, 
/// The inputs: two(MUST identifier "A" and "B") of [scalarref|vectorref|matrixref], 
/// The output: one(MUST identifier "result") of [scalar|vector|matrix], two-to-one type correspondence.
#[derive(Debug, Clone)]
pub struct Subtraction;

impl NodeType for Subtraction {}

impl fmt::Display for Subtraction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "subtraction")
    }
}

/// Multiplication node, 
/// The inputs: two(MUST identifier "A" and "B") of [scalarref|vectorref|matrixref], 
/// The output: one(MUST identifier "result") of [scalar|vector|matrix], two-to-one type correspondence.
#[derive(Debug, Clone)]
pub struct Multiplication;

impl NodeType for Multiplication {}

impl fmt::Display for Multiplication {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "multiplication")
    }
}

/// Division node,
/// The inputs: two(MUST identifier "A" and "B") of [scalarref|vectorref|matrixref], 
/// The output: one(MUST identifier "result") of [scalar|vector|matrix], two-to-one type correspondence.
#[derive(Debug, Clone)]
pub struct Division;

impl NodeType for Division {}

impl fmt::Display for Division {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "division")
    }
}

/// Constant scalar value,
/// The input: zero,
/// The output: one(MUST identifier "value") of [scalar].
#[derive(Debug, Clone)]
pub struct  Constant { value: f64 }

impl Constant {
    pub fn new(value: f64) -> Self {
        Self { value }
    }
}

impl NodeType for Constant {
    fn name(&self) -> String {
        "constant".to_string()
    }
}

impl fmt::Display for Constant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}|value={}", self.name(), self.value)
    }
}

/// Constant vector value, 
/// The input: zero,
/// The output: one(MUST identifier "vector") of [vector].
#[derive(Debug, Clone)]
pub struct ConstVec { x: f64, y: f64, z: f64 }

impl ConstVec {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}

impl NodeType for ConstVec {
    fn name(&self) -> String {
        "constvec".to_string()
    }
}

impl fmt::Display for ConstVec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}|x={}|y={}|z={}", self.name(), self.x, self.y, self.z)
    }
}

/// Constant matrix value, 
/// The input: zero,
/// The output: one(MUST identifier "matrix") of [matrix].
#[derive(Debug, Clone)]
pub struct  ConstMat { matrix: [f64; 16] }

impl ConstMat {
    pub fn new(matrix: [f64; 16]) -> Self {
        Self { matrix }
    }
}

impl NodeType for ConstMat {
    fn name(&self) -> String {
        "constmat".to_string()
    }
}

impl fmt::Display for ConstMat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}|matrix={}", self.name(), self.matrix.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(" "))
    }
}

/// Constant resource ID, 
/// The input: zero,
/// The output: one(MUST identifier "value") of [resourceid].
#[derive(Debug, Clone)]
pub struct ConstResourceId { value: u32 }

impl ConstResourceId {
    pub fn new(value: u32) -> Self {
        Self { value }
    }
}

impl NodeType for ConstResourceId {
    fn name(&self) -> String {
        "constresourceid".to_string()
    }
}

impl fmt::Display for ConstResourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}|value={}", self.name(), self.value)
    }
}

/// Compose vector from scalars, 
/// The inputs: three(MUST identifier "x", "y", "z") of [scalarref], 
/// The output: one(MUST identifier "result") of [vector].
#[derive(Debug, Clone)]
pub struct ComposeVector;

impl NodeType for ComposeVector {}

impl fmt::Display for ComposeVector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "composevector")
    }
}

/// Create vector from scalar, 
/// The input: one(MUST identifier "A") of [scalarref], 
/// The output: one(MUST identifier "result") of [vector].
#[derive(Debug, Clone)]
pub struct VectorFromScalar;

impl NodeType for VectorFromScalar {}

impl fmt::Display for VectorFromScalar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "vectorfromscalar")
    }
}

/// Decompose vector to scalars, 
/// The inputs: one(MUST identifier "A") of [vectorref], 
/// The outputs: three(MUST identifier "x", "y", "z") of [scalar].
#[derive(Debug, Clone)]
pub struct DecomposeVector;

impl NodeType for DecomposeVector {}

impl fmt::Display for DecomposeVector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "decomposevector")
    }
}

/// Compose matrix from scalars, 
/// The inputs: sixteen(MUST identifier "m00", "m01", "m02", "m03", "m10", "m11", "m12", "m13", "m20", "m21", "m22", "m23", "m30", "m31", "m32", "m33") of [scalarref], 
/// The output: one(MUST identifier "result") of [matrix].
#[derive(Debug, Clone)]
pub struct ComposeMatrix;

impl NodeType for ComposeMatrix {}

impl fmt::Display for ComposeMatrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "composematrix")
    }
}

/// Compose matrix from column vectors, 
/// The inputs: four(MUST identifier "A", "B", "C", "D") of [vectorref], 
/// The output: one(MUST identifier "result") of [matrix].
#[derive(Debug, Clone)]
pub struct MatrixFromColumns;

impl NodeType for MatrixFromColumns {}

impl fmt::Display for MatrixFromColumns {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "matrixfromcolumns")
    }
}

/// Compose matrix from row vectors, 
/// The inputs: four(MUST identifier "A", "B", "C", "D") of [vectorref], 
/// The output: one(MUST identifier "result") of [matrix].
#[derive(Debug, Clone)]
pub struct MatrixFromRows;

impl NodeType for MatrixFromRows {}

impl fmt::Display for MatrixFromRows {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "matrixfromrows")
    }
}

/// Dot product,
/// The inputs: two(MUST identifier "A" and "B") of [vectorref], 
/// The output: one(MUST identifier "result") of [scalar].
#[derive(Debug, Clone)]
pub struct Dot;

impl NodeType for Dot {}

impl fmt::Display for Dot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "dot")
    }
}

/// Cross product,
/// The inputs: two(MUST identifier "A" and "B") of [vectorref], 
/// The output: one(MUST identifier "result") of [vector].
#[derive(Debug, Clone)]
pub struct Cross;

impl NodeType for Cross {}

impl fmt::Display for Cross {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "cross")
    }
}

/// Matrix-vector multiplication,
/// The inputs: one(MUST identifier "A") of [matrixref] and one(MUST identifier "B") of [vectorref], 
/// The output: one(MUST identifier "result") of [vector].
#[derive(Debug, Clone)]
pub struct MatrixVectorMultiplication;

impl NodeType for MatrixVectorMultiplication {}

impl fmt::Display for MatrixVectorMultiplication {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "matrixvectormultiplication")
    }
}

/// Matrix transpose,
/// The input: one(MUST identifier "A") of [matrixref], 
/// The output: one(MUST identifier "result") of [matrix].
#[derive(Debug, Clone)]
pub struct Transpose;

impl NodeType for Transpose {}

impl fmt::Display for Transpose {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "transpose")
    }
}

/// Matrix inverse,
/// The input: one(MUST identifier "A") of [matrixref], 
/// The output: one(MUST identifier "result") of [matrix].
#[derive(Debug, Clone)]
pub struct Inverse;

impl NodeType for Inverse {}

impl fmt::Display for Inverse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "inverse")
    }
}

/// Sine function,
/// The input: one(MUST identifier "A") of [scalarref|vectorref], 
/// The output: one(MUST identifier "result") of [scalar|vector], one-to-one correspondence.
#[derive(Debug, Clone)]
pub struct Sin;

impl NodeType for Sin {}

impl fmt::Display for Sin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "sin")
    }
}

/// Cosine function,
/// The input: one(MUST identifier "A") of [scalarref|vectorref], 
/// The output: one(MUST identifier "result") of [scalar|vector], one-to-one correspondence.
#[derive(Debug, Clone)]
pub struct Cos;

impl NodeType for Cos {}

impl fmt::Display for Cos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "cos")
    }
}

/// Tangent function,
/// The input: one(MUST identifier "A") of [scalarref|vectorref], 
/// The output: one(MUST identifier "result") of [scalar|vector], one-to-one correspondence.
#[derive(Debug, Clone)]
pub struct Tan;

impl NodeType for Tan {}

impl fmt::Display for Tan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "tan")
    }
}

/// Arcsine function,
/// The input: one(MUST identifier "A") of [scalarref|vectorref], 
/// The output: one(MUST identifier "result") of [scalar|vector], one-to-one correspondence.
#[derive(Debug, Clone)]
pub struct Asin;

impl NodeType for Asin {}

impl fmt::Display for Asin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "asin")
    }
}

/// Arccosine function,
/// The input: one(MUST identifier "A") of [scalarref|vectorref], 
/// The output: one(MUST identifier "result") of [scalar|vector], one-to-one correspondence.
#[derive(Debug, Clone)]
pub struct Acos;

impl NodeType for Acos {}

impl fmt::Display for Acos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "acos")
    }
}

/// Arctangent function,
/// The input: one(MUST identifier "A") of [scalarref|vectorref], 
/// The output: one(MUST identifier "result") of [scalar|vector], one-to-one correspondence.
#[derive(Debug, Clone)]
pub struct Atan;

impl NodeType for Atan {}

impl fmt::Display for Atan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "atan")
    }
}

/// Two-argument arctangent function, 
/// The inputs: two(MUST identifier "A" and "B") of [scalarref|vectorref], 
/// The output: one(MUST identifier "result") of [scalar|vector], two-to-one type correspondence.
#[derive(Debug, Clone)]
pub struct Atan2;

impl NodeType for Atan2 {}

impl fmt::Display for Atan2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "atan2")
    }
}

/// Minimum value,
/// The inputs: two(MUST identifier "A" and "B") of [scalarref|vectorref|matrixref], 
/// The output: one(MUST identifier "result") of [scalar|vector|matrix], two-to-one type correspondence.
#[derive(Debug, Clone)]
pub struct Min;

impl NodeType for Min {}

impl fmt::Display for Min {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "min")
    }
}

/// Maximum value,
/// The inputs: two(MUST identifier "A" and "B") of [scalarref|vectorref|matrixref], 
/// The output: one(MUST identifier "result") of [scalar|vector|matrix], two-to-one type correspondence.
#[derive(Debug, Clone)]
pub struct Max;

impl NodeType for Max {}

impl fmt::Display for Max {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "max")
    }
}

/// Absolute value,
/// The input: one(MUST identifier "A") of [scalarref|vectorref|matrixref], 
/// The output: one(MUST identifier "result") of [scalar|vector|matrix], one-to-one correspondence.
#[derive(Debug, Clone)]
pub struct Abs;

impl NodeType for Abs {}

impl fmt::Display for Abs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "abs")
    }
}

/// Remainder of floating point division,
/// The inputs: two(MUST identifier "A" and "B") of [scalarref|vectorref|matrixref], 
/// The output: one(MUST identifier "result") of [scalar|vector|matrix], two-to-one type correspondence.
#[derive(Debug, Clone)]
pub struct Fmod;

impl NodeType for Fmod {}

impl fmt::Display for Fmod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "fmod")
    }
}

/// Modulo operation,
/// While fmod is the 'C' fmod, mod has the same behaviour as the glsl mod function,
/// The inputs: two(MUST identifier "A" and "B") of [scalarref|vectorref|matrixref], 
/// The output: one(MUST identifier "result") of [scalar|vector|matrix], two-to-one type correspondence.
#[derive(Debug, Clone)]
pub struct Mod;

impl NodeType for Mod {}

impl fmt::Display for Mod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "mod")
    }
}

/// Power function,
/// The inputs: two(MUST identifier "A" and "B") of [scalarref|vectorref|matrixref], 
/// The output: one(MUST identifier "result") of [scalar|vector|matrix], two-to-one type correspondence.
#[derive(Debug, Clone)]
pub struct Pow;

impl NodeType for Pow {}

impl fmt::Display for Pow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "pow")
    }
}

/// Square root,
/// The input: one(MUST identifier "A") of [scalarref|vectorref|matrixref], 
/// The output: one(MUST identifier "result") of [scalar|vector|matrix], one-to-one correspondence.
#[derive(Debug, Clone)]
pub struct Sqrt;

impl NodeType for Sqrt {}

impl fmt::Display for Sqrt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "sqrt")
    }
}

/// Exponential function,
/// The input: one(MUST identifier "A") of [scalarref|vectorref|matrixref], 
/// The output: one(MUST identifier "result") of [scalar|vector|matrix], one-to-one correspondence.
#[derive(Debug, Clone)]
pub struct Exp;

impl NodeType for Exp {}

impl fmt::Display for Exp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "exp")
    }
}

/// Natural logarithm,
/// The input: one(MUST identifier "A") of [scalarref|vectorref|matrixref], 
/// The output: one(MUST identifier "result") of [scalar|vector|matrix], one-to-one correspondence.
#[derive(Debug, Clone)]
pub struct Log;

impl NodeType for Log {}

impl fmt::Display for Log {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "log")
    }
}

/// Base 2 logarithm,
/// The input: one(MUST identifier "A") of [scalarref|vectorref|matrixref], 
/// The output: one(MUST identifier "result") of [scalar|vector|matrix], one-to-one correspondence.
#[derive(Debug, Clone)]
pub struct Log2;

impl NodeType for Log2 {}

impl fmt::Display for Log2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "log2")
    }
}

/// Base 10 logarithm,
/// The input: one(MUST identifier "A") of [scalarref|vectorref|matrixref], 
/// The output: one(MUST identifier "result") of [scalar|vector|matrix], one-to-one correspondence.
#[derive(Debug, Clone)]
pub struct Log10;

impl NodeType for Log10 {}

impl fmt::Display for Log10 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "log10")
    }
}

/// Selection operation, 
/// The inputs: four(MUST identifier "A", "B", "C", "D") of [scalarref|vectorref|matrixref], 
/// The output: one(MUST identifier "result") of [scalar|vector|matrix], four-to-one type correspondence,
/// If A < B then the result is C, otherwise D.
#[derive(Debug, Clone)]
pub struct Select;

impl NodeType for Select {}

impl fmt::Display for Select {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "select")
    }
}

/// Clamping operation,
/// The inputs: three(MUST identifier "A", "min", "max") of [scalarref|vectorref|matrixref], 
/// The output: one(MUST identifier "result") of [scalar|vector|matrix], three-to-one type correspondence.
#[derive(Debug, Clone)]
pub struct Clamp;

impl NodeType for Clamp {}

impl fmt::Display for Clamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "clamp")
    }
}

/// Hyperbolic cosine,
/// The input: one(MUST identifier "A") of [scalarref|vectorref|matrixref], 
/// The output: one(MUST identifier "result") of [scalar|vector|matrix], one-to-one correspondence.
#[derive(Debug, Clone)]
pub struct Cosh;

impl NodeType for Cosh {}

impl fmt::Display for Cosh {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "cosh")
    }
}

/// Hyperbolic sine,
/// The input: one(MUST identifier "A") of [scalarref|vectorref|matrixref], 
/// The output: one(MUST identifier "result") of [scalar|vector|matrix], one-to-one correspondence.
#[derive(Debug, Clone)]
pub struct Sinh;

impl NodeType for Sinh {}

impl fmt::Display for Sinh {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "sinh")
    }
}

/// Hyperbolic tangent,
/// The input: one(MUST identifier "A") of [scalarref|vectorref|matrixref], 
/// The output: one(MUST identifier "result") of [scalar|vector|matrix], one-to-one correspondence.
#[derive(Debug, Clone)]
pub struct Tanh;

impl NodeType for Tanh {}

impl fmt::Display for Tanh {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "tanh")
    }
}

/// Rounding operation, input: one(MUST identifier "A") of [scalarref|vectorref|matrixref], output: one(MUST identifier "result") of [scalar|vector|matrix], one-to-one correspondence.
/// The input: one(MUST identifier "A") of [scalarref|vectorref|matrixref], 
/// The output: one(MUST identifier "result") of [scalar|vector|matrix], one-to-one correspondence.
#[derive(Debug, Clone)]
pub struct Round;

impl NodeType for Round {}

impl fmt::Display for Round {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "round")
    }
}

/// Ceiling operation,
/// The input: one(MUST identifier "A") of [scalarref|vectorref|matrixref], 
/// The output: one(MUST identifier "result") of [scalar|vector|matrix], one-to-one correspondence.
#[derive(Debug, Clone)]
pub struct Ceil;

impl NodeType for Ceil {}

impl fmt::Display for Ceil {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ceil")
    }
}

/// Floor operation,
/// The input: one(MUST identifier "A") of [scalarref|vectorref|matrixref], 
/// The output: one(MUST identifier "result") of [scalar|vector|matrix], one-to-one correspondence.
#[derive(Debug, Clone)]
pub struct Floor;

impl NodeType for Floor {}

impl fmt::Display for Floor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "floor")
    }
}

/// Signum operation,
/// The input: one(MUST identifier "A") of [scalarref|vectorref|matrixref], 
/// The output: one(MUST identifier "result") of [scalar|vector|matrix], one-to-one correspondence.
#[derive(Debug, Clone)]
pub struct Sign;

impl NodeType for Sign {}

impl fmt::Display for Sign {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "sign")
    }
}

/// Fractional part extraction,
/// The input: one(MUST identifier "A") of [scalarref|vectorref|matrixref], 
/// The output: one(MUST identifier "result") of [scalar|vector|matrix], one-to-one correspondence.
#[derive(Debug, Clone)]
pub struct Fract;

impl NodeType for Fract {}

impl fmt::Display for Fract {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "fract")
    }
}

/// Length of a vector,
/// The input: one(MUST identifier "A") of [vectorref], 
/// The output: one(MUST identifier "result") of [scalar].
#[derive(Debug, Clone)]
pub struct Length;

impl NodeType for Length {}

impl fmt::Display for Length {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "length")
    }
}

/// Spatial gradient of a function, 
/// The inputs: one(MUST identifier "functionID") of [resourceref] and one(MUST identifier "step") of [scalarref] and any of [the argument references required by the referenced function], 
/// The outputs: one(MUST identifier "vector") of [vector] and one(MUST identifier "gradient") of [vector] and one(MUST identifier "magnitude") of [scalar].
#[derive(Debug, Clone)]
pub struct FunctionGradient {
    /// The attribute "scalaroutput" must name a scalar output of the referenced function.
    scalar_output: String,
    /// The attribute "vectorinput" must name a vector (float3) input of the referenced function with respect to which the gradient is computed.
    vector_input: String,
}

impl NodeType for FunctionGradient {
    fn name(&self) -> String {
        "functiongradient".to_string()
    }
}

impl fmt::Display for FunctionGradient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}|scalaroutput={}|vectorinput={}", self.name(), self.scalar_output, self.vector_input)
    }
}

/// Normalized distance from a function,
/// The inputs: one(MUST identifier "functionID") of [resourceref] and one(MUST identifier "step") of [scalarref] and any of [the argument references required by the referenced function], 
/// The output: one(MUST identifier "result") of [scalar].
#[derive(Debug, Clone)]
pub struct NormalizeDistance {
    /// The attribute "scalaroutput" must name a scalar output of the referenced function.
    scalar_output: String,
    /// The attribute "vectorinput" must name a vector (float3) input of the referenced function with respect to which the gradient is computed.
    vector_input: String,
}

impl NodeType for NormalizeDistance {
    fn name(&self) -> String {
        "normalizedistance".to_string()
    }
}

impl fmt::Display for NormalizeDistance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}|scalaroutput={}|vectorinput={}", self.name(), self.scalar_output, self.vector_input)
    }
}

/// Function call, 
/// The input: one(MUST identifier "functionID") of [resourceref],
/// The rest of the inputs and outputs must match the inputs and outputs of the function.
#[derive(Debug, Clone)]
pub struct FunctionCall;

impl NodeType for FunctionCall {}

impl fmt::Display for FunctionCall {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "functioncall")
    }
}

/// Signed distance to mesh, 
/// The inputs: one(MUST identifier "pos") of [vectorref] maybe one(identifier "mesh") of [resourceref],
/// The mesh is defined by a resource identifier, The mesh is assumed to be closed and watertight,
/// The output: one(MUST identifier "distance") of [scalar],
/// The distance is positive if the point is outside the mesh and negative if the point is inside the mesh.
#[derive(Debug, Clone)]
pub struct SignedMesh;

impl NodeType for SignedMesh {}

impl fmt::Display for SignedMesh {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "mesh")
    }
}

/// Unsigned distance to mesh,
/// The inputs: one(MUST identifier "pos") of [vectorref] maybe one(identifier "mesh") of [resourceref],
/// The mesh is defined by a resource identifier, The mesh may be open and is not required to be watertight,
/// The output: one(MUST identifier "distance") of [scalar],
/// The distance is always positive.
#[derive(Debug, Clone)]
pub struct UnsignedMesh;

impl NodeType for UnsignedMesh {}

impl fmt::Display for UnsignedMesh {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unsignedmesh")
    }
}

/// Signed distance to beam lattice,
/// The inputs: one(MUST identifier "pos") of [vectorref] maybe one(identifier "beamlattice") of [resourceref],
/// The beam lattice is defined by a resource identifier,
/// The output: one(MUST identifier "distance") of [scalar],
/// The distance is positive if the point is outside the beam lattice and negative if the point is inside the beam lattice.
#[derive(Debug, Clone)]
pub struct BeamLattice {
    /// The optional attribute "accuraterange" specifies a non-negative distance (in model units, same space as the input "pos") around the beam lattice within which the returned signed distance MUST be accurate.
    accurate_range: Option<f64>,
}

impl NodeType for BeamLattice {
    fn name(&self) -> String {
        "beamlattice".to_string()
    }
}

impl fmt::Display for BeamLattice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(range) = self.accurate_range {
            write!(f, "{}|accuraterange={}", self.name(), range)
        } else {
            write!(f, "{}", self.name())
        }
    }
}

impl FromStr for Box<dyn NodeType> {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let vs = s.split('|').collect::<Vec<_>>();
        let make = || {
            let mut kvs = HashMap::new();
            for v in vs[1..].iter() {
                let vs = v.split("=").collect::<Vec<_>>();
                if vs.len() == 2 {
                    kvs.insert(vs[0].to_string(), vs[1].to_string());
                }
            }
            kvs
        };
        let pick = |k: &str, kvs: &HashMap<String, String>| {
            kvs.get(k).map(|v| v.clone()).unwrap_or_default()
        };
        if vs.len() >= 1 {
            match vs[0] {
                "addition" => Ok(Box::new(Addition)),
                "subtraction" => Ok(Box::new(Subtraction)),
                "multiplication" => Ok(Box::new(Multiplication)),
                "division" => Ok(Box::new(Division)),
                "constant" => Ok(Box::new(Constant { value: pick("value", &make()).parse()? })),
                "constvec" => {
                    let kvs = make();
                    Ok(Box::new(ConstVec { x: pick("x", &kvs).parse()?, y: pick("y", &kvs).parse()?, z: pick("z", &kvs).parse()? }))
                },
                "constmat" => {
                    let x = pick("matrix", &make())
                        .split_whitespace()
                        .map(|v| v.parse::<f64>())
                        .collect::<std::result::Result<Vec<_>, _>>()
                        .map_err(|_| Error::InvalidMatrix(format!("cannot parse matrix: {}", s)))?;
                    let m = [
                        x[0], x[1], x[2], x[3],
                        x[4], x[5], x[6], x[7],
                        x[8], x[9], x[10], x[11],
                        x[12], x[13], x[14], x[15],
                    ];
                    Ok(Box::new(ConstMat { matrix: m }))
                },
                "constresourceid" => Ok(Box::new(ConstResourceId { value: pick("value", &make()).parse()? })),
                "composevector" => Ok(Box::new(ComposeVector)),
                "vectorfromscalar" => Ok(Box::new(VectorFromScalar)),
                "decomposevector" => Ok(Box::new(DecomposeVector)),
                "composematrix" => Ok(Box::new(ComposeMatrix)),
                "matrixfromcolumns" => Ok(Box::new(MatrixFromColumns)),
                "matrixfromrows" => Ok(Box::new(MatrixFromRows)),
                "dot" => Ok(Box::new(Dot)),
                "cross" => Ok(Box::new(Cross)),
                "matrixvectormultiplication" => Ok(Box::new(MatrixVectorMultiplication)),
                "transpose" => Ok(Box::new(Transpose)),
                "inverse" => Ok(Box::new(Inverse)),
                "sin" => Ok(Box::new(Sin)),
                "cos" => Ok(Box::new(Cos)),
                "tan" => Ok(Box::new(Tan)),
                "asin" => Ok(Box::new(Asin)),
                "acos" => Ok(Box::new(Acos)),
                "atan" => Ok(Box::new(Atan)),
                "atan2" => Ok(Box::new(Atan2)),
                "min" => Ok(Box::new(Min)),
                "max" => Ok(Box::new(Max)),
                "abs" => Ok(Box::new(Abs)),
                "fmod" => Ok(Box::new(Fmod)),
                "mod" => Ok(Box::new(Mod)),
                "pow" => Ok(Box::new(Pow)),
                "sqrt" => Ok(Box::new(Sqrt)),
                "exp" => Ok(Box::new(Exp)),
                "log" => Ok(Box::new(Log)),
                "log2" => Ok(Box::new(Log2)),
                "log10" => Ok(Box::new(Log10)),
                "select" => Ok(Box::new(Select)),
                "clamp" => Ok(Box::new(Clamp)),
                "cosh" => Ok(Box::new(Cosh)),
                "sinh" => Ok(Box::new(Sinh)),
                "tanh" => Ok(Box::new(Tanh)),
                "round" => Ok(Box::new(Round)),
                "ceil" => Ok(Box::new(Ceil)),
                "floor" => Ok(Box::new(Floor)),
                "sign" => Ok(Box::new(Sign)),
                "fract" => Ok(Box::new(Fract)),
                "length" => Ok(Box::new(Length)),
                "functiongradient" => {
                    let kvs = make();
                    Ok(Box::new(FunctionGradient {
                        scalar_output: pick("scalaroutput", &kvs), 
                        vector_input: pick("vectorinput", &kvs),
                    }))
                }
                "normalizedistance" => {
                    let kvs = make();
                    Ok(Box::new(NormalizeDistance {
                        scalar_output: pick("scalaroutput", &kvs),
                        vector_input: pick("vectorinput", &kvs),
                    }))
                },
                "functioncall" => Ok(Box::new(FunctionCall)),
                "mesh" => Ok(Box::new(SignedMesh)),
                "unsignedmesh" => Ok(Box::new(UnsignedMesh)),
                "beamlattice" => {
                    let range = if let Some(range) = make().get("accuraterange") {
                        Some(range.parse::<f64>()?)
                    } else {
                        None
                    };
                    Ok(Box::new(BeamLattice { accurate_range: range }))
                },
                _ => Err(Error::InvalidStructure(format!("unknown function node type: {}", s))),
            }
        } else {
            Err(Error::InvalidStructure(format!("invalid function node type: {}", s)))
        }
    }
}

/// Implicit function resources.
#[derive(Debug, Clone, Default)]
pub struct ImplicitResources {
    /// Implicit function definitions.
    pub functions: Vec<ImplicitFunction>,
}

impl ImplicitResources {
    /// Create a new implicit resources collection.
    pub fn new() -> Self {
        Self::default()
    }
}