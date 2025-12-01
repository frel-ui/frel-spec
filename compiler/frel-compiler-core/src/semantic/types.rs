// Type representation for Frel semantic analysis
//
// This module defines the internal type representation used during
// semantic analysis and type checking.

use super::symbol::SymbolId;
use serde::{Deserialize, Serialize};

/// Internal type representation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Type {
    // ========================================================================
    // Unit type
    // ========================================================================
    /// Unit type (void/no value)
    Unit,

    // ========================================================================
    // Boolean type
    // ========================================================================
    /// Boolean type (true/false)
    Bool,

    // ========================================================================
    // Integer types (signed)
    // ========================================================================
    /// 8-bit signed integer (-128 to 127)
    I8,
    /// 16-bit signed integer (-32,768 to 32,767)
    I16,
    /// 32-bit signed integer (-2,147,483,648 to 2,147,483,647)
    I32,
    /// 64-bit signed integer
    I64,

    // ========================================================================
    // Integer types (unsigned)
    // ========================================================================
    /// 8-bit unsigned integer (0 to 255)
    U8,
    /// 16-bit unsigned integer (0 to 65,535)
    U16,
    /// 32-bit unsigned integer (0 to 4,294,967,295)
    U32,
    /// 64-bit unsigned integer
    U64,

    // ========================================================================
    // Floating point types
    // ========================================================================
    /// 32-bit floating point
    F32,
    /// 64-bit floating point
    F64,

    // ========================================================================
    // Decimal type
    // ========================================================================
    /// Arbitrary-precision decimal for financial calculations
    Decimal,

    // ========================================================================
    // Text types
    // ========================================================================
    /// Unicode text string
    String,
    /// Sensitive string (passwords, API keys, tokens)
    Secret,

    // ========================================================================
    // Identifier types
    // ========================================================================
    /// Universally unique identifier (UUID)
    Uuid,
    /// URL/URI type with validation
    Url,

    // ========================================================================
    // Visual types
    // ========================================================================
    /// Color value (RGBA)
    Color,
    /// SVG graphics for icons and vector images
    Graphics,

    // ========================================================================
    // Binary data
    // ========================================================================
    /// Binary data (files, images, etc.)
    Blob,

    // ========================================================================
    // Temporal types
    // ========================================================================
    /// Specific moment in time (UTC timestamp)
    Instant,
    /// Calendar date without time (e.g., 2024-03-15)
    LocalDate,
    /// Time of day without date (e.g., 14:30:00)
    LocalTime,
    /// Date and time without timezone
    LocalDateTime,
    /// IANA timezone identifier (e.g., "America/New_York")
    Timezone,
    /// Length of time / time span
    Duration,

    // ========================================================================
    // Composite types (refer to declarations by SymbolId)
    // ========================================================================
    /// A scheme type
    Scheme(SymbolId),
    /// A backend type
    Backend(SymbolId),
    /// A blueprint type
    Blueprint(SymbolId),
    /// A contract type
    Contract(SymbolId),
    /// A theme type
    Theme(SymbolId),
    /// An enum type
    Enum(SymbolId),

    // ========================================================================
    // Type modifiers
    // ========================================================================
    /// Nullable type: T?
    Nullable(Box<Type>),
    /// Reference type: ref T
    Ref(Box<Type>),
    /// Draft (mutable copy) type: draft T
    Draft(Box<Type>),
    /// Asset type: asset T
    Asset(Box<Type>),

    // ========================================================================
    // Collection types
    // ========================================================================
    /// List/array type: [T]
    List(Box<Type>),
    /// Set type: set<T>
    Set(Box<Type>),
    /// Map type: map<K, V>
    Map(Box<Type>, Box<Type>),
    /// Tree type: tree<T>
    Tree(Box<Type>),

    // ========================================================================
    // Function types
    // ========================================================================
    /// Function type for methods, commands, etc.
    Function {
        params: Vec<Type>,
        ret: Box<Type>,
    },

    /// Blueprint instantiation type (callable that creates a fragment)
    BlueprintInstance {
        blueprint: SymbolId,
        params: Vec<Type>,
    },

    /// Accessor type for reactive bindings
    Accessor(Box<Type>),

    // ========================================================================
    // Special types
    // ========================================================================
    /// Error type (used for error recovery)
    Error,
    /// Unknown type (before resolution)
    Unknown,
    /// Never type (for expressions that don't return)
    Never,
}

