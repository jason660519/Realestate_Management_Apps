import { describe, expect, it } from 'vitest';
import { navigationItems } from './navigation';

describe('navigationItems', () => {
  it('starts with the workbench root route', () => {
    expect(navigationItems[0]?.to).toBe('/');
  });

  it('exposes the canonical job-based shell areas', () => {
    expect(navigationItems.map((item) => item.to)).toEqual([
      '/',
      '/properties',
      '/documents',
      '/ai-review',
      '/tasks',
      '/integrations',
      '/settings',
    ]);
  });

  it('every item has a tooltip hint distinct from its label', () => {
    for (const item of navigationItems) {
      expect(item.hint, `${item.label} should have a hint`).toBeTruthy();
      expect(item.hint).not.toBe(item.label);
    }
  });

  it('all routes are unique', () => {
    const routes = navigationItems.map((item) => item.to);
    expect(new Set(routes).size).toBe(routes.length);
  });
});
