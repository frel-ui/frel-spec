# Datum

In Frel, the **conceptual unit of runtime application data** is called **datum**.

A **datum** can be very simple such as a standalone number or very complex such as a tree of
deeply nested data.

Conceptually (before optimization), each **datum** has these properties:

| Property                | Description                                                        |
|-------------------------|--------------------------------------------------------------------|
| **Identity**            | A unique identifier of this **datum**.                             |
| **Type**                | The Frel type (e.g., `i32`, `String`, `List<User>`, `scheme Todo`) |
| **Payload**             | The actual data content.                                           |
| **Availability**        | The availability state (Loading/Ready/Error).                      |
| **Structural Revision** | Increments when the identities of contained data change.           |
| **Carried Revision**    | Increments when changes propagate from the payload.                |
| **Derivation**          | The function that can derive the datum from its dependencies.      |
| **Subscriptions**       | Identities of dependent data.                                      |
| **Metadata**            | Validation rules, constraints, etc. (not relevant to reactivity).  |

## Core tenets

1. Datum identity is unique, there cannot be two different payloads with the same identity.
2. The identity of a datum is **type-dependent**.
3. The reactivity behavior of a datum is **type-dependent**.
4. Each datum serves as the single source of truth for its identity.
5. Dependencies between data are expressed through subscriptions. When datum A depends on datum B, A
   subscribes to B.