impl Type {
    /// Check if this is an intrinsic (built-in) type
    pub fn is_intrinsic(&self) -> bool {
        matches!(
            self,
            Type::Unit
                | Type::Bool
                | Type::I8
                | Type::I16
                | Type::I32
                | Type::I64
                | Type::U8
                | Type::U16
                | Type::U32
                | Type::U64
                | Type::F32
                | Type::F64
                | Type::Decimal
                | Type::String
                | Type::Secret
                | Type::Uuid
                | Type::Url
                | Type::Color
                | Type::Graphics
                | Type::Blob
                | Type::Instant
                | Type::LocalDate
                | Type::LocalTime
                | Type::LocalDateTime
                | Type::Timezone
                | Type::Duration
        )
    }

    /// Check if this is a primitive type (simple value types)
    pub fn is_primitive(&self) -> bool {
        matches!(
            self,
            Type::Unit
                | Type::Bool
                | Type::I8
                | Type::I16
                | Type::I32
                | Type::I64
                | Type::U8
                | Type::U16
                | Type::U32
                | Type::U64
                | Type::F32
                | Type::F64
                | Type::Decimal
                | Type::String
                | Type::Secret
                | Type::Uuid
                | Type::Url
                | Type::Color
                | Type::Graphics
                | Type::Blob
                | Type::Instant
                | Type::LocalDate
                | Type::LocalTime
                | Type::LocalDateTime
                | Type::Timezone
                | Type::Duration
        )
    }

