// Error code registry for Frel compiler
//
// This module defines stable, documented error codes used throughout the compiler.
// Error codes are versioned and should not be removed or reassigned once released.
//
// Code ranges:
// - E01xx: Syntax errors (lexer)
// - E02xx: Parse errors (parser)
// - E03xx: Resolution errors (name resolution)
// - E04xx: Type errors (type system)
// - E05xx: Reactive errors (ownership/reactivity)
// - E06xx: Backend errors (composition)
// - E07xx: Blueprint errors (compilation)

use super::Severity;
use serde::{Deserialize, Serialize};

/// Category of error for grouping and filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    /// E01xx - Lexical/syntax errors
    Syntax,
    /// E02xx - Parser errors
    Parse,
    /// E03xx - Name resolution errors
    Resolution,
    /// E04xx - Type system errors
    Type,
    /// E05xx - Reactivity/ownership errors
    Reactive,
    /// E06xx - Backend composition errors
    Backend,
    /// E07xx - Blueprint compilation errors
    Blueprint,
}

impl Category {
    pub fn as_str(&self) -> &'static str {
        match self {
            Category::Syntax => "syntax",
            Category::Parse => "parse",
            Category::Resolution => "resolution",
            Category::Type => "type",
            Category::Reactive => "reactive",
            Category::Backend => "backend",
            Category::Blueprint => "blueprint",
        }
    }

    /// Returns the error code prefix for this category
    pub fn code_prefix(&self) -> &'static str {
        match self {
            Category::Syntax => "E01",
            Category::Parse => "E02",
            Category::Resolution => "E03",
            Category::Type => "E04",
            Category::Reactive => "E05",
            Category::Backend => "E06",
            Category::Blueprint => "E07",
        }
    }
}

/// A stable, documented error code
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorCode {
    /// The code string, e.g., "E0301"
    pub code: &'static str,
    /// Human-readable name, e.g., "undefined_name"
    pub name: &'static str,
    /// Category for grouping
    pub category: Category,
    /// Default severity (can be overridden by configuration)
    pub default_severity: Severity,
    /// Brief explanation for --explain support
    pub explanation: &'static str,
}

impl ErrorCode {
    pub const fn new(
        code: &'static str,
        name: &'static str,
        category: Category,
        default_severity: Severity,
        explanation: &'static str,
    ) -> Self {
        Self {
            code,
            name,
            category,
            default_severity,
            explanation,
        }
    }
}

// ============================================================================
// Syntax Errors (E01xx)
// ============================================================================

pub const E0101: ErrorCode = ErrorCode::new(
    "E0101",
    "invalid_token",
    Category::Syntax,
    Severity::Error,
    "The lexer encountered an unexpected character that cannot form a valid token.",
);

pub const E0102: ErrorCode = ErrorCode::new(
    "E0102",
    "unterminated_string",
    Category::Syntax,
    Severity::Error,
    "A string literal was started but not closed before the end of the line or file.",
);

pub const E0103: ErrorCode = ErrorCode::new(
    "E0103",
    "invalid_escape",
    Category::Syntax,
    Severity::Error,
    "An invalid escape sequence was used in a string literal.",
);

pub const E0104: ErrorCode = ErrorCode::new(
    "E0104",
    "invalid_number",
    Category::Syntax,
    Severity::Error,
    "A numeric literal has an invalid format.",
);

pub const E0105: ErrorCode = ErrorCode::new(
    "E0105",
    "invalid_color",
    Category::Syntax,
    Severity::Error,
    "A color literal has an invalid format. Expected #RGB, #RGBA, #RRGGBB, or #RRGGBBAA.",
);

// ============================================================================
// Parse Errors (E02xx)
// ============================================================================

pub const E0201: ErrorCode = ErrorCode::new(
    "E0201",
    "unexpected_token",
    Category::Parse,
    Severity::Error,
    "The parser encountered a token that was not expected at this position.",
);

pub const E0202: ErrorCode = ErrorCode::new(
    "E0202",
    "expected_identifier",
    Category::Parse,
    Severity::Error,
    "An identifier was expected but something else was found.",
);

pub const E0203: ErrorCode = ErrorCode::new(
    "E0203",
    "expected_type",
    Category::Parse,
    Severity::Error,
    "A type expression was expected but something else was found.",
);

pub const E0204: ErrorCode = ErrorCode::new(
    "E0204",
    "expected_expression",
    Category::Parse,
    Severity::Error,
    "An expression was expected but something else was found.",
);

pub const E0205: ErrorCode = ErrorCode::new(
    "E0205",
    "unclosed_delimiter",
    Category::Parse,
    Severity::Error,
    "A delimiter (brace, bracket, or parenthesis) was opened but never closed.",
);

pub const E0206: ErrorCode = ErrorCode::new(
    "E0206",
    "expected_declaration",
    Category::Parse,
    Severity::Error,
    "A top-level declaration was expected (blueprint, backend, scheme, etc.).",
);

