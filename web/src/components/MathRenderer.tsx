import React, { useEffect, useRef } from 'react';
import katex from 'katex';
import 'katex/dist/katex.min.css';

interface MathRendererProps {
  latex: string;
  display?: boolean;
  className?: string;
}

/**
 * Renders LaTeX mathematical expressions using KaTeX.
 */
export const MathRenderer: React.FC<MathRendererProps> = ({
  latex,
  display = false,
  className = '',
}) => {
  const containerRef = useRef<HTMLSpanElement>(null);

  useEffect(() => {
    if (containerRef.current && latex) {
      try {
        katex.render(latex, containerRef.current, {
          displayMode: display,
          throwOnError: false,
          errorColor: '#cc0000',
          trust: true,
        });
      } catch (error) {
        console.error('KaTeX rendering error:', error);
        if (containerRef.current) {
          containerRef.current.textContent = latex;
        }
      }
    }
  }, [latex, display]);

  return <span ref={containerRef} className={`math-renderer ${className}`} />;
};

export default MathRenderer;
