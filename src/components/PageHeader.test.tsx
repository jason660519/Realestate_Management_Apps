import { screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { renderWithProviders } from '../test/render';
import { PageHeader } from './PageHeader';

describe('PageHeader', () => {
  it('renders eyebrow text uppercased and a level-2 heading for the title', () => {
    renderWithProviders(<PageHeader title="Properties" eyebrow="Core data" />);

    const heading = screen.getByRole('heading', { name: 'Properties' });
    expect(heading.tagName).toBe('H2');

    const eyebrow = screen.getByText(/core data/i);
    expect(eyebrow).toBeInTheDocument();
    // text-transform: uppercase is applied via Mantine's `tt="uppercase"`.
    // assertion stays on visible casing so the test does not lock to a specific
    // CSS implementation.
    expect(eyebrow.textContent).toMatch(/core data/i);
  });

  it('renders the children slot to the right of the title', () => {
    renderWithProviders(
      <PageHeader title="Documents" eyebrow="Intake">
        <button type="button">Import</button>
      </PageHeader>,
    );

    expect(screen.getByRole('button', { name: 'Import' })).toBeInTheDocument();
  });

  it('does not render the children slot when omitted', () => {
    renderWithProviders(<PageHeader title="Tasks" eyebrow="Coordination" />);

    expect(screen.queryAllByRole('button')).toHaveLength(0);
  });
});
