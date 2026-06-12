import type { TFunction } from 'i18next';
import type { CalculationResult, CalculationStep } from '../types';

type StepParams = Record<string, string>;

interface ParsedCalculationStep {
  key: string;
  params?: StepParams;
}

function translateStepKey(
  t: TFunction,
  key: string,
  params: StepParams | undefined,
  fallback: string
): string {
  const translated = t(key, {
    ...(params || {}),
    defaultValue: fallback,
  });

  return translated === key ? fallback : String(translated);
}

function parseRawCalculationStep(step: string): ParsedCalculationStep | null {
  if (step === 'Evaluate grouped expression:') {
    return { key: 'steps.evaluateGroup' };
  }
  if (step === 'Computed symbolic result') {
    return { key: 'steps.computedSymbolicResult' };
  }
  if (step === 'Solve linear equation:') {
    return { key: 'steps.solveLinearEquation' };
  }
  if (step === 'Check equality:') {
    return { key: 'steps.checkEquality' };
  }

  const matchers: Array<{
    regex: RegExp;
    build: (match: RegExpMatchArray) => ParsedCalculationStep;
  }> = [
    {
      regex: /^Input expression: (.+)$/,
      build: match => ({
        key: 'steps.inputExpression',
        params: { expression: match[1] },
      }),
    },
    {
      regex: /^Input: (.+)$/,
      build: match => ({
        key: 'steps.input',
        params: { expression: match[1] },
      }),
    },
    {
      regex: /^Literal value: (.+)$/,
      build: match => ({
        key: 'steps.literalValue',
        params: { value: match[1] },
      }),
    },
    {
      regex: /^DateTime value: (.+)$/,
      build: match => ({
        key: 'steps.dateTimeValue',
        params: { value: match[1] },
      }),
    },
    {
      regex: /^UTC equivalent: (.+)$/,
      build: match => ({
        key: 'steps.utcEquivalent',
        params: { value: match[1] },
      }),
    },
    {
      regex: /^Time until: (.+)$/,
      build: match => ({
        key: 'steps.timeUntil',
        params: { duration: match[1] },
      }),
    },
    {
      regex: /^Time since: (.+) ago$/,
      build: match => ({
        key: 'steps.timeSince',
        params: { duration: match[1] },
      }),
    },
    {
      regex: /^Current time: (.+)$/,
      build: match => ({
        key: 'steps.currentTime',
        params: { time: match[1] },
      }),
    },
    {
      regex: /^Time until (.+): (.+)$/,
      build: match => ({
        key: 'steps.timeUntilTarget',
        params: { target: match[1], duration: match[2] },
      }),
    },
    {
      regex: /^Time since (.+): (.+) ago$/,
      build: match => ({
        key: 'steps.timeSinceTarget',
        params: { target: match[1], duration: match[2] },
      }),
    },
    {
      regex: /^Compute: (.+) \^ (.+)$/,
      build: match => ({
        key: 'steps.computePower',
        params: { base: match[1], exponent: match[2] },
      }),
    },
    {
      regex: /^Compute: (.+) ([+\-*/%]) (.+)$/,
      build: match => ({
        key: 'steps.compute',
        params: { left: match[1], op: match[2], right: match[3] },
      }),
    },
    {
      regex: /^= (.+)$/,
      build: match => ({
        key: 'steps.equals',
        params: { value: match[1] },
      }),
    },
    {
      regex: /^Negate: -(.+) = (.+)$/,
      build: match => ({
        key: 'steps.negate',
        params: { value: match[1], result: match[2] },
      }),
    },
    {
      regex: /^At time: (.+)$/,
      build: match => ({
        key: 'steps.atTime',
        params: { time: match[1] },
      }),
    },
    {
      regex: /^Call function: ([^(]+)\((.*)\)$/,
      build: match => ({
        key: 'steps.callFunction',
        params: { name: match[1], args: match[2] },
      }),
    },
    {
      regex: /^Numerical integration: (.+)\(\.\.\.\)$/,
      build: match => ({
        key: 'steps.numericalIntegration',
        params: { name: match[1] },
      }),
    },
    {
      regex: /^Parse first datetime: (.+)$/,
      build: match => ({
        key: 'steps.parseFirstDatetime',
        params: { datetime: match[1] },
      }),
    },
    {
      regex: /^Parse second datetime: (.+)$/,
      build: match => ({
        key: 'steps.parseSecondDatetime',
        params: { datetime: match[1] },
      }),
    },
    {
      regex: /^Calculate difference: (.+) - (.+)$/,
      build: match => ({
        key: 'steps.calculateDifference',
        params: { dt1: match[1], dt2: match[2] },
      }),
    },
    {
      regex: /^Result: (.+)$/,
      build: match => ({
        key: 'steps.result',
        params: { value: match[1] },
      }),
    },
    {
      regex: /^Final result: (.+)$/,
      build: match => ({
        key: 'steps.finalResult',
        params: { value: match[1] },
      }),
    },
    {
      regex: /^Exchange rate: 1 ([A-Za-z0-9]+) = (.+) ([A-Za-z0-9]+) \(source: (.+), date: (.+)\)$/,
      build: match => ({
        key: 'steps.exchangeRate',
        params: {
          from: match[1],
          rate: match[2],
          to: match[3],
          source: match[4],
          date: match[5],
        },
      }),
    },
    {
      regex: /^Convert: (.+) to (.+)$/,
      build: match => ({
        key: 'steps.convert',
        params: { value: match[1], unit: match[2] },
      }),
    },
    {
      regex: /^Indefinite integral: ∫ (.+) d(.+)$/,
      build: match => ({
        key: 'steps.indefiniteIntegral',
        params: { integrand: match[1], variable: match[2] },
      }),
    },
    {
      regex: /^Solution: (.+)$/,
      build: match => ({
        key: 'steps.solution',
        params: { value: match[1] },
      }),
    },
    {
      regex: /^Compare: (.+) (==|<=|>=|!=|<|>|vs) (.+)$/,
      build: match => ({
        key: 'steps.compareOperator',
        params: { left: match[1], op: match[2], right: match[3] },
      }),
    },
    {
      regex: /^Compare: (.+) = (.+)$/,
      build: match => ({
        key: 'steps.compare',
        params: { left: match[1], right: match[2] },
      }),
    },
  ];

  for (const matcher of matchers) {
    const match = step.match(matcher.regex);
    if (match) {
      return matcher.build(match);
    }
  }

  return null;
}

export function translateCalculationStep(
  t: TFunction,
  step: string | CalculationStep
): string {
  if (typeof step !== 'string') {
    if (!step.key) {
      return step.text;
    }

    return translateStepKey(t, step.key, step.params, step.text);
  }

  const parsed = parseRawCalculationStep(step);
  if (!parsed) {
    return step;
  }

  return translateStepKey(t, parsed.key, parsed.params, step);
}

export function localizeCalculationSteps(
  t: TFunction,
  result: Pick<CalculationResult, 'steps' | 'steps_i18n'>
): string[] {
  if (result.steps_i18n?.length === result.steps.length) {
    return result.steps_i18n.map((step, index) =>
      translateCalculationStep(t, {
        ...step,
        text: step.text || result.steps[index],
      })
    );
  }

  return result.steps.map(step => translateCalculationStep(t, step));
}
