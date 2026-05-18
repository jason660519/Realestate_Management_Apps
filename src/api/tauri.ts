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

export type StorageDiagnostics = {
  appDataDir: string;
  configPath: string;
  configExists: boolean;
  configReadable: boolean;
  configFileBytes?: number;
  error?: string;
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

export type PropertyKind = 'sale' | 'rental' | 'land_only' | 'commercial' | 'unknown';
export type PropertyStatus = 'draft' | 'active' | 'pending' | 'archived' | 'unknown';

// snake_case here is deliberate: this surface deserializes PostgREST responses
// straight through. See `docs/architecture/property-document-boundary.md` for
// the boundary note. Other typed surfaces (AppConfig, ServerHealth, …) stay
// camelCase.
export type PropertySummary = {
  id: string;
  display_name: string;
  kind: PropertyKind;
  status: PropertyStatus;
  address_raw: string | null;
  updated_at: string | null;
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

const fallbackStorageDiagnostics: StorageDiagnostics = {
  appDataDir: 'Preview mode',
  configPath: 'Preview mode: config.toml is created by the Tauri desktop runtime.',
  configExists: false,
  configReadable: false,
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

export async function getStorageDiagnostics(): Promise<StorageDiagnostics> {
  if (!isTauriRuntime()) {
    return fallbackStorageDiagnostics;
  }

  return invoke<StorageDiagnostics>('get_storage_diagnostics').catch((error: unknown) => ({
    ...fallbackStorageDiagnostics,
    error: error instanceof Error ? error.message : String(error),
  }));
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

export async function listPropertySummaries(): Promise<PropertySummary[]> {
  if (!isTauriRuntime()) {
    return fallbackPropertySummaries();
  }

  return invoke<PropertySummary[]>('list_property_summaries');
}

function fallbackPropertySummaries(): PropertySummary[] {
  return [
    {
      id: 'preview-1',
      display_name: 'Preview · 內湖 4 房',
      kind: 'sale',
      status: 'active',
      address_raw: 'Preview mode: real property data is fetched in the Tauri desktop runtime.',
      updated_at: null,
    },
    {
      id: 'preview-2',
      display_name: 'Preview · 信義 套房',
      kind: 'rental',
      status: 'draft',
      address_raw: null,
      updated_at: null,
    },
  ];
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
