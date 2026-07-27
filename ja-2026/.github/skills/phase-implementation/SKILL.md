---
name: phase-implementation
description: "Implement relay control architecture phases using test-driven development with frequent commits. Use when adding new domain behavior, fixing defects, or refactoring production code that can be covered by automated tests."
---

# Phase Implementation Workflow

## Objective

Deliver each phase as a series of small, committed red-green-refactor cycles that maintain a stable, testable codebase with clear historical markers.

## Core Principles

### 1. Always Test First (Red-Green-Refactor)

Before writing any production code:
- **Write a failing test** that expresses the missing behavior
- **Verify it fails** for the intended reason (not fixture errors or unrelated failures)
- **Implement minimum code** to make the test pass
- **Refactor** with tests remaining green
- **Repeat** for the next behavior slice

**See** [test-driven-development skill](../test-driven-development/SKILL.md) for detailed workflow.

### 2. Commit Often (Conventional Commits)

Commit once per completed red-green-refactor cycle:
- Commit the **failing test** first with type `test`
- **Amend** that commit when implementation is complete, changing type to `feat` (or `fix`)
- Resulting history shows one commit per behavior, not separate red/green steps
- Each commit is independently testable and runnable

**See** [conventional-commits skill](../conventional-commits/SKILL.md) for message format and cycle details.

## Implementation Checklist

- [ ] Read the phase specification and clarify ambiguities with the user
- [ ] Define constants and types in `config.rs`
- [ ] Write table-driven **failing tests** (Red)
- [ ] Run tests, confirm failures for intended reasons
- [ ] **Commit failing tests** with conventional message type `test`
- [ ] Implement production code (Green)
- [ ] Run focused and full test suite, confirm passes
- [ ] Amend commit, change type from `test` to `feat`/`fix`
- [ ] Refactor names, duplication, structure (Refactor)
- [ ] Rerun tests, amend same commit with `--no-edit`
- [ ] Verify host gate: `cargo test --lib --target x86_64-unknown-linux-gnu`
- [ ] Add boundary and regression cases in proportion to risk
- [ ] Generate implementation report (see below)

## Implementation Report Template

At the end of each phase implementation, provide a report containing:

### 1. Decisions Summary
Brief bullet list of key design choices:
- What was chosen and why (e.g., "Each relay independently owns speed, not shared state")
- Trade-offs considered
- Future extensibility points

### 2. Key Implementation Details
Concise overview of major types and functions:
- New types: RelayMode, RelayState, etc.
- New functions: apply_relay_action(), apply_speed_action(), etc.
- Important constants: SPEED_LEVELS_MS, BOOT_SPEED_MS, etc.
- Algorithm notes: direction-aware clamping, global inclusion mask, etc.

### 3. Test Summary
Table or checklist of all tests added in this phase:
```
- [✓] test_toggle_power_off_to_on
- [✓] test_toggle_power_on_to_off
- [✓] test_global_power_all_off_to_on
- [✗] test_unimplemented_behavior (if any failed)
```

### 4. Final Test Run Output
Show the relevant portion of `cargo test --lib --target x86_64-unknown-linux-gnu`:
```
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### 5. Phase Completion Checklist
Explicit yes/no confirmation:
- [ ] All failing tests were written first and committed
- [ ] All tests pass in the final suite
- [ ] Commit message follows conventional format
- [ ] Code review by user (if applicable)
- [ ] **Phase marked as [DONE] in plan** (if complete)

## Safety Rules

- Do not implement features speculatively; follow the phase specification exactly
- Do not delete a valid test to make an implementation pass
- Do not amend a commit that has been published or may be based on by other contributors
- Inspect staged diff before every commit
- Do not commit a red test until its failure has been observed and understood
- Do not amend until relevant tests pass

## Completion Check

- [ ] All phases up to the current one have committed red-green-refactor cycles
- [ ] Plan file shows `[DONE]` tags for completed phases
- [ ] Implementation report has been provided for this phase
- [ ] User has confirmed phase is ready for next iteration
