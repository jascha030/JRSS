

## Recommended file

Save this as:

```text
.opencode/instructions/comment-policy.md
```

## Ready-to-paste policy

# Comment Policy for Tauri 2 + SvelteKit 2 + Rust Project

## Priority Rule

If a comment can be eliminated by better naming, extraction, typing, or structure, eliminate the comment.

Comments must not compensate for unclear code.

## Core Principle

Prefer clarity in code over explanatory comments.

Code should communicate intent through:

* clear names
* explicit types
* small focused functions
* well-defined module boundaries
* separation of concerns

Single-line explanatory comments are generally a code smell.

**Exception:** test files are exempt from this restriction and may use comments freely when they improve clarity.

## Default Behavior for Generated Code

When writing or modifying code:

* do not add comments by default
* first improve naming and structure
* use doc comments for public APIs
* use inline comments only when they are required for correctness, safety, tooling, or non-obvious constraints
* in tests, comments are allowed and encouraged when they explain intent, edge cases, regressions, or setup

## Allowed Comments

### Public API documentation

Use doc comments for exported or public APIs.

This includes:

* exported TypeScript functions, types, interfaces, classes, and constants when the intent is not trivially obvious
* Svelte component APIs when they are reused or externally consumed
* public Rust items
* public `#[tauri::command]` functions
* Rust modules that benefit from module-level documentation

Preferred forms:

* TypeScript / JavaScript / Svelte: `/** ... */`
* Rust item docs: `///`
* Rust module docs: `//!`

### Tooling and static-analysis comments

Allowed when required by the toolchain, linter, compiler, or type system.

Examples include:

* `@ts-expect-error`
* ESLint suppression comments
* Rust `SAFETY:` comments for `unsafe` blocks
* narrowly scoped suppression comments with a concrete reason

These comments must explain **why** the exception is necessary.

### Non-obvious constraints

Allowed when the code is correct but the reason is not inferable from the code alone.

Examples include:

* protocol quirks
* platform-specific behavior
* interoperability constraints
* security boundaries
* performance-sensitive tradeoffs
* invariants that must remain true across layers

Keep these comments short and factual.

### Complex algorithms

Allowed only when the algorithm or formula is genuinely non-obvious and cannot be made clear enough through naming and decomposition alone.

Prefer doc comments above the function rather than scattered inline comments.

### Type-definition and boundary documentation

Allowed for files that define shared contracts or boundaries, such as:

* IPC command definitions
* serialization formats
* application configuration structures
* frontend-backend type mappings

## Prohibited Comments

Do not add comments that merely restate the code.

### Avoid single-line explanatory comments

Bad:

```ts
const result = data.filter(x => x.active); // Get active items
```

Good:

```ts
const activeItems = data.filter((item) => item.active);
```

### Avoid narrating obvious steps

Bad:

```rust
// Initialize counter
let mut count = 0;

// Increment counter
count += 1;
```

Good:

```rust
let mut count = 0;
count += 1;
```

### Avoid status comments without a concrete reference

Do not write vague comments like:

* `TODO: fix this later`
* `HACK: ugly but works`
* `BUG: broken on macOS`

If a note is necessary, reference a real issue, ticket, or upstream limitation.

Preferred style:

```ts
// See issue #142 for removal once upstream API typing is fixed.
```

### Avoid comments that restate control flow

Bad:

```ts
// Check if user is authenticated
if (isAuthenticated) {
    // Redirect to dashboard
    navigate('/dashboard');
}
```

Good:

```ts
if (isAuthenticated) {
    navigate('/dashboard');
}
```

### Avoid comments that restate type information

Bad:

```rust
// Create vector of integers
let mut numbers: Vec<i32> = Vec::new();
```

Good:

```rust
let mut numbers = Vec::new();
```

## Preferred Replacements for Comments

Use these techniques instead of explanatory comments:

### Use semantic names

Prefer names that encode intent.

Bad:

```ts
const x = users.filter((u) => u.permissions.includes('admin'));
```

Good:

```ts
const adminUsers = users.filter((user) => hasAdminPermission(user));
```

### Extract named functions

If a block needs comments to explain steps, extract those steps into clearly named functions.

Bad:

```ts
async function handleCommand(command: UserCommand): Promise<void> {
    // validate
    // execute
    // persist
}
```

Good:

```ts
async function handleCommand(command: UserCommand): Promise<void> {
    const validatedCommand = await validateCommandPermissions(command);
    const executionResult = await executeCommand(validatedCommand);
    await persistCommandHistory(executionResult);
}
```

### Use explicit types

Prefer strong typing over prose explanations.

### Separate responsibilities

Break up functions or modules that require commentary to be understandable.

## Test Files: Relaxed Standard

Test files are exempt from the no-single-line-comment rule.

In tests, comments are welcome when they explain:

* the purpose of the test
* a regression being protected
* an edge case
* unusual setup
* the reason a fixture or mock exists

Comments in tests should still be useful and specific.

## Tauri-Specific Rules

### `#[tauri::command]` functions

All public Tauri command functions should have doc comments.

Document:

* what the command does
* important arguments
* important return behavior
* error behavior when relevant

### Frontend-backend boundaries

Prefer clear documentation on:

* command names
* payload shapes
* return types
* serialization expectations
* side effects

### IPC and persistence boundaries

Use documentation comments where behavior crosses boundaries that are not obvious from local code alone, such as:

* frontend to Rust IPC
* Rust to filesystem
* Rust to database
* config loading and persistence

## Rust-Specific Rules

### `unsafe` blocks

Every `unsafe` block must include a `SAFETY:` comment explaining why the block is sound.

Example:

```rust
// SAFETY: The buffer is initialized, the source and destination do not overlap,
// and both pointers are valid for `len` bytes within this scope.
unsafe {
    ptr::copy_nonoverlapping(src, dest, len);
}
```

### Public items

Use `///` doc comments for public items whose purpose or contract is not trivial.

### Module docs

Use `//!` for modules that define a subsystem, boundary, or shared domain concept.

## TypeScript and Svelte-Specific Rules

### Public exported APIs

Use JSDoc or TSDoc for exported APIs when external callers rely on them.

### Suppression comments

When using:

* `@ts-expect-error`
* ESLint disable comments
* framework-specific suppressions

always include the concrete reason.

Bad:

```ts
// @ts-expect-error
const result = legacyCall();
```

Good:

```ts
// @ts-expect-error - Upstream Tauri typing does not yet model this legacy command shape.
const result = legacyCall();
```

## Enforcement Rules for LLMs

When generating or editing code, follow this order:

1. make the code clearer first
2. remove comments that only restate the code
3. add doc comments for public APIs and important boundaries
4. add inline comments only for safety, tooling, constraints, or non-obvious reasoning
5. allow broader comment usage in tests
6. if unsure whether a comment is necessary, do not add it

## Review Checklist

Before keeping a comment, ask:

* does this comment explain something the code cannot express well on its own?
* can better naming remove the need for this comment?
* can function extraction remove the need for this comment?
* is this comment documenting a public contract, safety requirement, or external constraint?
* if this is a test, does the comment improve clarity of intent?

If the answer is no, remove the comment.
