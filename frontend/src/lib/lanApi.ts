import { handleApiResponse, makeRequest } from './apiCore';
import type { LanEnableRequest, LanPairing, LanStatus } from '../../../shared/types';

export const isLocalDesktopOrigin = () =>
  typeof window !== 'undefined' && ['localhost', '127.0.0.1', '[::1]', '::1'].includes(window.location.hostname);

export const lanApi = {
  status: async (): Promise<LanStatus> => handleApiResponse(await makeRequest('/api/lan', { cache: 'no-store' })),
  enable: async (request: LanEnableRequest): Promise<LanPairing> => handleApiResponse(await makeRequest('/api/lan/enable', { method: 'POST', body: JSON.stringify(request) })),
  disable: async (): Promise<LanStatus> => handleApiResponse(await makeRequest('/api/lan/disable', { method: 'POST' })),
  rotate: async (): Promise<LanPairing> => handleApiResponse(await makeRequest('/api/lan/rotate', { method: 'POST' })),
};
