import { Stack } from '@mantine/core';
import { EmptyOperationalState } from '../components/EmptyOperationalState';
import { PageHeader } from '../components/PageHeader';

export function AiReviewPage() {
  return (
    <Stack gap="md">
      <PageHeader title="AI Review" eyebrow="Evidence first" />
      <EmptyOperationalState
        title="No processing runs are active"
        detail="Failed model/provider state, confidence, evidence source, and retry paths will appear here."
      />
    </Stack>
  );
}
