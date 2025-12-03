// Main state hook for compiler server interaction

import { useState, useEffect, useCallback, useRef } from 'react';
import { api } from '../api';
import type {
  ConnectionStatus,
  ModuleInfo,
  DiagnosticInfo,
  OutputTab,
  DevMode,
  CompareResponse,
  ScopeInfo,
} from '../types';

export interface CompilerState {
  // Connection
  status: ConnectionStatus;
  error: string | null;

  // Server state
  initialized: boolean;
  totalErrors: number;
  totalModules: number;

  // Modules
  modules: ModuleInfo[];
  selectedModule: string | null;
  selectedFile: string | null;

  // Source
  source: string;
  sourceLoading: boolean;
  isDirty: boolean;
  isCompiling: boolean;

  // Output panels
  activeTab: OutputTab;
  diagnostics: DiagnosticInfo[];
  ast: unknown | null;
  scopes: ScopeInfo[];
  generatedJs: string;

  // Compiler dev mode
  devMode: DevMode;
  comparison: CompareResponse | null;
  isSavingExpectations: boolean;

  // Actions
  refresh: () => Promise<void>;
  selectModule: (modulePath: string) => void;
  selectFile: (filePath: string) => void;
  setActiveTab: (tab: OutputTab) => void;
  updateSource: (content: string) => void;
  setDevMode: (mode: DevMode) => void;
  saveExpectations: () => Promise<void>;
  refreshComparison: () => Promise<void>;
}

