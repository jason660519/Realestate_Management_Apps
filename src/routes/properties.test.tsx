import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('@tauri-apps/api/core', async () => {
  const { invokeMock } = await import('../test/mockTauri');
  return { invoke: invokeMock };
});

import type { PropertySummary } from '../api/tauri';
import { renderWithProviders } from '../test/render';
import {
  installTauriRuntime,
  resetInvokeMock,
  setInvokeHandlers,
} from '../test/mockTauri';
import { PropertiesView } from './properties';

const SAMPLE: PropertySummary[] = [
  {
    id: '11111111-1111-1111-1111-111111111111',
    display_name: 'Test Sale Property',
    kind: 'sale',
    status: 'active',
    address_raw: 'Test address',
    updated_at: '2026-05-10T12:34:56Z',
  },
];

beforeAll(() => {
  installTauriRuntime();
});

beforeEach(() => {
  resetInvokeMock();
});

afterEach(() => {
  resetInvokeMock();
});

describe('PropertiesView', () => {
  it('renders rows returned by list_property_summaries', async () => {
    setInvokeHandlers({
      list_property_summaries: SAMPLE,
    });

    renderWithProviders(<PropertiesView serverConfigured />);

    expect(await screen.findByText('Test Sale Property')).toBeInTheDocument();
    expect(screen.getByText('Sale')).toBeInTheDocument();
    expect(screen.getByText('Active')).toBeInTheDocument();
    expect(screen.getByText('Test address')).toBeInTheDocument();
  });

  it('renders the not-configured empty state when server is unconfigured and list is empty', async () => {
    setInvokeHandlers({
      list_property_summaries: [],
    });

    renderWithProviders(<PropertiesView serverConfigured={false} />);

    expect(
      await screen.findByText('Server URL is not configured'),
    ).toBeInTheDocument();
  });

  it('renders the empty-but-configured state when server is configured and list is empty', async () => {
    setInvokeHandlers({
      list_property_summaries: [],
    });

    renderWithProviders(<PropertiesView serverConfigured />);

    expect(await screen.findByText('No properties yet')).toBeInTheDocument();
  });

  it('renders the error state and offers retry when invoke rejects', async () => {
    setInvokeHandlers({
      list_property_summaries: () => {
        throw new Error('Server unreachable: connection refused');
      },
    });

    renderWithProviders(<PropertiesView serverConfigured />);

    expect(
      await screen.findByText('Could not load properties'),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/connection refused/i),
    ).toBeInTheDocument();

    setInvokeHandlers({
      list_property_summaries: SAMPLE,
    });

    const user = userEvent.setup();
    await user.click(screen.getByRole('button', { name: /retry/i }));

    await waitFor(() => {
      expect(screen.getByText('Test Sale Property')).toBeInTheDocument();
    });
  });

  it('does not silently hide unknown kinds — they show up as Unknown badge', async () => {
    setInvokeHandlers({
      list_property_summaries: [
        {
          id: '22222222-2222-2222-2222-222222222222',
          display_name: 'Legacy Mixed-Use',
          kind: 'unknown',
          status: 'pending',
          address_raw: null,
          updated_at: null,
        },
      ] satisfies PropertySummary[],
    });

    renderWithProviders(<PropertiesView serverConfigured />);

    expect(await screen.findByText('Legacy Mixed-Use')).toBeInTheDocument();
    expect(screen.getByText('Unknown')).toBeInTheDocument();
    expect(screen.getByText('Pending')).toBeInTheDocument();
  });
});
