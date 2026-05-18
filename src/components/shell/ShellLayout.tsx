import {
  Badge,
  Box,
  Group,
  Text,
} from '@mantine/core';
import type { Icon as TablerIcon } from '@tabler/icons-react';
import { IconShieldCheck } from '@tabler/icons-react';
import { Link, Outlet, useRouterState } from '@tanstack/react-router';
import { getStatusColor } from '../../lib/status';
import { AppDataProvider, useAppData } from './appData';
import { type NavigationItem, navigationItems } from './navigation';
import { StatusPanel } from './StatusPanel';
import { PluginPanel } from './PluginPanel';

export function ShellLayout() {
  return (
    <AppDataProvider>
      <ShellFrame />
    </AppDataProvider>
  );
}

function ShellFrame() {
  const { health, plugins, refreshAppData } = useAppData();
  const statusColor = getStatusColor(health?.overall ?? 'not_configured');

  return (
    <Box className="app-root">
      <div className="app-grid-bg" />
      <div className="app-grid-overlay" />
      <div className="app-shell-grid">
        <aside className="app-rail">
          <div className="app-rail-mark-wrap">
            <div className="app-rail-mark">
              RE
            </div>
          </div>
          <nav className="app-rail-nav">
            {navigationItems.map((item) => (
              <RailLink key={item.to} item={item} />
            ))}
          </nav>
          <div className="app-rail-footer">
            <div className="app-rail-status" title={`Bridge: ${statusColor}`}>
              <IconShieldCheck size={16} />
              <Badge
                size="xs"
                color={statusColor}
                variant="filled"
                className="app-rail-status-badge"
              >
                {health?.overall ?? 'offline'}
              </Badge>
            </div>
          </div>
        </aside>

        <div className="app-main-col">
          <header className="app-header">
            <Group h="100%" px="md" justify="space-between" wrap="nowrap">
              <Group gap="sm" wrap="nowrap">
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
              </Group>
            </Group>
          </header>

          <main className="app-content">
            <Outlet />
          </main>

          <aside className="app-panels">
            <StatusPanel health={health} onRefresh={() => void refreshAppData()} />
            <PluginPanel plugins={plugins} />
          </aside>
        </div>
      </div>
    </Box>
  );
}

function RailLink(props: { item: NavigationItem }) {
  const { item } = props;
  const Icon = item.icon;
  const routerState = useRouterState();
  const currentPath = routerState.location.pathname;
  const isActive = item.to === '/'
    ? currentPath === '/'
    : currentPath.startsWith(item.to);

  return (
    <Link
      to={item.to}
      className={
        isActive
          ? 'app-rail-item app-rail-item--active'
          : 'app-rail-item'
      }
      title={item.hint ?? item.label}
    >
      <Icon size={17} />
    </Link>
  );
}
