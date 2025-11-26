// Abstract Syntax Tree for Frel
//
// This module defines the AST nodes that represent the structure of Frel programs.
// The AST is built from the parser and is used for semantic analysis,
// type checking, and code generation.
//
// All AST types are prefixed with "Fa" (Frel AST) to clearly identify them.

pub mod dump;
pub mod visitor;

pub use dump::DumpVisitor;
pub use visitor::FaVisitor;

use serde::{Deserialize, Serialize};

/// A Frel source file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaFile {
    pub module: String,
    pub imports: Vec<FaImport>,
    pub declarations: Vec<FaTopLevelDecl>,
}

/// Import statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaImport {
    pub module: String,
    pub name: String,
}

/// Top-level declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FaTopLevelDecl {
    Blueprint(FaBlueprint),
    Backend(FaBackend),
    Contract(FaContract),
    Scheme(FaScheme),
    Enum(FaEnum),
    Theme(FaTheme),
    Arena(FaArena),
}

/// Blueprint declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaBlueprint {
    pub name: String,
    pub params: Vec<FaParameter>,
    pub body: Vec<FaBlueprintStmt>,
}

/// Blueprint statement
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FaBlueprintStmt {
    With(String),
    LocalDecl(FaLocalDecl),
    FragmentCreation(FaFragmentCreation),
    Control(FaControlStmt),
    Instruction(FaInstruction),
    EventHandler(FaEventHandler),
    /// A standalone expression as content (e.g., "Hello" in text { "Hello" })
    ContentExpr(FaExpr),
}

/// Local declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaLocalDecl {
    pub name: String,
    pub type_expr: FaTypeExpr,
    pub init: FaExpr,
}

/// Fragment creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaFragmentCreation {
    pub name: String,
    pub args: Vec<FaArg>,
    pub body: Option<FaFragmentBody>,
    pub postfix: Vec<FaPostfixItem>,
}

/// Postfix item (instruction or event handler)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FaPostfixItem {
    Instruction(FaInstruction),
    EventHandler(FaEventHandler),
}

/// Fragment body
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FaFragmentBody {
    Default(Vec<FaBlueprintStmt>),
    Slots(Vec<FaSlotBinding>),
    /// Inline blueprint with parameters: { param -> body }
    InlineBlueprint {
        params: Vec<String>,
        body: Vec<FaBlueprintStmt>,
    },
}

/// Slot binding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaSlotBinding {
    pub slot_name: String,
    pub blueprint: FaBlueprintValue,
}

/// Blueprint value
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FaBlueprintValue {
    Inline {
        params: Vec<String>,
        body: Vec<FaBlueprintStmt>,
    },
    Reference(String),
}

/// Control statement
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FaControlStmt {
    When {
        condition: FaExpr,
        then_stmt: Box<FaBlueprintStmt>,
        else_stmt: Option<Box<FaBlueprintStmt>>,
    },
    Repeat {
        iterable: FaExpr,
        item_name: Option<String>,
        key_expr: Option<FaExpr>,
        body: Box<FaBlueprintStmt>,
    },
    Select {
        discriminant: Option<FaExpr>,
        branches: Vec<FaSelectBranch>,
        else_branch: Option<Box<FaBlueprintStmt>>,
    },
}

/// Select branch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaSelectBranch {
    pub condition: FaExpr,
    pub body: Box<FaBlueprintStmt>,
}

/// Instruction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaInstruction {
    pub name: String,
    pub params: Vec<(String, FaExpr)>,
}

/// Event handler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaEventHandler {
    pub event_name: String,
    pub param: Option<FaEventParam>,
    pub body: Vec<FaHandlerStmt>,
}

/// Event parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaEventParam {
    pub name: String,
    pub type_expr: Option<FaTypeExpr>,
}

/// Handler statement
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FaHandlerStmt {
    Assignment { name: String, value: FaExpr },
    CommandCall { name: String, args: Vec<FaExpr> },
}

/// Backend declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaBackend {
    pub name: String,
    pub params: Vec<FaParameter>,
    pub members: Vec<FaBackendMember>,
}

/// Backend member
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FaBackendMember {
    Include(String),
    Field(FaField),
    Method(FaMethod),
    Command(FaCommand),
}

/// Field declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaField {
    pub name: String,
    pub type_expr: FaTypeExpr,
    pub init: Option<FaExpr>,
}

