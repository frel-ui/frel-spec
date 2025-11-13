# Interop Basics

This document outlines the language-level connection points between Frel and host languages.

## Overview

Frel is a DSL that compiles to host language code. The interop layer defines how Frel concepts map to and interact with the host language at compile time and runtime.

## Connection Points

### 1. Scheme Representation

**What needs to be designed:**
- How schemes are represented in the host language
- Field access API
- Validation API (`is_valid()`, `errors()`, `validate()`)
- Serialization/deserialization support
- Generated methods for scheme instances

**References:**
- [Schemes](../10_data_modeling/60_schemes.md)

### 2. Enum Representation

**What needs to be designed:**
- Enum generation from Frel enum declarations
- Variant access (singleton values)
- String conversion (`to_string()`, `from_string()`)
- Enumeration (`all()` method)

**References:**
- [Enums](../10_data_modeling/50_enums.md)

### 3. Backend Representation

**What needs to be designed:**
- How backends are represented in the host language
- Store accessor types (`FrelMutableStore<T>` for writable stores, values for read-only)
- Parameter accessor methods
- Lifecycle hook signatures (`on_init`, `on_cleanup`)
- Command method signatures (async/suspend)
- Included backend accessors
- Backend implementation requirements

**References:**
- [Backends](../40_backends/10_backend_basics.md)

### 4. Backend Instantiation

**What needs to be designed:**
- How to create backend instances from the host language
- Constructor parameter passing
- Backend lifecycle (creation, initialization, cleanup, destruction)
- Backend composition (include mechanism)

**References:**
- [Backends](../40_backends/10_backend_basics.md)

### 5. Store Access API

Commands receive store handles (`FrelMutableStore<T>`) with the following API:
- `read() -> Option<&T>` - Read current value (None if not Ready)
- `status() -> FrelStatus` - Get current status
- `write(value: T)` - Set value and mark Ready
- `set_loading()` - Mark as Loading
- `set_error(error: FrelError)` - Mark as Error

All stores internally maintain:
- Status: `Loading`, `Ready`, or `Error(FrelError)`
- Value: Present only when status is `Ready`

**References:**
- [Store Basics](../20_reactive_state/10_store_basics.md)
- [Writable Stores](../20_reactive_state/30_writable_stores.md)

### 6. Store Initialization Expressions

**What needs to be designed:**
- How Frel expressions map to host language code
- Type inference from initialization expressions
- Parameter access in initializers
- Scope and visibility rules
- Code generation for Frel's expression language

**References:**
- [Frel Expressions](../15_expressions/10_expression_basics.md)
- [Store Basics](../20_reactive_state/10_store_basics.md#initialization-order)

### 7. Event Handler Statements

**What needs to be designed:**
- How host language statements are embedded in event handlers
- What side effects are allowed
- Store mutation syntax in handlers
- Event parameter types and binding
- Async operation support in handlers
- Error handling in handlers

**References:**
- [Event Handlers](../70_blueprint/70_event_handlers.md)
- [Language Overview](../00_language_overview.md)

### 8. Source Definition

Sources receive a `StoreKey` and `FrelRuntime` and dispatch updates via:
- `runtime.set_store_loading(key)` - Set Loading status
- `runtime.set_store_value(key, value)` - Set value and Ready status
- `runtime.set_store_error(key, error)` - Set Error status

Sources implement lifecycle methods:
- `start(runtime, store_key)` - Begin producing values
- `stop()` - Cleanup and cancel operations

All sources use `FrelStatus` (not `Status<E>`) and `FrelError` for errors.

**References:**
- [Sources](../20_reactive_state/50_sources.md)
- [Standard Sources](../20_reactive_state/60_standard_sources.md)

### 9. Contract System

**What needs to be designed:**
- Generated contract trait/interface structure
- Contract method signatures (all async, implicit Result)
- Runtime registration API (`register_contract::<T>(impl)`)
- Global registry access (`get_contract::<T>()`)
- Contract usage in backends (via `uses` clause)
- Contract usage in fragments (as sources)
- Error type (`ServiceError` or custom)

**References:**
- [Contracts](../../../10_language/30_contract/10_contracts.md)

### 10. Collection Wrapper Types

**What needs to be designed:**
- `FrelList<T>`, `FrelSet<T>`, `FrelMap<K,V>`, `FrelTree<T>` wrapper APIs
- Wrapping host languace collections
- Fine-grained change detection and reactivity
- Mutation methods that trigger updates
- Iteration and access patterns

**References:**
- [Data Basics](../10_data_modeling/10_data_basics.md)
- [Collections](../10_data_modeling/30_collections.md)

### 11. Type System Mapping

**What needs to be designed:**
- Mapping of Frel primitives to host language types
- Specialized type representations:
  - `Secret` (sensitive strings)
  - `Decimal` (arbitrary precision)
  - `Uuid`
  - `Url`
  - `Color`
  - `Blob` (binary data)
- DateTime type mappings:
  - `Instant`
  - `LocalDate`
  - `LocalTime`
  - `LocalDateTime`
  - `Timezone`
  - `Duration`
- Status and error types:
  - `FrelStatus` (Loading, Ready, Error)
  - `FrelError` (message, code, details)
- Runtime interop types:
  - `StoreKey` (opaque store reference for sources)
  - `FrelRuntime` (runtime interface for store updates)
  - `FrelMutableStore<T>` (store handle for commands)
- Optional types

**References:**
- [Data Basics](../10_data_modeling/10_data_basics.md)
- [Primitives](../10_data_modeling/20_primitives.md)
- [DateTime](../10_data_modeling/40_datetime.md)

### 12. Validation Infrastructure

**What needs to be designed:**
- Validation rule execution engine
- `FieldError` structure
- Validation method generation
- Custom validation closures
- Error message generation and localization
- Real-time vs on-demand validation

**References:**
- [Schemes](../10_data_modeling/60_schemes.md#validation-api)

### 13. Detached UI Integration

**What needs to be designed:**
- How to trigger modals, toasts, notifications from host language code
- API for detached UI components
- Lifecycle management of detached UI
- Return value handling (for modals with results)

**References:**
- [Detached UI](../70_blueprint/80_detached_ui.md)

## Design Principles

1. **Type Safety**: All interop should be type-safe and catch errors at compile time where possible
2. **Ergonomics**: The host language API should feel natural and idiomatic
3. **Performance**: Minimize overhead in the interop layer
4. **Clarity**: Clear separation between pure expressions and effectful statements
5. **Composability**: Interop mechanisms should compose well with host language patterns
6. **Portability**: Frel expressions should map cleanly to different host languages