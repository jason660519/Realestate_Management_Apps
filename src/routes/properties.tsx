import { Stack } from '@mantine/core';
import { EmptyOperationalState } from '../components/EmptyOperationalState';
import { PageHeader } from '../components/PageHeader';

export function PropertiesPage() {
  return (
    <Stack gap="md">
      <PageHeader title="Properties" eyebrow="Core data" />
      <EmptyOperationalState
        title="No property workspace is connected yet"
        detail="Phase 2 will add property list, draft editing, and evidence-backed field review."
      />
    </Stack>
  );
}
