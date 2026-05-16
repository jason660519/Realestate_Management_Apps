import { Badge, Card, Group, Text } from '@mantine/core';

export function MetricCard(props: {
  label: string;
  value: string;
  status: 'ready' | 'pending';
}) {
  return (
    <Card className="surface-card">
      <Text c="dimmed" size="xs" fw={700} tt="uppercase">
        {props.label}
      </Text>
      <Group justify="space-between" mt={8}>
        <Text fw={700}>{props.value}</Text>
        <Badge color={props.status === 'ready' ? 'green' : 'yellow'} variant="light">
          {props.status}
        </Badge>
      </Group>
    </Card>
  );
}
