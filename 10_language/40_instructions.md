# Instructions

**DIP**: Device Independent Pixel (floating point)

## Color

`<color>` is a 32-bit RGBA value representing a color.

**Supported formats:**

- `0xRRGGBB` - Hex RGB (alpha defaults to 255/opaque)
- `0xRRGGBBAA` - Hex RGBA
- `rgb(r, g, b)` - RGB components (0-255, alpha defaults to 255)
- `rgba(r, g, b, a)` - RGBA components (0-255)
- `<color-name>` - Named color constant (e.g., `Red`, `Blue`, `Transparent`)

**Examples:**

```dsl
background { color: 0xFF0000 }           // Red (opaque)
background { color: 0xFF0000FF }         // Red (opaque, explicit alpha)
background { color: rgb(255, 0, 0) }     // Red
background { color: rgba(255, 0, 0, 128) } // Red, 50% transparent
background { color: Red }                // Named color
border { color: Blue }                   // Named color in border
```

## Layout

### Position

`position { top : DIP left : DIP }`

Places the node at the given position.

**IMPORTANT**: Only positional containers (Box) support this instruction.

Note: `top` and `left` are relative to the `top` and `left` of the container **content box**
(that is without the surrounding).

Note: `bottom` and `right` are intentionally left out. Use `top` and `left` translated by 
the height and width of the node.

### Dimensions

`<dim> { <value> max : DIP min : DIP }`

`<dim>` may be one of:

- `width` : The width of the node.
- `height` : The height of the node.

`<value>` may be one of:

| Value       | Description                                                |
|-------------|------------------------------------------------------------|
| DIP         | The exact size of the node.                                |
| `expand`    | Use the space proposed by the container, grow if possible. |
| `container` | Use all the space proposed by the container.               |
| `content`   | Use the size of the child nodes plus the surrounding.      |

- `max` the maximum size of the node, optional
- `min` the minimum size of the node, optional

**Shorthands**

| Shorthand        | Full                                          |
|------------------|-----------------------------------------------|
| `fit_content`    | `width { content } .. height { content }`     |
| `fill_width`     | `width { container }`                         |
| `fill_height`    | `height { container }`                        |
| `fill`           | `width { container } .. height { container }` |
| `expand`         | `width { expand } .. height { expand }`       |

Note: percentages are not supported. For weight-based layouts use `grid` and
define a template.

### Surrounding

`<surrounding-type> { top : DIP right : DIP bottom : DIP left : DIP color : <color> }`

`<surrounding-type>` may be one of:

- `padding` : The padding surrounding the node.
- `border` : The border surrounding the node.
- `margin` : The margin surrounding the node.

All the dimensions are optional, but at least one must be specified.

`color` : The color of the border, optional, only for `border`.

**Shorthands**

NOTE: `color` may be used with any of the shorthands when `<type>` is `border`.

| Shorthand                     | Full                                                                  |
|-------------------------------|-----------------------------------------------------------------------|
| `<type> { horizontal : DIP }` | `<type> { left: DIP right : DIP }`                                    |
| `<type> { vertical : DIP }`   | `<type> { top: DIP bottom : DIP }`                                    |
| `<type> { DIP }`              | `<type> { top: DIP right : DIP bottom : DIP left : DIP }`             |
| `border { <color> DIP }`      | `border { top: DIP right: DIP bottom: DIP left: DIP color: <color> }` |

### Fill strategy

```
fill_strategy { constrain|constrain_reverse|resize_to_max }
```

Specifies how a directional algorithmic container (row and column) should fill its children.

- `constrain` : The children are measured one-by-one. The space used by **earlier** children is subtracted from the space available to **later** children.
- `constrain_reverse` : The children are measured one-by-one. The space used by **later** children is subtracted from the space available to **earlier** children.
- `resize_to_max` : Children are measured one-by-one, then resized to size of the largest child.

**Shorthands**

| Shorthand           | Full                                  |
|---------------------|---------------------------------------|
| `constrain`         | `fill_strategy { constrain }`         |
| `constrain_reverse` | `fill_strategy { constrain_reverse }` |
| `resize_to_max`     | `fill_strategy { resize_to_max }`     |

