import {
  ActionIcon,
  AppShell,
  Badge,
  Box,
  Button,
  Card,
  Divider,
  Group,
  Indicator,
  NavLink,
  Paper,
  SimpleGrid,
  Stack,
  Text,
  TextInput,
  Title,
} from '@mantine/core';
import { notifications } from '@mantine/notifications';
import {
  IconBuildingCommunity,
  IconChecklist,
  IconCloudCheck,
  IconFileText,
  IconHomeStats,
  IconPlugConnected,
  IconRefresh,
  IconRobot,
  IconSettings,
} from '@tabler/icons-react';
import {
  createRootRoute,
  createRoute,
  createRouter,
  Link,
  Outlet,
} from '@tanstack/react-router';
import { useEffect, useMemo, useState } from 'react';
import {
  AppConfig,
  checkServerHealth,
  getAppConfig,
  listPlugins,
  PluginStatus,
  ServerHealth,
  updateAppConfig,
} from './api/tauri';

const navigationItems = [
  { label: 'Workbench', to: '/', icon: IconHomeStats },
  { label: 'Properties', to: '/properties', icon: IconBuildingCommunity },
  { label: 'Documents', to: '/documents', icon: IconFileText },
  { label: 'AI Review', to: '/ai-review', icon: IconRobot },
  { label: 'Tasks', to: '/tasks', icon: IconChecklist },
  { label: 'Integrations', to: '/integrations', icon: IconPlugConnected },
  { label: 'Settings', to: '/settings', icon: IconSettings },
] as const;

function ShellLayout() {
  const [health, setHealth] = useState<ServerHealth | null>(null);
  const [plugins, setPlugins] = useState<PluginStatus[]>([]);
  const [loadingHealth, setLoadingHealth] = useState(false);

  const refresh = async () => {
    setLoadingHealth(true);
    const [nextHealth, nextPlugins] = await Promise.all([
      checkServerHealth(),
      listPlugins(),
    ]);
    setHealth(nextHealth);
    setPlugins(nextPlugins);
    setLoadingHealth(false);
  };

  useEffect(() => {
    void refresh();
  }, []);

  const statusColor = getStatusColor(health?.overall ?? 'not_configured');

  return (
    <AppShell
      header={{ height: 56 }}
      navbar={{ width: 248, breakpoint: 'sm' }}
      aside={{ width: 320, breakpoint: 'lg', collapsed: { mobile: true } }}
      padding="md"
    >
      <AppShell.Header className="app-header">
        <Group h="100%" px="md" justify="space-between" wrap="nowrap">
          <Group gap="sm" wrap="nowrap">
            <Indicator color={statusColor} size={8} offset={4}>
              <Box className="app-mark">RE</Box>
            </Indicator>
            <Box>
              <Text fw={700} size="sm">
                Realestate Management
              </Text>
              <Text c="dimmed" size="xs">
                Desktop operations workbench
              </Text>
            </Box>
          </Group>
          <Group gap="xs" wrap="nowrap">
            <Badge color={statusColor} variant="light">
              {health?.overall ?? 'not configured'}
            </Badge>
            <ActionIcon
              aria-label="Refresh service status"
              variant="subtle"
              loading={loadingHealth}
              onClick={() => void refresh()}
            >
              <IconRefresh size={16} />
            </ActionIcon>
          </Group>
        </Group>
      </AppShell.Header>

      <AppShell.Navbar p="sm" className="app-navbar">
        <Stack gap={4}>
          {navigationItems.map((item) => (
            <NavLink
              key={item.to}
              component={Link}
              to={item.to}
              label={item.label}
              leftSection={<item.icon size={17} />}
              className="nav-item"
              activeOptions={{ exact: item.to === '/' }}
            />
          ))}
        </Stack>
      </AppShell.Navbar>

      <AppShell.Main>
        <Outlet />
      </AppShell.Main>

      <AppShell.Aside p="md" className="app-aside">
        <Stack gap="md">
          <StatusPanel health={health} onRefresh={() => void refresh()} />
          <PluginPanel plugins={plugins} />
        </Stack>
      </AppShell.Aside>
    </AppShell>
  );
}

function PageHeader(props: { title: string; eyebrow: string; children?: React.ReactNode }) {
  return (
    <Group justify="space-between" align="flex-start" mb="md">
      <Box>
        <Text tt="uppercase" size="xs" fw={700} c="dimmed">
          {props.eyebrow}
        </Text>
        <Title order={2}>{props.title}</Title>
      </Box>
      {props.children}
    </Group>
  );
}

