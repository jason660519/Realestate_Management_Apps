import { describe, expect, it } from 'vitest';
import { getStatusColor } from './status';

describe('getStatusColor', () => {
  it.each(['ok', 'ready'])('maps healthy status %s to green', (status) => {
    expect(getStatusColor(status)).toBe('green');
  });

  it.each(['degraded', 'not_configured', 'pending'])(
    'maps warning status %s to yellow',
    (status) => {
      expect(getStatusColor(status)).toBe('yellow');
    },
  );

  it('maps disabled to gray', () => {
    expect(getStatusColor('disabled')).toBe('gray');
  });

  it.each(['failed', 'offline', 'unknown', ''])('falls back to red for %s', (status) => {
    expect(getStatusColor(status)).toBe('red');
  });
});