### Gap

`gap { width : DIP height : DIP }`

The gap between children. Positional containers ignore this instruction.

Both dimensions are optional, but at least one must be specified.

**Shorthands**

| Shorthand     | Full                              |
|---------------|-----------------------------------|
| `gap { DIP }` | `gap { width: DIP height : DIP }` |

### Inner Alignment

`<target> { horizontal: (start|center|end) vertical: (top|center|baseline|bottom) }`

`<target>` may be one of:

- `align_self` : Align the node on the given axis.
- `align_items` : Align all the children on the given axis.`

`align_self` has precedence over `align_items`.

**Shorthands**

| Shorthand                          | Full                                                          |
|------------------------------------|---------------------------------------------------------------|
| `<target>_center`                  | `<target> { horizontal : center vertical: center }`           |
| `<target>_<horizontal>_<vertical>` | `<target> { horizontal : <horizontal> vertical: <vertical> }` |

Examples:

```text
align_self { horizontal: start vertical: bottom }
align_items { horizontal: center vertical: top }

align_self_center
align_items_center

align_self_start_top
align_items_center_bottom
```

### Outer Alignment

`align_relative { horizontal: (before|start|center|end|after) vertical : (above|start|center|end|below)  }`

Note: `align_relative` is used primarily by popups to align themselves to the element they are relative to.
See [Detached UI - Tooltip](50_detached_ui.md#tooltip).

The following diagram shows the positions for each alignment. The corners/edges touch the node 
they are relative to (end/start at the previous/next pixel).

```text
  Before-Above  Start-Above    Center-Above    End-Above After-Above
              ┌─────────────────────────────────────────┐ 
  Before-Start│Start-Start    Center-Start     End-Start│After-Start
              │                                         │
 Before-Center│Start-Center   Center-Center   End-Center│After-Center
              │                                         │
   Before-End │Start-End       Center-End        End-End│After-End
              └─────────────────────────────────────────┘   
  Before-Below Start-Below   Center-Below      End-Below After-Below
```

Note: naming is asymmetric on purpose, so there is no conflict between horizontal and vertical.

### Spacing

- `space_around` : Distribute the space around the children.
- `space_between` : Distribute the space between the children.

### Scroll

`scroll { horizontal|vertical|both }`

The container’s content box area is scrollable (whatever you define as “inner size” in your docs). 
All child sizes (including your “margins are part of size” rule) contribute to the scrollable extent.

`scroll { <dir> }` and `<dim> { content }` are mutually exclusive in the given direction.
These combinations on the same node generate a compile-time error:

- `box { } .. width { content } .. scroll { horizontal } // ERROR`
- `box { } .. height { content } .. scroll { vertical } // ERROR`

All other combinations are valid, for example:

- `box { } .. height { container } .. scroll { vertical } // OK`
- `box { } .. width { content } .. scroll { vertical } // OK`

### Notes

**Overflow** is supported only by scrolling. In my experience overflow clip and hidden are
tools junior developers use to hide layout bugs.

## Decoration

**Notes:**

- Border is documented in the Layout section (Surrounding)
- Border is a mix of decoration and layout - its width affects layout calculations

### Background

`background { color: <color> }`
`background { color: <color> opacity: f32 }`
`background { gradient: <gradient> }`
`background { image: <image-resource> }`

Fills the background of the fragment's content box and padding area.

**Parameters:**

- `color` - Background color (see Color section above)
- `opacity` - Alpha multiplier (0.0 = transparent, 1.0 = opaque), applied to the color
- `gradient` - Gradient fill (linear or radial)
- `image` - Image resource to fill background

**Examples:**

```dsl
box { } .. background { color: White }
box { } .. background { color: Red opacity: 0.5 }
box { } .. background { gradient: linear(Red, Blue) }
box { } .. background { image: "background.png" }
```

**Notes:**

- Background is painted behind content and padding, but not margin
- When using `opacity`, it multiplies with the color's alpha channel
- Image backgrounds are stretched to fill the content box

### Corner Radius

`corner_radius { top_left: DIP top_right: DIP bottom_left: DIP bottom_right: DIP }`

