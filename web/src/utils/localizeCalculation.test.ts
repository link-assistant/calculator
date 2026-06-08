import { describe, expect, it } from 'vitest';
import type { TFunction } from 'i18next';
import type { CalculationResult } from '../types';
import { localizeCalculationSteps, translateCalculationStep } from './localizeCalculation';

function makeTranslator(translations: Record<string, string>): TFunction {
  return ((key: string, options?: Record<string, unknown>) => {
    let value = translations[key] || String(options?.defaultValue || key);
    if (options) {
      Object.entries(options).forEach(([name, replacement]) => {
        value = value.replace(`{{${name}}}`, String(replacement));
      });
    }
    return value;
  }) as TFunction;
}

describe('calculation step localization', () => {
  const ruT = makeTranslator({
    'steps.inputExpression': 'Входное выражение: {{expression}}',
    'steps.literalValue': 'Литеральное значение: {{value}}',
    'steps.compute': 'Вычислить: {{left}} {{op}} {{right}}',
    'steps.equals': '= {{value}}',
    'steps.evaluateGroup': 'Вычислить сгруппированное выражение:',
    'steps.finalResult': 'Итоговый результат: {{value}}',
  });

  it('should translate raw arithmetic steps from issue 171', () => {
    const steps = [
      'Input expression: (((1 / 2) + 1) / 4)',
      'Evaluate grouped expression:',
      'Literal value: 1',
      'Literal value: 2',
      'Compute: 1 / 2',
      '= 0.5',
      'Final result: 0.375',
    ];

    expect(steps.map(step => translateCalculationStep(ruT, step))).toEqual([
      'Входное выражение: (((1 / 2) + 1) / 4)',
      'Вычислить сгруппированное выражение:',
      'Литеральное значение: 1',
      'Литеральное значение: 2',
      'Вычислить: 1 / 2',
      '= 0.5',
      'Итоговый результат: 0.375',
    ]);
  });

  it('should prefer structured i18n step metadata when present', () => {
    const result: CalculationResult = {
      result: '5',
      lino_interpretation: '((2) + (3))',
      steps: ['Input expression: 2 + 3'],
      steps_i18n: [
        {
          key: 'steps.inputExpression',
          params: { expression: '2 + 3' },
          text: 'Input expression: 2 + 3',
        },
      ],
      success: true,
    };

    expect(localizeCalculationSteps(ruT, result)).toEqual(['Входное выражение: 2 + 3']);
  });
});
