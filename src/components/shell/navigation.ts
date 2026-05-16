import {
  IconBuildingCommunity,
  IconChecklist,
  IconFileText,
  IconHomeStats,
  IconPlugConnected,
  IconRobot,
  IconSettings,
} from '@tabler/icons-react';

export const navigationItems = [
  { label: 'Workbench', to: '/', icon: IconHomeStats },
  { label: 'Properties', to: '/properties', icon: IconBuildingCommunity },
  { label: 'Documents', to: '/documents', icon: IconFileText },
  { label: 'AI Review', to: '/ai-review', icon: IconRobot },
  { label: 'Tasks', to: '/tasks', icon: IconChecklist },
  { label: 'Integrations', to: '/integrations', icon: IconPlugConnected },
  { label: 'Settings', to: '/settings', icon: IconSettings },
] as const;
