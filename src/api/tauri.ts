import { invoke } from '@tauri-apps/api/core';

export type AppConfig = {
  server: {
    baseUrl: string;
    healthCheckIntervalSec: number;
    timeoutSec: number;
  };
  plugins: {
    saydoEnabled: boolean;
    projectManagerEnabled: boolean;
  };
};

export type AppConfigPatch = {
  serverBaseUrl?: string;
  saydoEnabled?: boolean;
  projectManagerEnabled?: boolean;
};

export type HealthService = {
  name: string;
  status: 'ok' | 'fail' | 'not_configured';
  latencyMs?: number;
  error?: string;
};

export type ServerHealth = {
  overall: 'ok' | 'degraded' | 'offline' | 'not_configured';
  checkedAt: string;
  baseUrl: string;
  services: HealthService[];
  error?: string;
};

export type PluginStatus = {
  id: 'saydo' | 'project_manager';
  name: string;
  enabled: boolean;
  permissionScope: string[];
  status: 'disabled' | 'ready' | 'degraded';
  lastHandshakeAt?: string;
  lastError?: string;
};

const fallbackConfig: AppConfig = {
  server: {
    baseUrl: 'http://192.168.1.6:8080',
    healthCheckIntervalSec: 30,
    timeoutSec: 10,
  },
  plugins: {
    saydoEnabled: false,
    projectManagerEnabled: false,
  },
};

function isTauriRuntime() {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

export async function getAppConfig(): Promise<AppConfig> {
  if (!isTauriRuntime()) {
    return fallbackConfig;
  }

  return invoke<AppConfig>('get_app_config').catch(() => fallbackConfig);
}

export async function updateAppConfig(patch: AppConfigPatch): Promise<AppConfig> {
  if (!isTauriRuntime()) {
    return {
      ...fallbackConfig,
      server: {
        ...fallbackConfig.server,
        baseUrl: patch.serverBaseUrl ?? fallbackConfig.server.baseUrl,
      },
      plugins: {
        saydoEnabled: patch.saydoEnabled ?? fallbackConfig.plugins.saydoEnabled,
        projectManagerEnabled:
          patch.projectManagerEnabled ?? fallbackConfig.plugins.projectManagerEnabled,
      },
    };
  }

  return invoke<AppConfig>('update_app_config', { patch });
}

export async function checkServerHealth(): Promise<ServerHealth> {
  if (!isTauriRuntime()) {
    return {
      overall: 'offline',
      checkedAt: new Date().toISOString(),
      baseUrl: fallbackConfig.server.baseUrl,
      services: [],
      error: 'Preview mode: Tauri commands are available in the desktop runtime.',
    };
  }

  return invoke<ServerHealth>('check_server_health').catch((error: unknown) => ({
    overall: 'offline',
    checkedAt: new Date().toISOString(),
    baseUrl: fallbackConfig.server.baseUrl,
    services: [],
    error: error instanceof Error ? error.message : String(error),
  }));
}

export async function listPlugins(): Promise<PluginStatus[]> {
  if (!isTauriRuntime()) {
    return fallbackPlugins();
  }

  return invoke<PluginStatus[]>('list_plugins').catch(() => fallbackPlugins());
}

function fallbackPlugins(): PluginStatus[] {
  return [
    {
      id: 'saydo',
      name: 'SayDo',
      enabled: false,
      permissionScope: ['text handoff draft only'],
      status: 'disabled',
    },
    {
      id: 'project_manager',
      name: 'Project-Manager',
      enabled: false,
      permissionScope: ['task export pending sync'],
      status: 'disabled',
    },
  ];
}
