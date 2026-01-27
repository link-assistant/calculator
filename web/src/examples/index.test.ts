import { describe, it, expect } from 'vitest';
import {
  getAllExamples,
  getExamplesByCategory,
  getCategories,
  getRandomExamples,
  getExamplesForDisplay,
  DEFAULT_EXAMPLES,
} from './index';

describe('Examples Module', () => {
  describe('getAllExamples', () => {
    it('should return an array of examples', () => {
      const examples = getAllExamples();
      expect(Array.isArray(examples)).toBe(true);
    });

    it('should have examples with required fields', () => {
      const examples = getAllExamples();
      if (examples.length > 0) {
        const example = examples[0];
        expect(example).toHaveProperty('expression');
        expect(example).toHaveProperty('description');
        expect(example).toHaveProperty('category');
      }
    });
  });

  describe('getCategories', () => {
    it('should return an array of categories', () => {
      const categories = getCategories();
      expect(Array.isArray(categories)).toBe(true);
    });

    it('should have categories with name and label', () => {
      const categories = getCategories();
      if (categories.length > 0) {
        const category = categories[0];
        expect(category).toHaveProperty('name');
        expect(category).toHaveProperty('label');
      }
    });
  });

  describe('getExamplesByCategory', () => {
    it('should return examples for a valid category', () => {
      const categories = getCategories();
      if (categories.length > 0) {
        const categoryName = categories[0].name;
        const examples = getExamplesByCategory(categoryName);
        expect(Array.isArray(examples)).toBe(true);
      }
    });

    it('should return empty array for invalid category', () => {
      const examples = getExamplesByCategory('nonexistent-category');
      expect(examples).toEqual([]);
    });
  });

  describe('getRandomExamples', () => {
    it('should return requested number of examples', () => {
      const examples = getRandomExamples(6);
      const allExamples = getAllExamples();
      const expectedCount = Math.min(6, allExamples.length);
      expect(examples.length).toBe(expectedCount);
    });

    it('should return unique examples', () => {
      const examples = getRandomExamples(6);
      const expressions = examples.map((e) => e.expression);
      const uniqueExpressions = new Set(expressions);
      expect(uniqueExpressions.size).toBe(expressions.length);
    });

    it('should return different results on subsequent calls (randomness)', () => {
      // Call multiple times and check if at least one result is different
      // (there's a small chance this could fail randomly, but very unlikely with many examples)
      const allExamples = getAllExamples();
      if (allExamples.length > 6) {
        const results = new Set<string>();
        for (let i = 0; i < 10; i++) {
          const examples = getRandomExamples(6);
          results.add(examples.map((e) => e.expression).sort().join(','));
        }
        // Should have at least 2 different results in 10 attempts
        expect(results.size).toBeGreaterThan(1);
      }
    });
  });

  describe('getExamplesForDisplay', () => {
    it('should return expressions as strings', () => {
      const expressions = getExamplesForDisplay(6);
      expect(Array.isArray(expressions)).toBe(true);
      expressions.forEach((expr) => {
        expect(typeof expr).toBe('string');
      });
    });

    it('should return 6 expressions by default', () => {
      const expressions = getExamplesForDisplay();
      const allExamples = getAllExamples();
      const expectedCount = Math.min(6, Math.max(allExamples.length, DEFAULT_EXAMPLES.length));
      expect(expressions.length).toBe(expectedCount);
    });
  });

  describe('DEFAULT_EXAMPLES', () => {
    it('should have 6 default examples', () => {
      expect(DEFAULT_EXAMPLES.length).toBe(6);
    });

    it('should have valid structure', () => {
      DEFAULT_EXAMPLES.forEach((example) => {
        expect(example).toHaveProperty('expression');
        expect(example).toHaveProperty('description');
        expect(example).toHaveProperty('category');
      });
    });
  });
});
