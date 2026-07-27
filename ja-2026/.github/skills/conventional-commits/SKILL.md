---
name: conventional-commits
description: "Create Conventional Commit messages and preserve one local commit per red-green-refactor cycle. Use when committing tested features, fixes, refactors, documentation, build changes, or failing tests during TDD."
---

# Conventional Commits

## Objective

Produce a readable history in which each completed behavior is one conventional commit, developed through a committed red-green-refactor cycle.

## Message Format

Use:

```text
<type>(<scope>): <imperative summary>
```

Omit the scope only when no concise domain or subsystem name is useful. Keep the subject lowercase after the colon, imperative, specific, and free of a trailing period.

Use these types:

- `feat`: add or change user-visible behavior.
- `fix`: correct defective behavior.
- `test`: add or correct tests without changing production behavior.
- `refactor`: restructure production code without changing behavior.
- `docs`: change documentation only.
- `build`: change dependencies, build scripts, targets, or toolchain configuration.
- `ci`: change continuous-integration configuration.
- `chore`: perform necessary maintenance not covered by another type.

Add a body when the reason, behavioral contract, or hardware constraint is not clear from the subject. Use `BREAKING CHANGE:` in the footer for an incompatible public contract.

## Red-Green-Refactor Commit Cycle

Keep each cycle narrow enough to represent one behavior.

1. **Red:** Add the smallest test that expresses the missing behavior, ensure it compiles (add minimal stubs for new APIs if needed), run it, and confirm it fails for the intended behavioral reason. Commit the failing test:

   ```text
   test(relay): specify independent blink timing
   ```

2. **Green:** Implement the minimum code needed, run the focused test and relevant suite, then amend the red commit. Change the message so the resulting commit describes the delivered behavior:

   ```text
   feat(relay): support independent blink timing
   ```

   For a defect, use `fix` instead of `feat`.

3. **Refactor:** Improve the same implementation without changing behavior, rerun the focused and relevant tests, then amend the same commit with `--no-edit`. Change the message only if it no longer describes the final result accurately.

The branch history should contain one final commit for the completed cycle, not separate red, green, and refactor commits.

## Safety Rules

- Amend only the current cycle's local, unpushed commit.
- Never amend a commit that has been published or may be based on by another contributor. Use a new conventional commit in that case.
- Inspect the staged diff before every commit or amend; exclude unrelated user changes.
- Do not amend when the new work belongs to a different behavior or scope. Start another red-green-refactor cycle instead.
- Do not commit a red test until its failure has been observed and understood.
- Do not amend green or refactor work until the relevant tests pass.
- Never bypass hooks or validation solely to create a commit.

## Examples

```text
test(input): specify same-tick shift chords
feat(input): resolve shift from stable input snapshots

fix(blink): preserve active phase after speed changes
refactor(action): separate mapping from dispatch
docs(readme): document diagnostic button mapping
build(cargo): add host-testable library target
```

## Completion Check

- The final subject follows Conventional Commits and describes the completed behavior.
- The failing test was committed before production implementation.
- Green and refactor work amended that same local commit.
- The final commit includes its tests and passes the relevant validation.
- No unrelated files are included.