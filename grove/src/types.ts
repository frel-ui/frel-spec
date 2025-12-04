// API response types matching frel-compiler-server/src/api.rs

export interface StatusResponse {
  initialized: boolean;
  error_count: number;
  module_count: number;
}

export interface ModuleInfo {
  path: string;
  source_files: string[];
  has_errors: boolean;
  error_count: number;
  warning_count: number;
}

export interface ModulesResponse {
  modules: ModuleInfo[];
}

export interface DiagnosticInfo {
  severity: string;
  code: string | null;
  message: string;
  file: string | null;
  line: number | null;
  column: number | null;
}

export interface DiagnosticsResponse {
  module: string | null;
  diagnostics: DiagnosticInfo[];
  error_count: number;
  warning_count: number;
}

export interface AstResponse {
  module: string;
  ast: unknown;
  dump: string;
}

export interface GeneratedResponse {
  module: string;
  javascript: string;
}

// === Scope types ===

export interface SymbolInfo {
  id: number;
  name: string;
  kind: string;
  body_scope: number | null;
  source_module: string | null;
}

export interface ScopeInfo {
  id: number;
  kind: string;
  parent: number | null;
  name: string | null;
  children: number[];
  symbols: SymbolInfo[];
}

export interface ScopeResponse {
  module: string;
  scopes: ScopeInfo[];
}

export interface SourceResponse {
  path: string;
  content: string;
  module: string | null;
}

export interface NotifyRequest {
  path: string;
}

export interface NotifyResponse {
  success: boolean;
  modules_rebuilt: string[];
  duration_ms: number;
  error_count: number;
}

export interface WriteRequest {
  path: string;
  content: string;
}

export interface WriteResponse {
  success: boolean;
  modules_rebuilt: string[];
  duration_ms: number;
  error_count: number;
}

// === Expectations types (compiler dev mode) ===

export interface ModuleExpectations {
  module: string;
  ast: unknown | null;
  diagnostics: DiagnosticInfo[];
  generated_js: string | null;
}

export interface ExpectationsResponse {
  module: string;
  exists: boolean;
  expectations: ModuleExpectations | null;
}

export interface SaveExpectationsResponse {
  success: boolean;
  module: string;
}

export interface CompareResponse {
  module: string;
  has_differences: boolean;
  ast_matches: boolean;
  diagnostics_match: boolean;
  generated_js_matches: boolean;
  current: ModuleExpectations;
  expected: ModuleExpectations | null;
}

// UI state types

export type ConnectionStatus = 'disconnected' | 'connecting' | 'connected' | 'error';

export type OutputTab = 'diagnostics' | 'ast' | 'scope' | 'generated';

export type DevMode = 'normal' | 'compiler-dev';

export interface FileNode {
  name: string;
  path: string;
  isDirectory: boolean;
  children?: FileNode[];
  module?: string;
  hasErrors?: boolean;
  errorCount?: number;
}
