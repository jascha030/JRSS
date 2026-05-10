# Comment Policy

## Core Rule

If information can be expressed through naming, types, structure, or module boundaries, do not use a comment. Comments must not compensate for unclear code.

Single-line explanatory comments are a code smell. Test files are exempt.

## What to Do

- Do not add comments by default
- Improve naming and structure first
- Use doc comments for public APIs and real boundaries
- Use inline comments only for correctness, safety, tooling, or non-obvious constraints
- In tests, comments are allowed and encouraged for intent, edge cases, regressions, or setup

## Allowed Comments

| Kind            | When                                                                                                                                             |
| --------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| Public API docs | Exported TS/Rust APIs, `#[tauri::command]`, Svelte components consumed externally, modules defining subsystems. Use `/** … */`, `///`, or `//!`. |
| Tooling         | `@ts-expect-error`, ESLint suppressions, Rust `SAFETY:` — always explain **why**.                                                                |
| Constraints     | Protocol quirks, platform behavior, security boundaries, performance tradeoffs, cross-layer invariants. Keep them short.                         |
| Algorithms      | Only when genuinely non-obvious and naming/decomposition is insufficient. Prefer doc comments above the function.                                |
| Boundaries      | IPC commands, serialization formats, config structures, frontend-backend type mappings.                                                          |

## Prohibited Comments

- Comments that merely restate the code (`// Get active items`)
- Narrating obvious steps (`// Initialize counter`)
- Vague status notes without a concrete reference (`TODO: fix this later`, `HACK: ugly but works`)
- Restating control flow or obvious type information
- Decorative/structural dividers (`// ----------`, `/* === Query === */`)

If code needs visual section labels, restructure the file instead.

## Language-Specific

- **Rust `unsafe`**: every block needs a `SAFETY:` comment explaining soundness.
- **Rust public items**: use `///` when purpose is non-trivial.
- **TypeScript/Svelte public exports**: use JSDoc/TSDoc when external callers rely on them.
- **Suppressions** (`@ts-expect-error`, ESLint): always include the concrete reason.

## Enforcement Order

1. Make the code clearer first
2. Remove comments that only restate the code
3. Remove decorative or structural comments
4. Add doc comments for public APIs and important boundaries
5. Add inline comments only for safety, tooling, constraints, or non-obvious reasoning
6. Allow broader comment usage in tests
7. If unsure whether a comment is necessary, do not add it

## Review Checklist

Before keeping a comment, ask:

- Does it explain something the code cannot express on its own?
- Can better naming, function extraction, or file structure remove the need?
- Is it documenting a public contract, safety requirement, or external constraint?
- If this is a test, does it improve clarity of intent?

If the answer is no, remove the comment.
