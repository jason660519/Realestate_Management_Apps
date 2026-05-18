import { ActionIcon, Badge, Box, Group, Stack, Text } from '@mantine/core';
import { IconRefresh } from '@tabler/icons-react';
import { ServerHealth } from '../../api/tauri';
import { getStatusColor } from '../../lib/status';

export function StatusPanel(props: { health: ServerHealth | null; onRefresh: () => void }) {
  const health = props.health;
  const services = health?.services ?? [];

  return (
    <Box className="surface" p="md" mb="md">
      <Group justify="space-between" mb="xs">
        <Text fw={700}>Service health</Text>
        <ActionIcon aria-label="Refresh service status" variant="subtle" onClick={props.onRefresh}>
          <IconRefresh size={16} />
        </ActionIcon>
      </Group>
      <Text size="xs" mb="sm" style={{ color: 'var(--text-muted)' }}>
        {health?.baseUrl ?? 'No server configured'}
      </Text>
      <Stack gap={8}>
        {services.length === 0 ? (
          <Text size="sm" style={{ color: 'var(--text-muted)' }}>
            No service response has been recorded.
          </Text>
        ) : (
          services.map((service) => (
            <Group key={service.name} justify="space-between" wrap="nowrap">
              <Text size="sm">{service.name}</Text>
              <Badge size="sm" color={getStatusColor(service.status)} variant="light">
                {service.status}
              </Badge>
            </Group>
          ))
        )}
      </Stack>
      {health?.error ? (
        <Text color="red.3" size="xs" mt="sm">
          {health.error}
        </Text>
      ) : null}
    </Box>
  );
}
