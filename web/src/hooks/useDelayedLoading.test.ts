import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useDelayedLoading } from './useDelayedLoading';

describe('useDelayedLoading', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('should not show loading immediately', () => {
    const { result } = renderHook(() => useDelayedLoading(true, 300));
    expect(result.current).toBe(false);
  });

  it('should show loading after delay', () => {
    const { result } = renderHook(() => useDelayedLoading(true, 300));

    expect(result.current).toBe(false);

    act(() => {
      vi.advanceTimersByTime(300);
    });

    expect(result.current).toBe(true);
  });

  it('should hide loading immediately when isLoading becomes false', () => {
    const { result, rerender } = renderHook(
      ({ isLoading }) => useDelayedLoading(isLoading, 300),
      { initialProps: { isLoading: true } }
    );

    act(() => {
      vi.advanceTimersByTime(300);
    });

    expect(result.current).toBe(true);

    rerender({ isLoading: false });

    expect(result.current).toBe(false);
  });

  it('should not show loading if isLoading becomes false before delay', () => {
    const { result, rerender } = renderHook(
      ({ isLoading }) => useDelayedLoading(isLoading, 300),
      { initialProps: { isLoading: true } }
    );

    act(() => {
      vi.advanceTimersByTime(200);
    });

    rerender({ isLoading: false });

    act(() => {
      vi.advanceTimersByTime(200);
    });

    expect(result.current).toBe(false);
  });

  it('should use default delay of 300ms', () => {
    const { result } = renderHook(() => useDelayedLoading(true));

    act(() => {
      vi.advanceTimersByTime(299);
    });

    expect(result.current).toBe(false);

    act(() => {
      vi.advanceTimersByTime(1);
    });

    expect(result.current).toBe(true);
  });

  it('should use custom delay', () => {
    const { result } = renderHook(() => useDelayedLoading(true, 500));

    act(() => {
      vi.advanceTimersByTime(499);
    });

    expect(result.current).toBe(false);

    act(() => {
      vi.advanceTimersByTime(1);
    });

    expect(result.current).toBe(true);
  });
});
