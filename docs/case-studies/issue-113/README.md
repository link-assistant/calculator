# Case Study: Issue #113 — Switching between interpretations makes first interpretation hidden

## Summary

When an ambiguous expression like `2 + 3 * 4` is entered, the calculator correctly shows multiple LINO interpretations (e.g., `(2 + (3 * 4))` and `((2 + 3) * 4)`). However, clicking on an alternative interpretation to switch to it caused all other interpretations to disappear, leaving only the selected one visible. Users could not switch back.

## Timeline / Sequence of Events

1. User enters ambiguous expression `2 + 3 * 4` and presses Enter.
2. Worker runs `calculator.plan("2 + 3 * 4")` which returns a plan with:
   - `lino_interpretation: "(2 + (3 * 4))"`
   - `alternative_lino: ["(2 + (3 * 4))", "((2 + 3) * 4)"]`
3. Plan message arrives → UI shows both interpretation buttons.
4. Worker runs `calculator.execute("2 + 3 * 4")` → result arrives with both alternatives.
5. User clicks `((2 + 3) * 4)` button → triggers `calculate("((2 + 3) * 4)")`.
6. Worker runs `calculator.plan("((2 + 3) * 4)")` — this fully-parenthesized expression has **only one** interpretation (itself), so:
   - `alternative_lino: ["((2 + 3) * 4)"]`
7. Plan message arrives → **overwrites** `result.alternative_lino` with single-element array.
8. UI condition `alternative_lino.length > 1` is now `false` → renders single `lino-value` div instead of button list.
9. Additionally, `setSelectedLinoIndex(0)` resets the selection.
10. Result arrives → also has single-element `alternative_lino` → state is now permanently collapsed.

## Root Cause

The root cause is in `App.tsx` in the interpretation click handler and worker message handlers:

1. **Click handler** (`line ~583-586`): When switching interpretations, `calculate(alt)` sends the LINO expression (e.g., `((2 + 3) * 4)`) to the worker for a full plan→execute cycle.

2. **Plan handler** (`line ~187-211`): The plan for an already-parenthesized expression returns only itself in `alternative_lino`, which overwrites the original full alternatives list.

3. **Result handler** (`line ~212-214`): `setSelectedLinoIndex(0)` resets the index, and the result also overwrites `alternative_lino`.

The fundamental issue: **switching interpretations triggers a re-calculation that replaces the alternatives list with a new one containing only the selected interpretation**.

## Solution

Introduced a `preservedAlternativesRef` that stores the full alternatives list when the user clicks an interpretation button. When plan/result messages arrive from an interpretation-switch calculation:

- The plan handler uses the preserved list instead of the incoming single-element list.
- The result handler restores the preserved list onto the result data before setting state.
- `selectedLinoIndex` is only reset to 0 for fresh calculations, not interpretation switches.
- The ref is cleared after the result arrives, so fresh calculations work normally.

### Files Changed

- `web/src/App.tsx` — Core fix: preserve alternatives across interpretation switches
- `web/src/App.test.tsx` — Added two test cases verifying the fix

### Test Coverage

- **"should preserve all alternatives after switching interpretation (issue #113)"**: Verifies that after clicking an alternative and receiving plan + result, both interpretation buttons remain visible.
- **"should allow switching back to first interpretation after switching (issue #113)"**: Verifies round-trip switching: first → second → first, with all buttons preserved.

## Screenshots from Issue

- **Before switch**: Both `(2 + (3 * 4))` and `((2 + 3) * 4)` visible as clickable buttons.
- **After switch (bug)**: Only `((2 + 3) * 4)` visible, `(2 + (3 * 4))` disappeared.

See: `docs/issue-113-img1.png` and `docs/issue-113-img2.png` in the repository root.

## Related Patterns

This is a common UI state management issue where a user action triggers a data refresh that overwrites broader UI state. Similar patterns:

- React state derived from server data being overwritten by partial updates
- Optimistic UI updates conflicting with server responses

The ref-based approach is lightweight and avoids adding new state management complexity. An alternative approach would be to add a separate `executeOnly` worker message type that skips the plan step entirely when switching interpretations, but the ref approach is simpler and sufficient.
