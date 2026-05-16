import { Stack } from '@mantine/core';
import { EmptyOperationalState } from '../components/EmptyOperationalState';
import { PageHeader } from '../components/PageHeader';

export function TasksPage() {
  return (
    <Stack gap="md">
      <PageHeader title="Tasks" eyebrow="Project-Manager contract" />
      <EmptyOperationalState
        title="No exported tasks"
        detail="Task export starts as a local pending record until Project-Manager acknowledges the payload."
      />
    </Stack>
  );
}
