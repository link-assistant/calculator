import React, { useMemo } from 'react';

interface Props {
  lino: string;
}

// Color palette for parentheses (cycling through these)
const PAREN_COLORS = [
  '#6366f1', // indigo
  '#f59e0b', // amber
  '#10b981', // emerald
  '#ef4444', // red
  '#8b5cf6', // violet
  '#06b6d4', // cyan
  '#f97316', // orange
  '#84cc16', // lime
];

interface Token {
  type: 'paren' | 'text';
  value: string;
  depth?: number;
}

/**
 * Parses the LINO expression into tokens with depth information for parentheses.
 */
function tokenize(lino: string): Token[] {
  const tokens: Token[] = [];
  let depth = 0;
  let currentText = '';

  for (const char of lino) {
    if (char === '(') {
      if (currentText) {
        tokens.push({ type: 'text', value: currentText });
        currentText = '';
      }
      tokens.push({ type: 'paren', value: '(', depth });
      depth++;
    } else if (char === ')') {
      if (currentText) {
        tokens.push({ type: 'text', value: currentText });
        currentText = '';
      }
      depth--;
      tokens.push({ type: 'paren', value: ')', depth });
    } else {
      currentText += char;
    }
  }

  if (currentText) {
    tokens.push({ type: 'text', value: currentText });
  }

  return tokens;
}

/**
 * ColorCodedLino displays LINO expressions with color-coded parentheses.
 * Each nesting level has a different color, making it easier to match
 * opening and closing parentheses.
 */
const ColorCodedLino: React.FC<Props> = ({ lino }) => {
  const tokens = useMemo(() => tokenize(lino), [lino]);

  return (
    <code className="lino-colored">
      {tokens.map((token, idx) => {
        if (token.type === 'paren' && token.depth !== undefined) {
          const color = PAREN_COLORS[token.depth % PAREN_COLORS.length];
          return (
            <span
              key={idx}
              style={{ color, fontWeight: 600 }}
              title={`Depth: ${token.depth + 1}`}
            >
              {token.value}
            </span>
          );
        }
        return <span key={idx}>{token.value}</span>;
      })}
    </code>
  );
};

export default ColorCodedLino;
