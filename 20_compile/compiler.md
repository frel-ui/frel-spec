# Fragment Compiler

The compiler is a Rust procedural macro that turns [fragment definitions](../10_language/00_language_overview.md) into [Fragment IR](fir.md).

## Surrounding Rewriter (Lowering Pass)

**Purpose:**
Ensure that all layout and decoration instructions (`padding`, `border`, `margin`, etc.)
are applied to container nodes.

**Transformation rule:**  
If a basic fragment receives surrounding instructions,
the compiler emits an enclosing container with those instructions
and moves the basic fragment inside it.

Example:

```frel
text { "Hello" } .. padding { 8 }
```

is transformed into:

```frel
box {
    padding { 8 }
    text { "Hello" }
}
```
