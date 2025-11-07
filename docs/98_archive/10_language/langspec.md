# AI Coding Assistant Instructions for Frel Language Specification

**CRITICAL: These instructions apply to the Frel language specification in `docs/10_language/`, NOT the other parts of the project.**

When working on the language spec, treat it as documentation of what Frel IS, not how it's implemented.

## Required Context

Before working on the Frel language specification, read these foundational documents to understand the project:

1. **Project Overview**: [README.md](../../../README.md) - Understand that Frel is in early design stage
2. **Language Overview**: [docs/10_language/00_language_overview.md](00_language_overview.md) - Core concepts, glossary, and how components fit together
3. **Data Modeling**: [docs/10_language/10_data_modeling/10_data_basics.md](10_data_modeling/10_data_basics.md) - Type system, primitives, collections, enums, schemes
4. **Expression Basics**: [docs/10_language/15_expressions/10_expression_basics.md](15_expressions/10_expression_basics.md) - The pure expression language that forms the foundation
5. **Store Basics**: [docs/10_language/20_reactive_state/10_store_basics.md](20_reactive_state/10_store_basics.md) - Reactive state management semantics
6. **Contract Basics**: [docs/10_language/30_contracts/10_contract_basics.md](30_contracts/10_contract_basics.md) - External service interfaces
7. **Backend Basics**: [docs/10_language/40_backends/10_backend_basics.md](40_backends/10_backend_basics.md) - Business logic containers with stores and commands
8. **Blueprint Basics**: [docs/10_language/70_blueprint/10_blueprint_basics.md](70_blueprint/10_blueprint_basics.md) - UI component templates

These documents define what IS available in Frel. If a feature isn't documented there, it doesn't exist yet.

## Scope: These Rules Apply to Frel Code Only

**All rules in this document apply to code within `frel` code fences only.**

When working with interop examples using `rust`, `typescript`, `kotlin`, or other host language code
fences, you MAY use host language features as needed for the implementation. However, you MUST still 
respect the Frel specification boundaries:

- Host language code can use language-specific features (e.g., `forEach`, `println`, `vec![]`)
- Host language code can use implementation-specific types (e.g., `FrelList<T>`, `FrelStore<T>`)
- Host language code CANNOT assume Frel features that don't exist in the specification
- When Frel code calls a backend command, only pass data according to Frel's type system

**Example - CORRECT interop usage:**

```frel
blueprint Example {
    with DumpBackend
    decl nums: List<u32> = [1, 2, 3]
    button { "Dump" } .. on_click { dump(nums) }
}

backend DumpBackend {
    command dump(nums: List<u32>)
}
```

```kotlin
// Host language implementation - Kotlin-specific features are OK here
fun DumpBackend.dump(nums: FrelList<Int>) {
    nums.toList().forEach { println(it) }  // Kotlin features OK in kotlin fence
}
```

The same `.forEach` and `println` would be WRONG in a `frel` code fence.

## Mandatory Rules

### 1. ONLY Use Explicitly Specified Features

- If you see a `TODO` comment in an example, that feature does NOT exist yet
- DO NOT generate code using unspecified features
- DO NOT assume standard library functions exist unless explicitly documented
- DO NOT extrapolate or infer features from incomplete examples

### 2. Respect the Minimal Expression Language

The Frel expression language is pure and minimal. Read [10_language/15_expressions/10_expression_basics.md](15_expressions/10_expression_basics.md) to understand what IS available.

**Not yet specified (DO NOT USE):**

- List operations: `.push()`, `.append()`, `.contains()`, `.sort()`, `.slice()`
- Most string methods beyond what's documented
- Most array methods beyond what's documented
- Any operation not explicitly listed in the expression documentation

**If an operation isn't documented in the specification, it DOES NOT EXIST.**

### 3. No Host Language Syntax in Frel Examples

Examples must use ONLY Frel syntax as defined in this specification.

**DO NOT use:**

- Rust syntax: `vec![]`, `.len()`, `.is_empty()`, `.clone()`, `&`, `mut`
- TypeScript/JavaScript syntax: `.length`, `.push()`, `[]`, `console.log()`
- Python syntax: `len()`, `.append()`, `[]`
- Any other host language syntax

**Frel is host-independent.** Examples that mix host language syntax are incorrect.

### 4. Store Operations Are Strictly Defined

Read [10_language/20_reactive_state/10_store_basics.md](20_reactive_state/10_store_basics.md) for complete semantics.

- **`decl` stores**: Read-only, reactive, initialized with pure Frel expressions
- **`writable` stores**: Can be assigned in event handlers, no automatic dependencies
- **`fanin` stores**: Reactive expression that can be temporarily overridden by manual assignment
- **`source` stores**: Async producers, have `.latest()` and `.status()`, support `on_value` handlers

