import { screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { renderWithProviders } from '../test/render';
import { EmptyOperationalState } from './EmptyOperationalState';

describe('EmptyOperationalState', () => {
  it('renders the title and detail text', () => {
    renderWithProviders(
      <EmptyOperationalState
        title="No properties yet"
        detail="Add one to seed the workspace."
      />,
    );

    expect(screen.getByText('No properties yet')).toBeInTheDocument();
    expect(screen.getByText('Add one to seed the workspace.')).toBeInTheDocument();
  });

  it('renders title as a heading at level 4 per design system §Section title', () => {
    renderWithProviders(
      <EmptyOperationalState title="Workbench is empty" detail="Connect a server." />,
    );

    const heading = screen.getByRole('heading', { name: 'Workbench is empty' });
    expect(heading.tagName).toBe('H4');
  });
});
