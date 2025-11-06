# Primitives

Each supported platform provides the following primitives.

## Group

A logical parent with no paint of its own.

Supported styling:

- none

## Rectangle

A generic rectangle to paint background and borders.

Supported styling:

- background
- border
- corner_radius
- shadow

## Text

A non-breakable string of characters.

Supported styling:

- font { name, size, weight, color }
- line_height
- no_select
- underline
- small_caps
- letter_spacing
- text_overflow { visible / ellipsis }

## Raster image

An image (PNG, JPG).

Size of the image is specified by layout.

Supported styling:

- none

## Icon

A tiny, stylable symbol (font-glyph or vector).

Size of the icon is specified by the layout.

Supported styling:

- color

## Native text input

A platform native text input.

Supported styling:

- font { name, size, weight, color }
- line_height
- letter_spacing

### Native host

Renders native platform widgets, provides a way to use special features of a given platform.

Supported styling:

- none