# Frel Standard Library

Higher-level UI widgets and components written in Frel.

## Contents

This library provides common UI patterns and widgets built on top of the standard blueprints (text, image, icon, box, column, row) provided by the platform adapters.

### Planned Components

- **Button**: Interactive button with styling
- **Card**: Container with shadow and border
- **Input**: Text input field
- **Checkbox**: Checkbox input
- **Radio**: Radio button
- **Select**: Dropdown selection
- **Modal**: Dialog overlay
- **Toast**: Notification messages
- **Tooltip**: Hover information
- **List**: Scrollable list of items
- **Grid**: Grid layout
- **Form**: Form container with validation

## Usage

```frel
import frel.stdlib.Button
import frel.stdlib.Card

blueprint MyApp {
    Card {
        Button {
            text { "Click me!" }
        }
    }
}
```

## Development

These widgets are written entirely in Frel and compiled to JavaScript using the Frel compiler.