pub const E0207: ErrorCode = ErrorCode::new(
    "E0207",
    "missing_module",
    Category::Parse,
    Severity::Error,
    "Every Frel file must start with a module declaration.",
);

// ============================================================================
// Resolution Errors (E03xx)
// ============================================================================

pub const E0301: ErrorCode = ErrorCode::new(
    "E0301",
    "undefined_name",
    Category::Resolution,
    Severity::Error,
    "The name could not be found in the current scope or any parent scope.",
);

pub const E0302: ErrorCode = ErrorCode::new(
    "E0302",
    "duplicate_definition",
    Category::Resolution,
    Severity::Error,
    "A name is defined more than once in the same scope. Frel does not allow shadowing.",
);

pub const E0303: ErrorCode = ErrorCode::new(
    "E0303",
    "shadowing_not_allowed",
    Category::Resolution,
    Severity::Error,
    "Frel does not allow shadowing. The name is already defined in an outer scope.",
);

pub const E0304: ErrorCode = ErrorCode::new(
    "E0304",
    "unresolved_import",
    Category::Resolution,
    Severity::Error,
    "The import could not be resolved. The file or declaration does not exist.",
);

pub const E0305: ErrorCode = ErrorCode::new(
    "E0305",
    "circular_import",
    Category::Resolution,
    Severity::Error,
    "Circular imports are not allowed between files.",
);

pub const E0306: ErrorCode = ErrorCode::new(
    "E0306",
    "unresolved_qualified_name",
    Category::Resolution,
    Severity::Error,
    "The qualified name could not be resolved. Check that all path segments exist.",
);

// ============================================================================
// Type Errors (E04xx)
// ============================================================================

pub const E0401: ErrorCode = ErrorCode::new(
    "E0401",
    "type_mismatch",
    Category::Type,
    Severity::Error,
    "The type of the expression does not match the expected type.",
);

pub const E0402: ErrorCode = ErrorCode::new(
    "E0402",
    "unknown_type",
    Category::Type,
    Severity::Error,
    "The type name could not be found. Check spelling or imports.",
);

pub const E0403: ErrorCode = ErrorCode::new(
    "E0403",
    "invalid_type_modifier",
    Category::Type,
    Severity::Error,
    "The type modifier cannot be applied to this type.",
);

pub const E0404: ErrorCode = ErrorCode::new(
    "E0404",
    "ref_requires_identity",
    Category::Type,
    Severity::Error,
    "A ref type can only reference a scheme that has an identity field.",
);

pub const E0405: ErrorCode = ErrorCode::new(
    "E0405",
    "incompatible_types",
    Category::Type,
    Severity::Error,
    "The types are not compatible for this operation.",
);

pub const E0406: ErrorCode = ErrorCode::new(
    "E0406",
    "nullable_access",
    Category::Type,
    Severity::Error,
    "Cannot access a field on a nullable type without optional chaining (?.).",
);

pub const E0407: ErrorCode = ErrorCode::new(
    "E0407",
    "parameter_backend_type_mismatch",
    Category::Type,
    Severity::Error,
    "Parameter and backend field have the same name but different types. Types must match when merging.",
);

pub const E0408: ErrorCode = ErrorCode::new(
    "E0408",
    "conflicting_defaults",
    Category::Type,
    Severity::Error,
    "Both parameter and backend field have default values. Only one can have a default.",
);

// ============================================================================
// Reactive Errors (E05xx)
// ============================================================================

pub const E0501: ErrorCode = ErrorCode::new(
    "E0501",
    "ownership_cycle",
    Category::Reactive,
    Severity::Error,
    "Ownership must form a tree. A cycle was detected in the ownership graph.",
);

pub const E0502: ErrorCode = ErrorCode::new(
    "E0502",
    "invalid_draft_usage",
    Category::Reactive,
    Severity::Error,
    "Draft values can only be assigned to draft fields or local variables.",
);

pub const E0503: ErrorCode = ErrorCode::new(
    "E0503",
    "non_draft_assignment",
    Category::Reactive,
    Severity::Error,
    "Cannot assign a non-draft composite to another location. Use draft T for mutable copies.",
);

pub const E0504: ErrorCode = ErrorCode::new(
    "E0504",
    "virtual_field_cycle",
    Category::Reactive,
    Severity::Error,
    "Virtual fields cannot have circular dependencies.",
);

// ============================================================================
// Backend Errors (E06xx)
// ============================================================================

pub const E0601: ErrorCode = ErrorCode::new(
    "E0601",
    "backend_include_conflict",
    Category::Backend,
    Severity::Error,
    "A field from an included backend conflicts with an existing field.",
);

pub const E0602: ErrorCode = ErrorCode::new(
    "E0602",
    "circular_include",
    Category::Backend,
    Severity::Error,
    "Backends cannot include each other in a cycle.",
);

