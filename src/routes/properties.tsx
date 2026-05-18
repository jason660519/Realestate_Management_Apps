import {
  Alert,
  Badge,
  Button,
  Group,
  Paper,
  Skeleton,
  Stack,
  Table,
  Text,
} from '@mantine/core';
import { IconAlertCircle, IconCloudOff, IconRefresh } from '@tabler/icons-react';
import { useQuery } from '@tanstack/react-query';
import {
  listPropertySummaries,
  PropertyKind,
  PropertyStatus,
  PropertySummariesResult,
  PropertySummary,
} from '../api/tauri';
import { EmptyOperationalState } from '../components/EmptyOperationalState';
import { PageHeader } from '../components/PageHeader';
import { useAppData } from '../components/shell/appData';

export function PropertiesPage() {
  const { health } = useAppData();
  const serverConfigured = (health?.overall ?? 'not_configured') !== 'not_configured';
  return <PropertiesView serverConfigured={serverConfigured} />;
}

export function PropertiesView(props: { serverConfigured: boolean }) {
  const query = useQuery({
    queryKey: ['property-summaries'],
    queryFn: listPropertySummaries,
  });

  return (
    <Stack gap="md">
      <PageHeader title="Properties" eyebrow="Core data">
        <Button
          variant="default"
          leftSection={<IconRefresh size={14} />}
          onClick={() => void query.refetch()}
          loading={query.isFetching}
        >
          Refresh
        </Button>
      </PageHeader>
      <PropertiesBody
        serverConfigured={props.serverConfigured}
        query={query}
      />
    </Stack>
  );
}

type PropertiesQuery = ReturnType<typeof useQuery<PropertySummariesResult>>;

function PropertiesBody(props: {
  serverConfigured: boolean;
  query: PropertiesQuery;
}) {
  const { query, serverConfigured } = props;

  if (query.isPending) {
    return (
      <Paper p="md" className="surface">
        <Stack gap="xs">
          <Skeleton height={20} radius={4} />
          <Skeleton height={20} radius={4} />
          <Skeleton height={20} radius={4} />
        </Stack>
      </Paper>
    );
  }

  if (query.isError) {
    return (
      <Alert
        icon={<IconAlertCircle size={16} />}
        color="red"
        variant="light"
        title="Could not load properties"
      >
        <Stack gap="xs">
          <Text size="sm">
            The desktop app preserved its cached view; nothing was changed on the server.
            Reason: {errorMessage(query.error)}
          </Text>
          <Group gap="xs">
            <Button size="xs" variant="default" onClick={() => void query.refetch()}>
              Retry
            </Button>
          </Group>
        </Stack>
      </Alert>
    );
  }

  const result = query.data;
  if (!result) {
    return null;
  }

  if (result.source === 'empty') {
    if (!serverConfigured) {
      return (
        <EmptyOperationalState
          title="Server URL is not configured"
          detail="Open Settings to point the desktop app at the internal server. Property data will load once a base URL is saved and the server is reachable."
        />
      );
    }
    return (
      <EmptyOperationalState
        title="No properties yet"
        detail={
          result.error
            ? `Server reachable but the cache is empty. Last error: ${result.error}`
            : 'The server is reachable but no property records have been created. Add one from the desktop app or import a document to seed the workspace.'
        }
      />
    );
  }

  return (
    <Stack gap="md">
      {result.source === 'cache' ? (
        <StaleCacheBanner
          lastSyncedAt={result.lastSyncedAt}
          reason={result.error}
        />
      ) : null}
      <PropertiesTable rows={result.rows} />
    </Stack>
  );
}

function StaleCacheBanner(props: {
  lastSyncedAt: string | null;
  reason: string | null;
}) {
  return (
    <Alert
      icon={<IconCloudOff size={16} />}
      color="yellow"
      variant="light"
      title="Showing cached property list"
    >
      <Stack gap={4}>
        <Text size="sm">
          The server is unreachable, so the desktop app fell back to its local SQLite cache.
          Confirmation actions and saves are disabled until the server is back.
        </Text>
        <Text size="xs" c="dimmed">
          Last synced: {formatTimestamp(props.lastSyncedAt) ?? 'never'}
          {props.reason ? ` · Reason: ${props.reason}` : ''}
        </Text>
      </Stack>
    </Alert>
  );
}

function PropertiesTable(props: { rows: PropertySummary[] }) {
  return (
    <Paper className="surface" p="xs">
      <Table verticalSpacing="xs" highlightOnHover>
        <Table.Thead>
          <Table.Tr>
            <Table.Th style={{ width: 96 }}>Status</Table.Th>
            <Table.Th>Display name</Table.Th>
            <Table.Th style={{ width: 110 }}>Kind</Table.Th>
            <Table.Th>Address</Table.Th>
            <Table.Th style={{ width: 160 }}>Updated</Table.Th>
          </Table.Tr>
        </Table.Thead>
        <Table.Tbody>
          {props.rows.map((row) => (
            <Table.Tr key={row.id}>
              <Table.Td>
                <Badge color={statusColor(row.status)} variant="light">
                  {formatStatus(row.status)}
                </Badge>
              </Table.Td>
              <Table.Td>
                <Text size="sm" fw={600}>
                  {row.display_name}
                </Text>
              </Table.Td>
              <Table.Td>
                <Badge variant="outline" color="gray">
                  {formatKind(row.kind)}
                </Badge>
              </Table.Td>
              <Table.Td>
                <Text size="sm" c="dimmed">
                  {row.address_raw ?? '—'}
                </Text>
              </Table.Td>
              <Table.Td>
                <Text size="xs" c="dimmed">
                  {formatTimestamp(row.updated_at) ?? '—'}
                </Text>
              </Table.Td>
            </Table.Tr>
          ))}
        </Table.Tbody>
      </Table>
    </Paper>
  );
}

function statusColor(status: PropertyStatus): string {
  switch (status) {
    case 'active':
      return 'green';
    case 'draft':
      return 'gray';
    case 'pending':
      return 'yellow';
    case 'archived':
      return 'dark';
    case 'unknown':
    default:
      return 'red';
  }
}

const KIND_LABELS: Record<PropertyKind, string> = {
  sale: 'Sale',
  rental: 'Rental',
  land_only: 'Land',
  commercial: 'Commercial',
  unknown: 'Unknown',
};

const STATUS_LABELS: Record<PropertyStatus, string> = {
  draft: 'Draft',
  active: 'Active',
  pending: 'Pending',
  archived: 'Archived',
  unknown: 'Unknown',
};

function formatKind(kind: PropertyKind): string {
  return KIND_LABELS[kind] ?? 'Unknown';
}

function formatStatus(status: PropertyStatus): string {
  return STATUS_LABELS[status] ?? 'Unknown';
}

function formatTimestamp(value: string | null): string | null {
  if (!value) return null;
  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) return value;
  return parsed.toLocaleString();
}

function errorMessage(error: unknown): string {
  if (error instanceof Error) return error.message;
  if (typeof error === 'string') return error;
  if (error && typeof error === 'object' && 'message' in error) {
    const message = (error as { message?: unknown }).message;
    if (typeof message === 'string') return message;
  }
  return String(error);
}
