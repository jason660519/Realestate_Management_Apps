import { createTheme } from '@mantine/core';

export const theme = createTheme({
  fontFamily:
    'Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
  primaryColor: 'blue',
  defaultRadius: 6,
  headings: {
    fontWeight: '650',
  },
  components: {
    Button: {
      defaultProps: {
        size: 'xs',
      },
    },
    Badge: {
      defaultProps: {
        radius: 4,
      },
    },
    Card: {
      defaultProps: {
        radius: 6,
        withBorder: true,
      },
    },
  },
});
