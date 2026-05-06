import { describe, it, expect } from 'vitest';
import { formatDuration, formatDate } from '$lib/utils/format';

describe('formatDuration', () => {
	it('returns 0:00 for zero', () => {
		expect(formatDuration(0)).toBe('0:00');
	});

	it('returns 0:00 for negative values', () => {
		expect(formatDuration(-60)).toBe('0:00');
	});

	it('returns 0:00 for NaN', () => {
		expect(formatDuration(NaN)).toBe('0:00');
	});

	it('returns 0:00 for Infinity', () => {
		expect(formatDuration(Infinity)).toBe('0:00');
	});

	it('formats sub-minute durations correctly', () => {
		expect(formatDuration(9)).toBe('0:09');
		expect(formatDuration(59)).toBe('0:59');
	});

	it('formats minute durations correctly', () => {
		expect(formatDuration(60)).toBe('1:00');
		expect(formatDuration(90)).toBe('1:30');
		expect(formatDuration(3599)).toBe('59:59');
	});

	it('formats hour durations with zero-padded minutes and seconds', () => {
		expect(formatDuration(3600)).toBe('1:00:00');
		expect(formatDuration(3661)).toBe('1:01:01');
		expect(formatDuration(7322)).toBe('2:02:02');
	});

	it('floors fractional seconds', () => {
		expect(formatDuration(59.9)).toBe('0:59');
		expect(formatDuration(60.1)).toBe('1:00');
	});
});

describe('formatDate', () => {
	it('returns a non-empty string for a valid ISO date', () => {
		const result = formatDate('2024-01-15T10:30:00Z');
		expect(typeof result).toBe('string');
		expect(result.length).toBeGreaterThan(0);
	});

	it('includes the year in the output', () => {
		expect(formatDate('2024-06-01T00:00:00Z')).toContain('2024');
	});
});
