import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { AutoResizeTextarea } from './AutoResizeTextarea';

// Helper to mock getComputedStyle
const mockComputedStyle = (overrides: Partial<CSSStyleDeclaration> = {}) => {
  const defaultStyle = {
    lineHeight: '24px',
    fontSize: '16px',
    paddingTop: '16px',
    paddingBottom: '16px',
    borderTopWidth: '2px',
    borderBottomWidth: '2px',
    ...overrides,
  } as CSSStyleDeclaration;

  return vi.spyOn(window, 'getComputedStyle').mockReturnValue(defaultStyle);
};

describe('AutoResizeTextarea', () => {
  let observeMock: ReturnType<typeof vi.fn>;
  let disconnectMock: ReturnType<typeof vi.fn>;
  let originalResizeObserver: typeof ResizeObserver;

  beforeEach(() => {
    observeMock = vi.fn();
    disconnectMock = vi.fn();

    originalResizeObserver = window.ResizeObserver;

    // Create a proper mock class
    window.ResizeObserver = class MockResizeObserver {
      callback: ResizeObserverCallback;

      constructor(callback: ResizeObserverCallback) {
        this.callback = callback;
      }

      observe = observeMock;
      unobserve = vi.fn();
      disconnect = disconnectMock;
    } as unknown as typeof ResizeObserver;
  });

  afterEach(() => {
    window.ResizeObserver = originalResizeObserver;
    vi.restoreAllMocks();
  });

  it('should render a textarea element', () => {
    render(<AutoResizeTextarea data-testid="textarea" />);
    const textarea = screen.getByTestId('textarea');
    expect(textarea).toBeInTheDocument();
    expect(textarea.tagName).toBe('TEXTAREA');
  });

  it('should pass through standard textarea props', () => {
    render(
      <AutoResizeTextarea
        data-testid="textarea"
        placeholder="Enter text..."
        disabled
        id="my-textarea"
      />
    );
    const textarea = screen.getByTestId('textarea');

    expect(textarea).toHaveAttribute('placeholder', 'Enter text...');
    expect(textarea).toBeDisabled();
    expect(textarea).toHaveAttribute('id', 'my-textarea');
  });

  it('should apply className prop', () => {
    render(<AutoResizeTextarea data-testid="textarea" className="custom-class" />);
    const textarea = screen.getByTestId('textarea');
    expect(textarea).toHaveClass('custom-class');
  });

  it('should call onChange handler when text changes', () => {
    const handleChange = vi.fn();
    render(<AutoResizeTextarea data-testid="textarea" onChange={handleChange} />);
    const textarea = screen.getByTestId('textarea');

    fireEvent.change(textarea, { target: { value: 'Hello World' } });

    expect(handleChange).toHaveBeenCalled();
  });

  it('should have resize: none style (auto-resize only)', () => {
    render(<AutoResizeTextarea data-testid="textarea" />);
    const textarea = screen.getByTestId('textarea');
    expect(textarea).toHaveStyle({ resize: 'none' });
  });

  it('should have overflow: hidden style', () => {
    render(<AutoResizeTextarea data-testid="textarea" />);
    const textarea = screen.getByTestId('textarea');
    expect(textarea).toHaveStyle({ overflow: 'hidden' });
  });

  it('should setup ResizeObserver on mount', () => {
    render(<AutoResizeTextarea data-testid="textarea" />);

    expect(observeMock).toHaveBeenCalled();
  });

  it('should disconnect ResizeObserver on unmount', () => {
    const { unmount } = render(<AutoResizeTextarea data-testid="textarea" />);

    unmount();

    expect(disconnectMock).toHaveBeenCalled();
  });

  it('should accept minRows prop', () => {
    render(<AutoResizeTextarea data-testid="textarea" minRows={3} />);
    const textarea = screen.getByTestId('textarea');
    expect(textarea).toBeInTheDocument();
  });

  it('should accept maxRows prop', () => {
    render(<AutoResizeTextarea data-testid="textarea" maxRows={5} />);
    const textarea = screen.getByTestId('textarea');
    expect(textarea).toBeInTheDocument();
  });

  it('should accept controlled value prop', () => {
    const { rerender } = render(
      <AutoResizeTextarea data-testid="textarea" value="Initial" onChange={() => {}} />
    );
    const textarea = screen.getByTestId('textarea') as HTMLTextAreaElement;

    expect(textarea.value).toBe('Initial');

    rerender(
      <AutoResizeTextarea data-testid="textarea" value="Updated" onChange={() => {}} />
    );

    expect(textarea.value).toBe('Updated');
  });

  it('should merge custom style with default styles', () => {
    render(
      <AutoResizeTextarea
        data-testid="textarea"
        style={{ maxWidth: '500px' }}
      />
    );
    const textarea = screen.getByTestId('textarea');

    // Check that default styles are applied
    expect(textarea).toHaveStyle({
      resize: 'none',
      overflow: 'hidden',
    });

    // Check that custom style is also applied
    expect(textarea).toHaveStyle({
      maxWidth: '500px',
    });
  });
});

describe('AutoResizeTextarea height calculations', () => {
  let originalResizeObserver: typeof ResizeObserver;

  beforeEach(() => {
    originalResizeObserver = window.ResizeObserver;
    window.ResizeObserver = class MockResizeObserver {
      observe = vi.fn();
      unobserve = vi.fn();
      disconnect = vi.fn();
    } as unknown as typeof ResizeObserver;
  });

  afterEach(() => {
    window.ResizeObserver = originalResizeObserver;
    vi.restoreAllMocks();
  });

  it('should use fallback line-height when element is not available', () => {
    // This tests that the component handles null refs gracefully
    render(<AutoResizeTextarea data-testid="textarea" />);
    const textarea = screen.getByTestId('textarea');
    expect(textarea).toBeInTheDocument();
  });

  it('should handle normal line-height value', () => {
    const getComputedStyleSpy = mockComputedStyle({ lineHeight: 'normal', fontSize: '16px' });

    render(<AutoResizeTextarea data-testid="textarea" />);

    expect(getComputedStyleSpy).toHaveBeenCalled();
  });

  it('should handle pixel line-height value', () => {
    const getComputedStyleSpy = mockComputedStyle({ lineHeight: '24px' });

    render(<AutoResizeTextarea data-testid="textarea" />);

    expect(getComputedStyleSpy).toHaveBeenCalled();
  });
});

describe('AutoResizeTextarea ref', () => {
  let originalResizeObserver: typeof ResizeObserver;

  beforeEach(() => {
    originalResizeObserver = window.ResizeObserver;
    window.ResizeObserver = class MockResizeObserver {
      observe = vi.fn();
      unobserve = vi.fn();
      disconnect = vi.fn();
    } as unknown as typeof ResizeObserver;
  });

  afterEach(() => {
    window.ResizeObserver = originalResizeObserver;
  });

  it('should expose textarea element via ref', () => {
    const ref = { current: null as { textarea: HTMLTextAreaElement | null; resize: () => void } | null };

    render(<AutoResizeTextarea ref={ref} data-testid="textarea" />);

    expect(ref.current).not.toBeNull();
    expect(ref.current?.textarea).toBeInstanceOf(HTMLTextAreaElement);
  });

  it('should expose resize method via ref', () => {
    const ref = { current: null as { textarea: HTMLTextAreaElement | null; resize: () => void } | null };

    render(<AutoResizeTextarea ref={ref} data-testid="textarea" />);

    expect(ref.current?.resize).toBeDefined();
    expect(typeof ref.current?.resize).toBe('function');
  });
});
