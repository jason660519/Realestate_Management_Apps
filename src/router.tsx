import { createRootRoute, createRoute, createRouter } from '@tanstack/react-router';
import { ShellLayout } from './components/shell/ShellLayout';
import { AiReviewPage } from './routes/ai-review';
import { DocumentsPage } from './routes/documents';
import { IntegrationsPage } from './routes/integrations';
import { PropertiesPage } from './routes/properties';
import { SettingsPage } from './routes/settings';
import { TasksPage } from './routes/tasks';
import { WorkbenchPage } from './routes/workbench';

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
