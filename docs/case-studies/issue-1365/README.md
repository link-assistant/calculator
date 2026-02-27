# Case Study: Issue #1365 - Interpretation Switching Does Not Work and Inconsistent Styling

**Repository**: [link-assistant/hive-mind#1365](https://github.com/link-assistant/hive-mind/issues/1365)
**Fix PR**: [link-assistant/calculator#86](https://github.com/link-assistant/calculator/pull/86)

---

## Summary

When a user entered an expression with multiple possible interpretations (e.g., `1 + 2 * 3`), the calculator correctly detected and displayed both interpretations as clickable buttons. However:

1. **Functional bug**: Clicking an alternative interpretation button only changed the visual highlight (which button appeared "selected") but did NOT re-run the calculation. The result shown to the user remained unchanged regardless of which interpretation was clicked.

2. **Style inconsistency**: The alternative interpretation buttons (`lino-alt-button`) had visually different styling from the single-interpretation display (`lino-value`): different font, different text color in dark mode (defaulted to browser button text color instead of `var(--text)`), and tighter spacing.

---

## Timeline of Events

### 1. Feature Introduction (commit `5f60133d`, Feb 27 2026)

Alternative interpretation support was added as part of PR #85 / issue #84. The feature introduced:
- A new `alternative_lino` field on `CalculationResult`
- WASM-side generation of alternatives for ambiguous expressions
- Frontend UI with clickable `lino-alt-button` elements
- Initial CSS for the buttons

**The bug was introduced here.** The `onClick` handler was:
```tsx
onClick={() => setSelectedLinoIndex(idx)}
```

This only updated the `selectedLinoIndex` state (for visual selection), but never called `calculate(alt)` to actually recompute using the chosen interpretation.

### 2. Bug Reported (issue #1365)

User observed two problems:
- Clicking different interpretations did nothing to the result
- Alternative interpretation buttons looked visually inconsistent with the single-interpretation display (wrong font, wrong text color in dark mode, tighter spacing)

### 3. Fix Applied (commit `bae689d4`, PR #86)

The `onClick` handler was updated to also call `calculate(alt)`:

```tsx
onClick={() => {
  setSelectedLinoIndex(idx);
  calculate(alt);
}}
```

CSS was also updated to add `font-family`, `font-size`, and `color: var(--text)` to `.lino-alt-button`, matching `.lino-value` appearance.

---

## Root Cause Analysis

### Bug 1: Clicking interpretation does not trigger recalculation

**Root cause**: Incomplete implementation in the initial feature commit.

The `calculate` function in `App.tsx` accepts an optional `expression` parameter. When called without arguments it uses the current input field value; when called with an argument it evaluates that specific expression.

The feature author correctly implemented:
- State tracking which interpretation is visually selected (`selectedLinoIndex`)
- Rendering a highlight on the selected button

But **forgot** to also trigger actual recalculation by calling `calculate(alt)` — only the visual state was updated.

**Code trace** (before fix, `web/src/App.tsx`):
```tsx
// Line 153: state for visual selection
const [selectedLinoIndex, setSelectedLinoIndex] = useState(0);

// Line 456: onClick only updates visual state, no recalculation
onClick={() => setSelectedLinoIndex(idx)}
```

**After fix:**
```tsx
onClick={() => {
  setSelectedLinoIndex(idx);   // visual selection
  calculate(alt);               // actual recalculation
}}
```

### Bug 2: Inconsistent styling between single and multi-interpretation displays

**Root cause**: When implementing the alternative interpretation UI, the CSS for `.lino-alt-button` was written independently without referencing the existing `.lino-value` styles.

**Before fix** — `.lino-alt-button` lacked:
- `font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace` (inherited browser button font)
- `font-size: 0.75rem` (was using browser default button font size)
- `color: var(--text)` (defaulted to browser button text color — black in dark mode)
- Adequate gap between items (only `0.5rem` vs `0.75rem`/`1rem` for `.lino-value`)

**After fix** — `.lino-alt-button` gained explicit font, size, color, and responsive gap matching `.lino-value`.

---

## Code Changes

### `web/src/App.tsx`
| Before | After |
|--------|-------|
| `onClick={() => setSelectedLinoIndex(idx)}` | `onClick={() => { setSelectedLinoIndex(idx); calculate(alt); }}` |

### `web/src/index.css`
| Property | Before | After |
|----------|--------|-------|
| `.lino-alternatives` gap | `0.5rem` | `0.75rem` (mobile), `1rem` (desktop ≥640px) |
| `.lino-alt-button` font-family | (browser default) | `'SF Mono', 'Fira Code', 'Consolas', monospace` |
| `.lino-alt-button` font-size | (browser default) | `0.75rem` (mobile), `0.875rem` (desktop) |
| `.lino-alt-button` color | (browser default) | `var(--text)` |
| `.lino-alt-button .lino-colored` color | (inherited) | `var(--text)` |

---

## Impact Analysis

### Affected users
All users entering ambiguous arithmetic expressions (e.g., mixed `+` and `*` without explicit parentheses) who tried to switch between interpretations. The UI falsely suggested the feature worked by showing a visual selection change, while the displayed result never changed.

### Severity
**Medium-High** — The feature appeared to work (visual feedback was present) but silently produced incorrect behavior (result was not updated). This is a misleading UX bug: users would believe the switching worked but get wrong answers.

### Test coverage gap
The original feature tests covered:
- That alternative buttons are rendered
- That the correct button receives the `selected` class on click

But did NOT verify that `calculate()` was called with the clicked interpretation's expression — allowing the incomplete `onClick` to pass tests.

**New tests added** (5 tests in `App.test.tsx`):
1. Renders alternative interpretation buttons when result has multiple interpretations
2. Highlights the first interpretation as selected by default
3. **Triggers recalculation** (calls `postMessage` with selected interpretation) when clicking
4. Updates the visual selection highlight on click
5. Shows single `lino-value` (not buttons) when there is only one interpretation

---

## Proposed Solutions (Applied)

### Solution 1: Fix the onClick handler ✅

Minimal change — add `calculate(alt)` call alongside `setSelectedLinoIndex(idx)`.

**Pros**: Minimal change, follows existing patterns
**Cons**: None

### Solution 2: Unify styling ✅

Explicitly set `font-family`, `font-size`, and `color` on `.lino-alt-button` to match `.lino-value`.

**Pros**: Consistent UX, proper dark mode support
**Cons**: None

### Solution 3 (Alternative, not applied): Derive result from WASM state

Instead of calling `calculate(alt)` (which posts a message to the WASM worker), the app could derive the result for each interpretation from cached WASM computation results. This would avoid redundant re-calculation.

**Pros**: No extra WASM evaluation needed
**Cons**: Would require significant state management changes; the expressions are already short and fast to evaluate, so the overhead of `calculate(alt)` is negligible

---

## Relevant Libraries and References

- **React state hooks** (`useState`, `useCallback`): The fix correctly uses the existing `calculate` callback already defined in the component — no new state or effects needed.
- **CSS custom properties (`var(--text)`, `var(--surface)`)**: The project already uses a design-token based theming system. The fix correctly uses existing tokens instead of hardcoding color values.
- **Web Workers**: The `calculate` function posts a `{ type: 'calculate', expression }` message to a Web Worker that runs the WASM calculator. Calling `calculate(alt)` with the selected interpretation string correctly re-evaluates via the same code path used for the main expression.

---

## Lessons Learned

1. **Test the behavior, not just the UI**: Tests that only check CSS classes (`selected`) can miss functional bugs where the correct function is never called.

2. **When building clickable UI elements that change computed state, always verify the computation is triggered**: It's easy to wire up visual state changes and forget the data/computation update.

3. **Design consistency**: New UI components should reference existing similar components' styles rather than starting from scratch to avoid visual inconsistencies.

4. **Test mocks must match actual hook interfaces**: The test failure (CI failure in `bae689d4`) was partly due to `wasLoadedFromUrl` missing from the `useUrlExpression` mock — a reminder to keep mocks in sync with evolving hook signatures.
