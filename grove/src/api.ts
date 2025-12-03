// HTTP client for frel-compiler-server

import type {
  StatusResponse,
  ModulesResponse,
  DiagnosticsResponse,
  AstResponse,
  GeneratedResponse,
  ScopeResponse,
  SourceResponse,
  NotifyResponse,
  WriteResponse,
  ExpectationsResponse,
  SaveExpectationsResponse,
  CompareResponse,
} from './types';

const API_BASE = '/api';

async function fetchJson<T>(path: string, options?: RequestInit): Promise<T> {
  const response = await fetch(`${API_BASE}${path}`, options);
  if (!response.ok) {
    throw new Error(`API error: ${response.status} ${response.statusText}`);
  }
  return response.json();
}

export const api = {
  async getStatus(): Promise<StatusResponse> {
    return fetchJson('/status');
  },

  async getModules(): Promise<ModulesResponse> {
    return fetchJson('/modules');
  },

  async getDiagnostics(module?: string): Promise<DiagnosticsResponse> {
    const path = module
      ? `/diagnostics/${encodeURIComponent(module)}`
      : '/diagnostics';
    return fetchJson(path);
  },

  async getAst(module: string): Promise<AstResponse> {
    return fetchJson(`/ast/${encodeURIComponent(module)}`);
  },

  async getGenerated(module: string): Promise<GeneratedResponse> {
    return fetchJson(`/generated/${encodeURIComponent(module)}`);
  },

  async getScope(module: string): Promise<ScopeResponse> {
    return fetchJson(`/scope/${encodeURIComponent(module)}`);
  },

  async getSource(filePath: string): Promise<SourceResponse> {
    return fetchJson(`/source/${encodeURIComponent(filePath)}`);
  },

  async notify(filePath: string): Promise<NotifyResponse> {
    return fetchJson('/notify', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ path: filePath }),
    });
  },

  async write(filePath: string, content: string): Promise<WriteResponse> {
    return fetchJson('/write', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ path: filePath, content }),
    });
  },

  // Expectations endpoints (compiler dev mode)
  async getExpectations(module: string): Promise<ExpectationsResponse> {
    return fetchJson(`/expectations/${encodeURIComponent(module)}`);
  },

  async saveExpectations(module: string): Promise<SaveExpectationsResponse> {
    return fetchJson(`/expectations/${encodeURIComponent(module)}/save`, {
      method: 'POST',
    });
  },

  async compare(module: string): Promise<CompareResponse> {
    return fetchJson(`/compare/${encodeURIComponent(module)}`);
  },
};
