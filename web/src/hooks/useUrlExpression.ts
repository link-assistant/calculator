import { useState, useEffect, useCallback, useRef } from 'react';

/**
 * Convert standard base64 to URL-safe base64 (base64url, RFC 4648 Section 5)
 * - Replaces + with -
 * - Replaces / with _
 * - Removes padding =
 */
function toBase64Url(base64: string): string {
  return base64.replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

/**
 * Convert URL-safe base64 (base64url) back to standard base64
 * - Replaces - with +
 * - Replaces _ with /
 * - Adds padding = as needed
 */
function fromBase64Url(base64url: string): string {
  let base64 = base64url.replace(/-/g, '+').replace(/_/g, '/');
  // Add padding
  const padding = (4 - (base64.length % 4)) % 4;
  base64 += '='.repeat(padding);
  return base64;
}

/**
 * Check if encoded string is in old base64 format (not base64url)
 * Old format uses +, /, or = characters which get URL-encoded
 */
function isLegacyBase64Format(encoded: string): boolean {
  // If it contains padding (=) or standard base64 chars (+, /), it's legacy
  // Also if it doesn't contain any base64url-specific chars (- or _) and is valid base64
  return encoded.includes('=') || encoded.includes('+') || encoded.includes('/');
}

/**
 * Encode an expression to a URL-safe Links Notation format using base64url encoding.
 * Expression is wrapped in a LINO format and then base64url encoded.
 * This encoding uses only a-zA-Z0-9, -, _ characters (no URL encoding needed).
 */
export function encodeExpression(expression: string): string {
  if (!expression.trim()) return '';

  try {
    // Encode as Links Notation: (expression "...")
    const lino = `(expression "${expression.replace(/"/g, '\\"')}")`;
    // Use base64url encoding (URL-safe, no padding)
    const base64 = btoa(encodeURIComponent(lino));
    return toBase64Url(base64);
  } catch {
    // Fallback to simple base64url encoding
    const base64 = btoa(encodeURIComponent(expression));
    return toBase64Url(base64);
  }
}

/**
 * Decode an expression from either base64url (new) or standard base64 (legacy) format.
 * Supports backward compatibility with old links.
 */
export function decodeExpression(encoded: string): string {
  if (!encoded) return '';

  try {
    // Determine if legacy or new format and convert to standard base64
    const base64 = isLegacyBase64Format(encoded) ? encoded : fromBase64Url(encoded);

    // Decode base64
    const decoded = decodeURIComponent(atob(base64));

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
 * Check if the encoded query parameter is in legacy base64 format
 */
export function isLegacyFormat(encoded: string): boolean {
  return isLegacyBase64Format(encoded);
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
  // Track if the expression was loaded from URL (for auto-calculation)
  const loadedFromUrl = useRef(false);
  // Track the last URL-encoded expression to detect duplicates
  const lastUrlExpression = useRef<string | null>(null);

  const [expression, setExpression] = useState<string>(() => {
    // Check URL for initial expression
    const params = new URLSearchParams(window.location.search);
    const q = params.get('q');
    if (q) {
      const decoded = decodeExpression(q);
      if (decoded) {
        loadedFromUrl.current = true;
        lastUrlExpression.current = decoded;

        // If legacy format, redirect to new format immediately
        if (isLegacyFormat(q)) {
          const url = new URL(window.location.href);
          url.searchParams.set('q', encodeExpression(decoded));
          window.history.replaceState({}, '', url.toString());
        }

        return decoded;
      }
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
        // Only push to history if the expression actually changed
        if (lastUrlExpression.current !== expression) {
          window.history.pushState({}, '', url.toString());
          lastUrlExpression.current = expression;
        } else {
          // Same expression, just replace state
          window.history.replaceState({}, '', url.toString());
        }
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
      lastUrlExpression.current = decoded;
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

  // Function to check if expression was initially loaded from URL
  const wasLoadedFromUrl = useCallback(() => {
    const result = loadedFromUrl.current;
    loadedFromUrl.current = false; // Reset after checking
    return result;
  }, []);

  return { expression, setExpression, copyShareLink, wasLoadedFromUrl };
}
