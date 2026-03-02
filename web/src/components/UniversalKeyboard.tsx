import { useCallback } from 'react';
import { useTranslation } from 'react-i18next';

export interface UniversalKeyboardProps {
  /** Callback when a key is pressed - receives the text to insert */
  onKeyPress: (text: string) => void;
  /** Callback when backspace is pressed */
  onBackspace: () => void;
  /** Callback when Enter/Calculate is triggered */
  onCalculate: () => void;
  /** Whether the calculator is ready */
  disabled?: boolean;
}

interface KeyDef {
  label: string;
  value: string;
  /** Additional CSS class */
  className?: string;
  /** Width multiplier (default 1) */
  wide?: boolean;
}

const KEYBOARD_ROWS: KeyDef[][] = [
  // Row 1: digits and basic operators
  [
    { label: '7', value: '7' },
    { label: '8', value: '8' },
    { label: '9', value: '9' },
    { label: '÷', value: ' / ' },
    { label: '(', value: '(' },
    { label: ')', value: ')' },
    { label: '^', value: '^' },
  ],
  // Row 2: digits and basic operators
  [
    { label: '4', value: '4' },
    { label: '5', value: '5' },
    { label: '6', value: '6' },
    { label: '×', value: ' * ' },
    { label: 'sin', value: 'sin(' },
    { label: 'cos', value: 'cos(' },
    { label: 'tan', value: 'tan(' },
  ],
  // Row 3: digits and basic operators
  [
    { label: '1', value: '1' },
    { label: '2', value: '2' },
    { label: '3', value: '3' },
    { label: '−', value: ' - ' },
    { label: 'ln', value: 'ln(' },
    { label: 'log', value: 'log(' },
    { label: '√', value: 'sqrt(' },
  ],
  // Row 4: zero, decimal, space, plus
  [
    { label: '0', value: '0' },
    { label: '.', value: '.' },
    { label: '_', value: ' ' },
    { label: '+', value: ' + ' },
    { label: 'π', value: 'pi' },
    { label: 'e', value: 'e' },
    { label: '%', value: '%' },
  ],
];

/**
 * A universal on-screen keyboard for mathematical expressions.
 * Allows input of digits, operators, and math functions.
 * Collapsed by default, toggled by a button below the input field.
 */
export function UniversalKeyboard({ onKeyPress, onBackspace, onCalculate, disabled }: UniversalKeyboardProps) {
  const { t } = useTranslation();

  const handleKey = useCallback((value: string) => {
    if (!disabled) {
      onKeyPress(value);
    }
  }, [onKeyPress, disabled]);

  const handleBackspace = useCallback(() => {
    if (!disabled) {
      onBackspace();
    }
  }, [onBackspace, disabled]);

  const handleCalculate = useCallback(() => {
    if (!disabled) {
      onCalculate();
    }
  }, [onCalculate, disabled]);

  return (
    <div className="universal-keyboard" aria-label={t('keyboard.label', 'Universal Keyboard')}>
      {KEYBOARD_ROWS.map((row, rowIdx) => (
        <div key={rowIdx} className="keyboard-row">
          {row.map((key) => (
            <button
              key={key.label}
              className={`keyboard-key${key.className ? ` ${key.className}` : ''}`}
              onClick={() => handleKey(key.value)}
              disabled={disabled}
              aria-label={key.label}
              type="button"
            >
              {key.label}
            </button>
          ))}
        </div>
      ))}
      <div className="keyboard-row keyboard-row-actions">
        <button
          className="keyboard-key keyboard-key-backspace"
          onClick={handleBackspace}
          disabled={disabled}
          aria-label={t('keyboard.backspace', 'Backspace')}
          type="button"
        >
          ⌫
        </button>
        <button
          className="keyboard-key keyboard-key-calculate"
          onClick={handleCalculate}
          disabled={disabled}
          aria-label={t('keyboard.calculate', 'Calculate')}
          type="button"
        >
          =
        </button>
      </div>
    </div>
  );
}

export default UniversalKeyboard;
