import { Badge, Box, Group, Paper, SimpleGrid, Stack, Text, Title } from '@mantine/core';
import { PageHeader } from '../components/PageHeader';
import { MetricCard } from '../components/MetricCard';

export function WorkbenchPage() {
  return (
    <Stack gap="md">
      <PageHeader title="Workbench" eyebrow="Phase 1 shell" />
      <SimpleGrid cols={{ base: 1, md: 2, xl: 3 }}>
        <MetricCard label="Server connection" value="Health visible" status="ready" />
        <MetricCard label="Evidence workflow" value="Draft stages" status="pending" />
        <MetricCard label="Plugin boundary" value="Contracts drafted" status="ready" />
      </SimpleGrid>
      <Paper p="md" className="surface">
        <Group justify="space-between" align="flex-start">
          <Box>
            <Title order={4}>Document evidence stages</Title>
            <Text c="dimmed" size="sm">
              Detect, parse, review, human confirm, and save remain separated.
            </Text>
          </Box>
          <Badge variant="light" color="blue">
            scaffold
          </Badge>
        </Group>
        <SimpleGrid cols={{ base: 1, md: 5 }} mt="md">
          {['Detect', 'Parse', 'Review', 'Human confirm', 'Save'].map((stage) => (
            <Box key={stage} className="stage-box">
              <Text fw={700} size="sm">
                {stage}
              </Text>
              <Text c="dimmed" size="xs">
                Waiting for Phase 2 data
              </Text>
            </Box>
          ))}
        </SimpleGrid>
      </Paper>
    </Stack>
  );
}
