import { useRef, useEffect, useCallback, forwardRef, useImperativeHandle, TextareaHTMLAttributes } from 'react';

export interface AutoResizeTextareaProps extends Omit<TextareaHTMLAttributes<HTMLTextAreaElement>, 'style'> {
  /** Minimum number of visible rows (default: 1) */
  minRows?: number;
  /** Maximum number of visible rows (default: 10) */
  maxRows?: number;
  /** Custom className for the textarea */
  className?: string;
  /** Additional inline styles */
  style?: React.CSSProperties;
}

export interface AutoResizeTextareaRef {
  /** The underlying textarea element */
  textarea: HTMLTextAreaElement | null;
  /** Manually trigger resize calculation */
  resize: () => void;
}

/**
 * A textarea component that automatically resizes based on content.
 *
 * Features:
 * - Auto-grows when content increases
 * - Constrains manual resize to discrete line-height steps (no partial lines)
 * - Enforces minimum height based on content (cannot resize smaller than content)
 * - Maintains smooth UX with proper snapping
 */
export const AutoResizeTextarea = forwardRef<AutoResizeTextareaRef, AutoResizeTextareaProps>(
  ({ minRows = 1, maxRows = 10, className, style, onChange, ...props }, ref) => {
    const textareaRef = useRef<HTMLTextAreaElement>(null);
    const resizeObserverRef = useRef<ResizeObserver | null>(null);
    const isResizingRef = useRef(false);
    const lastContentHeightRef = useRef<number>(0);

    /**
     * Get computed line height in pixels
     */
    const getLineHeight = useCallback((): number => {
      const textarea = textareaRef.current;
      if (!textarea) return 24; // fallback

      const computedStyle = getComputedStyle(textarea);
      const lineHeight = computedStyle.lineHeight;

      if (lineHeight === 'normal') {
        // 'normal' is typically 1.2 times font-size
        const fontSize = parseFloat(computedStyle.fontSize);
        return Math.round(fontSize * 1.5); // Using 1.5 as per CSS (line-height: 1.5)
      }

      return parseFloat(lineHeight);
    }, []);

    /**
     * Get vertical padding (top + bottom)
     */
    const getVerticalPadding = useCallback((): number => {
      const textarea = textareaRef.current;
      if (!textarea) return 32; // fallback (1rem * 2)

      const computedStyle = getComputedStyle(textarea);
      return parseFloat(computedStyle.paddingTop) + parseFloat(computedStyle.paddingBottom);
    }, []);

    /**
     * Get border heights (top + bottom)
     */
    const getBorderHeight = useCallback((): number => {
      const textarea = textareaRef.current;
      if (!textarea) return 4; // fallback (2px * 2)

      const computedStyle = getComputedStyle(textarea);
      return parseFloat(computedStyle.borderTopWidth) + parseFloat(computedStyle.borderBottomWidth);
    }, []);

    /**
     * Calculate minimum height required for content
     * Returns height that fits all content without scrolling
     */
    const calculateContentHeight = useCallback((): number => {
      const textarea = textareaRef.current;
      if (!textarea) return 0;

      const lineHeight = getLineHeight();
      const padding = getVerticalPadding();
      const border = getBorderHeight();

      // Store current height
      const currentHeight = textarea.style.height;

      // Temporarily set height to auto to measure scrollHeight
      textarea.style.height = 'auto';
      const scrollHeight = textarea.scrollHeight;
      textarea.style.height = currentHeight;

      // Calculate lines needed (content area only, excluding padding)
      const contentHeight = scrollHeight - padding;
      const linesNeeded = Math.ceil(contentHeight / lineHeight);

      // Ensure at least minRows, at most maxRows
      const clampedLines = Math.max(minRows, Math.min(maxRows, linesNeeded));

      // Calculate total height: lines * lineHeight + padding + border
      const totalHeight = (clampedLines * lineHeight) + padding + border;

      return totalHeight;
    }, [getLineHeight, getVerticalPadding, getBorderHeight, minRows, maxRows]);

    /**
     * Snap a height value to the nearest line-height multiple
     * Also ensures height is at least the content minimum
     */
    const snapToLineHeight = useCallback((height: number): number => {
      const textarea = textareaRef.current;
      if (!textarea) return height;

      const lineHeight = getLineHeight();
      const padding = getVerticalPadding();
      const border = getBorderHeight();

      // Calculate content area height (without padding/border)
      const contentAreaHeight = height - padding - border;

      // Round to nearest line
      const lines = Math.round(contentAreaHeight / lineHeight);

      // Clamp to min/max rows
      const clampedLines = Math.max(minRows, Math.min(maxRows, lines));

      // Calculate snapped height
      const snappedHeight = (clampedLines * lineHeight) + padding + border;

      // Get minimum content height
      const minContentHeight = lastContentHeightRef.current;

      // Return the larger of snapped height or content minimum
      return Math.max(snappedHeight, minContentHeight);
    }, [getLineHeight, getVerticalPadding, getBorderHeight, minRows, maxRows]);

    /**
     * Update textarea height based on content
     */
    const updateHeight = useCallback(() => {
      const textarea = textareaRef.current;
      if (!textarea || isResizingRef.current) return;

      const newHeight = calculateContentHeight();
      lastContentHeightRef.current = newHeight;
      textarea.style.height = `${newHeight}px`;
    }, [calculateContentHeight]);

    /**
     * Handle manual resize (via resize handle)
     * Snaps to discrete line-height steps and enforces minimum content height
     */
    const handleResize = useCallback(() => {
      const textarea = textareaRef.current;
      if (!textarea) return;

      isResizingRef.current = true;

      // Get current height
      const currentHeight = textarea.getBoundingClientRect().height;

      // Snap to line-height
      const snappedHeight = snapToLineHeight(currentHeight);

      // Apply snapped height
      if (Math.abs(currentHeight - snappedHeight) > 1) {
        textarea.style.height = `${snappedHeight}px`;
      }

      // Use requestAnimationFrame to prevent immediate re-triggering
      requestAnimationFrame(() => {
        isResizingRef.current = false;
      });
    }, [snapToLineHeight]);

    // Expose ref methods
    useImperativeHandle(ref, () => ({
      textarea: textareaRef.current,
      resize: updateHeight,
    }), [updateHeight]);

    // Setup ResizeObserver to handle manual resize
    useEffect(() => {
      const textarea = textareaRef.current;
      if (!textarea) return;

      resizeObserverRef.current = new ResizeObserver(() => {
        handleResize();
      });

      resizeObserverRef.current.observe(textarea);

      return () => {
        resizeObserverRef.current?.disconnect();
      };
    }, [handleResize]);

    // Update height when content changes
    useEffect(() => {
      updateHeight();
    }, [props.value, updateHeight]);

    // Initial height calculation
    useEffect(() => {
      // Small delay to ensure styles are computed
      const timeoutId = setTimeout(updateHeight, 0);
      return () => clearTimeout(timeoutId);
    }, [updateHeight]);

    // Handle change event
    const handleChange = useCallback((e: React.ChangeEvent<HTMLTextAreaElement>) => {
      onChange?.(e);
      // Height will be updated via the value prop change effect
    }, [onChange]);

    return (
      <textarea
        ref={textareaRef}
        className={className}
        onChange={handleChange}
        style={{
          ...style,
          resize: 'vertical',
          overflow: 'hidden',
        }}
        {...props}
      />
    );
  }
);

AutoResizeTextarea.displayName = 'AutoResizeTextarea';

export default AutoResizeTextarea;