    /// Check if this is a numeric type
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            Type::I8
                | Type::I16
                | Type::I32
                | Type::I64
                | Type::U8
                | Type::U16
                | Type::U32
                | Type::U64
                | Type::F32
                | Type::F64
                | Type::Decimal
        )
    }

    /// Check if this is an integer type (signed or unsigned)
    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            Type::I8
                | Type::I16
                | Type::I32
                | Type::I64
                | Type::U8
                | Type::U16
                | Type::U32
                | Type::U64
        )
    }

    /// Check if this is a signed integer type
    pub fn is_signed_integer(&self) -> bool {
        matches!(self, Type::I8 | Type::I16 | Type::I32 | Type::I64)
    }

    /// Check if this is an unsigned integer type
    pub fn is_unsigned_integer(&self) -> bool {
        matches!(self, Type::U8 | Type::U16 | Type::U32 | Type::U64)
    }

    /// Check if this is a floating point type
    pub fn is_float(&self) -> bool {
        matches!(self, Type::F32 | Type::F64)
    }

    /// Check if this is a text type (String or Secret)
    pub fn is_text(&self) -> bool {
        matches!(self, Type::String | Type::Secret)
    }

    /// Check if this is a temporal type
    pub fn is_temporal(&self) -> bool {
        matches!(
            self,
            Type::Instant
                | Type::LocalDate
                | Type::LocalTime
                | Type::LocalDateTime
                | Type::Timezone
                | Type::Duration
        )
    }

    /// Check if this is a composite (scheme, backend, etc.) type
    pub fn is_composite(&self) -> bool {
        matches!(
            self,
            Type::Scheme(_)
                | Type::Backend(_)
                | Type::Blueprint(_)
                | Type::Contract(_)
                | Type::Theme(_)
                | Type::Enum(_)
        )
    }

    /// Check if this is a collection type
    pub fn is_collection(&self) -> bool {
        matches!(
            self,
            Type::List(_) | Type::Set(_) | Type::Map(_, _) | Type::Tree(_)
        )
    }

    /// Check if this type is nullable
    pub fn is_nullable(&self) -> bool {
        matches!(self, Type::Nullable(_))
    }

    /// Check if this is a draft type
    pub fn is_draft(&self) -> bool {
        matches!(self, Type::Draft(_))
    }

    /// Check if this is a ref type
    pub fn is_ref(&self) -> bool {
        matches!(self, Type::Ref(_))
    }

    /// Check if this is an error type (for error recovery)
    pub fn is_error(&self) -> bool {
        matches!(self, Type::Error)
    }

    /// Check if this type is known (not unknown or error)
    pub fn is_known(&self) -> bool {
        !matches!(self, Type::Unknown | Type::Error)
    }

    /// Get the inner type of a nullable
    pub fn nullable_inner(&self) -> Option<&Type> {
        match self {
            Type::Nullable(inner) => Some(inner),
            _ => None,
        }
    }

    /// Get the inner type of a draft
    pub fn draft_inner(&self) -> Option<&Type> {
        match self {
            Type::Draft(inner) => Some(inner),
            _ => None,
        }
    }

    /// Get the inner type of a ref
    pub fn ref_inner(&self) -> Option<&Type> {
        match self {
            Type::Ref(inner) => Some(inner),
            _ => None,
        }
    }

    /// Get the element type of a collection
    pub fn element_type(&self) -> Option<&Type> {
        match self {
            Type::List(elem) | Type::Set(elem) | Type::Tree(elem) => Some(elem),
            _ => None,
        }
    }

    /// Strip all modifiers (nullable, ref, draft, asset) to get base type
    pub fn base_type(&self) -> &Type {
        match self {
            Type::Nullable(inner)
            | Type::Ref(inner)
            | Type::Draft(inner)
            | Type::Asset(inner) => inner.base_type(),
            _ => self,
        }
    }

    /// Make this type nullable if it isn't already
    pub fn make_nullable(self) -> Type {
        if self.is_nullable() {
            self
        } else {
            Type::Nullable(Box::new(self))
        }
    }

    /// Create a function type
    pub fn function(params: impl Into<Vec<Type>>, ret: Type) -> Type {
        Type::Function {
            params: params.into(),
            ret: Box::new(ret),
        }
    }

    /// Try to parse an intrinsic type from its name
    ///
    /// Returns None if the name doesn't match any intrinsic type.
    /// Type names are case-sensitive as defined in the Frel spec.
    pub fn from_intrinsic_name(name: &str) -> Option<Type> {
        match name {
            // Unit
            "unit" => Some(Type::Unit),
            // Boolean
            "bool" => Some(Type::Bool),
            // Signed integers
            "i8" => Some(Type::I8),
            "i16" => Some(Type::I16),
            "i32" => Some(Type::I32),
            "i64" => Some(Type::I64),
            // Unsigned integers
            "u8" => Some(Type::U8),
            "u16" => Some(Type::U16),
            "u32" => Some(Type::U32),
            "u64" => Some(Type::U64),
            // Floating point
            "f32" => Some(Type::F32),
            "f64" => Some(Type::F64),
            // Decimal
            "Decimal" => Some(Type::Decimal),
            // Text types
            "String" => Some(Type::String),
            "Secret" => Some(Type::Secret),
            // Identifier types
            "Uuid" => Some(Type::Uuid),
            "Url" => Some(Type::Url),
            // Visual types
            "Color" => Some(Type::Color),
            "Graphics" => Some(Type::Graphics),
            // Binary data
            "Blob" => Some(Type::Blob),
            // Temporal types
            "Instant" => Some(Type::Instant),
            "LocalDate" => Some(Type::LocalDate),
            "LocalTime" => Some(Type::LocalTime),
            "LocalDateTime" => Some(Type::LocalDateTime),
            "Timezone" => Some(Type::Timezone),
            "Duration" => Some(Type::Duration),
            _ => None,
        }
    }

    /// Get all intrinsic type names
    pub fn intrinsic_type_names() -> &'static [&'static str] {
        &[
            "unit",
            "bool",
            "i8",
            "i16",
            "i32",
            "i64",
            "u8",
            "u16",
            "u32",
            "u64",
            "f32",
            "f64",
            "Decimal",
            "String",
            "Secret",
            "Uuid",
            "Url",
            "Color",
            "Graphics",
            "Blob",
            "Instant",
            "LocalDate",
            "LocalTime",
            "LocalDateTime",
            "Timezone",
            "Duration",
        ]
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Unit
            Type::Unit => write!(f, "unit"),
            // Boolean
            Type::Bool => write!(f, "bool"),
            // Signed integers
            Type::I8 => write!(f, "i8"),
            Type::I16 => write!(f, "i16"),
            Type::I32 => write!(f, "i32"),
            Type::I64 => write!(f, "i64"),
            // Unsigned integers
            Type::U8 => write!(f, "u8"),
            Type::U16 => write!(f, "u16"),
            Type::U32 => write!(f, "u32"),
            Type::U64 => write!(f, "u64"),
            // Floating point
            Type::F32 => write!(f, "f32"),
            Type::F64 => write!(f, "f64"),
            // Decimal
            Type::Decimal => write!(f, "Decimal"),
            // Text types
            Type::String => write!(f, "String"),
            Type::Secret => write!(f, "Secret"),
            // Identifier types
            Type::Uuid => write!(f, "Uuid"),
            Type::Url => write!(f, "Url"),
            // Visual types
            Type::Color => write!(f, "Color"),
            Type::Graphics => write!(f, "Graphics"),
            // Binary data
            Type::Blob => write!(f, "Blob"),
            // Temporal types
            Type::Instant => write!(f, "Instant"),
            Type::LocalDate => write!(f, "LocalDate"),
            Type::LocalTime => write!(f, "LocalTime"),
            Type::LocalDateTime => write!(f, "LocalDateTime"),
            Type::Timezone => write!(f, "Timezone"),
            Type::Duration => write!(f, "Duration"),
            Type::Scheme(id) => write!(f, "scheme#{}", id.0),
            Type::Backend(id) => write!(f, "backend#{}", id.0),
            Type::Blueprint(id) => write!(f, "blueprint#{}", id.0),
            Type::Contract(id) => write!(f, "contract#{}", id.0),
            Type::Theme(id) => write!(f, "theme#{}", id.0),
            Type::Enum(id) => write!(f, "enum#{}", id.0),
            Type::Nullable(inner) => write!(f, "{}?", inner),
            Type::Ref(inner) => write!(f, "ref {}", inner),
            Type::Draft(inner) => write!(f, "draft {}", inner),
            Type::Asset(inner) => write!(f, "asset {}", inner),
            Type::List(elem) => write!(f, "[{}]", elem),
            Type::Set(elem) => write!(f, "set<{}>", elem),
            Type::Map(k, v) => write!(f, "map<{}, {}>", k, v),
            Type::Tree(elem) => write!(f, "tree<{}>", elem),
            Type::Function { params, ret } => {
                write!(f, "fn(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", p)?;
                }
                write!(f, ") -> {}", ret)
            }
            Type::BlueprintInstance { blueprint, params } => {
                write!(f, "blueprint#{}(", blueprint.0)?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", p)?;
                }
                write!(f, ")")
            }
            Type::Accessor(inner) => write!(f, "accessor<{}>", inner),
            Type::Error => write!(f, "<error>"),
            Type::Unknown => write!(f, "<unknown>"),
            Type::Never => write!(f, "never"),
        }
    }
}

