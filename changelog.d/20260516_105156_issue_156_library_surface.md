---
bump: minor
---

### Added
- Expanded the public Rust library surface so downstream crates (such as
  [`formal-ai`](https://github.com/link-assistant/formal-ai)) can drive
  `link-calculator` end-to-end without crossing any `pub(crate)` or private
  boundaries (#156).
  - `Calculator::calculate_with_value` — a single call that returns the
    parsed `Expression`, the structured `Value`, the per-step explanation
    list, and the Links Notation re-encoding, so no information produced
    during evaluation is lost when consuming the library as a Rust crate.
  - `Calculator::parser` / `Calculator::parser_mut` — borrow the underlying
    `ExpressionParser` to reuse its parse cache and primitives.
  - `ExpressionParser::evaluate_with_steps`,
    `ExpressionParser::evaluate_expr`,
    `ExpressionParser::evaluate_expr_with_steps`,
    `ExpressionParser::apply_binary_op`,
    `ExpressionParser::evaluate_integrate`,
    `ExpressionParser::evaluate_at`, and
    `ExpressionParser::evaluate_expr_with_var` are now `pub`, with
    documentation explaining the downstream-consumer use case.
  - `grammar::evaluate_power` is re-exported at the `grammar` module level
    so callers can apply the same power semantics as the evaluator.
  - The `utils` module is promoted to `pub`, and `generate_issue_link` plus
    `truncate` are re-exported at the crate root.
- `examples/library_consumer.rs` — a runnable example modelled on the
  `formal-ai` use case, demonstrating both the high-level
  `calculate_with_value` API and the lower-level `ExpressionParser`
  primitives.
- `tests/issue_156_library_surface_tests.rs` — 19 outside-the-crate
  integration tests asserting that the full pipeline (parse → plan →
  evaluate-with-steps → re-encode as LINO) is reachable through public
  APIs only.
- Case study at `docs/case-studies/issue-156/` documenting requirements,
  options considered, and the chosen fix.
