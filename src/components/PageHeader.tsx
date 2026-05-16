import { Box, Group, Text, Title } from '@mantine/core';
import { ReactNode } from 'react';

export function PageHeader(props: {
  title: string;
  eyebrow: string;
  children?: ReactNode;
}) {
  return (
    <Group justify="space-between" align="flex-start" mb="md">
      <Box>
        <Text tt="uppercase" size="xs" fw={700} c="dimmed">
          {props.eyebrow}
        </Text>
        <Title order={2}>{props.title}</Title>
      </Box>
      {props.children}
    </Group>
  );
}
