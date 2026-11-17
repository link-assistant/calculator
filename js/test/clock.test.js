import { describe, expect, it } from 'vitest';
import { now, setFixedNow } from '../src/clock.js';
import { Utc } from '../src/rust/chrono.js';

describe('clock', () => {
  it('fixed instant is returned until cleared', () => {
    const instant = Utc.withYmdAndHms(2026, 3, 15, 12, 34, 56).unwrap();
    setFixedNow(instant);
    expect(now().equals(instant)).toBe(true);
    setFixedNow(null);
    expect(now().equals(instant)).toBe(false);
  });
});
