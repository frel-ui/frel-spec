# Theme Basics

Themes provide reusable styling configurations for blueprints. They contain styling values,
instruction sets, assets, and variants to enable consistent visual design across an application.

## Declaration

Themes are declared at the module level:

```frel
theme AppTheme {
    primary_color : asset Color
    padding : u32 = 16

    set button_style {
        corner_radius { 8 }
        padding { padding }
    }
}
```

## Theme Components

A theme consists of four kinds of members:

### 1. Fields

Fields hold styling values and reactive computations:

```frel
theme AppTheme {
    base_padding : u32 = 16
    double_padding : u32 = base_padding * 2
    corner_radius : u32 = 8
}
```

### 2. Asset Fields

Asset fields use asset types to represent externally-loaded UI resources:

```frel
theme AppTheme {
    primary_color : asset Color
    app_name : asset String
    logo : asset Graphics
}
```

See [Asset Types](../20_data_model/02_type_system.md#asset-types) for details.

### 3. Instruction Sets

Instruction sets are named collections of instructions that can be applied to fragments:

```frel
theme AppTheme {
    primary_color : asset Color
    padding : u32 = 16

    set button_style {
        corner_radius { 8 }
        padding { padding }
        background { color: primary_color }
    }

    set card_style {
        border { 0xCCCCCC 1 }
        corner_radius { 12 }
        padding { padding * 2 }
    }
}
```

**Instruction set properties:**

- Declared with `set` keyword followed by a name
- Body contains zero or more instructions
- Type is `Set<Instruction>`
- Can reference theme fields and asset fields
- Applied to fragments using postfix or inner syntax

**Usage in blueprints:**

```frel
blueprint Button {
    with ButtonBackend

    // Postfix syntax
    box { } .. theme.button_style

    // Inner syntax
    box {
        theme.button_style
    }
}
```

### 4. Variants

Variants are theme instances with overridden field values:

```frel
theme MessageTheme {
    background : asset Color
    text_color : asset Color

    padding : u32 = 16

    set container {
        background { color: background }
        padding { padding }
    }

    variant Dark {
        background = 0x1E1E1E
        text_color = 0xFFFFFF
    }

    variant Compact {
        padding = 8
    }
}
```

**Variant properties:**

- Declared with `variant` keyword followed by a name
- Body contains field assignments that override theme field values
- Type is the same as the parent theme type
- Identity is distinct from parent theme
- Can only override existing fields (cannot add new fields)
- Inherits all instruction sets from parent theme
- All overridden fields must be compatible with their declared types

**Usage:**

```frel
backend AppBackend {
    theme : ref MessageTheme = MessageTheme.Dark
}
```

## Theme Composition

Themes can be composed using the `include` keyword:

```frel
theme BaseTheme {
    padding : u32 = 16
    corner_radius : u32 = 8

    set base_container {
        padding { padding }
        corner_radius { corner_radius }
    }
}

theme AppTheme {
    include BaseTheme

    primary_color : asset Color

    set button_style {
        base_container
        background { color: primary_color }
    }
}
```

**Composition behavior:**

- All fields are added to the including theme
- All asset fields are added to the including theme
- All instruction sets are added to the including theme
- All variants are added to the including theme
- Members are added flat - no namespacing needed
- Multiple themes can be included
- Name conflicts result in compilation error
- Diamond inclusion (same theme via multiple paths) includes once

## Using Themes

Themes are fully-fledged types and can be used like any other data:

### Theme References

```frel
backend AppBackend {
    theme : ref AppTheme
    current_padding : u32 = theme.padding
}

blueprint Button(theme: ref AppTheme) {
    box { } .. theme.button_style
}
```

### Accessing Theme Fields

```frel
blueprint Card {
    with AppBackend

    // Access regular fields
    spacing : u32 = theme.padding * 2

    // Use in expressions
    text { "Padding: ${theme.padding}" }

    // Use in instructions
    box { } .. padding { theme.padding }
}
```

### Applying Instruction Sets

```frel
blueprint StyledBox {
    with AppBackend

    // Postfix syntax
    box { } .. theme.card_style

    // Inner syntax
    box {
        theme.card_style
        text { "Content" }
    }

    // Conditional application
    box { } .. is_important ? theme.important_style : theme.normal_style
}
```

### Using Variants

```frel
backend ThemeBackend {
    theme : ref AppTheme = AppTheme.Dark
    use_compact : bool = false

    command switch_to_compact() {
        theme = AppTheme.Compact
    }
}

blueprint ThemedUI(theme_variant: ref AppTheme) {
    box { } .. theme_variant.button_style
}
```

## Visibility

All theme members are public and accessible via the theme reference.