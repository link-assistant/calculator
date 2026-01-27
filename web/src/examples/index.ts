/**
 * Examples module for Link.Calculator
 *
 * This module loads calculator examples from data/examples.lino
 * and provides utilities for selecting random examples for display.
 */

import { Parser } from 'links-notation';

// Import examples .lino file as raw text
import examplesLino from '../../../data/examples.lino?raw';

export interface Example {
  expression: string;
  description: string;
  result?: string;
  category: string;
}

interface ExamplesCategory {
  label: string;
  examples: Example[];
}

interface ExamplesData {
  categories: Record<string, ExamplesCategory>;
  allExamples: Example[];
}

// Parser for Links Notation
const parser = new Parser();

/**
 * Extract a field value from a parsed link's values array.
 * Links notation parses (field "value") as [{ id: "field" }, { id: "value" }]
 */
function extractField(values: Array<{ id: string | null; values: unknown[] }>, fieldName: string): string | null {
  for (let i = 0; i < values.length - 1; i++) {
    const current = values[i];
    const next = values[i + 1];
    if (current?.id === fieldName && next?.id) {
      return next.id;
    }
  }
  return null;
}

/**
 * Parse the examples.lino file into a structured format.
 * The file uses flat S-expression format:
 * (example (expression "...") (description "...") (category "..."))
 */
function parseExamplesLino(linoContent: string): ExamplesData {
  const categories: Record<string, ExamplesCategory> = {};
  const allExamples: Example[] = [];

  try {
    const links = parser.parse(linoContent);

    // Each top-level link should be an example
    for (const link of links) {
      // Check if this is an example link
      // Format: (example (expression "...") (description "...") (category "..."))
      // Parsed as: { id: null, values: [{ id: "example" }, { id: null, values: [...] }, ...] }
      if (!link.values || link.values.length === 0) continue;

      const firstValue = link.values[0];
      if (firstValue?.id !== 'example') continue;

      // Extract fields from the nested values
      let expression = '';
      let description = '';
      let category = '';

      for (const value of link.values) {
        if (value.id === null && value.values && Array.isArray(value.values)) {
          // This is a nested S-expression like (expression "2 + 3")
          const fieldValues = value.values as Array<{ id: string | null; values: unknown[] }>;
          const fieldResult = extractField(fieldValues, 'expression');
          if (fieldResult) expression = fieldResult;

          const descResult = extractField(fieldValues, 'description');
          if (descResult) description = descResult;

          const catResult = extractField(fieldValues, 'category');
          if (catResult) category = catResult;
        }
      }

      if (expression && category) {
        const example: Example = {
          expression,
          description: description || '',
          category,
        };
        allExamples.push(example);

        // Add to category
        if (!categories[category]) {
          categories[category] = {
            label: category.charAt(0).toUpperCase() + category.slice(1),
            examples: [],
          };
        }
        categories[category].examples.push(example);
      }
    }
  } catch (error) {
    console.warn('Failed to parse examples.lino:', error);
  }

  return { categories, allExamples };
}

// Parse examples on module load
const examplesData = parseExamplesLino(examplesLino);

/**
 * Get all examples from all categories.
 */
export function getAllExamples(): Example[] {
  return examplesData.allExamples;
}

/**
 * Get examples by category.
 */
export function getExamplesByCategory(category: string): Example[] {
  return examplesData.categories[category]?.examples || [];
}

/**
 * Get all categories with their labels.
 */
export function getCategories(): Array<{ name: string; label: string }> {
  return Object.entries(examplesData.categories).map(([name, data]) => ({
    name,
    label: data.label,
  }));
}

/**
 * Select random examples from the pool, ensuring variety across categories.
 *
 * @param count Number of examples to select (default: 6)
 * @returns Array of randomly selected examples
 */
export function getRandomExamples(count: number = 6): Example[] {
  const all = getAllExamples();

  if (all.length <= count) {
    return [...all];
  }

  // Try to get examples from different categories for variety
  const categoryNames = Object.keys(examplesData.categories);
  const selected: Example[] = [];
  const usedExpressions = new Set<string>();

  // First, try to get one example from each category (up to count)
  for (const category of categoryNames) {
    if (selected.length >= count) break;

    const categoryExamples = examplesData.categories[category]?.examples || [];
    if (categoryExamples.length > 0) {
      const randomIndex = Math.floor(Math.random() * categoryExamples.length);
      const example = categoryExamples[randomIndex];
      if (!usedExpressions.has(example.expression)) {
        selected.push(example);
        usedExpressions.add(example.expression);
      }
    }
  }

  // Fill remaining slots with random examples
  while (selected.length < count && usedExpressions.size < all.length) {
    const randomIndex = Math.floor(Math.random() * all.length);
    const example = all[randomIndex];
    if (!usedExpressions.has(example.expression)) {
      selected.push(example);
      usedExpressions.add(example.expression);
    }
  }

  // Shuffle the final selection
  for (let i = selected.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [selected[i], selected[j]] = [selected[j], selected[i]];
  }

  return selected;
}

/**
 * Default examples (fallback if parsing fails).
 */
export const DEFAULT_EXAMPLES: Example[] = [
  { expression: '2 + 3', description: 'Simple addition', category: 'arithmetic' },
  { expression: '(2 + 3) * 4', description: 'Parentheses', category: 'arithmetic' },
  { expression: '84 USD - 34 EUR', description: 'Currency conversion', category: 'currency' },
  { expression: '100 * 1.5', description: 'Multiplication', category: 'arithmetic' },
  { expression: 'integrate sin(x)/x dx', description: 'Integral', category: 'integration' },
  { expression: 'integrate(x^2, x, 0, 3)', description: 'Definite integral', category: 'integration' },
];

/**
 * Get examples for display, with fallback to defaults.
 */
export function getExamplesForDisplay(count: number = 6): string[] {
  const examples = getAllExamples();
  if (examples.length === 0) {
    return DEFAULT_EXAMPLES.slice(0, count).map((e) => e.expression);
  }
  return getRandomExamples(count).map((e) => e.expression);
}
