---
name: phase-implementation
description: "Implement relay control architecture phases using test-driven development with frequent commits. Use when adding new domain behavior, fixing defects, or refactoring production code that can be covered by automated tests."
---

# Phase Implementation Workflow

## Objective

Implement a phase specification from .github/agents/relay-input-control-rewrite-plan.md as a series of small, committed red-green-refactor cycles that maintain a stable, testable codebase with clear historical markers.

## Starting Prompt Template

When beginning a phase implementation, the user should provide:

```
I want to implement [Phase N - Brief Title] from the relay control architecture plan.

**Phase Specification:** See .github/agents/relay-input-control-rewrite-plan.md, [Phase N] section

**Instructions:**
1. Before starting, clarify the specification with me using vscode_askQuestions
2. Ask **one targeted question at a time** about areas that are unclear or ambiguous
3. Continue asking until you understand the specification well enough to proceed
4. Then show me the step-by-step implementation plan for approval

**Implementation Workflow:**
- Follow this skill for TDD, commits, and reporting
- Write failing tests first (Red), commit with type `test`
- Implement to make tests pass (Green), amend commit to type `feat`
- Refactor while keeping tests green (Refactor), amend same commit
- Follow Conventional Commits and phase-implementation skill guidelines

**Scope:**
- Host test gate required: `cargo test --lib --target x86_64-unknown-linux-gnu` must pass
- All new behavior must have tests in `tests/unit/` or `tests/behavior/`
- [Optional: List any specific gates or constraints for this phase]

**Report at End:**
Provide implementation report with:
1. Decisions Summary (key design choices)
2. Key Implementation Details (types, functions, constants)
3. Test Summary (all tests added, with ✅/❌ status)
4. Final Test Run Output
5. Phase Completion Checklist
6. Question: "Is [Phase N] ready to mark [DONE]?"
```

## Core Principles

### 0. Clarify Before Implementing (Agent Responsibility)

Before writing any code:
- **Read the phase specification** in .github/agents/relay-input-control-rewrite-plan.md
- **Identify unclear or ambiguous areas** (design decisions, edge cases, field names, algorithms)
- **Ask the user one targeted question at a time** using vscode_askQuestions tool
- **Continue asking until you have enough clarity** to proceed with confidence
- **Do not ask redundant questions**; if the spec is clear, move on
- **Show the user your implementation plan** (step-by-step checklist) before starting
- **Wait for user approval** before writing production code

This ensures the agent and user are aligned on the phase requirements before effort is invested.

### 1. Always Test First (Red-Green-Refactor)

Before writing any production code:
- **Write a failing test** that expresses the missing behavior
- **Make the test compile first**; if the test references a new API, add a minimal compiling stub (for example, a no-op `tick_1ms()`) before evaluating red-state failure
- **Verify it fails** for the intended reason (not compile errors, fixture errors, or unrelated failures)
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

- [ ] Read the phase specification from .github/agents/relay-input-control-rewrite-plan.md
- [ ] Ask clarifying questions one at a time (if specification has ambiguous areas)
- [ ] Continue asking until specification is understood
- [ ] Show the user a step-by-step implementation plan for approval
- [ ] Wait for user approval before proceeding
- [ ] Define constants and types in appropriate files
- [ ] Place tests in `tests/unit/` (module-focused) or `tests/behavior/` (grouped behaviors)
- [ ] Write table-driven **failing tests** (Red)
- [ ] Ensure tests compile (add minimal stubs for new APIs when needed)
- [ ] Run tests, confirm behavioral failures for intended reasons
- [ ] **Commit failing tests** with conventional message type `test`
- [ ] Implement production code (Green)
- [ ] Run focused and full test suite, confirm passes
- [ ] Amend commit, change type from `test` to `feat`/`fix`
- [ ] Refactor names, duplication, structure (Refactor)
- [ ] Rerun tests, amend same commit with `--no-edit`
- [ ] Verify host gate: `cargo test --lib --target x86_64-unknown-linux-gnu`
- [ ] Add boundary and regression cases in proportion to risk
- [ ] Generate implementation report (see below)
- [ ] Ask user: "Is [Phase N] ready to mark [DONE]?"

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
- Do not treat compilation failures as red-state test failures; fix compilation (including temporary stubs) first
- Do not amend a commit that has been published or may be based on by other contributors
- Inspect staged diff before every commit
- Do not commit a red test until its failure has been observed and understood
- Do not amend until relevant tests pass

## Completion Check

- [ ] All phases up to the current one have committed red-green-refactor cycles
- [ ] Plan file shows `[DONE]` tags for completed phases
- [ ] Implementation report has been provided for this phase
- [ ] User has confirmed phase is ready for next iteration
