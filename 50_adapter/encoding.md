# Encoding

Adapters pass encoded events to the runtime and receive encoded patches from the runtime.

The encoding is designed to be **platform independent**, **Rust-friendly** and **fast**.

**Note:** little-endianness is a core requirement for the library, therefore, encodings can
safely assume little-endian.

## Principles

- Framed records in contiguous buffers. No varints; Just fixed-width little-endian.
- 4-byte alignment. When ending non-aligned, add padding (part of the record size, decoders must
  ignore it)
- Time = monotonic microseconds (u64) since page timeOrigin.
- DIP everywhere for layout-related lengths (f32).
- Colors are u32 RGBA.
- IDs (u32) for nodes/instances (stable across page lifetime).
- Bitmasks for modifiers and style fields.

## Event encoding

Event record header

| Field      | Type | Notes                                    |
|------------|------|------------------------------------------|
| version    | u8   | Start at `1`                             |
| kind       | u8   | `EventKind` (see below)                  |
| flags      | u16  | Reserved (0 for now)                     |
| size_bytes | u32  | Total size of this record (incl. header) |
| seq        | u32  | Producer-assigned, strictly increasing   |
| time_us    | u64  | Microseconds since timeOrigin            |

>> I don't like this list of events. They are way too low-level for
>> normal applications.

```text
0 = Pointer
1 = Wheel
2 = Key
3 = Resize
```

### Pointer

All bitmasks are LSB = bit0.

| Field        | Type | Notes                                      |
|--------------|------|--------------------------------------------|
| target_id    | u32  | Logical node/instance id                   |
| phase        | u8   | 0 move, 1 down, 2 up, 3 enter, 4 leave     |
| pointer_kind | u8   | 0 mouse, 1 touch, 2 pen                    |
| button       | u8   | primary=0, aux=1, secondary=2              |
| reserved0    | u8   | 0                                          |
| buttons_mask | u16  | bitmask of primary=0, aux=1, secondary=2   |
| modifiers    | u16  | bit0 Shift, bit1 Ctrl, bit2 Alt, bit3 Meta |
| pointer_id   | u32  |                                            |
| x_dip        | f32  | DIP                                        |
| y_dip        | f32  | DIP                                        |
| pressure     | f32  | 0..1 (mouse 0/0.5/1 as available)          |
| tilt_x       | i16  | pen tilt deg                               |
| tilt_y       | i16  | pen tilt deg                               |
| tangential   | f32  | 0 if N/A                                   |

### Wheel

| Field     | Type | Notes                    |
|-----------|------|--------------------------|
| target_id | u32  |                          |
| modifiers | u16  |                          |
| reserved0 | u16  |                          |
| delta_x   | f32  | DIP                      |
| delta_y   | f32  | DIP                      |
| phase     | u8   | 0=update, 1=begin, 2=end |

### Key

| Field     | Type   | Notes                             |
|-----------|--------|-----------------------------------|
| target_id | u32    |                                   |
| action    | u8     | 0 down, 1 up, 2 repeat            |
| reserved0 | u8     |                                   |
| modifiers | u16    |                                   |
| text_len  | u16    | UTF-8 bytes following (0 if none) |
| …text…    | \[u8\] | `text_len` bytes UTF-8            |

**Note:** IME composition and dead-key handling are in the scope of the adapter.
The runtime does not need to get an event for those.

### Resize

Resize event is sent when the whole scene (window) is resized.

| Field     | Type | Notes |
|-----------|------|-------|
| w_dip     | f32  |       |
| h_dip     | f32  |       |

## Patch encoding

Patch record header (12 bytes)

| Field      | Type |                           |
|------------|------|---------------------------|
| version    | u8   | Start at `1`              |
| opcode     | u8   | `PatchOpcode` below       |
| flags      | u16  | Reserved                  |
| size_bytes | u32  | Total size (incl. header) |
| seq        | u32  | Producer-assigned (WASM)  |

```text
0  = CreateNode
1  = RemoveNode
2  = SetText
3  = SetScroll
4  = SetStyle
5  = SetPosition
```

**Note:** Node move (parent change) is not supported directly (you can remove/add a node to move nodes).

### CreateNode

| Field     | Type | Notes                                                                   |
|-----------|------|-------------------------------------------------------------------------|
| node_id   | u32  |                                                                         |
| parent_id | u32  |                                                                         |
| type      | u16  | 0 group, 1 rect, 2 text, 3 image, 4 icon, 5 native_input, 6 native_host |
| reserved  | u16  |                                                                         |

### RemoveNode

| Field   | Type | Notes |
|---------|------|-------|
| node_id | u32  |       |

### SetText

| Field    | Type |
|----------|------|
| node_id  | u32  |
| len_utf8 | u16  |
| …bytes…  | [u8] |

**Note:** UTF-8 is used for text. Text size limit is intentional and non-negotiable. Larger 
texts will be transferred by other means.

### SetScroll

| Field   | Type |
|---------|------|
| node_id | u32  |
| x_dip   | f32  |
| y_dip   | f32  |

### SetStyle

Updates supplied style/render data in one shot. The mask tells the decoder which fields
follow and in what order.

| Field   | Type | Bit |
|---------|------|-----|
| node_id | u32  | —   |
| mask    | u32  | —   |

Then, in increasing bit order, the fields below **only if the bit is set**:

| Bit | Field(s)                           | Type                  |
|-----|------------------------------------|-----------------------|
| 0   | padding_top/right/bottom/left      | f32 × 4               |
| 1   | border_top/right/bottom/left/color | f32 × 4 + u32         |
| 2   | margin_top/right/bottom/left       | f32 × 4               |
| 3   | background_rgba                    | u32                   |
| 4   | corner_radius_t/r/b/l              | f32 × 4               |
| 5   | shadow_rgba, off_x, off_y, dev     | u32, f32×3            |
| 6   | text_overflow                      | u8 (0 vis,1 ellipsis) |
| 7   | no_select                          | u8 (0/1)              |
| 8   | font_name_len + name bytes         | u16 + [u8]            |
| 9   | font_size_sp                       | f32                   |
| 10  | font_weight                        | u16                   |
| 11  | font_color_rgba                    | u32                   |
| 12  | letter_spacing                     | f64                   |
| 13  | line_height_dip                    | f32                   |

### SetPosition

| Field   | Type |
|---------|------|
| node_id | u32  |
| x_dip   | f32  |
| y_dip   | f32  |

## Notes

- Unknown bits/fields result in panic. It is not possible to skip unknown fields as sizes are unknown.
- PX to DIP conversion is adapter-specific, so it is not part of the encoding.
- The encoding has nothing to do with (hence not specified here):
    - Backpressure
    - Event coalescing