function WorkbenchPage() {
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

function PropertiesPage() {
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

function DocumentsPage() {
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

function AiReviewPage() {
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

function TasksPage() {
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

function IntegrationsPage() {
  const [plugins, setPlugins] = useState<PluginStatus[]>([]);

  useEffect(() => {
    void listPlugins().then(setPlugins);
  }, []);

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

function SettingsPage() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [serverBaseUrl, setServerBaseUrl] = useState('');
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    void getAppConfig().then((nextConfig) => {
      setConfig(nextConfig);
      setServerBaseUrl(nextConfig.server.baseUrl);
    });
  }, []);

  const save = async () => {
    setSaving(true);
    try {
      const nextConfig = await updateAppConfig({ serverBaseUrl });
      setConfig(nextConfig);
      notifications.show({
        title: 'Settings saved',
        message: 'Server configuration updated for this app session.',
        color: 'green',
      });
    } catch (error) {
      notifications.show({
        title: 'Settings failed',
        message: error instanceof Error ? error.message : String(error),
        color: 'red',
      });
    } finally {
      setSaving(false);
    }
  };

  return (
    <Stack gap="md">
      <PageHeader title="Settings" eyebrow="Local app config">
        <Button onClick={() => void save()} loading={saving}>
          Save
        </Button>
      </PageHeader>
      <Paper p="md" className="surface">
        <Stack gap="md" maw={560}>
          <TextInput
            label="Server base URL"
            value={serverBaseUrl}
            onChange={(event) => setServerBaseUrl(event.currentTarget.value)}
            placeholder="http://192.168.1.6:8080"
          />
          <Divider />
          <Group gap="sm">
            <Badge color={config?.plugins.saydoEnabled ? 'green' : 'gray'} variant="light">
              SayDo {config?.plugins.saydoEnabled ? 'enabled' : 'disabled'}
            </Badge>
            <Badge
              color={config?.plugins.projectManagerEnabled ? 'green' : 'gray'}
              variant="light"
            >
              Project-Manager {config?.plugins.projectManagerEnabled ? 'enabled' : 'disabled'}
            </Badge>
          </Group>
        </Stack>
      </Paper>
    </Stack>
  );
}

function StatusPanel(props: { health: ServerHealth | null; onRefresh: () => void }) {
  const health = props.health;
  const services = health?.services ?? [];

  return (
    <Card className="surface-card">
      <Group justify="space-between" mb="xs">
        <Text fw={700}>Service health</Text>
        <ActionIcon aria-label="Refresh service status" variant="subtle" onClick={props.onRefresh}>
          <IconRefresh size={16} />
        </ActionIcon>
      </Group>
      <Text c="dimmed" size="xs" mb="sm">
        {health?.baseUrl ?? 'No server configured'}
      </Text>
      <Stack gap={8}>
        {services.length === 0 ? (
          <Text c="dimmed" size="sm">
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
        <Text c="red.3" size="xs" mt="sm">
          {health.error}
        </Text>
      ) : null}
    </Card>
  );
}

function PluginPanel(props: { plugins: PluginStatus[] }) {
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

function MetricCard(props: { label: string; value: string; status: 'ready' | 'pending' }) {
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

function EmptyOperationalState(props: { title: string; detail: string }) {
  return (
    <Paper p="lg" className="surface">
      <Stack gap="xs">
        <Title order={4}>{props.title}</Title>
        <Text c="dimmed" size="sm">
          {props.detail}
        </Text>
      </Stack>
    </Paper>
  );
}

function getStatusColor(status: string) {
  if (status === 'ok' || status === 'ready') {
    return 'green';
  }

  if (status === 'degraded' || status === 'not_configured' || status === 'pending') {
    return 'yellow';
  }

  if (status === 'disabled') {
    return 'gray';
  }

  return 'red';
}

const rootRoute = createRootRoute({
  component: ShellLayout,
});

const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/',
  component: WorkbenchPage,
});

const propertiesRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/properties',
  component: PropertiesPage,
});

const documentsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/documents',
  component: DocumentsPage,
});

const aiReviewRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/ai-review',
  component: AiReviewPage,
});

const tasksRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/tasks',
  component: TasksPage,
});

const integrationsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/integrations',
  component: IntegrationsPage,
});

const settingsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/settings',
  component: SettingsPage,
});

const routeTree = rootRoute.addChildren([
  indexRoute,
  propertiesRoute,
  documentsRoute,
  aiReviewRoute,
  tasksRoute,
  integrationsRoute,
  settingsRoute,
]);

export const router = createRouter({ routeTree });

declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router;
  }
}
