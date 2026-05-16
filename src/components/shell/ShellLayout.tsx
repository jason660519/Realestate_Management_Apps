import {
  ActionIcon,
  AppShell,
  Badge,
  Box,
  Group,
  Indicator,
  NavLink,
  Stack,
  Text,
} from '@mantine/core';
import { IconRefresh } from '@tabler/icons-react';
import { Link, Outlet } from '@tanstack/react-router';
import { getStatusColor } from '../../lib/status';
import { AppDataProvider, useAppData } from './appData';
import { navigationItems } from './navigation';
import { PluginPanel } from './PluginPanel';
import { StatusPanel } from './StatusPanel';

export function ShellLayout() {
  return (
    <AppDataProvider>
      <ShellFrame />
    </AppDataProvider>
  );
}

function ShellFrame() {
  const { health, plugins, loadingHealth, refreshAppData } = useAppData();
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
              onClick={() => void refreshAppData()}
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
          <StatusPanel health={health} onRefresh={() => void refreshAppData()} />
          <PluginPanel plugins={plugins} />
        </Stack>
      </AppShell.Aside>
    </AppShell>
  );
}
