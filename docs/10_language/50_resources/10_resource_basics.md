# Resource Basics

Resource declarations define application assets such as images, strings, fonts, and themes. Resources are typically auto-generated from the filesystem structure but can also be declared manually for programmatic or computed resources.

## Syntax

```text
<resource> ::= "resource" <category> "{" <resource-item>* "}"
<resource-item> ::= <name> [ "(" <param-list> ")" ] [ "=" <expr> ]

<category> ::= "Strings" | "Images" | "Fonts" | "Themes" | "Documents" | "Files" | "Graphics"
```

## Semantics

- **Category determines type**: All items in a `resource` block have the same type based on category
- **Auto-generated**: Resource declarations are typically generated from filesystem structure
- **Manually extensible**: Can be declared manually for dynamic or computed resources
- **Qualified variants**: Resources support qualifiers (language, density, theme) resolved at runtime

## Auto-Generated Resources

Resource files in the filesystem automatically generate declarations:

**Filesystem:**
```
resources/
  images/
    avatar.png
    background-hdpi.png
  strings/
    strings-en.xml
    strings-hu.xml
```

**Generated:**
```frel
resource Images {
    avatar
    background
}

resource Strings {
    welcome
    greeting
}
```

## Manual Declaration

For programmatic or computed resources:

```frel
resource Images {
    // Static (from filesystem)
    logo
    icon_home

    // Dynamic (programmatic)
    user_avatar(user_id: u32) = load_avatar(user_id)
}

resource Strings {
    // With interpolation parameters
    welcome(name: String)
    item_count(count: u32)
}
```

## Resource Categories

| Category    | Type       | Description                  |
|-------------|------------|------------------------------|
| `Strings`   | `String`   | Localized text               |
| `Images`    | `Image`    | Raster graphics (PNG, JPG)   |
| `Graphics`  | `Graphic`  | Vector graphics (SVG)        |
| `Fonts`     | `Font`     | Font files (TTF, WOFF2)      |
| `Themes`    | `Theme`    | Component styling            |
| `Documents` | `Document` | Structured text (Markdown)   |
| `Files`     | `File`     | Raw files (PDF, JSON, etc.)  |

## Usage

Resources are accessed via their category namespace:

```frel
blueprint UserProfile() {
    column {
        image { Images.avatar }
        text { Strings.welcome }
        text { Strings.greeting(name: "Alice") }
    } .. font { name: Fonts.heading }
}
```

## Qualifiers

Resources support environment-based variants (light/dark theme, language, density):

```frel
resource Images {
    // Compiler knows variants from filesystem:
    // - background-mdpi.png
    // - background-hdpi.png
    // - background-xhdpi.png
    background
}

resource Strings {
    // Compiler knows variants from filesystem:
    // - strings-en.xml
    // - strings-hu.xml
    welcome
}
```

Qualification is resolved automatically at runtime based on the current environment.

See [Resource System](20_resource_system.md) for detailed information about qualifiers, resolution, and the resource subsystem.

## Example

```frel
// Auto-generated from filesystem
resource Images {
    logo
    avatar
    background
}

// Extended with dynamic resources
resource Images {
    user_avatar(user_id: u32) = load_avatar(user_id)
    fallback_image(name: String) = Images.lookup(name).unwrap_or(Images.logo)
}

// String resources with parameters
resource Strings {
    welcome
    greeting(name: String)
    item_count(count: u32)
}
```
