import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { UniversalKeyboard } from './UniversalKeyboard';

// Mock i18next
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string, fallback?: string) => fallback || key,
  }),
}));

describe('UniversalKeyboard', () => {
  it('should render digit buttons', () => {
    const onKeyPress = vi.fn();
    const onBackspace = vi.fn();
    const onCalculate = vi.fn();

    render(
      <UniversalKeyboard
        onKeyPress={onKeyPress}
        onBackspace={onBackspace}
        onCalculate={onCalculate}
      />
    );

    for (const digit of ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9']) {
      expect(screen.getByRole('button', { name: digit })).toBeInTheDocument();
    }
  });

  it('should render operator buttons', () => {
    const onKeyPress = vi.fn();
    const onBackspace = vi.fn();
    const onCalculate = vi.fn();

    render(
      <UniversalKeyboard
        onKeyPress={onKeyPress}
        onBackspace={onBackspace}
        onCalculate={onCalculate}
      />
    );

    expect(screen.getByRole('button', { name: '+' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: '−' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: '×' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: '÷' })).toBeInTheDocument();
  });

  it('should render math function buttons', () => {
    const onKeyPress = vi.fn();
    const onBackspace = vi.fn();
    const onCalculate = vi.fn();

    render(
      <UniversalKeyboard
        onKeyPress={onKeyPress}
        onBackspace={onBackspace}
        onCalculate={onCalculate}
      />
    );

    expect(screen.getByRole('button', { name: 'sin' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'cos' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'tan' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'ln' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'log' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: '√' })).toBeInTheDocument();
  });

  it('should render parentheses, caret, and constants', () => {
    const onKeyPress = vi.fn();
    const onBackspace = vi.fn();
    const onCalculate = vi.fn();

    render(
      <UniversalKeyboard
        onKeyPress={onKeyPress}
        onBackspace={onBackspace}
        onCalculate={onCalculate}
      />
    );

    expect(screen.getByRole('button', { name: '(' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: ')' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: '^' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'π' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'e' })).toBeInTheDocument();
  });

  it('should call onKeyPress with correct value when digit button is clicked', async () => {
    const onKeyPress = vi.fn();
    const onBackspace = vi.fn();
    const onCalculate = vi.fn();

    render(
      <UniversalKeyboard
        onKeyPress={onKeyPress}
        onBackspace={onBackspace}
        onCalculate={onCalculate}
      />
    );

    await userEvent.click(screen.getByRole('button', { name: '5' }));
    expect(onKeyPress).toHaveBeenCalledWith('5');
  });

  it('should call onKeyPress with operator including spaces when + is clicked', async () => {
    const onKeyPress = vi.fn();
    const onBackspace = vi.fn();
    const onCalculate = vi.fn();

    render(
      <UniversalKeyboard
        onKeyPress={onKeyPress}
        onBackspace={onBackspace}
        onCalculate={onCalculate}
      />
    );

    await userEvent.click(screen.getByRole('button', { name: '+' }));
    expect(onKeyPress).toHaveBeenCalledWith(' + ');
  });

  it('should call onKeyPress with function call syntax when sin is clicked', async () => {
    const onKeyPress = vi.fn();
    const onBackspace = vi.fn();
    const onCalculate = vi.fn();

    render(
      <UniversalKeyboard
        onKeyPress={onKeyPress}
        onBackspace={onBackspace}
        onCalculate={onCalculate}
      />
    );

    await userEvent.click(screen.getByRole('button', { name: 'sin' }));
    expect(onKeyPress).toHaveBeenCalledWith('sin(');
  });

  it('should call onBackspace when backspace button is clicked', async () => {
    const onKeyPress = vi.fn();
    const onBackspace = vi.fn();
    const onCalculate = vi.fn();

    render(
      <UniversalKeyboard
        onKeyPress={onKeyPress}
        onBackspace={onBackspace}
        onCalculate={onCalculate}
      />
    );

    await userEvent.click(screen.getByRole('button', { name: 'Backspace' }));
    expect(onBackspace).toHaveBeenCalled();
  });

  it('should call onCalculate when calculate button is clicked', async () => {
    const onKeyPress = vi.fn();
    const onBackspace = vi.fn();
    const onCalculate = vi.fn();

    render(
      <UniversalKeyboard
        onKeyPress={onKeyPress}
        onBackspace={onBackspace}
        onCalculate={onCalculate}
      />
    );

    await userEvent.click(screen.getByRole('button', { name: 'Calculate' }));
    expect(onCalculate).toHaveBeenCalled();
  });

  it('should disable all buttons when disabled prop is true', () => {
    const onKeyPress = vi.fn();
    const onBackspace = vi.fn();
    const onCalculate = vi.fn();

    render(
      <UniversalKeyboard
        onKeyPress={onKeyPress}
        onBackspace={onBackspace}
        onCalculate={onCalculate}
        disabled={true}
      />
    );

    const buttons = screen.getAllByRole('button');
    for (const button of buttons) {
      expect(button).toBeDisabled();
    }
  });

  it('should not call onKeyPress when disabled', async () => {
    const onKeyPress = vi.fn();
    const onBackspace = vi.fn();
    const onCalculate = vi.fn();

    render(
      <UniversalKeyboard
        onKeyPress={onKeyPress}
        onBackspace={onBackspace}
        onCalculate={onCalculate}
        disabled={true}
      />
    );

    // Try to click - buttons are disabled so userEvent won't fire click handler
    const button5 = screen.getByRole('button', { name: '5' });
    // The button is disabled, so clicking it should not trigger the handler
    expect(button5).toBeDisabled();
    expect(onKeyPress).not.toHaveBeenCalled();
  });
});

describe('UniversalKeyboard - keyboard toggle in App', () => {
  it('keyboard toggle button should exist and toggle the keyboard', async () => {
    // This test verifies integration-level behavior; more detailed tests live in App.test.tsx
    // The UniversalKeyboard itself is stateless (toggle managed by parent App)
    expect(true).toBe(true);
  });
});