Rounds the corners of the fragment's border box.

**Parameters:**

- `top_left` - Top-left corner radius
- `top_right` - Top-right corner radius
- `bottom_left` - Bottom-left corner radius
- `bottom_right` - Bottom-right corner radius

All parameters are optional.

**Shorthands:**

| Shorthand                       | Full                                                                                |
|---------------------------------|-------------------------------------------------------------------------------------|
| `corner_radius { DIP }`         | `corner_radius { top_left: DIP top_right: DIP bottom_left: DIP bottom_right: DIP }` |
| `corner_radius { bottom: DIP }` | `corner_radius { bottom_left: DIP bottom_right: DIP }`                              |
| `corner_radius { top: DIP }`    | `corner_radius { top_left: DIP top_right: DIP }`                                    |
| `corner_radius { left: DIP }`   | `corner_radius { top_left: DIP bottom_left: DIP }`                                  |
| `corner_radius { right: DIP }`  | `corner_radius { top_right: DIP bottom_right: DIP }`                                |


**Examples:**

```dsl
box { } .. corner_radius { 8 }                               // All corners 8 DIP
box { } .. corner_radius { top_left: 8 bottom_right: 4 }     // Individual corners
box { } .. corner_radius { right: 8 }                        // Top right and bottom right
```

**Notes:**

- Corner radius is clamped to half the width/height to prevent overlapping
- Applied to background, border, and shadow

### Shadow

`shadow { color: <color> offset_x: DIP offset_y: DIP blur: DIP }`

Adds a drop shadow to the fragment.

**Parameters:**

- `color` - Shadow color (including alpha for transparency)
- `offset_x` - Horizontal offset (positive = right, negative = left)
- `offset_y` - Vertical offset (positive = down, negative = up)
- `blur` - Blur radius (0 = sharp, higher = softer)

**Examples:**

```dsl
box { } .. shadow { color: rgba(0, 0, 0, 128) offset_x: 0 offset_y: 2 blur: 4 }
box { } .. shadow { color: 0x00000080 offset_x: 4 offset_y: 4 blur: 8 }
```

**Notes:**

- Shadow is rendered outside the element's box (does not affect layout)
- Multiple shadows are not currently supported
- Shadow follows the corner radius if specified

### Cursor

`cursor { default|pointer|text|crosshair|move|none|grab|grabbing }`

Sets the mouse cursor appearance when hovering over the fragment.

**Values:**

- `default` - Standard arrow cursor
- `pointer` - Pointing hand (for clickable elements)
- `text` - I-beam cursor (for text input)
- `crosshair` - Crosshair
- `move` - Move cursor (for draggable elements)
- `none` - Do not show cursor
- `grab` - Open hand (for draggable)
- `grabbing` - Closed hand (while dragging)

**Example:**

```dsl
button { } .. cursor { pointer }
text_input { } .. cursor { text }
```

## Focus

### Focusable

`focusable { order: i32|false|programmatic }`

> [!NOTE]
> 
> Scrolling automatically to the focused node is an implementation/adapter detail, it is
> not specified in the language.
> 

Makes the node focusable.

- `order` determines tab order (lower values focused first)
- the default order is 0 (document order used for ties)
- negative values are allowed (focused before positive)
- `false` makes the node not focusable
- `programmatic` makes the node skipped during tab navigation but focusable programmatically
- when the focused node is removed, the next focusable node is focused

**Shorthands**

| Shorthand       | Full                  |
|-----------------|-----------------------|
| `focusable`     | `focusable { 0 }`     |
| `not_focusable` | `focusable { false }` |

### Autofocus

`autofocus`

- Node receives focus when created.
- When more than one fragment has `autofocus`, the last-rendered wins.

### Focus Trap

`focus_trap`

Tab/Shift+Tab cycles focus within this subtree.

Escape key does not automatically exit the trap. Fragments may implement this behavior.

In the case of nested focus traps, the inner trap takes precedence; tab cycles within the inner trap.
When the inner trap is removed, Tab resumes in the outer trap.

## Stereotype

Stereotypes add semantic behavior to fragments.

