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
 * Parse the examples.lino file into a structured format.
 */
function parseExamplesLino(linoContent: string): ExamplesData {
  const categories: Record<string, ExamplesCategory> = {};
  const allExamples: Example[] = [];

  try {
    const links = parser.parse(linoContent);

    // Navigate to examples > categories
    for (const link of links) {
      if (link.id === 'examples' && link.values) {
        for (const child of link.values) {
          if (child.id === 'categories' && child.values) {
            // Each child is a category like 'arithmetic', 'currency', etc.
            for (const categoryLink of child.values) {
              if (!categoryLink.id || !categoryLink.values) continue;

              const categoryName = categoryLink.id;
              let categoryLabel = categoryName;
              const categoryExamples: Example[] = [];

              for (const prop of categoryLink.values) {
                if (prop.id === 'label' && prop.values && prop.values.length > 0) {
                  // Extract label value
                  categoryLabel = prop.values[0]?.id || categoryName;
                } else if (prop.id === 'examples' && prop.values) {
                  // Parse examples array
                  for (const exampleLink of prop.values) {
                    if (!exampleLink.values) continue;

                    let expression = '';
                    let description = '';
                    let result = '';

                    for (const field of exampleLink.values) {
                      if (field.id === 'expression' && field.values && field.values.length > 0) {
                        expression = field.values[0]?.id || '';
                      } else if (field.id === 'description' && field.values && field.values.length > 0) {
                        description = field.values[0]?.id || '';
                      } else if (field.id === 'result' && field.values && field.values.length > 0) {
                        result = field.values[0]?.id || '';
                      }
                    }

                    if (expression) {
                      const example: Example = {
                        expression,
                        description,
                        result,
                        category: categoryName,
                      };
                      categoryExamples.push(example);
                      allExamples.push(example);
                    }
                  }
                }
              }

              categories[categoryName] = {
                label: categoryLabel,
                examples: categoryExamples,
              };
            }
          }
        }
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
