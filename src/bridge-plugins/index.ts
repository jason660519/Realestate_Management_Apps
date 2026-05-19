import { registerAllCapabilities, listCapabilitiesByApp, OpenClawBridge, AppHealth } from '@jason66/shared-bridge';

export { registerAllCapabilities, listCapabilitiesByApp, OpenClawBridge };
export type { AppHealth };

registerAllCapabilities();

export function getOpenClawBridge(): OpenClawBridge {
  return new OpenClawBridge('http://127.0.0.1:18790');
}
