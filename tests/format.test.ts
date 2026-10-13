import { describe, expect, it, vi, afterEach } from 'vitest';
import {
  formatBytes,
  formatNumber,
  formatPercentage,
  getQuotaColor,
  getQuotaColorClass,
  formatTimeRemaining,
  formatDate,
  formatRelativeTime,
  truncate,
  maskApiKey,
} from '../src/utils/format';
import { cn } from '../src/utils/cn';

describe('formatBytes', () => {
  it('handles zero and unit boundaries', () => {
    expect(formatBytes(0)).toBe('0 B');
    expect(formatBytes(512)).toBe('512 B');
    expect(formatBytes(1024)).toBe('1 KB');
    expect(formatBytes(1536)).toBe('1.5 KB');
    expect(formatBytes(1024 ** 3)).toBe('1 GB');
  });
});

describe('formatNumber', () => {
  it('abbreviates thousands and millions', () => {
    expect(formatNumber(999)).toBe('999');
    expect(formatNumber(1000)).toBe('1.0K');
    expect(formatNumber(2_500_000)).toBe('2.5M');
  });
});

describe('formatPercentage', () => {
  it('rounds and guards against division by zero', () => {
    expect(formatPercentage(1, 3)).toBe(33);
    expect(formatPercentage(5, 0)).toBe(0);
  });
});

describe('quota colours', () => {
  it('maps remaining percentage to a severity', () => {
    expect(getQuotaColor(80)).toBe('success');
    expect(getQuotaColor(50)).toBe('success');
    expect(getQuotaColor(20)).toBe('warning');
    expect(getQuotaColor(5)).toBe('error');
    expect(getQuotaColor(0)).toBe('neutral');
  });

  it('maps severity to tailwind classes', () => {
    expect(getQuotaColorClass(90)).toContain('emerald');
    expect(getQuotaColorClass(30)).toContain('amber');
    expect(getQuotaColorClass(1)).toContain('rose');
    expect(getQuotaColorClass(0)).toContain('gray');
  });
});

describe('time formatting', () => {
  afterEach(() => vi.useRealTimers());

  it('formats the time remaining until a reset', () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date('2026-01-01T00:00:00Z'));
    expect(formatTimeRemaining(undefined)).toBe('N/A');
    expect(formatTimeRemaining('2025-12-31T00:00:00Z')).toBe('Reset');
    expect(formatTimeRemaining('2026-01-01T02:30:00Z')).toBe('2h 30m');
    expect(formatTimeRemaining('2026-01-03T05:00:00Z')).toBe('2d 5h');
  });

  it('formats relative timestamps (unix seconds)', () => {
    vi.useFakeTimers();
    const now = new Date('2026-01-01T12:00:00Z');
    vi.setSystemTime(now);
    const secs = now.getTime() / 1000;
    expect(formatRelativeTime(undefined)).toBe('Never');
    expect(formatRelativeTime(secs - 10)).toBe('Just now');
    expect(formatRelativeTime(secs - 5 * 60)).toBe('5m ago');
    expect(formatRelativeTime(secs - 3 * 3600)).toBe('3h ago');
    expect(formatRelativeTime(secs - 2 * 86400)).toBe('2d ago');
  });

  it('returns Never for a missing date', () => {
    expect(formatDate(undefined)).toBe('Never');
    expect(formatDate(1_700_000_000)).not.toBe('Never');
  });
});

describe('string helpers', () => {
  it('truncates long strings only', () => {
    expect(truncate('short', 10)).toBe('short');
    expect(truncate('abcdefghij', 4)).toBe('abcd...');
  });

  it('masks API keys without leaking the middle', () => {
    expect(maskApiKey('sk-1234567890abcdef')).toBe('sk-1****cdef');
    expect(maskApiKey('short')).toBe('****');
  });

  it('joins only truthy class names', () => {
    expect(cn('a', undefined, false, null, 'b')).toBe('a b');
  });
});