`stereotype { cancel|save }`

**Scope:**

- Search upward from a stereotyped element.
- Fire handler on the first ancestor that has the event.
- If there is no handler in the channel (considering only direct upward fragments, no horizontal search): 
  - generate an error for pointer events
  - ignore for keyboard events

**Behavior:**

- `cancel`: Triggers `on_cancel` event.
- `save`: Triggers `on_save` event. 

**Pointer handling:**

- `cancel`: Clicking on the fragment triggers `on_cancel` event.
- `save`: Clicking on the fragment triggers `on_save` event.

**Keyboard shortcuts:**

1. If there is no handler in the scope, the keyboard event is ignored.
2. Otherwise, the current focus specifies what happens.

> [!NOTE]
> 
> The focus-based behavior is designed to match user expectations
> and avoid accidental navigation.
>

`Escape` → `on_cancel`
  - there is no focused fragment in the scene **or**
  - the focused fragment is the one with the `on_cancel` handler

`Ctrl+S` → `on_save`
  - with any focus situation
      
`Enter` → `on_save` **or** `on_cancel`
  - current focus has `save` stereotype -> `on_save`
  - current focus has `cancel` stereotype -> `on_cancel`

```rust
fragment! { 
    Confirm(text: &str) {
        DefaultModal {
      
            text { "Are you sure?" }
            button { "No" } .. stereotype { cancel }
            button { "Yes" } .. stereotype { save }
            
            on_save { /* ... */ } 
        }
    }
}
```

## Event

### Pointer events

**Event propagation:** Pointer events fire only on the target fragment (no bubbling).

```rust
// mod frel::pointer

struct PointerEvent {
    target_id    : u32,   //  Logical node/instance id
    phase        : u8,    //  0 move, 1 down, 2 up, 3 enter, 4 leave
    pointer_kind : u8,    //  0 mouse, 1 touch, 2 pen
    button       : u8,    //  primary=0, aux=1, secondary=2
    buttons_mask : u16,   //  bitmask of primary=0, aux=1, secondary=2
    modifiers    : u16,   //  same as in KeyEvent
    pointer_id   : u32,   //
    x_dip        : f32,   //  DIP - X coordinate relative to the fragment's top-left corner
    y_dip        : f32,   //  DIP - Y coordinate relative to the fragment's top-left corner
    pressure     : f32,   //  0..1 (mouse 0/0.5/1 as available)
    tilt_x       : i16,   //  pen tilt deg
    tilt_y       : i16,   //  pen tilt deg
    tangential   : f32    //  0 if N/A
}

const PHASE_MOVE: u8 = 0;
const PHASE_DOWN: u8 = 1;
const PHASE_UP: u8 = 2;
const PHASE_ENTER: u8 = 3;
const PHASE_LEAVE: u8 = 4;

const POINTER_KIND_MOUSE: u8 = 0;
const POINTER_KIND_TOUCH: u8 = 1;
const POINTER_KIND_PEN: u8 = 2;

const BUTTON_PRIMARY: u8 = 0;
const BUTTON_AUX: u8 = 1;
const BUTTON_SECONDARY: u8 = 2;
```

```
on_click |event: PointerEvent| { <event-handler> }
on_double_click |event: PointerEvent| { <event-handler> }
on_pointer_move |event: PointerEvent| { <event-handler> }
on_pointer_enter |event: PointerEvent| { <event-handler> }
on_pointer_leave |event: PointerEvent| { <event-handler> }
on_primary_down |event: PointerEvent| { <event-handler> }
on_primary_up |event: PointerEvent| { <event-handler> }
on_secondary_down |event: PointerEvent| { <event-handler> }
on_secondary_up |event: PointerEvent| { <event-handler> }
```

#### Suppressing pointer events

`pointer_events { enabled|disabled }`

`enabled` enables the pointer events for the fragment and **all** its children.
`disabled` disables the pointer events for the fragment and **all** its children.

When `enabled` is used in a child of a `disabled` parent, the child receives pointer events.

**Shorthands**

| Shorthand             | Full                          |
|-----------------------|-------------------------------|
| `with_pointer_events` | `pointer_events { enabled }`  |
| `no_pointer_events`   | `pointer_events { disabled }` |

