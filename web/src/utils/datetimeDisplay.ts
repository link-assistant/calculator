import type { DateTimeResult } from '../types';

function getPart(parts: Intl.DateTimeFormatPart[], type: Intl.DateTimeFormatPartTypes): string {
  return parts.find(part => part.type === type)?.value || '';
}

/**
 * Formats a backend datetime instant in the requested IANA timezone.
 * When no timezone is supplied, the browser's local timezone is used.
 */
export function formatDateTimeInZone(
  info: DateTimeResult,
  timeZone?: string
): string {
  const options: Intl.DateTimeFormatOptions = {
    hour12: false,
    hourCycle: 'h23',
  };

  if (timeZone) {
    options.timeZone = timeZone;
  }

  if (info.has_date) {
    options.year = 'numeric';
    options.month = '2-digit';
    options.day = '2-digit';
  }

  if (info.has_time) {
    options.hour = '2-digit';
    options.minute = '2-digit';
    options.second = '2-digit';
    options.timeZoneName = 'short';
  }

  const date = new Date(info.epoch_milliseconds);
  const formatter = new Intl.DateTimeFormat('en-US', options);
  const parts = formatter.formatToParts(date);

  const datePart = info.has_date
    ? `${getPart(parts, 'year')}-${getPart(parts, 'month')}-${getPart(parts, 'day')}`
    : '';
  const timePart = info.has_time
    ? `${getPart(parts, 'hour')}:${getPart(parts, 'minute')}:${getPart(parts, 'second')}`
    : '';
  const zonePart = info.has_time ? getPart(parts, 'timeZoneName') : '';

  if (datePart && timePart) {
    return `${datePart} ${timePart} ${zonePart}`.trim();
  }
  if (timePart) {
    return `${timePart} ${zonePart}`.trim();
  }
  if (datePart) {
    return datePart;
  }

  return formatter.format(date);
}

export function formatLocalDateTime(info: DateTimeResult): string {
  return formatDateTimeInZone(info);
}

export function formatUtcDateTime(info: DateTimeResult): string {
  return info.utc || formatDateTimeInZone(info, 'UTC');
}