/// Represents a resolved type annotation from the AST
#[derive(Debug, Clone)]
pub struct ResolvedType {
    /// The resolved type
    pub ty: Type,
    /// Whether this type was explicitly annotated or inferred
    pub explicit: bool,
}

impl ResolvedType {
    pub fn explicit(ty: Type) -> Self {
        Self { ty, explicit: true }
    }

    pub fn inferred(ty: Type) -> Self {
        Self {
            ty,
            explicit: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_predicates() {
        // Integer types
        assert!(Type::I32.is_primitive());
        assert!(Type::I32.is_numeric());
        assert!(Type::I32.is_integer());
        assert!(Type::I32.is_signed_integer());
        assert!(!Type::I32.is_unsigned_integer());
        assert!(!Type::I32.is_float());

        // Unsigned integers
        assert!(Type::U64.is_integer());
        assert!(Type::U64.is_unsigned_integer());
        assert!(!Type::U64.is_signed_integer());

        // Float types
        assert!(Type::F64.is_float());
        assert!(Type::F64.is_numeric());
        assert!(!Type::F64.is_integer());

        // Decimal
        assert!(Type::Decimal.is_numeric());
        assert!(!Type::Decimal.is_integer());
        assert!(!Type::Decimal.is_float());

        // Text types
        assert!(Type::String.is_text());
        assert!(Type::Secret.is_text());

        // Temporal types
        assert!(Type::Instant.is_temporal());
        assert!(Type::LocalDate.is_temporal());
        assert!(Type::Duration.is_temporal());
        assert!(!Type::String.is_temporal());

        // Type modifiers
        assert!(Type::Nullable(Box::new(Type::I32)).is_nullable());
        assert!(Type::Draft(Box::new(Type::String)).is_draft());
        assert!(Type::Ref(Box::new(Type::Scheme(SymbolId(0)))).is_ref());

        // Collections
        assert!(Type::List(Box::new(Type::I32)).is_collection());
        assert!(Type::Set(Box::new(Type::String)).is_collection());
        assert!(Type::Map(Box::new(Type::String), Box::new(Type::I32)).is_collection());

        // Composites
        assert!(Type::Scheme(SymbolId(0)).is_composite());
        assert!(Type::Backend(SymbolId(0)).is_composite());

        // Intrinsic check
        assert!(Type::Uuid.is_intrinsic());
        assert!(Type::Graphics.is_intrinsic());
        assert!(Type::Blob.is_intrinsic());
        assert!(!Type::Scheme(SymbolId(0)).is_intrinsic());
    }

    #[test]
    fn test_base_type() {
        let nested = Type::Nullable(Box::new(Type::Draft(Box::new(Type::Ref(Box::new(
            Type::Scheme(SymbolId(5)),
        ))))));

        assert_eq!(*nested.base_type(), Type::Scheme(SymbolId(5)));
    }

    #[test]
    fn test_type_display() {
        assert_eq!(format!("{}", Type::I32), "i32");
        assert_eq!(format!("{}", Type::U8), "u8");
        assert_eq!(format!("{}", Type::Decimal), "Decimal");
        assert_eq!(format!("{}", Type::String), "String");
        assert_eq!(format!("{}", Type::Secret), "Secret");
        assert_eq!(format!("{}", Type::Uuid), "Uuid");
        assert_eq!(format!("{}", Type::Instant), "Instant");
        assert_eq!(format!("{}", Type::LocalDateTime), "LocalDateTime");
        assert_eq!(format!("{}", Type::Duration), "Duration");
        assert_eq!(
            format!("{}", Type::Nullable(Box::new(Type::String))),
            "String?"
        );
        assert_eq!(
            format!("{}", Type::List(Box::new(Type::I32))),
            "[i32]"
        );
        assert_eq!(
            format!(
                "{}",
                Type::Function {
                    params: vec![Type::I32, Type::String],
                    ret: Box::new(Type::Bool)
                }
            ),
            "fn(i32, String) -> bool"
        );
    }

    #[test]
    fn test_from_intrinsic_name() {
        // Lowercase types
        assert_eq!(Type::from_intrinsic_name("i32"), Some(Type::I32));
        assert_eq!(Type::from_intrinsic_name("u64"), Some(Type::U64));
        assert_eq!(Type::from_intrinsic_name("f32"), Some(Type::F32));
        assert_eq!(Type::from_intrinsic_name("bool"), Some(Type::Bool));

        // PascalCase types
        assert_eq!(Type::from_intrinsic_name("String"), Some(Type::String));
        assert_eq!(Type::from_intrinsic_name("Decimal"), Some(Type::Decimal));
        assert_eq!(Type::from_intrinsic_name("Uuid"), Some(Type::Uuid));
        assert_eq!(Type::from_intrinsic_name("Instant"), Some(Type::Instant));
        assert_eq!(Type::from_intrinsic_name("LocalDate"), Some(Type::LocalDate));
        assert_eq!(Type::from_intrinsic_name("Duration"), Some(Type::Duration));
        assert_eq!(Type::from_intrinsic_name("Graphics"), Some(Type::Graphics));
        assert_eq!(Type::from_intrinsic_name("Blob"), Some(Type::Blob));
        assert_eq!(Type::from_intrinsic_name("Secret"), Some(Type::Secret));

        // Unknown type
        assert_eq!(Type::from_intrinsic_name("unknown"), None);
        assert_eq!(Type::from_intrinsic_name("MyScheme"), None);

        // Case sensitivity
        assert_eq!(Type::from_intrinsic_name("string"), None); // Must be "String"
        assert_eq!(Type::from_intrinsic_name("I32"), None); // Must be "i32"
    }

    #[test]
    fn test_intrinsic_type_names() {
        let names = Type::intrinsic_type_names();
        assert!(names.contains(&"i32"));
        assert!(names.contains(&"String"));
        assert!(names.contains(&"Uuid"));
        assert!(names.contains(&"Duration"));
        assert_eq!(names.len(), 26); // Total intrinsic types
    }

    #[test]
    fn test_make_nullable() {
        let t = Type::I32;
        let nullable = t.make_nullable();
        assert!(nullable.is_nullable());

        // Should not double-wrap
        let double = nullable.make_nullable();
        assert!(matches!(double, Type::Nullable(_)));
        if let Type::Nullable(inner) = double {
            assert!(!inner.is_nullable());
        }
    }
}
