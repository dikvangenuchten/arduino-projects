## Plan: Rewrite Relay Input Control Architecture

Replace the monolithic control policy with a `no_std`, host-testable library containing per-relay mode and timing state, semantic actions, input debounce/mapping, and read-only diagnostics. Every relay owns its speed, phase, and countdown; no timing state is shared. Keep `Io22d08Api` as the only hardware boundary and make `Io22d08Controller` a thin AVR orchestrator. Follow red-green-refactor within each phase.

**Steps**

### Phase 0 - Prove the host test harness [DONE]
1. Add a library target rooted at `src/lib.rs`, using `#![no_std]`; add test-only `std` access only if a test utility actually requires it.
2. Run `cargo test --lib --target x86_64-unknown-linux-gnu`. If package-level AVR dependencies block host compilation, move AVR-only dependencies into `target_arch = "avr"` Cargo dependency sections. Do not begin domain implementation until this command passes.
3. Keep the AVR binary target test-disabled and verify the empty/minimal library still builds into the AVR firmware with `cargo build`.

### Phase 1 - Pure relay and action domain [DONE]
4. Define shared behavioral constants and fixed-size types in the library. Let the library own relay/input counts used by domain arrays; keep pin assignments and display/button hardware counts in `board.rs`.
5. Write table-driven failing tests for all relay transitions, then implement a concrete `RelayBank` and `RelayMode::{Off, Blink, On}`. Each `RelayState` owns its `speed: u16` (raw ticks in milliseconds from [1200, 800, 500, 300, 150]), and placeholder fields for blink phase and remaining countdown (deferred to Phase 2). Do not add a `RelayBank` trait or fake: the concrete bank is directly testable. Note: `speed_index` is a UI-layer abstraction introduced in Phase 4; Phase 1 works with raw speed values only.
6. Use resolved semantic actions such as `TogglePower(relay)`, `ToggleBlink(relay)`, `GlobalPower`, `GlobalBlink`, `SpeedAllUp`, `SpeedAllDown`, and `Reserved`. The mapper consumes Shift; the dispatcher never needs an `ActionContext`.
7. Implement and test these contracts: power toggle maps Off->On and On/Blink->Off; blink toggle maps Off/On->Blink and Blink->On; global power sets included relays On only when all included relays are Off, otherwise Off; global blink forces included relays to Blink without changing their stored speeds.
8. Speed actions move each relay's speed toward shorter durations (SpeedAllUp) or longer durations (SpeedAllDown), stopping at the nearest defined level [1200, 800, 500, 300, 150], clamping each relay independently. Examples: 300→150 (up), 301→300 (up, next level), 301→500 (down, next level), 499→500 (down). Tests must begin with deliberately different relay speeds to prove the action does not rely on shared speed state.
9. Add a compile-time global inclusion mask and test with a non-default mask that excluded relays remain untouched by global mode actions. Speed actions target all relays, independent of this mode-action mask.

### Phase 2 - Per-relay blink engine
10. Write failing synthetic-tick tests, then implement independent per-relay timing using half-cycle values `[1200, 800, 500, 300, 150]` ms. Every relay boots with speed index `2`; there is no controller-wide current speed.
11. Entering Blink starts that relay On with a full half-cycle selected from its stored speed. Leaving Blink immediately produces the commanded steady output. Re-entering Blink uses that relay's current stored speed and starts a new independent phase origin.
12. When `SpeedAllUp` or `SpeedAllDown` changes stored indices, every active relay finishes its current half-cycle and loads its own new duration at its next phase transition. Relays at different countdown offsets remain offset.
13. Route half-cycle selection through one small per-relay function or timing-policy enum rather than indexing the table directly throughout the engine. Initially it implements only fixed speed levels; a future `Random` per-relay policy can choose a new duration at each phase transition with an injected PRNG source. Random blinking and PRNG implementation are out of current scope.
14. State explicitly that this replaces the existing asymmetric `RelayPattern::Blink { on_ticks, off_ticks }` API. Fixed mode remains 50/50, but relay speeds may differ.

### Phase 3 - Debounce and input mapping
15. Separate debouncing from mapping. Debounce all eight external inputs and four built-in buttons after active-low normalization, accepting a change after 10 consecutive 1 ms samples.
16. Test press edge, held input, release/repress, bounce reset, simultaneous actions, and Shift/action acceptance on the same tick. Commit one stable snapshot first, then resolve Shift before emitting action press edges, so Shift and action accepted together are shifted.
17. Implement the default external mapping in one static configuration: `I0-I3` relay keys 1-4, `I4` Shift, `I5` Global, `I6` Speed, `I7` Reserved. Without Shift, relay keys target relays 1-4 and emit power toggles; with Shift, they target relays 5-8 and emit blink toggles. Shift also selects global blink and `SpeedAllDown`; unshifted Speed emits `SpeedAllUp`.
18. Keep Shift mode represented by a small enum/constant with momentary as the only required behavior; a future latched variant may be added later, but do not implement unrequested latch state now.

