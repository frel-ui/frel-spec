# Layout Statement

The layout statement provides a visual ASCII-table syntax for defining grid-based layouts. It keeps
the layout structure together with its content, making the code easier to read and maintain.

## Motivation

Traditional nested `row`/`column` constructs can become verbose and hard to follow:

```frel
// Verbose nested approach
column {
    row {
        content1()
        content2()
    }
    row {
        content3()
        content4()
    }
}
```

The layout statement provides a visual alternative:

```frel
"""layout
| slot1 | slot2 |
| slot3 | slot4 |
"""
at slot1: { content1() }
at slot2: { content2() }
at slot3: { content3() }
at slot4: { content4() }
```

## Syntax

### Basic Structure

A layout statement uses triple-quote fencing with the `layout` keyword:

```frel
blueprint Example {
    """layout
    | slot1 | slot2 |
    | slot3 | slot4 |
    """
    at slot1: { content1() }
    at slot2: { content2() }
    at slot3: { content3() }
    at slot4: { content4() }
}
```

**Rules:**

- Each row starts and ends with `|`
- Cells are separated by `|`
- Cell content is the slot name plus optional modifiers
- Empty cells (containing only spaces) have no content
- The `"""layout` fence preserves formatting (no auto-format)

### Slot Bindings

Slots are bound using the existing `at <slot>:` syntax:

```frel
"""layout
| header  |
| content |
| footer  |
"""
at header: { text { "Title" } }
at content: { MainContent() }
at footer: { text { "Copyright 2024" } }
```

Slot bindings can use inline blueprints or blueprint references:

```frel
at header: { text { "Title" } }     // Inline blueprint
at content: MainContentBlueprint    // Blueprint reference
```

**Note:** Slots without bindings are simply empty cells. This is not an error.

## Sizing

Sizes are specified around the grid, not inside cells.

### Column Sizes

Column sizes appear on the first line, before any grid rows:

```frel
"""layout
  ~0.5    ~0.8
| slot1 | slot2 |
| slot3 | slot4 |
"""
```

### Row Sizes

Row sizes appear before the `|` on each grid row:

```frel
"""layout
  24 | slot1 | slot2 |
 ~1  | slot3 | slot4 |
"""
```

### Both Column and Row Sizes

```frel
"""layout
       ~0.5    ~0.8
  24 | slot1 | slot2 |
 ~1  | slot3 | slot4 |
"""
```

### Size Values

| Value  | Meaning              |
|--------|----------------------|
| `123`  | Fixed size in DIP    |
| `~0.5` | Weight-based sizing  |
| `#`    | Content-based sizing |

**Rules:**

- If any size is specified for a dimension, all sizes for that dimension must be specified
- Sizes are matched to columns/rows by order (not visual alignment)
- When no sizes are specified, all columns/rows default to content sizing

**Example with mixed sizes:**

```frel
"""layout
         #      ~0.8
  24 | slot1 | slot2 |
   # | slot3 | slot4 |
"""
```

## Cell Modifiers

Cell modifiers control alignment within each cell.

### Alignment Modifiers

| Modifier | Meaning                              |
|----------|--------------------------------------|
| `<`      | Horizontal align left (default)      |
| `!`      | Horizontal align center              |
| `>`      | Horizontal align right               |
| `^`      | Vertical align top                   |
| `=`      | Vertical align center                |
| `_`      | Vertical align baseline (default)    |
| `.`      | Vertical align bottom                |

**Defaults:** left horizontal, baseline vertical.

**Note:** Slot names cannot start with `_` to avoid ambiguity with the baseline modifier.

**Examples:**

```frel
"""layout
| < slot1 | ! slot2 | > slot3 |
| ^ slot4 | = slot5 | . slot6 |
"""
```

Combined alignment:

```frel
"""layout
| < ^ header |     // left, top
| ! = center |     // center, center
| > . footer |     // right, bottom
"""
```

**Note:** Layout alignment takes precedence over `align_self` in slot content.

### Merge Modifiers

Cells can be merged to create spans:

| Modifier | Meaning                      |
|----------|------------------------------|
| `<--`    | Merge with cell to the left  |
| `-->`    | Merge with cell to the right |
| `^--`    | Merge with cell above        |
| `v--`    | Merge with cell below        |

**Horizontal span:**

```frel
"""layout
| header | <--    |
| left   | right  |
"""
at header: { text { "Full Width Header" } }
```

**Vertical span:**

```frel
"""layout
| sidebar | content |
| v--     | footer  |
"""
at sidebar: { Navigation() }
```

**Rectangular span (2x2):**

```frel
"""layout
| main | <--  | side |
| v--  | v--  | info |
"""
at main: { MainContent() }  // Spans 2x2
```

**Rules:**

- Only rectangular spans are allowed
- Non-rectangular merges (L-shaped, etc.) cause a compile error
- Merge arrows pointing to non-existent cells cause a compile error
- Each column keeps its own size; merging spans content across columns

## Instructions

Since only one layout is allowed per blueprint, all instructions in the blueprint body apply to
the layout. These placements are equivalent:

```frel
// Inside the layout fence (recommended)
"""layout
.. gap { 8 }
.. padding { 16 }
| slot1 | slot2 |
"""

// After the layout
"""layout
| slot1 | slot2 |
"""
.. gap { 8 }
.. padding { 16 }

// Before the layout
.. gap { 8 }
.. padding { 16 }
"""layout
| slot1 | slot2 |
"""
```

**Recommendation:** Place instructions inside the layout fence before the grid for best readability.

## Complete Example

```frel
blueprint MessageComposer {
    """layout
    .. gap { 8 }
    .. padding { 16 }
             #       ~1
      #  | avatar | header   |
     ~1  | v--    | messages |
     48  |        | input    |
    """
    at avatar: {
        image { user.avatar }
            .. size { 48 }
            .. corner_radius { 24 }
    }
    at header: {
        text { user.name }
            .. font { size: 18 weight: 700 }
    }
    at messages: {
        column {
            .. scroll { vertical }
            repeat on messages as msg {
                MessageBubble(msg)
            }
        }
    }
    at input: {
        row {
            .. gap { 8 }
            text_editor { draft }
                .. width { expand }
            icon { "send" }
                .. on_click { send_message() }
        }
    }
}
```

## Error Conditions

| Condition                                 | Result          |
|-------------------------------------------|-----------------|
| Non-rectangular merge (L-shaped, etc.)    | Compile error   |
| Merge arrow points to non-existent cell   | Compile error   |
| `at slot:` references slot not in layout  | Compile error   |
| Slot in layout has no `at slot:` binding  | OK (empty cell) |
| Size count doesn't match column/row count | Compile error   |
| More than one layout per blueprint        | Compile error   |

## Design Rationale

The layout statement addresses several pain points:

1. **Visual correspondence**: The ASCII table visually represents the actual layout structure
2. **Colocation**: Layout definition and sizing stay together (unlike CSS Grid's separated templates)
3. **Reduced nesting**: Eliminates deeply nested `row`/`column` structures
4. **Formatting preservation**: The `"""` fence syntax prevents auto-formatters from breaking the visual structure

## See Also

- [Blueprint Basics](10_blueprint_basics.md) - Blueprint structure and body statements
- [Fragment Creation](40_fragment_creation.md) - Slot syntax (`at slot:`)
- [Standard Blueprints](30_standard_blueprints.md) - `row`, `column`, `box` containers
- [Instructions](60_instructions.md) - Layout instructions (`gap`, `padding`, etc.)
