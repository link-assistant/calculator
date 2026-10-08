---
title: "Notepad state: variables, line references, totals, booleans, strings, comments and conditionals"
---
Part of #227. Soulver, Numi, Calca, Parsify, Raycast and Hurmet are multi-line notepads; fend, Qalculate!, Numbat and math.js have persistent variables and booleans.

## Evidence

| Expression | Source | Expected | Actual |
| --- | --- | --- | --- |
| `true and true`, `not true` | math.js/fend | true, false | `Unexpected identifier` (`true` 17, `false` 12, `not` 30 rows) |
| `"Hello, world!"` | fend | Hello, world! | `Unexpected character '"'` (49 rows) |
| `# comment`, `x = 5; x * 2` | Soulver/Qalculate! | | `Unexpected character '#'` (49), `';'` (35) |
| `@line1 + 1`, `total`, `sum`, `prev` | Soulver/Numi | | `Unexpected character '@'` (17), `Unexpected identifier` |
| `if 5 > 3 then 1 else 0` | Soulver | 1 | unsupported |
| `price = $20` then `price * 3` | Soulver variables page | $60 | second line has no context |

## Acceptance criteria

- The library exposes an evaluation context (`Session`) with variable assignment, line results, `total`/`sum`/`prev`/`@n` references, headings and `#`/`//` comments; CLI interactive mode and the React notepad use it; one-shot CLI stays stateless.
- Boolean values, comparison and logical operators, `if … then … else`, string literals with `+` concatenation and `to string`.
- Multi-statement input separated by `;` or newline.

## Solution options

1. A `Session` struct holding a symbol table and line results, separate from equation solving (`x = 2` is assignment when `x` is unknown and the right side is constant; otherwise it is an equation). The issue-220 plan already lists this. Recommended.
2. Rewrite the notepad in the React layer only. Rejected: the CLI and WASM API would diverge.

## Blockers

- Blocked by: #229.
