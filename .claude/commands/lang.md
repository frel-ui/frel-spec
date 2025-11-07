**CRITICAL: This file describes the rules to follow when working on the Frel language specification.**

Read [README.md](../../README.md) to get an overview of the Frel project.

The Frel language specification is in the `docs/10_language` directory, the rules described here
are applied to all files in that directory.

The most important files in the specification are:

- [Reactivity model](../../docs/10_language/10_data_model/10_reactivity_model.md)

## Naming

The Frel project considers naming one of the utmost important aspects of its design.

All names **MUST** be chosen very carefully to avoid misunderstandings and confusion.

We prefer **deliberately non-intuitive** names in some contexts when we want to force
the reader to **NOT ASSUME** something.

For example, `mut` is a keyword in Rust which has a very specific meaning. If we do not
assign the **exact same** meaning to `mut`, we should not use it.

**CRITICAL: Do not propose or perform naming changes without being asked explicitly.**

## APIs

Frel APIs are designed carefully from the perspective of a UI language.

All available Frel APIs are described explicitly in the language specification.

If an API is not described in the language specification, it is not available, do not use it.

**CRITICAL: Do not propose or perform API changes without being asked explicitly.**

## Examples

**CRITICAL: Do not invent syntax and APIs when generating examples. If a function and/or 
language construct does not exist in the language specification, do not use it.**

### Code fences

Pay extra attention to the language of the code fences. When the language is `frel`,
apply all rules described in this section.

When the language is not `frel`, do not apply any rules, but apply the rules of the given language.

### Verification Before Generating Code

Before generating ANY Frel code or examples:

1. ✓ Check language specification in `docs/10_language/`
2. ✓ Verify syntax exists in specification
3. ✓ Verify APIs are documented
4. ✓ Confirm it's pure Frel, not borrowed syntax
5. ✗ If uncertain → STOP and ASK

**CRITICAL: DO NOT assume. DO NOT extrapolate. ASK questions.**

### ❌ WRONG - Imports from other languages

This is wrong because:

- ":" is not valid in Frel, other languages use `:` for type annotations, Frel does not
- `Option<T>` is not valid in Frel, it is a Rust construct, not a Frel construct
- `?` is not a valid scheme field declaration in Frel, it is valid at other places but not here

```frel
scheme Example {
    name : Option<String>? // <- this is a bad example
}
```

### ✅ CORRECT - Uses Frel Syntax

This is correct because:

- `..` is a valid Frel syntax for scheme field type declarations
- `.. optional` is a valid Frel syntax for optional values (values that may be absent)

```frel
scheme Example {
    name .. String .. optional // <- this is a good example
}
```

### ❌ WRONG - Assumes Unspecified Features

This is wrong because:

- `.push()` method is NOT specified!

```frel
items: List<Item> = []
items.push(item)
```


### ✅ CORRECT - Acknowledges Limitation

This is correct because:

- marks a missing feature but does not invent/guess it

```frel
items: List<Item> = []
// TODO: list append operation not yet specified
```