### 5. Data Types Are Strictly Defined

Frel has a specific type system documented in [docs/10_language/10_data_modeling/](../../docs/98_archive/10_language/10_data_modeling/).

**Available types:**
- Primitives: `i32`, `f64`, `bool`, `String`, etc.
- Collections: `List<T>`, `Set<T>`, `Map<K,V>`, `Tree<T>`
- Optional: `T?`
- User-defined: `enum`, `scheme`

**DO NOT use:**
- Host language types: `Vec<T>`, `Array<T>`, `vector<T>`, `ArrayList<T>`
- Generic type parameters beyond what's specified
- Any type not explicitly documented

### 6. Event Handlers Have Limited Syntax

Event handlers contain a sequence of statements. Each statement must be ONE of:

1. **Store assignment**: `store_name = <pure_frel_expression>`
2. **Backend command call**: `command_name(arg1, arg2, ...)`

**NOT allowed in event handlers:**

- Imperative control flow: `if`, `for`, `while`, `match`
- Multiple statements per line
- Complex imperative logic
- Host language constructs

For control flow, use Frel's declarative constructs (`when`, `select`) in the blueprint body, or implement complex logic as backend commands.

See [docs/10_language/15_expressions/10_expression_basics.md](15_expressions/10_expression_basics.md) section "Statement Context" for details.

### 7. Handle Incomplete Examples Correctly

Many examples in this repository contain placeholders for unspecified features:

```frel
// Example with TODO marker
new_item .. on_value { item: Item ->
    // TODO: conditional append - if item not in list, append it
    items = items  // placeholder
}
```

**When you encounter these:**

- DO NOT complete the TODO by inventing syntax
- DO NOT replace placeholders with "real" code
- These markers indicate the feature is intentionally not yet designed
- **ASK the user** what they want instead of guessing

### 8. Verification Before Generating Code

Before generating ANY Frel code or examples:

1. Verify ALL operations are documented in [docs/10_language/15_expressions/](../../docs/98_archive/10_language/15_expressions/)
2. Verify ALL store operations match [docs/10_language/20_reactive_state/](../../docs/98_archive/10_language/20_reactive_state/)
3. Verify blueprint/backend/contract syntax matches the basics docs (listed in Required Context above)
4. Verify syntax is pure Frel, not host language syntax
5. If ANY operation is uncertain, STOP and ASK the user

**DO NOT assume. DO NOT extrapolate. ASK questions.**

## Examples

### ❌ WRONG - Assumes Unspecified Features

```frel
writable items: List<Item> = []

new_items .. on_value { item: Item ->
    items.push(item)  // .push() method is NOT specified!
}
```

### ✅ CORRECT - Acknowledges Limitation

```frel
writable items: List<Item> = []

new_items .. on_value { item: Item ->
    // TODO: list append operation not yet specified
}
```

### ✅ EVEN BETTER - Ask First

"I notice the Frel specification doesn't define list mutation operations yet. The example shows a placeholder for list append. Would you like me to:

1. Help design the list append operation specification?
2. Suggest an alternative approach that works with current Frel features?
3. Document this as a known limitation?"

### ❌ WRONG - Uses Host Language Syntax

```frel
writable count: Vec<String> = vec![]  // Rust syntax!

decl length = count.len()  // Rust syntax!
```

### ✅ CORRECT - Uses Frel Syntax

```frel
writable count: List<String> = []  // Frel syntax

decl length = count.length  // If .length is specified for lists
```

### ❌ WRONG - Complex Event Handler

```frel
button { "Process" }
    .. on_click {
        if items.is_empty() {  // No control flow in handlers!
            error = "No items"
        } else {
            for item in items {  // No loops in handlers!
                process(item)
            }
        }
    }
```

### ✅ CORRECT - Simple Command Call

```frel
button { "Process" }
    .. on_click {
        process_items()  // Logic in backend command
    }
```

## When In Doubt

1. **Check the specification files** in [docs/10_language/](../../docs/98_archive/10_language/)
2. **Search for the operation** in the expression and reactive state docs (listed in Required Context above)
3. **If not found, ASK the user** - do not invent or assume

## Summary

This is a language specification being developed. Many features are intentionally incomplete or marked as TODO. Your role is to:

- Work ONLY with features that are explicitly specified
- Identify gaps and limitations rather than paper over them
- Ask clarifying questions instead of making assumptions
- Help design new features when requested, not proactively fill in blanks

**Respect the specification boundaries. Do not hallucinate features.**
