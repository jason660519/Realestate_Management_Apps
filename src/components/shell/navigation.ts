import {
  IconBuildingCommunity,
  IconChecklist,
  IconFileText,
  IconHomeStats,
  IconPlugConnected,
  IconRobot,
  IconSettings,
} from '@tabler/icons-react';

import type { Icon } from '@tabler/icons-react';

export interface NavigationItem {
  label: string;
  to: string;
  icon: Icon;
  hint: string;
}

export const navigationItems: NavigationItem[] = [
  { label: 'Workbench', to: '/', icon: IconHomeStats, hint: 'Operational overview and document evidence stages.' },
  { label: 'Properties', to: '/properties', icon: IconBuildingCommunity, hint: 'Browse and manage property records.' },
  { label: 'Documents', to: '/documents', icon: IconFileText, hint: 'Intake, parse, and organize property documents.' },
  { label: 'AI Review', to: '/ai-review', icon: IconRobot, hint: 'AI-generated extraction review with source and confidence.' },
  { label: 'Tasks', to: '/tasks', icon: IconChecklist, hint: 'Coordinate property and document work items.' },
  { label: 'Integrations', to: '/integrations', icon: IconPlugConnected, hint: 'Plugin contracts for SayDo, Project-Manager, and services.' },
  { label: 'Settings', to: '/settings', icon: IconSettings, hint: 'Local app configuration and storage diagnostics.' },
];
