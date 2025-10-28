# Box model

```text
    Top
Left┌────────────────────────────────────────────┐   ┬
    │                  MARGIN                    │   │
    │  ┌──────────────────────────────────────┐  │   │
    │  │               BORDER                 │  │   │
    │  │  ┌───────────────────────────────┐   │  │   │
    │  │  │           PADDING             │   │  │   │  Height
    │  │  │  ┌─────────────────────────┐  │   │  │   │
    │  │  │  │        CONTENT          │  │   │  │   │
    │  │  │  └─────────────────────────┘  │   │  │   │
    │  │  └───────────────────────────────┘   │  │   │
    │  └──────────────────────────────────────┘  │   │
    └────────────────────────────────────────────┘   ┴ 
    ├────────────────── Width ───────────────────┤
```

The first three: margin, border and padding together are called **surrounding**.

`content`, called **content box** is the actual content of the node, for layouts it
contains the children, for non-layout nodes it contains the actual content such as
text, image, etc.

Width and height together define the size of the node.

In contrast with other layout systems, **margin is counted into the size** of a node.

As the picture above shows, top-left means the top-left corner of the surrounding,
margins included.

- **padding** uses the background of the node.
- **border** uses the specified border color.
- **margin** is transparent.