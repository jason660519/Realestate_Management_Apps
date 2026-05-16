import { Stack } from '@mantine/core';
import { EmptyOperationalState } from '../components/EmptyOperationalState';
import { PageHeader } from '../components/PageHeader';

export function DocumentsPage() {
  return (
    <Stack gap="md">
      <PageHeader title="Documents" eyebrow="Intake" />
      <EmptyOperationalState
        title="Document intake is gated behind server storage"
        detail="Original files, AI outputs, and processing runs will stay separated when Phase 2 storage is wired."
      />
    </Stack>
  );
}