### Wheel events

`on_wheel |event: WheelEvent| { <event-handler> }`

```rust
// mod frel::wheel

struct WheelEvent {
    modifiers : u16,    // Uses KEY_MODIFIER_* constants from keyboard module
    delta_x   : f32,    // DIP - horizontal scroll delta
    delta_y   : f32,    // DIP - vertical scroll delta  
    phase     : u8,     // 0=update, 1=begin, 2=end
}

const WHEEL_PHASE_UPDATE: u8 = 0;
const WHEEL_PHASE_BEGIN: u8 = 1;
const WHEEL_PHASE_END: u8 = 2;
```

### Keyboard events

Keyboard events are triggered when a keyboard action is performed by the user while the fragment 
or **any of its children** has focus.

**Event propagation:** The first event handler upwards (same channel, no horizontal) from the
focused fragment is called.

`on_key |event: KeyEvent| { <event-handler> }`

Convenience handlers:

`on_enter { <event-handler> }`
`on_escape { <event-handler> }`

Convenience handlers fire once when the key is **first pressed** (Down action only).
Repeats and key releases are ignored.

Input events are triggered only on text input primitives when the composed text is ready:

`on_input |event: InputEvent| { <event-handler> }`

```rust
// mod frel::keyboard

struct KeyEvent {
    action: KeyAction,
    modifiers: u16,
    key_name: &str,      // Physical key (e.g., "KeyA", "Digit1")
}

struct InputEvent {
    character: &str      // Composed character (e.g., "A", "1", "Ω") - empty for non-printable
}

enum KeyAction {
    Down,
    Up,
    Repeat
}

const KEY_MODIFIER_SHIFT: u16 = 0x01;
const KEY_MODIFIER_CTRL: u16 = 0x02;
const KEY_MODIFIER_ALT: u16 = 0x04;
const KEY_MODIFIER_CMD: u16 = 0x08;


mod key {
    // Arrow keys
    const ARROW_LEFT: &str = "ArrowLeft";
    const ARROW_RIGHT: &str = "ArrowRight";
    const ARROW_UP: &str = "ArrowUp";
    const ARROW_DOWN: &str = "ArrowDown";

    // Other keys
    const BACKSPACE: &str = "Backspace";
    const TAB: &str = "Tab";
    const ENTER: &str = "Enter";
    const ESCAPE: &str = "Escape";
    const SPACE: &str = "Space";
    const HOME: &str = "Home";
    const END: &str = "End";
    const PAGE_UP: &str = "PageUp";
    const PAGE_DOWN: &str = "PageDown";
}
```

### On Resize

`on_resize |rect : Option<Rect>| { <event-handler> }`

The event handler is called when the node is resized. The call takes place after all layout changes
are applied.

- `rect` contains the position and size of the node.
- If the sizes do not change during a layout pass, the event handler is not called.
- Size comparison uses a small epsilon to avoid false positives.

**NOTE** Be careful with this event handler, so you don't trigger infinite loops.

### Focus events

Focus events are triggered when the fragment or **any of its children** gains or loses focus. 

`on_focus { <event-handler> }`
`on_blur { <event-handler> }`

### Stereotype events

Stereotype events provide semantic handlers for stereotype generated events.

`on_cancel { <event-handler> }`
`on_save { <event-handler> }`

## Text

**SP (Scaled Pixel):** A unit that scales with the platform's text scaling settings.

Unlike DIP which remains constant, SP adjusts when users change their system font size preferences.
Use SP for font sizes to respect accessibility settings.

### Font

`font { name: String size: SP weight: u32 color: <color> }`

Sets font properties for text content.

**Parameters:**

- `name` - Font family name (e.g., "Arial", "system-ui", or custom font identifier)
- `size` - Font size in SP (Scaled Pixels)
- `weight` - Font weight from 100 (thin) to 900 (black)
  - 100 = Thin
  - 200 = Extra Light
  - 300 = Light
  - 400 = Normal/Regular
  - 500 = Medium
  - 600 = Semi Bold
  - 700 = Bold
  - 800 = Extra Bold
  - 900 = Black
