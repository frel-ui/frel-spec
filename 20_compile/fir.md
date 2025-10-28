# Fragment IR

- The struct `FragmentIR` contains the IR of a fragment template.
- The Fragment Linker processes the IR to link fragments (creating handles, stores, and subscriptions).
- Once a fragment is built, the IR is not used anymore by that fragment.

Binary structure of the IR:

1. The IR is a byte stream of instructions.
2. Instructions are variable-length, the first byte of the instruction contains
   1. Opcode in the low 6 bits
   2. An optional `length of arguments - 1` (in bytes) in the high 2 bits, zero if the instruction has no arguments.
   3. Argument length is a means to compress the argument; it is not opcode-dependent.
3. This allows for:
   1. 64 basic instructions.
   2. Up to 4 bytes for arguments.

This format has been created to minimize the size of the generated code.

```rust
const ARG_LEN_1 = 0x00;         // 1 byte for argument
const ARG_LEN_2 = 0x01 << 6;    // 2 byte for argument
const ARG_LEN_3 = 0x02 << 6;    // 3 byte for argument
const ARG_LEN_4 = 0x03 << 6;    // 4 byte for argument

#[repr(C)]
pub struct FragmentIR {
   pub node_count: u16,
   pub ext_store_count: u16,
   pub own_store_count: u16,
   pub resources: &'static ResourceTable,
   pub dependencies: &'static [FragmentIR],
   pub events_handlers: &'static [EventHandler], 
   pub derived_handlers: &'static [Fn()],
   pub ops: &'static [u8],
}

// instruction set

const OP_VERSION:       u8 = 0; // version of the IR format

const OP_CONST:         u8 = 1; // create a const store
const OP_READABLE:      u8 = 2; // create a readable store
const OP_DERIVED:       u8 = 3; // create a derived store
const OP_WRITABLE:      u8 = 4; // create a writable store

const OP_BEGIN:         u8 = 5; // begin a new fragment (create handle)
const OP_ARG_PASS:      u8 = 6; // pass through an existing store to the current fragment
const OP_ARG_CONST:     u8 = 7; // create a const store and use it as an external store for the current fragment
const OP_ARG_READABLE:  u8 = 8; // create a readable store and use it as an external store for the current fragment
const OP_ARG_DERIVED:   u8 = 9; // create a derived store and use it as an external store for the current fragment
const OP_ARG_WRITABLE:  u8 = 10; // create a writable store and use it as an external store for the current fragment
const OP_ARG_EH:        u8 = 11; // event handler

const OP_END:           u8 = 62; // end of the current fragment
```

Generated code example:

```rust
/// This is a conceptual example for readability.
/// The macro would emit a constant binary BLOB for `ops`.
///
/// stores:
///   0 - external, the label parameter
///   1 - internal, writable, counter
///   2 - internal, derived, "${label}: ${count}"
pub static COUNTER_DESC = FragmentIR {

    resources: &[
        Const(0),
        Const("Click me"),
        Const([Padding(16), Border(Red, 1)]),
        Const([TextSmall()])
    ],

    dependencies: &[
        COLUMN_DESC,
        BUTTON_DESC,
        TEXT_DESC,
    ],

    events_handlers: &[
        EhDesc(&[1], counter_increment_fn), // uses store 1 (count)
    ],

    derived_handlers: &[
        DeriveDesc(&[0, 1], label_derive_fn), // uses stores 0 (label) and 1 (count)
    ],

    // inline ops
    ops: &[
        op!(OP_VERSION, 1), // version of the IR format

        op!(OP_WRITABLE, 0), // Creates a writable store by copying the value from Const(0) in resources
        op!(OP_DERIVED, 0),  // Creates a derived store from the first description in derived_handlers

        op!(OP_BEGIN, 0),    // COLUMN_DESC in dependencies
        op!(OP_ARG_CONST, 2), // Const([Padding(16), Border(Red, 1)]) in resources

        op!(OP_BEGIN, 1),    // BUTTON_DESC in dependencies
        op!(OP_ARG_EH, 0),   // first of the events_handlers

        op!(OP_BEGIN, 2),    // TEXT_DESC in dependencies
        op!(OP_ARG_CONST, 1), // Const("Click me") in resources
        op!(OP_END, 0),      // text

        op!(OP_END, 0),      // button

        op!(OP_BEGIN, 2),    // TEXT_DESC in dependencies
        op!(OP_ARG_DERIVED, 2), // content of the derived store at store index 2
        op!(OP_END, 0),      // text

        op!(OP_END, 0),      // column
    ],
};
```