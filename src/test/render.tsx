import { MantineProvider } from '@mantine/core';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { render, type RenderOptions, type RenderResult } from '@testing-library/react';
import type { ReactElement, ReactNode } from 'react';
import { theme } from '../theme';

// React Query defaults retry once and keep cache across tests, which causes
// flaky assertions in jsdom. This factory disables retry and zeroes gcTime so
// each `renderWithProviders` call starts from a clean slate.
function makeQueryClient(): QueryClient {
  return new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
        gcTime: 0,
        staleTime: 0,
      },
      mutations: {
        retry: false,
      },
    },
  });
}

type RenderProvidersOptions = Omit<RenderOptions, 'wrapper'> & {
  queryClient?: QueryClient;
};

export function renderWithProviders(
  ui: ReactElement,
  options: RenderProvidersOptions = {},
): RenderResult & { queryClient: QueryClient } {
  const { queryClient = makeQueryClient(), ...renderOptions } = options;

  function Wrapper({ children }: { children: ReactNode }) {
    return (
      <QueryClientProvider client={queryClient}>
        <MantineProvider theme={theme} defaultColorScheme="dark">
          {children}
        </MantineProvider>
      </QueryClientProvider>
    );
  }

  return {
    ...render(ui, { wrapper: Wrapper, ...renderOptions }),
    queryClient,
  };
}
