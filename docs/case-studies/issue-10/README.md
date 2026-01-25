# Case Study: Issue #10 - Settings Menu Z-Index Problem

## Issue Description

The settings dropdown menu (containing Theme and Language options) was appearing **behind** other elements on the page, specifically behind the calculator card, making it impossible for users to interact with the settings.

## Screenshots

### Original Issue (Before Fix)

**Screenshot 1** (`screenshot1.png`): Shows the settings dropdown appearing to work initially but actually being rendered behind content.

**Screenshot 2** (`screenshot2.png`): Browser developer tools showing the dropdown is behind the calculator card, with the `.settings-dropdown` element highlighted.

### After Fix

**Screenshot** (`screenshot-fix-test.png`): Shows the settings dropdown correctly appearing above all other elements.

## Timeline of Events

1. **User opens settings menu** - Clicks the gear icon in the header
2. **Dropdown appears** - But is rendered behind the calculator card
3. **User cannot interact** - Theme and language options are obscured
4. **Issue reported** - User provides screenshots showing the problem

## Root Cause Analysis

### The Problem: CSS Stacking Contexts

The issue was caused by CSS **stacking context** behavior, not simply a missing z-index value.

#### Understanding Stacking Contexts

In CSS, `z-index` only works within a stacking context. A new stacking context is created when an element has:
- `position: relative/absolute/fixed` with a `z-index` value
- `opacity` less than 1
- `transform` property
- And several other properties

#### The Bug

Looking at the original CSS structure:

```css
/* Header with position: relative creates stacking context */
.header-top {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 1rem;
  margin-bottom: 0.5rem;
  position: relative;
  /* NO z-index defined! */
}

/* Settings wrapper positioned absolutely within header */
.settings-wrapper {
  position: absolute;
  right: 0;
  top: 50%;
  transform: translateY(-50%);
}

/* Dropdown has high z-index but it's trapped */
.settings-dropdown {
  position: absolute;
  top: calc(100% + 8px);
  z-index: 1000;  /* This value is ignored outside its stacking context! */
}
```

The `.settings-dropdown` had `z-index: 1000`, but this value was only relative to elements **within the same stacking context** (the `.header-top` element).

Since `.header-top` had `position: relative` but **no z-index**, it was treated as having `z-index: auto`, which doesn't create an elevated stacking context. The entire header (including the dropdown) was rendered at the same level as other content.

The `.calculator` card, being a sibling element that comes **after** the header in DOM order, naturally painted over the header's content due to paint order rules.

## Solution

### The Fix (1 line change)

Add `z-index: 10` to `.header-top`:

```css
.header-top {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 1rem;
  margin-bottom: 0.5rem;
  position: relative;
  z-index: 10;  /* Added: Elevates the entire header above siblings */
}
```

### Why This Works

By giving `.header-top` an explicit `z-index`, we:
1. Create a proper stacking context for the header
2. Elevate the entire header (including all children like the dropdown) above sibling elements
3. Allow the dropdown's `z-index: 1000` to work correctly within its parent's elevated context

### Why `z-index: 10` Instead of Higher?

- We use a moderate value (10) because we only need to be above the adjacent content
- The dropdown itself has `z-index: 1000` for any internal layering needs
- This follows the best practice of using a centralized z-index scale with reasonable values

## Research References

Based on web research, this is a common CSS issue:

1. **Z-index values are NOT global** - They only apply relative to siblings within the same stacking context
2. **Parent elements limit children** - A child element cannot appear above something outside its parent's stacking context, regardless of its z-index value
3. **Transform creates stacking context** - The `transform: translateY(-50%)` on `.settings-wrapper` also creates a stacking context, but this wasn't the main issue

## Prevention Strategies

1. **Always set z-index when using position: relative/absolute for containers that have dropdown menus**
2. **Use a centralized z-index scale** in CSS custom properties:
   ```css
   :root {
     --z-index-header: 100;
     --z-index-dropdown: 200;
     --z-index-modal: 300;
   }
   ```
3. **Test dropdown menus** with content below them during development
4. **Use browser dev tools** to inspect stacking contexts (Firefox has excellent stacking context visualization)

## Files Changed

- `web/src/index.css`: Added `z-index: 10` to `.header-top` class

## Verification

The fix was verified by:
1. Building the WASM package
2. Running the development server
3. Using Playwright to automate browser testing
4. Taking a screenshot showing the dropdown correctly overlapping the calculator card
