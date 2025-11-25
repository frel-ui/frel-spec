// Abstract Syntax Tree for Frel
//
// This module defines the AST nodes that represent the structure of Frel programs.
// The AST is built from the PEST parse tree and is used for semantic analysis,
// type checking, and code generation.

use serde::{Deserialize, Serialize};

/// A Frel source file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
    pub module: String,
    pub imports: Vec<Import>,
    pub declarations: Vec<TopLevelDecl>,
}

/// Import statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Import {
    pub module: String,
    pub name: String,
}

/// Top-level declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TopLevelDecl {
    Blueprint(Blueprint),
    Backend(Backend),
    Contract(Contract),
    Scheme(Scheme),
    Enum(Enum),
    Theme(Theme),
    Arena(Arena),
}

/// Blueprint declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blueprint {
    pub name: String,
    pub params: Vec<Parameter>,
    pub body: Vec<BlueprintStmt>,
}

/// Blueprint statement
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlueprintStmt {
    With(String),
    LocalDecl(LocalDecl),
    FragmentCreation(FragmentCreation),
    Control(ControlStmt),
    Instruction(Instruction),
    EventHandler(EventHandler),
    /// A standalone expression as content (e.g., "Hello" in text { "Hello" })
    ContentExpr(Expr),
}

/// Local declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalDecl {
    pub name: String,
    pub type_expr: TypeExpr,
    pub init: Expr,
}

/// Fragment creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentCreation {
    pub name: String,
    pub args: Vec<Arg>,
    pub body: Option<FragmentBody>,
    pub instructions: Vec<Instruction>,
}

/// Fragment body
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FragmentBody {
    Default(Vec<BlueprintStmt>),
    Slots(Vec<SlotBinding>),
}

/// Slot binding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotBinding {
    pub slot_name: String,
    pub blueprint: BlueprintValue,
}

/// Blueprint value
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlueprintValue {
    Inline {
        params: Vec<String>,
        body: Vec<BlueprintStmt>,
    },
    Reference(String),
}

/// Control statement
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlStmt {
    When {
        condition: Expr,
        then_stmt: Box<BlueprintStmt>,
        else_stmt: Option<Box<BlueprintStmt>>,
    },
    Repeat {
        iterable: Expr,
        item_name: Option<String>,
        key_expr: Option<Expr>,
        body: Box<BlueprintStmt>,
    },
    Select {
        discriminant: Option<Expr>,
        branches: Vec<SelectBranch>,
        else_branch: Option<Box<BlueprintStmt>>,
    },
}

/// Select branch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectBranch {
    pub condition: Expr,
    pub body: Box<BlueprintStmt>,
}

/// Instruction (placeholder - to be expanded)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instruction {
    pub name: String,
    pub params: Vec<(String, Expr)>,
}

/// Event handler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventHandler {
    pub event_name: String,
    pub param: Option<EventParam>,
    pub body: Vec<HandlerStmt>,
}

/// Event parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventParam {
    pub name: String,
    pub type_expr: Option<TypeExpr>,
}

/// Handler statement
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HandlerStmt {
    Assignment { name: String, value: Expr },
    CommandCall { name: String, args: Vec<Expr> },
}

/// Backend declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Backend {
    pub name: String,
    pub params: Vec<Parameter>,
    pub members: Vec<BackendMember>,
}

/// Backend member
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackendMember {
    Include(String),
    Field(Field),
    Method(Method),
    Command(Command),
}

/// Field declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub type_expr: TypeExpr,
    pub init: Option<Expr>,
}

/// Method declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Method {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: TypeExpr,
}

/// Command declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    pub name: String,
    pub params: Vec<Parameter>,
}

/// Contract declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    pub name: String,
    pub methods: Vec<ContractMethod>,
}

/// Contract method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractMethod {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Option<TypeExpr>,
}

/// Scheme declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scheme {
    pub name: String,
    pub members: Vec<SchemeMember>,
}

/// Scheme member
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SchemeMember {
    Field(SchemeField),
    Virtual(VirtualField),
}

/// Scheme field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemeField {
    pub name: String,
    pub type_expr: TypeExpr,
    pub instructions: Vec<FieldInstruction>,
}

/// Virtual field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualField {
    pub name: String,
    pub type_expr: TypeExpr,
    pub expr: Expr,
}

/// Field instruction (placeholder)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldInstruction {
    pub name: String,
    pub value: Option<Expr>,
}

/// Enum declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enum {
    pub name: String,
    pub variants: Vec<String>,
}

/// Theme declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub members: Vec<ThemeMember>,
}

/// Theme member
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeMember {
    Include(String),
    Field(ThemeField),
    InstructionSet(InstructionSet),
    Variant(ThemeVariant),
}

/// Theme field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeField {
    pub name: String,
    pub is_asset: bool,
    pub type_expr: TypeExpr,
    pub init: Option<Expr>,
}

/// Instruction set
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionSet {
    pub name: String,
    pub instructions: Vec<Instruction>,
}

/// Theme variant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeVariant {
    pub name: String,
    pub overrides: Vec<(String, Expr)>,
}

/// Arena declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Arena {
    pub name: String,
    pub scheme_name: String,
    pub contract: Option<String>,
}

/// Parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub type_expr: TypeExpr,
    pub default: Option<Expr>,
}

/// Argument
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Arg {
    pub name: Option<String>,
    pub value: Expr,
}

/// Type expression
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TypeExpr {
    Named(String),
    Nullable(Box<TypeExpr>),
    Ref(Box<TypeExpr>),
    Draft(Box<TypeExpr>),
    Asset(Box<TypeExpr>),
    Blueprint(Vec<TypeExpr>),
    Accessor(Box<TypeExpr>),
    List(Box<TypeExpr>),
    Set(Box<TypeExpr>),
    Map(Box<TypeExpr>, Box<TypeExpr>),
    Tree(Box<TypeExpr>),
}

/// Expression
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Expr {
    // Literals
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    StringTemplate(Vec<TemplateElement>),
    List(Vec<Expr>),
    Object(Vec<(String, Expr)>),

    // Identifiers
    Identifier(String),
    QualifiedName(Vec<String>),

    // Operators
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Ternary {
        condition: Box<Expr>,
        then_expr: Box<Expr>,
        else_expr: Box<Expr>,
    },

    // Field access
    FieldAccess {
        base: Box<Expr>,
        field: String,
    },
    OptionalChain {
        base: Box<Expr>,
        field: String,
    },

    // Function call
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
}

/// Template element for string interpolation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemplateElement {
    Text(String),
    Interpolation(Box<Expr>),
}

/// Binary operators
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BinaryOp {
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
pub enum UnaryOp {
    Not,
    Neg,
    Pos,
}
