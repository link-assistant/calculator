// Compares js/src/{error,utils}.js, js/src/types/dimension.js and
// js/src/grammar/locale_numbers.js with the JSON that
// experiments/issue-254/foundation-vectors prints (issue #254).
//
//   node experiments/issue-254/foundation-compare.mjs /tmp/foundation-vectors.json

import { readFileSync } from 'node:fs';
import { CalculatorError } from '../../js/src/error.js';
import { generateIssueLink, truncate } from '../../js/src/utils.js';
import { Dimension } from '../../js/src/types/dimension.js';
import { scan } from '../../js/src/grammar/locale_numbers.js';
import { Err, Panic } from '../../js/src/rust/core.js';

const vectors = JSON.parse(readFileSync(process.argv[2], 'utf8'));
let failures = 0;
const check = (label, actual, expected) => {
  const a = JSON.stringify(actual);
  const e = JSON.stringify(expected);
  if (a !== e) {
    failures += 1;
    if (failures <= 30) {
      console.log(`MISMATCH ${label}\n  js:   ${a}\n  rust: ${e}`);
    }
  }
};

const errors = [
  CalculatorError.parse('invalid syntax'),
  CalculatorError.unexpectedToken('+', 'number', 5),
  CalculatorError.unitMismatch('add', 'USD', 'hours'),
  CalculatorError.eval('bad "quote"\n\tline é \u{7f} \u{200b}'),
  CalculatorError.DivisionByZero,
  CalculatorError.InvalidDateTime('not a date'),
  CalculatorError.UnknownCurrency('XYZ'),
  CalculatorError.CurrencyConversion({ from: 'USD', to: 'EUR', reason: 'no rate' }),
  CalculatorError.NoHistoricalRate({ currency: 'USD', date: '2020-01-01' }),
  CalculatorError.Overflow,
  CalculatorError.InvalidOperation('Cannot add'),
  CalculatorError.EmptyInput,
  CalculatorError.unknownFunction('foo'),
  CalculatorError.invalidArgs('sqrt', 'needs 1 argument'),
  CalculatorError.domain('sqrt of negative'),
  CalculatorError.SymbolicResult({
    expression: 'integrate x dx',
    result: 'x^2/2 + C',
    latexInput: '\\int x\\,dx',
    latexResult: '\\frac{x^{2}}{2} + C',
  }),
];
errors.forEach((error, index) => {
  const info = error.toErrorInfo();
  const json = JSON.parse(JSON.stringify(info));
  check(`error ${index}`, {
    debug: error.fmtDebug(),
    display: error.toString(),
    json_keys: Object.keys(json),
    key: info.key,
    params: info.params === null ? null : [...info.params].sort(([a], [b]) => (a < b ? -1 : 1)),
  }, vectors.errors[index]);
});

for (const [input, error, link] of vectors.links) {
  check(`link ${JSON.stringify(input)}`, generateIssueLink(input, error), link);
}
for (const [s, n, expected] of vectors.truncates) {
  check(`truncate ${JSON.stringify(s)} ${n}`, truncate(s, n), expected);
}

const length = Dimension.base('length');
const mass = Dimension.base('mass');
const time = Dimension.base('time');
const currency = Dimension.base('currency');
const energy = mass.mul(length.pow(2)).div(time.pow(2));
const str = (d) => (d === null ? null : d.toString());
check('dimensions', {
  currency_per_mass: currency.div(mass).toString(),
  default: Dimension.default().toString(),
  energy: energy.toString(),
  energy_pow_neg3: energy.pow(-3).toString(),
  energy_root0: str(energy.root(0)),
  energy_root2: str(energy.root(2)),
  energy_root_neg2: str(energy.root(-2)),
  neg_root: str(length.pow(-4).root(2)),
  neg_root_odd: str(length.pow(-3).root(2)),
  none: Dimension.NONE.toString(),
  pow0: energy.pow(0).toString(),
  speed: length.div(time).toString(),
  unknown: str(Dimension.base('speed')),
}, vectors.dimensions);

const runScan = (input, start, allowComma, decimalComma) => {
  try {
    const [text, end] = scan([...input], start, allowComma, decimalComma);
    return { ok: [text, end] };
  } catch (thrown) {
    if (thrown instanceof Err) {
      return { err: thrown.error.fmtDebug() };
    }
    if (thrown instanceof Panic) {
      return { panic: true };
    }
    throw thrown;
  }
};
let scans = 0;
for (const list of [vectors.scans, vectors.fuzz]) {
  for (const [input, start, allowComma, decimalComma, outcome] of list) {
    scans += 1;
    check(`scan ${JSON.stringify([input, start, allowComma, decimalComma])}`,
      runScan(input, start, allowComma, decimalComma), outcome);
  }
}
const outcomes = { ok: 0, err: 0, panic: 0 };
for (const [, , , , outcome] of vectors.fuzz) {
  outcomes[Object.keys(outcome)[0]] += 1;
}
console.log(`errors ${errors.length}, links ${vectors.links.length}, truncates ${vectors.truncates.length}, scans ${scans} (fuzz outcomes ${JSON.stringify(outcomes)})`);
console.log(failures === 0 ? 'all match' : `${failures} mismatches`);
process.exitCode = failures === 0 ? 0 : 1;
