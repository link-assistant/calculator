import { readFileSync } from "node:fs";
import { describe, expect, test } from "vitest";
import { createPlan, emptyPlan, errorPlan } from "../src/plan.js";
import { BinaryOp, Decimal, Expression, Unit } from "../src/types/index.js";

// `src/plan.rs` has no unit tests; these check `toJSON()` against the Rust
// plans recorded in `parity/goldens/default.json`, with the expressions built
// by hand (no parser).
const goldens = JSON.parse(
  readFileSync(
    new URL("../../parity/goldens/default.json", import.meta.url),
    "utf8",
  ),
);

/** @param {string} input */
function goldenPlan(input) {
  const text = JSON.stringify(goldens);
  const re = /"plan":(\{[^{}]*\})/g;
  for (const m of text.matchAll(re)) {
    const plan = JSON.parse(m[1]);
    if (plan.expression === input) return plan;
  }
  throw new Error(`no golden plan for ${input}`);
}

/** @param {number} n @param {string | null} currency */
const num = (n, currency) =>
  Expression.Number({
    value: Decimal.fromI64(BigInt(n)),
    unit: currency === null ? Unit.None : Unit.Currency(currency),
    alternativeUnits: [],
  });

const KEY_ORDER = [
  "expression",
  "lino_interpretation",
  "alternative_lino",
  "required_sources",
  "currencies",
  "is_live_time",
  "success",
  "error",
];

/** @param {object} json */
function expectSerdeKeyOrder(json) {
  const keys = Object.keys(json);
  expect(keys).toEqual(KEY_ORDER.filter((k) => keys.includes(k)));
}

describe("plan", () => {
  const cases = [
    [
      "2 + 3",
      () =>
        Expression.Binary({
          left: num(2, null),
          op: BinaryOp.Add,
          right: num(3, null),
        }),
    ],
    ["100 USD", () => num(100, "USD")],
    [
      "84 USD - 34 EUR",
      () =>
        Expression.Binary({
          left: num(84, "USD"),
          op: BinaryOp.Subtract,
          right: num(34, "EUR"),
        }),
    ],
    [
      "0 INR + 100 RUB",
      () =>
        Expression.Binary({
          left: num(0, "INR"),
          op: BinaryOp.Add,
          right: num(100, "RUB"),
        }),
    ],
    [
      "0 RUB + 1 USD",
      () =>
        Expression.Binary({
          left: num(0, "RUB"),
          op: BinaryOp.Add,
          right: num(1, "USD"),
        }),
    ],
    ["now", () => Expression.Now()],
  ];
  for (const [input, build] of cases) {
    test(`create_plan matches golden: ${input}`, () => {
      const json = createPlan(
        /** @type {string} */ (input),
        /** @type {() => Expression} */ (build)(),
      ).toJSON();
      expect(json).toEqual(goldenPlan(/** @type {string} */ (input)));
      expectSerdeKeyOrder(json);
    });
  }

  test("empty_plan and error_plan", () => {
    expect(JSON.stringify(emptyPlan())).toBe(
      '{"expression":"","lino_interpretation":"","required_sources":[],"currencies":[],"is_live_time":false,"success":false,"error":"Empty input"}',
    );
    expect(errorPlan("2 +", "oops").toJSON()).toEqual({
      expression: "2 +",
      lino_interpretation: "",
      required_sources: [],
      currencies: [],
      is_live_time: false,
      success: false,
      error: "oops",
    });
  });
});