export function useCompiler(): CompilerState {
  // Connection state
  const [status, setStatus] = useState<ConnectionStatus>('disconnected');
  const [error, setError] = useState<string | null>(null);

  // Server state
  const [initialized, setInitialized] = useState(false);
  const [totalErrors, setTotalErrors] = useState(0);
  const [totalModules, setTotalModules] = useState(0);

  // Modules
  const [modules, setModules] = useState<ModuleInfo[]>([]);
  const [selectedModule, setSelectedModule] = useState<string | null>(null);
  const [selectedFile, setSelectedFile] = useState<string | null>(null);

  // Source
  const [source, setSource] = useState('');
  const [sourceLoading, setSourceLoading] = useState(false);
  const [isDirty, setIsDirty] = useState(false);
  const [isCompiling, setIsCompiling] = useState(false);
  const originalSourceRef = useRef('');
  const debounceTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const selectedFileRef = useRef<string | null>(null);
  const selectedModuleRef = useRef<string | null>(null);
  const devModeRef = useRef<DevMode>('normal');

  // Keep refs in sync
  useEffect(() => {
    selectedFileRef.current = selectedFile;
  }, [selectedFile]);

  useEffect(() => {
    selectedModuleRef.current = selectedModule;
  }, [selectedModule]);

  // Output panels
  const [activeTab, setActiveTab] = useState<OutputTab>('diagnostics');
  const [diagnostics, setDiagnostics] = useState<DiagnosticInfo[]>([]);
  const [ast, setAst] = useState<unknown | null>(null);
  const [scopes, setScopes] = useState<ScopeInfo[]>([]);
  const [generatedJs, setGeneratedJs] = useState('');

  // Compiler dev mode
  const [devMode, setDevMode] = useState<DevMode>('normal');
  const [comparison, setComparison] = useState<CompareResponse | null>(null);
  const [isSavingExpectations, setIsSavingExpectations] = useState(false);

  useEffect(() => {
    devModeRef.current = devMode;
  }, [devMode]);

  // Fetch server status and modules
  const refresh = useCallback(async () => {
    setStatus('connecting');
    setError(null);

    try {
      const [statusRes, modulesRes] = await Promise.all([
        api.getStatus(),
        api.getModules(),
      ]);

      setInitialized(statusRes.initialized);
      setTotalErrors(statusRes.error_count);
      setTotalModules(statusRes.module_count);
      setModules(modulesRes.modules);
      setStatus('connected');
    } catch (err) {
      setStatus('error');
      setError(err instanceof Error ? err.message : 'Connection failed');
    }
  }, []);

  // Load module data when selected
  const selectModule = useCallback(async (modulePath: string, modulesList: ModuleInfo[] = modules) => {
    setSelectedModule(modulePath);

    try {
      const [diagRes, astRes, scopeRes, genRes] = await Promise.all([
        api.getDiagnostics(modulePath),
        api.getAst(modulePath).catch(() => ({ module: modulePath, ast: null })),
        api.getScope(modulePath).catch(() => ({ module: modulePath, scopes: [] })),
        api.getGenerated(modulePath).catch(() => ({ module: modulePath, javascript: '' })),
      ]);

      setDiagnostics(diagRes.diagnostics);
      setAst(astRes.ast);
      setScopes(scopeRes.scopes);
      setGeneratedJs(genRes.javascript);

      // Load source for first file in module
      const mod = modulesList.find(m => m.path === modulePath);
      if (mod && mod.source_files.length > 0) {
        const firstFile = mod.source_files[0];
        setSelectedFile(firstFile);
        setSourceLoading(true);
        try {
          const sourceRes = await api.getSource(firstFile);
          setSource(sourceRes.content);
          originalSourceRef.current = sourceRes.content;
          setIsDirty(false);
        } catch {
          setSource(`// Failed to load source for ${firstFile}`);
          originalSourceRef.current = '';
        } finally {
          setSourceLoading(false);
        }
      }
    } catch (err) {
      console.error('Failed to load module data:', err);
    }
  }, [modules]);

  // Select a specific file (for source viewing)
  const selectFile = useCallback(async (filePath: string) => {
    setSelectedFile(filePath);
    setSourceLoading(true);

    try {
      const sourceRes = await api.getSource(filePath);
      setSource(sourceRes.content);
      originalSourceRef.current = sourceRes.content;
      setIsDirty(false);
    } catch (err) {
      console.error('Failed to load file:', err);
      setSource(`// Failed to load source for ${filePath}`);
      originalSourceRef.current = '';
    } finally {
      setSourceLoading(false);
    }
  }, []);

  // Update source content (for editing) with debounced save
  const updateSource = useCallback((content: string) => {
    setSource(content);
    setIsDirty(content !== originalSourceRef.current);

    // Clear previous debounce timer
    if (debounceTimer.current) {
      clearTimeout(debounceTimer.current);
    }

    // Debounced write and compile
    debounceTimer.current = setTimeout(async () => {
      const currentFile = selectedFileRef.current;
      const currentModule = selectedModuleRef.current;

      if (!currentFile || content === originalSourceRef.current) return;

      setIsCompiling(true);
      try {
        await api.write(currentFile, content);
        originalSourceRef.current = content;
        setIsDirty(false);

        // Refresh module data if we have a selected module
        if (currentModule) {
          const fetchPromises: Promise<unknown>[] = [
            api.getDiagnostics(currentModule),
            api.getAst(currentModule).catch(() => ({ module: currentModule, ast: null })),
            api.getScope(currentModule).catch(() => ({ module: currentModule, scopes: [] })),
            api.getGenerated(currentModule).catch(() => ({ module: currentModule, javascript: '' })),
            api.getStatus(),
          ];

          // In compiler dev mode, also refresh comparison
          if (devModeRef.current === 'compiler-dev') {
            fetchPromises.push(api.compare(currentModule));
          }

          const results = await Promise.all(fetchPromises);
          const [diagRes, astRes, scopeRes, genRes, statusRes] = results as [
            { diagnostics: DiagnosticInfo[] },
            { ast: unknown },
            { scopes: ScopeInfo[] },
            { javascript: string },
            { error_count: number },
          ];

          setDiagnostics(diagRes.diagnostics);
          setAst(astRes.ast);
          setScopes(scopeRes.scopes);
          setGeneratedJs(genRes.javascript);
          setTotalErrors(statusRes.error_count);

          // Update comparison if in dev mode
          if (devModeRef.current === 'compiler-dev' && results[5]) {
            setComparison(results[5] as CompareResponse);
          }
        }
      } catch (err) {
        console.error('Failed to save and compile:', err);
      } finally {
        setIsCompiling(false);
      }
    }, 500); // 500ms debounce
  }, []);

  // Initial fetch
  useEffect(() => {
    refresh();
  }, [refresh]);

  // Auto-select first module if none selected
  useEffect(() => {
    if (modules.length > 0 && !selectedModule) {
      selectModule(modules[0].path, modules);
    }
  }, [modules, selectedModule, selectModule]);

  // Refresh comparison for current module (in compiler dev mode)
  const refreshComparison = useCallback(async () => {
    const currentModule = selectedModuleRef.current;
    if (!currentModule) return;

    try {
      const compareRes = await api.compare(currentModule);
      setComparison(compareRes);
    } catch (err) {
      console.error('Failed to fetch comparison:', err);
      setComparison(null);
    }
  }, []);

  // Save current results as expectations
  const saveExpectations = useCallback(async () => {
    const currentModule = selectedModuleRef.current;
    if (!currentModule) return;

    setIsSavingExpectations(true);
    try {
      await api.saveExpectations(currentModule);
      // Refresh comparison after saving
      await refreshComparison();
    } catch (err) {
      console.error('Failed to save expectations:', err);
    } finally {
      setIsSavingExpectations(false);
    }
  }, [refreshComparison]);

  // Fetch comparison when module changes in dev mode
  useEffect(() => {
    if (devMode === 'compiler-dev' && selectedModule) {
      refreshComparison();
    } else {
      setComparison(null);
    }
  }, [devMode, selectedModule, refreshComparison]);

  // Cleanup debounce timer on unmount
  useEffect(() => {
    return () => {
      if (debounceTimer.current) {
        clearTimeout(debounceTimer.current);
      }
    };
  }, []);

  return {
    status,
    error,
    initialized,
    totalErrors,
    totalModules,
    modules,
    selectedModule,
    selectedFile,
    source,
    sourceLoading,
    isDirty,
    isCompiling,
    activeTab,
    diagnostics,
    ast,
    scopes,
    generatedJs,
    devMode,
    comparison,
    isSavingExpectations,
    refresh,
    selectModule,
    selectFile,
    setActiveTab,
    updateSource,
    setDevMode,
    saveExpectations,
    refreshComparison,
  };
}
