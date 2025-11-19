# Goals

**Main objective:** Build a modern, reactive, truly platform-independent UI library that
emphasizes clarity and expressiveness — a concise DSL where code reads like intent rather 
than syntax.

**Agnostic**: The library aims to be programming language-agnostic and platform-agnostic.
This is not 100% possible at the moment, but everything should be designed so that the
language- and/or platform-specific parts are clearly separated and easy to implement.

>
> [!NOTE]
> First implementation language will be Rust with WebAssembly target. Examples in the
> specification use Rust.
>

1. **Declarative Composition:**
    Enable developers to define application structure and behavior through a concise, clean, 
    declarative syntax. The library should make it easy to express reactive and reusable UI 
    elements without focusing on platform or implementation details.
2. **Compact Runtime Footprint:**
    Ensure generated code remains small and efficient. The library should avoid unnecessary
    monomorphization and strive for minimal runtime overhead even with many components.
3. **Platform Independence:**
    Support multiple platforms through a clean separation between the declarative layer and 
    platform-specific rendering backends. Core logic and layout behavior must remain portable.
4. **Deterministic Rendering and Layout:**
    Provide a layout and rendering pipeline that behaves predictably across backends. 
    Platform-dependent operations should be minimal, deterministic, and isolated from reactive logic.

# Conceptual Model

The library uses **fragments** to transform a tree of reactive **stores** into a **scene** 
which is then rendered into target backend **primitives**.

```text
  Application         State to UI             Scene          Platform dependent    
state (reactive)     transformation                             UI elements

  ┌────────┐          ┌───────────┐         ┌───────┐         ┌────────────┐
  │ Stores │    →     │ Fragments │    →    │ Nodes │    →    │ Primitives │
  └────────┘          └───────────┘         └───────┘         └────────────┘

├─────────────────────────────────┤     ├───────────────────────────────────┤
    Defined in source code                        Built during runtime
```

* [**Store**](../30_runtime/stores.md)
  A backend-neutral state container exposing reactive notifications to fragments and other stores.

* **Fragment Definition**
  A declarative definition authored in the Frel DSL. Definitions are compiled into Fragment IR.

* **Fragment**
  A runtime composition derived from a fragment definition. It consists of a handle (lifetime), stores (data),
  and subscriptions (reactivity). A fragment can be headless (logic-only) or produce renderable content.

* [**Scene**](../../10_language/70_blueprints/15_scene_model.md)
  A collection of platform-independent UI nodes. Organizes all visual content into **channels**,
  **viewports**, and **layers**.

# Architecture

The library provides a structured way to define, compile, and execute declarative 
programs that can target multiple backends — such as interactive UIs, PDF documents, or image exports.

It separates **compile-time** and **runtime** responsibilities, making the system 
both modular and extensible.

## Requirements

**Endianness requirement:** Only little-endian targets are supported.
Big-endian targets are unsupported and may produce incorrect results.

## High-Level Flow

```text
DSL Source
   ↓
[Fragment Compiler] → Fragment IR (FIR)
   ↓
[Fragment Linker] → Linked runtime tree of fragments (handles + stores)
   ↓
[Fragment Renderer] → Output (UI / PDF / PNG / etc.)
```

## Stages and Responsibilities

| Stage                  | Component                                | Responsibility                                                                                                                                           |
|------------------------|------------------------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------|
| **1. Source Code**     | **Frel DSL**                             | **Defines** declarative fragment definitions written by the developer.                                                                                   |
| **2. Compile Stage**   | **Fragment Compiler** (procedural macro) | **Transforms** the DSL into backend-neutral **Fragment IR (FIR)**; performs validation, symbol resolution, and metadata embedding.                       |
| **3. Link Stage**      | **Fragment Linker** (runtime)            | **Instantiates and links** fragments (creates handles), stores, and handlers into a connected runtime tree.                                              |
| **4. Execution Stage** | **Fragment Renderer** (runtime)          | **Applies** the linked runtime tree to the target backend. After the initial build, updates are propagated incrementally through reactive notifications. |

## Core Concepts

* [**Frel**](../10_language/00_language_overview.md)
  A declarative language for describing user interfaces.

* [**Fragment Compiler (FC)**](../20_compile/compiler.md)
  The procedural macro that compiles the Frel DSL into **Fragment IR (FIR)**.

* [**Fragment Intermediate Representation (FIR)**](../20_compile/fir.md)
  A backend-agnostic program that describes how a fragment should be built, connected, and parameterized for later linking and rendering.

* [**Fragment Linker (FL)**](../30_runtime/linker.md)
  The runtime component that takes FIR definitions, instantiates fragments (creates handles), and links them into a connected runtime tree with stores, actions, and derived values.

* [**Fragment Renderer (FR)**](../40_render/renderer.md)
  The runtime component that applies the linked runtime tree to a specific backend. It performs the initial build and then reacts to fine-grained notifications.

* [**Fragment Handle**](../30_runtime/handles.md)
  A lightweight runtime handle owning the lifetime and cleanup of a fragment’s resources (stores, subscriptions, children).
