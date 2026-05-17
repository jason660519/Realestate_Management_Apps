import {
  Badge,
  Button,
  Divider,
  Group,
  Paper,
  Stack,
  Switch,
  Text,
  TextInput,
  Title,
} from '@mantine/core';
import { notifications } from '@mantine/notifications';
import { useEffect, useState } from 'react';
import {
  AppConfig,
  getAppConfig,
  getStorageDiagnostics,
  StorageDiagnostics,
  updateAppConfig,
} from '../api/tauri';
import { PageHeader } from '../components/PageHeader';
import { useAppData } from '../components/shell/appData';

export function SettingsPage() {
  const { refreshAppData } = useAppData();
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [serverBaseUrl, setServerBaseUrl] = useState('');
  const [saydoEnabled, setSaydoEnabled] = useState(false);
  const [projectManagerEnabled, setProjectManagerEnabled] = useState(false);
  const [storageDiagnostics, setStorageDiagnostics] =
    useState<StorageDiagnostics | null>(null);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    void Promise.all([getAppConfig(), getStorageDiagnostics()]).then(
      ([nextConfig, nextStorageDiagnostics]) => {
        setConfig(nextConfig);
        setServerBaseUrl(nextConfig.server.baseUrl);
        setSaydoEnabled(nextConfig.plugins.saydoEnabled);
        setProjectManagerEnabled(nextConfig.plugins.projectManagerEnabled);
        setStorageDiagnostics(nextStorageDiagnostics);
      },
    );
  }, []);

  const save = async () => {
    setSaving(true);
    try {
      const nextConfig = await updateAppConfig({
        serverBaseUrl,
        saydoEnabled,
        projectManagerEnabled,
      });
      setConfig(nextConfig);
      setServerBaseUrl(nextConfig.server.baseUrl);
      setSaydoEnabled(nextConfig.plugins.saydoEnabled);
      setProjectManagerEnabled(nextConfig.plugins.projectManagerEnabled);
      setStorageDiagnostics(await getStorageDiagnostics());
      await refreshAppData();
      notifications.show({
        title: 'Settings saved',
        message: 'Server and plugin configuration saved to local app data.',
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
          <Stack gap="sm">
            <Switch
              label="SayDo"
              description="Allow draft text handoff from SayDo after plugin transport is implemented."
              checked={saydoEnabled}
              onChange={(event) => setSaydoEnabled(event.currentTarget.checked)}
            />
            <Switch
              label="Project-Manager"
              description="Allow pending task export after plugin transport is implemented."
              checked={projectManagerEnabled}
              onChange={(event) => setProjectManagerEnabled(event.currentTarget.checked)}
            />
          </Stack>
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
      <StorageDiagnosticsPanel diagnostics={storageDiagnostics} />
    </Stack>
  );
}

function StorageDiagnosticsPanel(props: { diagnostics: StorageDiagnostics | null }) {
  const diagnostics = props.diagnostics;

  return (
    <Paper p="md" className="surface">
      <Stack gap="sm">
        <Group justify="space-between" align="flex-start">
          <Title order={4}>Local storage</Title>
          <Badge
            color={diagnostics?.configReadable ? 'green' : 'yellow'}
            variant="light"
          >
            {diagnostics?.configReadable ? 'readable' : 'pending'}
          </Badge>
        </Group>
        <StorageRow label="App data directory" value={diagnostics?.appDataDir} />
        <StorageRow label="Config file" value={diagnostics?.configPath} />
        <Group gap="sm">
          <Badge color={diagnostics?.configExists ? 'green' : 'gray'} variant="light">
            {diagnostics?.configExists ? 'config exists' : 'config missing'}
          </Badge>
          <Badge color={diagnostics?.configReadable ? 'green' : 'yellow'} variant="light">
            {diagnostics?.configReadable ? 'config readable' : 'not readable'}
          </Badge>
          {typeof diagnostics?.configFileBytes === 'number' ? (
            <Badge color="blue" variant="light">
              {diagnostics.configFileBytes} bytes
            </Badge>
          ) : null}
        </Group>
        {diagnostics?.error ? (
          <Text c="red.3" size="sm">
            {diagnostics.error}
          </Text>
        ) : null}
      </Stack>
    </Paper>
  );
}

function StorageRow(props: { label: string; value?: string }) {
  return (
    <Stack gap={2}>
      <Text c="dimmed" size="xs" fw={700} tt="uppercase">
        {props.label}
      </Text>
      <Text size="sm" className="path-text">
        {props.value ?? 'Loading...'}
      </Text>
    </Stack>
  );
}