/// Method declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaMethod {
    pub name: String,
    pub params: Vec<FaParameter>,
    pub return_type: FaTypeExpr,
}

/// Command declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaCommand {
    pub name: String,
    pub params: Vec<FaParameter>,
}

/// Contract declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaContract {
    pub name: String,
    pub methods: Vec<FaContractMethod>,
}

/// Contract method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaContractMethod {
    pub name: String,
    pub params: Vec<FaParameter>,
    pub return_type: Option<FaTypeExpr>,
}

/// Scheme declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaScheme {
    pub name: String,
    pub members: Vec<FaSchemeMember>,
}

/// Scheme member
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FaSchemeMember {
    Field(FaSchemeField),
    Virtual(FaVirtualField),
}

/// Scheme field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaSchemeField {
    pub name: String,
    pub type_expr: FaTypeExpr,
    pub instructions: Vec<FaFieldInstruction>,
}

/// Virtual field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaVirtualField {
    pub name: String,
    pub type_expr: FaTypeExpr,
    pub expr: FaExpr,
}

/// Field instruction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaFieldInstruction {
    pub name: String,
    pub value: Option<FaExpr>,
}

/// Enum declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaEnum {
    pub name: String,
    pub variants: Vec<String>,
}

/// Theme declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaTheme {
    pub name: String,
    pub members: Vec<FaThemeMember>,
}

/// Theme member
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FaThemeMember {
    Include(String),
    Field(FaThemeField),
    InstructionSet(FaInstructionSet),
    Variant(FaThemeVariant),
}

/// Theme field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaThemeField {
    pub name: String,
    pub is_asset: bool,
    pub type_expr: FaTypeExpr,
    pub init: Option<FaExpr>,
}

/// Instruction set
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaInstructionSet {
    pub name: String,
    pub instructions: Vec<FaInstruction>,
}

/// Theme variant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaThemeVariant {
    pub name: String,
    pub overrides: Vec<(String, FaExpr)>,
}

/// Arena declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaArena {
    pub name: String,
    pub scheme_name: String,
    pub contract: Option<String>,
}

/// Parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaParameter {
    pub name: String,
    pub type_expr: FaTypeExpr,
    pub default: Option<FaExpr>,
}

/// Argument
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaArg {
    pub name: Option<String>,
    pub value: FaExpr,
}

/// Type expression
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FaTypeExpr {
    Named(String),
    Nullable(Box<FaTypeExpr>),
    Ref(Box<FaTypeExpr>),
    Draft(Box<FaTypeExpr>),
    Asset(Box<FaTypeExpr>),
    Blueprint(Vec<FaTypeExpr>),
    Accessor(Box<FaTypeExpr>),
    List(Box<FaTypeExpr>),
    Set(Box<FaTypeExpr>),
    Map(Box<FaTypeExpr>, Box<FaTypeExpr>),
    Tree(Box<FaTypeExpr>),
}

/// Expression
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FaExpr {
    // Literals
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Color(u32),
    String(String),
    StringTemplate(Vec<FaTemplateElement>),
    List(Vec<FaExpr>),
    Object(Vec<(String, FaExpr)>),

    // Identifiers
    Identifier(String),
    QualifiedName(Vec<String>),

    // Operators
    Binary {
        op: FaBinaryOp,
        left: Box<FaExpr>,
        right: Box<FaExpr>,
    },
    Unary {
        op: FaUnaryOp,
        expr: Box<FaExpr>,
    },
    Ternary {
        condition: Box<FaExpr>,
        then_expr: Box<FaExpr>,
        else_expr: Box<FaExpr>,
    },

    // Field access
    FieldAccess {
        base: Box<FaExpr>,
        field: String,
    },
    OptionalChain {
        base: Box<FaExpr>,
        field: String,
    },

    // Function call
    Call {
        callee: Box<FaExpr>,
        args: Vec<FaExpr>,
    },
}

/// Template element for string interpolation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FaTemplateElement {
    Text(String),
    Interpolation(Box<FaExpr>),
}

/// Binary operators
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FaBinaryOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,

    // Comparison
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,

    // Logical
    And,
    Or,

    // Null coalescing
    Elvis,
}

/// Unary operators
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FaUnaryOp {
    Not,
    Neg,
    Pos,
}
