import { Paper, Stack, Text, Title } from '@mantine/core';

export function EmptyOperationalState(props: { title: string; detail: string }) {
  return (
    <Paper p="lg" className="surface">
      <Stack gap="xs">
        <Title order={4}>{props.title}</Title>
        <Text c="dimmed" size="sm">
          {props.detail}
        </Text>
      </Stack>
    </Paper>
  );
}
