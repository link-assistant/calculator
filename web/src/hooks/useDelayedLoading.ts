import { useState, useEffect, useRef } from 'react';

/**
 * Hook that shows loading indicator only after a delay (300ms by default)
 * This prevents flickering for fast operations
 */
export function useDelayedLoading(isLoading: boolean, delay: number = 300) {
  const [showLoading, setShowLoading] = useState(false);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    if (isLoading) {
      // Start timer to show loading indicator after delay
      timerRef.current = setTimeout(() => {
        setShowLoading(true);
      }, delay);
    } else {
      // Clear timer and hide loading immediately when done
      if (timerRef.current) {
        clearTimeout(timerRef.current);
        timerRef.current = null;
      }
      setShowLoading(false);
    }

    return () => {
      if (timerRef.current) {
        clearTimeout(timerRef.current);
      }
    };
  }, [isLoading, delay]);

  return showLoading;
}