pub const E0603: ErrorCode = ErrorCode::new(
    "E0603",
    "command_outside_handler",
    Category::Backend,
    Severity::Error,
    "Commands can only be called from event handlers, not from expressions.",
);

pub const E0604: ErrorCode = ErrorCode::new(
    "E0604",
    "method_in_handler",
    Category::Backend,
    Severity::Warning,
    "Methods are pure and should be called from expressions, not event handlers.",
);

// ============================================================================
// Blueprint Errors (E07xx)
// ============================================================================

pub const E0701: ErrorCode = ErrorCode::new(
    "E0701",
    "invalid_slot",
    Category::Blueprint,
    Severity::Error,
    "The slot name does not exist in the target blueprint.",
);

pub const E0702: ErrorCode = ErrorCode::new(
    "E0702",
    "parameter_arity",
    Category::Blueprint,
    Severity::Error,
    "The number of arguments does not match the number of parameters.",
);

pub const E0703: ErrorCode = ErrorCode::new(
    "E0703",
    "multiple_with",
    Category::Blueprint,
    Severity::Error,
    "A blueprint can only have one 'with' statement.",
);

pub const E0704: ErrorCode = ErrorCode::new(
    "E0704",
    "invalid_closure_capture",
    Category::Blueprint,
    Severity::Error,
    "The captured variable is not available in the closure scope.",
);

pub const E0705: ErrorCode = ErrorCode::new(
    "E0705",
    "invalid_instruction_keyword",
    Category::Blueprint,
    Severity::Error,
    "The value is not a valid keyword for this instruction parameter.",
);

// ============================================================================
// Error code lookup
// ============================================================================

/// Look up an error code by its string identifier
pub fn lookup(code: &str) -> Option<&'static ErrorCode> {
    match code {
        // Syntax
        "E0101" => Some(&E0101),
        "E0102" => Some(&E0102),
        "E0103" => Some(&E0103),
        "E0104" => Some(&E0104),
        "E0105" => Some(&E0105),
        // Parse
        "E0201" => Some(&E0201),
        "E0202" => Some(&E0202),
        "E0203" => Some(&E0203),
        "E0204" => Some(&E0204),
        "E0205" => Some(&E0205),
        "E0206" => Some(&E0206),
        "E0207" => Some(&E0207),
        // Resolution
        "E0301" => Some(&E0301),
        "E0302" => Some(&E0302),
        "E0303" => Some(&E0303),
        "E0304" => Some(&E0304),
        "E0305" => Some(&E0305),
        "E0306" => Some(&E0306),
        // Type
        "E0401" => Some(&E0401),
        "E0402" => Some(&E0402),
        "E0403" => Some(&E0403),
        "E0404" => Some(&E0404),
        "E0405" => Some(&E0405),
        "E0406" => Some(&E0406),
        "E0407" => Some(&E0407),
        "E0408" => Some(&E0408),
        // Reactive
        "E0501" => Some(&E0501),
        "E0502" => Some(&E0502),
        "E0503" => Some(&E0503),
        "E0504" => Some(&E0504),
        // Backend
        "E0601" => Some(&E0601),
        "E0602" => Some(&E0602),
        "E0603" => Some(&E0603),
        "E0604" => Some(&E0604),
        // Blueprint
        "E0701" => Some(&E0701),
        "E0702" => Some(&E0702),
        "E0703" => Some(&E0703),
        "E0704" => Some(&E0704),
        "E0705" => Some(&E0705),
        _ => None,
    }
}

/// Get all error codes for a category
pub fn by_category(category: Category) -> Vec<&'static ErrorCode> {
    let all = [
        // Syntax
        &E0101, &E0102, &E0103, &E0104, &E0105,
        // Parse
        &E0201, &E0202, &E0203, &E0204, &E0205, &E0206, &E0207,
        // Resolution
        &E0301, &E0302, &E0303, &E0304, &E0305, &E0306,
        // Type
        &E0401, &E0402, &E0403, &E0404, &E0405, &E0406, &E0407, &E0408,
        // Reactive
        &E0501, &E0502, &E0503, &E0504,
        // Backend
        &E0601, &E0602, &E0603, &E0604,
        // Blueprint
        &E0701, &E0702, &E0703, &E0704, &E0705,
    ];
    all.into_iter().filter(|c| c.category == category).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup() {
        assert_eq!(lookup("E0301").map(|c| c.name), Some("undefined_name"));
        assert_eq!(lookup("E9999"), None);
    }

    #[test]
    fn test_category_codes() {
        let resolution = by_category(Category::Resolution);
        assert!(resolution.iter().all(|c| c.category == Category::Resolution));
        assert!(resolution.len() >= 6);
    }

    #[test]
    fn test_code_format() {
        // All codes should match format E0Nxx where N is category digit
        for code in by_category(Category::Resolution) {
            assert!(code.code.starts_with("E03"));
        }
        for code in by_category(Category::Type) {
            assert!(code.code.starts_with("E04"));
        }
    }
}