### Phase 4 - Read-only diagnostics
19. Build diagnostics from immutable snapshots/data only; no diagnostic API receives mutable relay/controller state.
20. Debounce built-in buttons using the same 10 ms rule. Direct view selection is `B0` Speed, `B1` Inputs, `B2` Relay, `B3` Reserved.
21. Speed view computes the mode of all eight stored per-relay periods and displays it as `1200`, `0800`, `0500`, `0300`, or `0150`. If multiple periods have equal highest frequency, choose the largest millisecond value. Test uniform, unique-mode, two-way tie, and all-distinct cases.
22. Inputs view displays four raw logical input states as `0/1`; repeated `B1` presses toggle pages `I0-I3` and `I4-I7`.
23. Relay view displays four modes as `0=Off`, `1=Blink`, `2=On`; repeated `B2` presses toggle pages relays 1-4 and 5-8. Test selection, repeated-press paging, and reserved-button no-op behavior.

### Phase 5 - AVR integration
24. Refactor `Io22d08Controller` into the orchestrator: read normalized hardware snapshots through `Io22d08Api`, debounce/map external inputs, dispatch all emitted semantic actions, tick each relay's blink engine once per consumed 1 ms timer tick, flush changed relay outputs, update the selected diagnostic display, and call the board tick.
25. Remove the example counter and direct button-to-relay policy from `main.rs`; retain peripheral setup, interrupt enabling, controller construction, and the pending-tick loop.
26. Boot with all relays Off and each relay's speed index set to `2`; add no EEPROM persistence.
27. Update README controls, mode transitions, per-relay speed ownership, half-cycle timing, diagnostics, and boot defaults. Note that the current Speed key changes all stored relay speeds even though the model permits them to diverge.

### Phase 6 - Regression and refactor
28. Run all host tests after each phase and after cleanup. Refactor only while tests remain green.
29. Build the AVR firmware and perform the on-device checks below.

**Relevant files**
- `/home/dik/arduino-projects/ja-2026/Cargo.toml` - add library target and, if the host spike proves necessary, AVR-target dependency gating.
- `/home/dik/arduino-projects/ja-2026/src/lib.rs` - `no_std` pure crate root.
- `/home/dik/arduino-projects/ja-2026/src/config.rs` - behavior mapping, counts, speed-level table/default, debounce threshold, inclusion mask, Shift mode.
- `/home/dik/arduino-projects/ja-2026/src/relay.rs` - `RelayMode`, concrete `RelayBank`, per-relay speed/profile, phase, countdown, and next-duration selection boundary.
- `/home/dik/arduino-projects/ja-2026/src/action.rs` - semantic actions and pure mode/all-relay-speed transition functions.
- `/home/dik/arduino-projects/ja-2026/src/input.rs` - stable-snapshot debouncer, press edges, Shift-aware static mapping.
- `/home/dik/arduino-projects/ja-2026/src/debug.rs` - immutable diagnostic state/view formatting, modal-speed calculation, and page selection.
- `/home/dik/arduino-projects/ja-2026/src/wrapper.rs` - retain `Io22d08Controller`, but reduce it to hardware/core orchestration.
- `/home/dik/arduino-projects/ja-2026/src/board.rs` - retain `Io22d08Api`, active-low normalization, pin wiring, relay/display primitives; modify only shared-count imports or integration details.
- `/home/dik/arduino-projects/ja-2026/src/main.rs` - remove demo policy and run the integrated controller.
- `/home/dik/arduino-projects/ja-2026/README.md` - document the final behavior contract.

**Verification**
1. Host gate: `cargo test --lib --target x86_64-unknown-linux-gnu` passes before and after each pure-domain phase.
2. AVR gate: `cargo build` succeeds after the harness spike and final integration.
3. Unit tests cover every mode/action transition, mixed global states and exclusions, independently clamped speed indices, all-relay speed actions starting from divergent indices, active-phase speed changes, independent blink offsets and periods, full debounce sequences, simultaneous presses, same-tick Shift chords, every mapping slot, modal-speed tie-breaking, and diagnostic pages.
4. On device, configure or initialize relays with different speeds and verify independent 1200/800/500/300/150 ms half-cycles and offsets; verify an all-relay speed change preserves current phases and independently clamps boundary values.
5. On device, verify one action per held press, same-tick and prior-held Shift behavior, all eight relay mappings, mixed global behavior, B0/B1/B2 selection, B1/B2 paging, B3 no-op, modal speed display, boot defaults, and that diagnostics never change relay modes or speeds.

**Decisions**
- Full control-layer rewrite; backward compatibility with per-relay asymmetric On/Off timing is excluded.
- Every relay independently owns speed index, timing policy, phase, and countdown. There is no shared current speed or shared blink phase.
- The current Speed key changes all eight stored speed indices; each is clamped independently. Global Blink preserves per-relay speeds.
- Fixed speed values are per half-cycle, not full-cycle periods. Active phases finish before adopting changed speed indices.
- Future random blinking is per relay and should be added through the localized next-duration policy with an injected PRNG; randomness itself is excluded from this implementation.
- Speed diagnostics display the most common stored period; ties select the largest millisecond value.
- Eight external inputs control actions; four built-in buttons are diagnostics-only.
- Default mapping: I0-I3 relay keys, I4 Shift, I5 Global, I6 Speed, I7 Reserved.
- Shift is momentary; a Shift and action accepted in the same debounced snapshot count as shifted.
- Global inclusion is compile-time configuration for mode actions.
- Reserved external input and B3 are no-ops.
- EEPROM persistence and latched Shift behavior are deliberately excluded.
