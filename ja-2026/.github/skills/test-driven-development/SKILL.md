---
name: test-driven-development
description: "Apply test-driven development with a red-green-refactor loop. Use when implementing behavior, fixing a defect, changing a contract, or refactoring code that can be covered by automated tests."
---

# Test-Driven Development

## Objective

Make behavior explicit before implementation and keep each change small enough that a focused test can disprove it.

## Workflow

1. State the behavior as an observable input/output contract.
2. Add the smallest focused test that fails for the intended reason.
3. Ensure the test target compiles; if a new API is referenced, add a minimal compiling stub (for example, a no-op `tick_1ms()`) before evaluating test failure semantics.
4. Run that test and confirm the failure demonstrates the missing behavior, not a compile error, broken fixture, or unrelated error.
5. Implement only enough production code to satisfy the contract.
6. Run the focused test immediately, then the relevant surrounding suite.
7. Refactor names, duplication, and structure while tests remain green.
8. Add boundary and regression cases in proportion to the change's risk.

## Constraints

- Test public behavior and stable contracts rather than private implementation details.
- Keep tests deterministic; inject time, randomness, hardware, and external I/O behind controllable boundaries.
- A "red" test is valid only when it compiles and fails on behavior/assertions; unresolved symbols or type errors are setup failures, not red-state evidence.
- Place tests in the dedicated structure: `tests/unit/*` for module-focused tests, `tests/behavior/*` for grouped behaviors, and `tests/end_to_end/` for broad scenarios.
- Keep nested test directories wired through the library test harness in `src/lib.rs` using `#[path = "..."]` modules.
- Prefer table-driven tests when several cases express one rule.
- Do not weaken assertions or delete a valid test merely to make an implementation pass.
- Do not defer all testing until after a large implementation is complete.

## Test Commands

- Full host gate (includes `tests/` hierarchy via `src/lib.rs`):
	- `cargo test --lib --target x86_64-unknown-linux-gnu`

## Completion Check

- The new test compiled, failed before the implementation for the intended behavioral reason, and passes afterward.
- Relevant existing tests still pass.
- The tests explain the changed behavior without requiring knowledge of internal control flow.
