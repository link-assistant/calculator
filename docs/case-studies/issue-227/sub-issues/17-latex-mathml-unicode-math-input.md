---
title: "Accept LaTeX, MathML and Unicode math notation copied from papers"
---
Part of #227. Scientific papers are written in LaTeX; copying `$E = mc^2$`, `\frac{1}{2}mv^2` or MathML from a PDF/HTML paper should compute when every symbol is bound.

## Evidence

- `\frac{1}{299792458}` and `c^{2}` style input is rejected by the lexer (`Unexpected character '\'` 3 rows; `{` and `}` are not tokens).
- `π²/6`, `1⁄299792458`, `¼`, `‰`, `∨`, `∧`, `¬`, `→`, `✕` are rejected (one row each in the current corpus; see #236 for the superscript subset).
- `E = mc²` with `m = 1 kg` and `c = 299792458 m/s` needs assignment (#242) and dimensional units (#235) to print `89875517873681764 J`.

## Acceptance criteria

- A LaTeX subset to expression-tree converter: `\frac`, `\sqrt[n]`, `\cdot`, `\times`, `\div`, `\pm`, `^{}`, `_{}` (ignored index or variable suffix), `\pi`, `\left( \right)`, `\mathrm{m}` units, `\,` spacing, `\%`, `\times 10^{8}`.
- MathML presentation subset (`<mfrac>`, `<msup>`, `<mrow>`, `<mi>`, `<mn>`, `<mo>`), useful for HTML papers.
- Unicode math operators `∙ ⋅ × ÷ ∕ ⁄ − ± ∓ ∞ √ ∛ ∜ ∑ ∏ ≤ ≥ ≠ ≈ ∧ ∨ ¬ → ⇒`.
- Rows in the quotes corpus tagged `latex` pass.

## Solution options

1. Small hand-written LaTeX tokenizer feeding the existing parser (similar in size to `latex2mathml` 0.2.3, MIT). Recommended.
2. Reuse MathCAT (Rust, MIT; 16 language rule sets) or Speech Rule Engine (JS, Apache-2.0) which parse MathML to semantic trees. Candidate for the MathML path; both target speech output, so only their parsing layer would be used.
3. `latex2sympy` (Python). Not usable in WASM.

## Blockers

- Blocked by: #236 (Unicode superscripts, fraction slash and scale notation), #242 (assignments for symbol binding).
