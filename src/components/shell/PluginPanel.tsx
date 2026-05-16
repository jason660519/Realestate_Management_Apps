import { Badge, Box, Card, Group, Stack, Text } from '@mantine/core';
import { PluginStatus } from '../../api/tauri';
import { getStatusColor } from '../../lib/status';

export function PluginPanel(props: { plugins: PluginStatus[] }) {
  return (
    <Card className="surface-card">
      <Text fw={700} mb="sm">
        Plugin boundary
      </Text>
      <Stack gap={10}>
        {props.plugins.map((plugin) => (
          <Box key={plugin.id}>
            <Group justify="space-between" wrap="nowrap">
              <Text size="sm">{plugin.name}</Text>
              <Badge size="sm" color={getStatusColor(plugin.status)} variant="light">
                {plugin.enabled ? plugin.status : 'disabled'}
              </Badge>
            </Group>
            <Text c="dimmed" size="xs">
              {plugin.permissionScope.join(', ')}
            </Text>
          </Box>
        ))}
      </Stack>
    </Card>
  );
}