- `color` - Text color (see [Color](#color))

All parameters are optional.

**Examples:**

```dsl
text { "Hello" } .. font { name: "Arial" size: 16 weight: 400 color: Black }
text { "Title" } .. font { size: 24 weight: 700 }
text { "Body" } .. font { name: "system-ui" }
```

**Shorthands:**

Common font combinations can be defined as style templates (see Resources documentation).

**Notes:**

- If font name is not available, platform-specific fallback is used
- Weight values between defined values may be rounded to nearest available weight
- SP scales with user's accessibility settings, DIP does not

### Line Height

`line_height { height: DIP }`

Sets the vertical spacing between lines of text.

**Parameters:**

- `height` - Line height in DIP (Device Independent Pixels)

**Examples:**

```dsl
text { "..." } .. line_height { 20 }     // Fixed 20 DIP
text { "..." } .. line_height { 24 }     // More spacious
```

**Notes:**

- Line height is the total height from baseline to baseline
- Typical ratio is 1.2-1.5 times the font size
- Measured in DIP (not SP) for consistent spacing regardless of text scaling

### Text Selection

`no_select`

Prevents text from being selectable by the user.

**Example:**

```dsl
text { "Button Label" } .. no_select
```

**Notes:**
- Useful for UI chrome, buttons, and labels
- By default, text is selectable

### Text Wrapping

`text_wrap { none|wrap }`

Controls how text handles overflow.

**Values:**

- `none` - Single line, text overflows if too wide
- `wrap` - Multi-line, wraps at word boundaries

**Examples:**

```dsl
text { "Short" } .. text_wrap { none }
text { "Long paragraph..." } .. text_wrap { wrap }
```

**Notes:**
- `none` combined with width constraints may clip text
- Use `text_overflow` from primitives for ellipsis behavior
- Wrapping respects word boundaries (no mid-word breaks by default)

### Text Overflow

`text_overflow { visible|ellipsis }`

Controls how overflowing text is displayed (only applies to single-line text).

**Values:**

- `visible` - Text overflows its container
- `ellipsis` - Text is clipped and "..." is shown

**Example:**

```dsl
text { "Very long text..." } .. text_wrap { none } .. text_overflow { ellipsis }
```

**Notes:**
- Only effective when `text_wrap { none }` is set
- Requires a width constraint on the text element

### Text Decoration

`underline`

Adds an underline to the text.

**Example:**

```dsl
text { "Link" } .. underline .. font { color: Blue }
```

**Notes:**
- Underline color matches the text color
- Other decorations (strikethrough, overline) are not currently supported

### Text Transform

`small_caps`

Renders lowercase letters as smaller capitals.

**Example:**

```dsl
text { "Title" } .. small_caps
```

**Notes:**
- True small caps use font's small-caps glyphs if available
- Otherwise, synthesized by scaling capitals

### Letter Spacing

`letter_spacing { value: f64 }`

Adjusts spacing between characters.

**Parameters:**

- `value` - Additional spacing in DIP (can be negative)

**Examples:**

```dsl
text { "Spaced" } .. letter_spacing { 2.0 }     // Wider spacing
text { "Tight" } .. letter_spacing { -0.5 }     // Tighter spacing
text { "Normal" } .. letter_spacing { 0.0 }     // Default
```

**Notes:**
- Positive values increase spacing
- Negative values decrease spacing
- Measured in DIP for consistent spacing
- Affects all characters equally

### Text Alignment

Text alignment is handled by the container's `align_items` or the `text` fragment's 
`align_self` instruction (see [Layout](#layout)).

Text alignment is layout-based and is handled by the container. The `text` fragment 
does not align the text within itself.

For horizontal alignment within a container:

- `align_items { horizontal: start }` - Left-aligned text
- `align_items { horizontal: center }` - Center-aligned text
- `align_items { horizontal: end }` - Right-aligned text
- `text { "Hello" } .. align_self_start` - Left-aligned text
- `text { "Hello" } .. align_self_center` - Center-aligned text
- `text { "Hello" } .. align_self_end` - Right-aligned text