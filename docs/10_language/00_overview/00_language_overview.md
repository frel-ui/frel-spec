# Frel

**Frel:** a language for building reactive, declarative, and composable user interfaces. It is
accompanied by a compiler and a runtime. Frel is a DSL (domain-specific language) that is
compiled to a host language.

## Glossary

**Host Language:** The programming language used to implement backends, commands, and complex business
logic. Examples: Rust, TypeScript, Kotlin. Each host language needs a compiler plugin that generates
appropriate code from the Frel DSL.

**Host Platform:** The UI platform that the application runs on. Examples: web browser, iOS, Android,
GTK, desktop (via Skia or similar). Each host platform needs a runtime adapter that provides the
necessary integrations.

## Core Concepts

