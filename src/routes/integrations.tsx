import { Badge, Card, Group, SimpleGrid, Stack, Text } from '@mantine/core';
import { PageHeader } from '../components/PageHeader';
import { useAppData } from '../components/shell/appData';
import { getStatusColor } from '../lib/status';

export function IntegrationsPage() {
  const { plugins } = useAppData();

  return (
    <Stack gap="md">
      <PageHeader title="Integrations" eyebrow="Plugin registry" />
      <SimpleGrid cols={{ base: 1, md: 2 }}>
        {plugins.map((plugin) => (
          <Card key={plugin.id} className="surface-card">
            <Group justify="space-between" mb="xs">
              <Text fw={700}>{plugin.name}</Text>
              <Badge color={getStatusColor(plugin.status)} variant="light">
                {plugin.status}
              </Badge>
            </Group>
            <Stack gap={6}>
              {plugin.permissionScope.map((scope) => (
                <Text key={scope} c="dimmed" size="sm">
                  {scope}
                </Text>
              ))}
            </Stack>
          </Card>
        ))}
      </SimpleGrid>
    </Stack>
  );
}
