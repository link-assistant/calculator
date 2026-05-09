import { describe, expect, it } from 'vitest';
import { formatDateTimeInZone, formatUtcDateTime } from './datetimeDisplay';
import type { DateTimeResult } from '../types';

const mskTime: DateTimeResult = {
  source: '12:30:00 MSK',
  utc: '09:30:00 UTC',
  epoch_milliseconds: Date.UTC(2026, 0, 1, 9, 30, 0),
  has_date: false,
  has_time: true,
  timezone: 'MSK',
  offset_seconds: 10_800,
};

describe('datetime display helpers', () => {
  it('uses the backend UTC display string', () => {
    expect(formatUtcDateTime(mskTime)).toBe('09:30:00 UTC');
  });

  it('can format the instant in an explicit timezone', () => {
    expect(formatDateTimeInZone(mskTime, 'UTC')).toBe('09:30:00 UTC');
  });
});
