import { useState, useEffect, useCallback, useRef } from 'react';

/**
 * Encode an expression to a URL-safe Links Notation format
 * Expression is wrapped in a link and then base64 encoded
 */
export function encodeExpression(expression: string): string {
  if (!expression.trim()) return '';

  try {
    // Encode as Links Notation: (expression "...")
    const lino = `(expression "${expression.replace(/"/g, '\\"')}")`;
    // Use base64 encoding for URL safety
    const encoded = btoa(encodeURIComponent(lino));
    return encoded;
  } catch {
    // Fallback to simple base64 encoding
    return btoa(encodeURIComponent(expression));
  }
}

/**
 * Decode an expression from URL-safe Links Notation format
 */
export function decodeExpression(encoded: string): string {
  if (!encoded) return '';

  try {
    // Decode base64
    const decoded = decodeURIComponent(atob(encoded));

    // Check if it looks like our LINO format: (expression "...")
    const linoMatch = decoded.match(/^\(expression\s+"(.*)"\)$/s);
    if (linoMatch) {
      return linoMatch[1].replace(/\\"/g, '"');
    }

    // Final fallback: return decoded string directly (for backwards compatibility)
    return decoded;
  } catch {
    // If all else fails, return empty
    return '';
  }
}

/**
 * Generate a shareable URL for the current expression
 */
export function generateShareUrl(expression: string): string {
  const encoded = encodeExpression(expression);
  const baseUrl = window.location.origin + window.location.pathname;
  return encoded ? `${baseUrl}?q=${encoded}` : baseUrl;
}

/**
 * Hook to sync expression with URL
 */
export function useUrlExpression(initialExpression: string = '') {
  const [expression, setExpression] = useState<string>(() => {
    // Check URL for initial expression
    const params = new URLSearchParams(window.location.search);
    const q = params.get('q');
    if (q) {
      const decoded = decodeExpression(q);
      if (decoded) return decoded;
    }
    return initialExpression;
  });

  // Track if this is the first update to avoid creating unnecessary history entries
  const isFirstUpdate = useRef(true);

  // Update URL when expression changes (debounced)
  useEffect(() => {
    const timeout = setTimeout(() => {
      const url = new URL(window.location.href);
      if (expression.trim()) {
        url.searchParams.set('q', encodeExpression(expression));
      } else {
        url.searchParams.delete('q');
      }

      // Use replaceState for the first update (initial load), pushState for subsequent changes
      if (isFirstUpdate.current) {
        window.history.replaceState({}, '', url.toString());
        isFirstUpdate.current = false;
      } else {
        window.history.pushState({}, '', url.toString());
      }
    }, 500);

    return () => clearTimeout(timeout);
  }, [expression]);

  // Handle browser back/forward navigation
  useEffect(() => {
    const handlePopState = () => {
      const params = new URLSearchParams(window.location.search);
      const q = params.get('q');
      const decoded = q ? decodeExpression(q) : '';
      setExpression(decoded);
    };

    window.addEventListener('popstate', handlePopState);
    return () => window.removeEventListener('popstate', handlePopState);
  }, []);

  const copyShareLink = useCallback(async () => {
    const url = generateShareUrl(expression);
    try {
      await navigator.clipboard.writeText(url);
      return true;
    } catch {
      return false;
    }
  }, [expression]);

  return { expression, setExpression, copyShareLink };
}
