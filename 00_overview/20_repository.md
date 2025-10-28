# Repository Structure

```text
/doc/                        # design docs, living specs

/crates/frel/                # Public API surface

/crates/frel-core            # Shared by the compiler and the runtime, must keep it macro-safe
/crates/frel-core/types      # Core types shared across crates (DIP, Color, Instruction enums, errors)
/crates/frel-core/fir        # FIR data structures + opcodes + tiny encoder/decoder helpers

/crates/frel-compiler        # Fragment Compiler (proc-macro) → emits FIR blobs
/crates/frel-compiler/dsl    # DSL surface (token types, pest grammar) for tests/tools only

/crates/frel-encoding        # Event/Patch binary encodings shared by renderer<->adapter

/crates/frel-runtime/        # 
/crates/frel-runtime/linker  # FIR -> fragment linker (creates handles, stores, subscriptions)
/crates/frel-runtime/store   # Reactive store implementation
/crates/frel-runtime/render  # Instruction applier + layout engine + patch generation

/crates/frel-adapter         # Adapter crates
/crates/frel-adapter/frel-adapter-browser  # Browser adapter PoC (inline mode first, worker later)

/crates/frel-lib             # Higher level fragments (tables, trees, editors etc.)

/tests/                      # Integration tests 

/utils/                      # shared utilities (can be split later)
```

## Published crates

```text
frel                   # Public API surface 
frel-core              # Core types shared across crates
frel-compiler          # proc-macro for compiling DSL to FIR
frel-encoding          # Separated so adapters can depend on it without pulling in the runtime
frel-runtime           # General, adapter-independent runtime
frel-adapter-browser   # Browser adapter
frel-lib               # Higher level fragments
```
