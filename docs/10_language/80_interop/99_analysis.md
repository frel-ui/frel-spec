# Interop Analysis

We have these top-level declarations in Frel:

- `enum`
- `scheme`
- `contract`
- `backend`
- `blueprint`
- `theme`
- `resource`

## Enum

Enum is typically supported by languages:

| Language   | Support  |
|------------|----------|
| Rust       | has enum |
| TypeScript | has enum |
| Kotlin     | has enum |

## Scheme

Conceptually, the scheme can be modeled as an interface. The actual data storage method
is not relevant but may provide performance benefits.

What is important:

- store the actual data somehow
- provide a way to access the data
- provide a way to update the data
- provide a way to access the metadata

## Contract

Contract is an interface, it defines functions.

Can be modeled as a list of function references in the host language.

## Backend

Backend is a mix of a scheme and a contract. Host language model can be
the combination of a data storage and a function reference list.

## Blueprint

Blueprints can be modeled as an AST. As they can contain only pure Frel expressions,
the AST can be interpreted at runtime.

## Theme

Themes are simply schemes with constant data.

## Resources

Can be an accessor/loader function list or as a data storage.




