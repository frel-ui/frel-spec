# Standard Blueprints

These blueprint definitions are provided by the standard library and serve as the building blocks for
Frel applications. They are compiled to efficient native rendering primitives specific to each
host platform.

## Basic Blueprints

### text

Renders a string of text.

**Syntax:**

```frel
text { <string-expr> }
```

**Parameters:**

- `content` (template slot) - String expression to display

**Examples:**

```frel
// Simple text
text { "Hello, World!" }

// With styling
text { "Title" } .. font { size: 24 weight: 700 color: Black }

// Reactive text
count : u32 = 0
text { "Count: ${count}" }

// Multi-line with wrapping
text { user.bio } .. text_wrap { wrap } .. width { 300 }

// Styled inline text
text { "Click here" }
    .. font { size: 14 color: Blue }
    .. underline
    .. cursor { pointer }
    .. on_click { /* ... */ }
```

**Notes:**

- Supports all standard instructions - see [Instructions](60_instructions.md)
- Text-specific instructions: `font`, `line_height`, `text_wrap`, `text_overflow`, `underline`, `small_caps`, `letter_spacing`, `no_select`
- String interpolation using `${}` is supported and reactive
- Without `text_wrap { wrap }`, text renders as a single line
- Text inherits parent's text styling instructions when not explicitly set
- Selection is enabled by default (use `no_select` to disable)

### image

Displays a raster image from a resource.

**Syntax:**

```frel
image { <image-resource-ref> }
```

**Parameters:**

- `content` (template slot) - Image resource reference

**Examples:**

```frel
// Fixed size
image { "profile.png" } .. width { 100 } .. height { 100 }

// Resource reference
image { Images.user_profile }

// Rounded corners
image { user.avatar_url }
    .. width { 64 } .. height { 64 }
    .. corner_radius { 32 }

// Clickable image
image { "icon.png" }
    .. width { 24 } .. height { 24 }
    .. cursor { pointer }
    .. on_click { /* ... */ }
```

**Notes:**

- Supports all standard instructions - see [Instructions](60_instructions.md)
- Use explicit `width` and `height` to prevent layout shift during loading
- Images are loaded asynchronously; loading state depends on platform adapter
- Resource references are resolved via the resource system (see [Resources](60_resources.md))
- Supports both embedded resources and dynamic URLs (platform-dependent)

### icon

Displays a vector icon from a graphic resource or icon font.

**Syntax:**

```frel
icon { <graphic-resource-ref> }
```

**Parameters:**

- `content` (template slot) - Graphic resource reference or icon identifier

**Examples:**

```frel
// Basic icon
icon { "close" } .. size { 24 } .. tint { Black }

// Resource reference
icon { Icons.settings } .. width { 20 } .. height { 20 }

// Clickable icon button
icon { "save" }
    .. width { 20 } .. height { 20 }
    .. tint { White }
    .. cursor { pointer }
    .. on_click { /* save action */ }

// Dynamic icon from state
icon_name = is_playing ? "pause" : "play"
icon { icon_name } .. width { 32 } .. height { 32 } .. tint { Blue }
```

**Notes:**

- Supports all standard instructions - see [Instructions](60_instructions.md)
- Icon-specific instruction: `tint { <color> }` - tint color for monochrome icons
- Icons are typically vector graphics (SVG or icon fonts)
- `tint` only affects monochrome/symbolic icons; multi-color icons ignore it
- Use `image` for raster icons

## Containers

Containers are fragments that position and organize child fragments using layout algorithms.

### box

A positional container that places children using absolute coordinates or alignment.

**Syntax:**

```frel
box {
    // children with position instructions
}
```

**Parameters:**

- `content` (blueprint)

**Layout Algorithm:**

Children are positioned using:
1. Explicit positioning: `position { top left }` instruction on children
2. Alignment: `align_self` instruction on children (requires parent dimensions)
3. Default: Children without positioning or alignment are placed at (0, 0)

**Examples:**

