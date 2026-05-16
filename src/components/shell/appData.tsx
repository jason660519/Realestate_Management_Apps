import { createContext, ReactNode, useContext, useEffect, useState } from 'react';
import {
  checkServerHealth,
  listPlugins,
  PluginStatus,
  ServerHealth,
} from '../../api/tauri';

type AppDataContextValue = {
  health: ServerHealth | null;
  plugins: PluginStatus[];
  loadingHealth: boolean;
  refreshAppData: () => Promise<void>;
};

const AppDataContext = createContext<AppDataContextValue | null>(null);

export function AppDataProvider(props: { children: ReactNode }) {
  const [health, setHealth] = useState<ServerHealth | null>(null);
  const [plugins, setPlugins] = useState<PluginStatus[]>([]);
  const [loadingHealth, setLoadingHealth] = useState(false);

  const refreshAppData = async () => {
    setLoadingHealth(true);
    try {
      const [nextHealth, nextPlugins] = await Promise.all([
        checkServerHealth(),
        listPlugins(),
      ]);
      setHealth(nextHealth);
      setPlugins(nextPlugins);
    } finally {
      setLoadingHealth(false);
    }
  };

  useEffect(() => {
    void refreshAppData();
  }, []);

  return (
    <AppDataContext.Provider
      value={{ health, plugins, loadingHealth, refreshAppData }}
    >
      {props.children}
    </AppDataContext.Provider>
  );
}

export function useAppData() {
  const value = useContext(AppDataContext);

  if (!value) {
    throw new Error('useAppData must be used within AppDataProvider');
  }

  return value;
}
