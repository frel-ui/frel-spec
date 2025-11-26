// AST Visitor Pattern for Frel
//
// This module defines a visitor trait for traversing the Frel AST.
// Implementors can use this trait to process AST nodes in a structured way.

use super::*;

/// Visitor trait for traversing the Frel AST.
///
/// Implement this trait to process AST nodes. All methods have default
/// implementations that traverse into child nodes, so you only need to
/// override the methods for nodes you care about.
pub trait FaVisitor {
    /// The result type produced by visit methods
    type Result;

    // =========================================================================
    // File-level
    // =========================================================================

    /// Visit a file
    fn visit_file(&mut self, file: &FaFile) -> Self::Result;

    /// Visit an import
    fn visit_import(&mut self, import: &FaImport) -> Self::Result;

    // =========================================================================
    // Top-level declarations
    // =========================================================================

    /// Visit a top-level declaration
    fn visit_top_level_decl(&mut self, decl: &FaTopLevelDecl) -> Self::Result;

    /// Visit a blueprint declaration
    fn visit_blueprint(&mut self, blueprint: &FaBlueprint) -> Self::Result;

    /// Visit a backend declaration
    fn visit_backend(&mut self, backend: &FaBackend) -> Self::Result;

    /// Visit a contract declaration
    fn visit_contract(&mut self, contract: &FaContract) -> Self::Result;

    /// Visit a scheme declaration
    fn visit_scheme(&mut self, scheme: &FaScheme) -> Self::Result;

    /// Visit an enum declaration
    fn visit_enum(&mut self, enum_decl: &FaEnum) -> Self::Result;

    /// Visit a theme declaration
    fn visit_theme(&mut self, theme: &FaTheme) -> Self::Result;

    /// Visit an arena declaration
    fn visit_arena(&mut self, arena: &FaArena) -> Self::Result;

    // =========================================================================
    // Blueprint members
    // =========================================================================

    /// Visit a blueprint statement
    fn visit_blueprint_stmt(&mut self, stmt: &FaBlueprintStmt) -> Self::Result;

    /// Visit a local declaration
    fn visit_local_decl(&mut self, decl: &FaLocalDecl) -> Self::Result;

    /// Visit a fragment creation
    fn visit_fragment_creation(&mut self, frag: &FaFragmentCreation) -> Self::Result;

    /// Visit a fragment body
    fn visit_fragment_body(&mut self, body: &FaFragmentBody) -> Self::Result;

    /// Visit a slot binding
    fn visit_slot_binding(&mut self, binding: &FaSlotBinding) -> Self::Result;

    /// Visit a blueprint value (inline or reference)
    fn visit_blueprint_value(&mut self, value: &FaBlueprintValue) -> Self::Result;

    /// Visit a control statement
    fn visit_control_stmt(&mut self, ctrl: &FaControlStmt) -> Self::Result;

    /// Visit a select branch
    fn visit_select_branch(&mut self, branch: &FaSelectBranch) -> Self::Result;

    /// Visit an instruction
    fn visit_instruction(&mut self, instr: &FaInstruction) -> Self::Result;

    /// Visit an event handler
    fn visit_event_handler(&mut self, handler: &FaEventHandler) -> Self::Result;

    /// Visit an event parameter
    fn visit_event_param(&mut self, param: &FaEventParam) -> Self::Result;

    /// Visit a handler statement
    fn visit_handler_stmt(&mut self, stmt: &FaHandlerStmt) -> Self::Result;

    // =========================================================================
    // Backend members
    // =========================================================================

    /// Visit a backend member
    fn visit_backend_member(&mut self, member: &FaBackendMember) -> Self::Result;

    /// Visit a field declaration
    fn visit_field(&mut self, field: &FaField) -> Self::Result;

    /// Visit a method declaration
    fn visit_method(&mut self, method: &FaMethod) -> Self::Result;

    /// Visit a command declaration
    fn visit_command(&mut self, command: &FaCommand) -> Self::Result;

    // =========================================================================
    // Contract members
    // =========================================================================

    /// Visit a contract method
    fn visit_contract_method(&mut self, method: &FaContractMethod) -> Self::Result;

    // =========================================================================
    // Scheme members
    // =========================================================================

    /// Visit a scheme member
    fn visit_scheme_member(&mut self, member: &FaSchemeMember) -> Self::Result;

    /// Visit a scheme field
    fn visit_scheme_field(&mut self, field: &FaSchemeField) -> Self::Result;

    /// Visit a virtual field
    fn visit_virtual_field(&mut self, field: &FaVirtualField) -> Self::Result;

    /// Visit a field instruction
    fn visit_field_instruction(&mut self, instr: &FaFieldInstruction) -> Self::Result;

    // =========================================================================
    // Theme members
    // =========================================================================

    /// Visit a theme member
    fn visit_theme_member(&mut self, member: &FaThemeMember) -> Self::Result;

    /// Visit a theme field
    fn visit_theme_field(&mut self, field: &FaThemeField) -> Self::Result;

    /// Visit an instruction set
    fn visit_instruction_set(&mut self, set: &FaInstructionSet) -> Self::Result;

    /// Visit a theme variant
    fn visit_theme_variant(&mut self, variant: &FaThemeVariant) -> Self::Result;

    // =========================================================================
    // Common elements
    // =========================================================================

    /// Visit a parameter
    fn visit_parameter(&mut self, param: &FaParameter) -> Self::Result;

    /// Visit an argument
    fn visit_arg(&mut self, arg: &FaArg) -> Self::Result;

    // =========================================================================
    // Types
    // =========================================================================

    /// Visit a type expression
    fn visit_type_expr(&mut self, type_expr: &FaTypeExpr) -> Self::Result;

    // =========================================================================
    // Expressions
    // =========================================================================

    /// Visit an expression
    fn visit_expr(&mut self, expr: &FaExpr) -> Self::Result;

    /// Visit a template element
    fn visit_template_element(&mut self, elem: &FaTemplateElement) -> Self::Result;

    /// Visit a binary operator
    fn visit_binary_op(&mut self, op: &FaBinaryOp) -> Self::Result;

    /// Visit a unary operator
    fn visit_unary_op(&mut self, op: &FaUnaryOp) -> Self::Result;
}