```frel
// Absolute positioning
box {
    .. width { 400 } .. height { 300 }
    .. background { color: White }

    text { "Top left" } .. position { top: 10 left: 10 }
    text { "Bottom right" } .. position { top: 270 left: 300 }
}

// Centered content
box {
    .. width { 400 } .. height { 300 }

    text { "Centered" } .. align_self_center
}

// Layered content (z-order matches source order)
box {
    .. width { 200 } .. height { 200 }

    // Background fills entire box (no position needed, defaults to 0,0)
    image { "background.jpg" } .. fill

    // Text centered on top
    text { "Overlay" }
        .. align_self_center
        .. font { color: White weight: 700 }
}

// Mixed positioning
box {
    .. width { 500 } .. height { 400 }

    // Absolute
    icon { "close" }
        .. position { top: 8 left: 8 }
        .. icon_size { 24 }

    // Aligned
    column {
        .. gap { 8 }
        text { "Title" }
        text { "Content" }
    } .. align_self_center
}
```

**Notes:**

- Supports all standard instructions - see [Instructions](60_instructions.md)
- Children without `position` or `align_self` are positioned at (0, 0)
- Coordinates in `position { top left }` are relative to content box (excludes surrounding)
- Later children render on top of earlier children (painter's algorithm)
- Box does not automatically size to children; use explicit dimensions or `expand`
- Use `column` or `row` for automatic vertical/horizontal layouts

### column

A vertical layout container that stacks children top-to-bottom.

**Syntax:**

```frel
column {
    // children rendered vertically
}
```

**Parameters:**

- `content` (blueprint)

**Examples:**

```frel
// Simple vertical stack
column {
    text { "First" }
    text { "Second" }
    text { "Third" }
}

// With spacing and padding
column {
    .. gap { 16 }
    .. padding { 20 }

    text { "Header" } .. font { size: 20 weight: 700 }
    text { "Body content" }
}

// Centered items
column {
    .. width { 400 }
    .. align_items { horizontal: center }
    .. gap { 8 }

    image { "logo.png" } .. width { 100 } .. height { 100 }
    text { "Welcome" }
}

// Mixed heights
column {
    .. height { 500 }
    .. gap { 8 }

    text { "Header" } .. height { 40 }
    box { /* content */ } .. height { expand }  // Takes remaining space
    row { /* buttons */ } .. height { 60 }
}

// Scrollable column
column {
    .. height { 300 }
    .. scroll { vertical }
    .. gap { 8 }

    repeat on items as item {
        text { item.name }
    }
}
```

**Notes:**

- Supports all standard instructions - see [Instructions](60_instructions.md)
- Container-specific: `fill_strategy`, `gap`, `align_items`, `space_between`, `space_around`
- Children with `height { expand }` share remaining space equally
- Children with `height { container }` try to take all available space
- Use `gap` instead of margin between children for cleaner spacing

### row

A horizontal layout container that arranges children left-to-right.

**Syntax:**

```frel
row {
    // children rendered horizontally
}
```

**Parameters:**

- `content` (blueprint)

**Examples:**

```frel
// Simple horizontal layout
row {
    text { "Left" }
    text { "Center" }
    text { "Right" }
}

// With spacing and padding
row {
    .. gap { 8 }
    .. padding { 16 }

    text { "Item 1" }
    text { "Item 2" }
}

// Vertically centered items
row {
    .. height { 60 }
    .. align_items { vertical: center }
    .. gap { 12 }

    icon { "user" } .. size { 32 } .. tint { Black }
    text { user.name }
}

// Mixed widths
row {
    .. width { 800 }
    .. gap { 16 }

    column { /* sidebar */ } .. width { 200 }
    column { /* main */ } .. width { expand }  // Takes remaining space
}

// Scrollable row
row {
    .. width { 400 }
    .. scroll { horizontal }
    .. gap { 8 }

    repeat on images as img {
        image { img.url } .. width { 100 } .. height { 100 }
    }
}
```

**Notes:**

- Supports all standard instructions - see [Instructions](60_instructions.md)
- Container-specific: `fill_strategy`, `gap`, `align_items`, `space_between`, `space_around`
- Children with `width { expand }` share remaining space equally
- Children with `width { container }` try to take all available space
- Use `gap` instead of margin between children for cleaner spacing
- Without explicit `width`, row sizes to fit children
