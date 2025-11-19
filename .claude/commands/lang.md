**CRITICAL: This file describes the rules to follow when working on the Frel language specification.**

Read [README.md](../../README.md) to get an overview of the Frel project.

The Frel language specification is in the `docs/10_language` directory, the rules described here
are applied to all files in that directory.

The most important files in the specification are:

- [Language overview](../../docs/10_language/00_overview/00_language_overview.md)
- [Reactivity model](../../docs/10_language/20_data_model/10_reactivity_model.md)
- [Top-level declarations](../../docs/10_language/10_organization/10_top_level.md)

## Naming

The Frel project considers naming one of the utmost important aspects of its design.

All names **MUST** be chosen very carefully to avoid misunderstandings and confusion.

We prefer **deliberately non-intuitive** names in some contexts when we want to force
the reader to **NOT ASSUME** something.

For example, `mut` is a keyword in Rust which has a very specific meaning. If we do not
assign the **exact same** meaning to `mut`, we should not use it.

**CRITICAL: Propose or perform naming changes only when asked explicitly.**

## APIs

Frel APIs are designed carefully from the perspective of a UI language.

All available Frel APIs are described explicitly in the language specification.

Use only Frel APIs that are explicitly described in the language specification.

**CRITICAL: Propose or perform API changes only when asked explicitly.**

## Examples

**CRITICAL: Use only explicitly defined APIs when generating examples. Use only
function and/or language constructs which are explicitly described in the language specification.**

### Code fences

Pay extra attention to the language of the code fences. When the language is `frel`,
apply all rules described in this section.

When the language is not `frel` apply the rules of the given language.

### Verification Before Generating Code

Before generating ANY Frel code or examples:

1. ✓ Verify semantics and constraints match what is described anywhere in `docs/10_language/`
2. ✓ Verify syntax matches what is described anywhere in `docs/10_language/`
3. ✓ Verify APIs match what is described anywhere in `docs/10_language/`
4. ✓ Confirm it's pure Frel, not borrowed syntax
5. ✗ If uncertain → STOP and ASK

**CRITICAL: Use only what is explicitly specified. If something is missing, ASK questions.**

### ❌ WRONG - Imports from other languages

This is wrong because:

- `Option<T>` is not valid in Frel, it is a Rust construct, not a Frel construct

```frel
scheme Example {
    name : Option<String> // <- this is a bad example
}
```

### ✅ CORRECT - Uses Frel Syntax

This is correct because:

- `?` is a valid Frel syntax for scheme field type declarations

```frel
scheme Example {
    name : String? // <- this is a good example
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

## Scope

All the Frel language specification is in the `docs/10_language` directory.

**CRITICAL: When working on the language specification, use only information/files in the `docs/10_language`
directory, except when asked explicitly.**

Especially ignore the content of the `docs/98_archive` directory as it contains old, archived documentation.

## File operations

**CRITICAL: When moving files, use `git mv` to keep repository information.**