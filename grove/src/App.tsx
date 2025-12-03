import { useState } from 'react';
import { useCompiler } from './hooks/useCompiler';
import type { OutputTab, DiagnosticInfo, ModuleInfo, DevMode, CompareResponse, ScopeInfo } from './types';

function App() {
  const compiler = useCompiler();

  return (
    <div className="app">
      <Sidebar
        modules={compiler.modules}
        selectedModule={compiler.selectedModule}
        selectedFile={compiler.selectedFile}
        onSelectModule={compiler.selectModule}
        onSelectFile={compiler.selectFile}
      />
      <EditorArea
        selectedFile={compiler.selectedFile}
        source={compiler.source}
        isDirty={compiler.isDirty}
        isCompiling={compiler.isCompiling}
        onSourceChange={compiler.updateSource}
      />
      <OutputArea
        activeTab={compiler.activeTab}
        onTabChange={compiler.setActiveTab}
        diagnostics={compiler.diagnostics}
        ast={compiler.ast}
        scopes={compiler.scopes}
        generatedJs={compiler.generatedJs}
      />
      <StatusBar
        status={compiler.status}
        error={compiler.error}
        moduleCount={compiler.totalModules}
        errorCount={compiler.totalErrors}
        devMode={compiler.devMode}
        comparison={compiler.comparison}
        isSavingExpectations={compiler.isSavingExpectations}
        onRefresh={compiler.refresh}
        onDevModeChange={compiler.setDevMode}
        onSaveExpectations={compiler.saveExpectations}
      />
    </div>
  );
}

// Sidebar component
interface SidebarProps {
  modules: ModuleInfo[];
  selectedModule: string | null;
  selectedFile: string | null;
  onSelectModule: (path: string) => void;
  onSelectFile: (path: string) => void;
}

function Sidebar({ modules, selectedModule, selectedFile, onSelectModule, onSelectFile }: SidebarProps) {
  const [expandedModules, setExpandedModules] = useState<Set<string>>(new Set());

  const toggleModule = (modulePath: string) => {
    setExpandedModules(prev => {
      const next = new Set(prev);
      if (next.has(modulePath)) {
        next.delete(modulePath);
      } else {
        next.add(modulePath);
      }
      return next;
    });
    onSelectModule(modulePath);
  };

  const getFileName = (path: string) => {
    const parts = path.split('/');
    return parts[parts.length - 1];
  };

  return (
    <div className="sidebar">
      <div className="sidebar-header">Modules</div>
      <div className="module-list">
        {modules.map((mod) => {
          const isExpanded = expandedModules.has(mod.path) || selectedModule === mod.path;
          return (
            <div key={mod.path} className="module-group">
              <div
                className={`module-item ${selectedModule === mod.path ? 'selected' : ''}`}
                onClick={() => toggleModule(mod.path)}
              >
                <span className="icon">{isExpanded ? '▼' : '▶'}</span>
                <span className="name">{mod.path}</span>
                {mod.error_count > 0 && (
                  <span className="badge">{mod.error_count}</span>
                )}
              </div>
              {isExpanded && mod.source_files.map((file) => (
                <div
                  key={file}
                  className={`file-item ${selectedFile === file ? 'selected' : ''}`}
                  onClick={(e) => {
                    e.stopPropagation();
                    onSelectFile(file);
                  }}
                >
                  <span className="icon">📄</span>
                  <span className="name">{getFileName(file)}</span>
                </div>
              ))}
            </div>
          );
        })}
        {modules.length === 0 && (
          <div className="empty-state">
            <p>No modules found</p>
          </div>
        )}
      </div>
    </div>
  );
}

// Editor area component
interface EditorAreaProps {
  selectedFile: string | null;
  source: string;
  isDirty: boolean;
  isCompiling: boolean;
  onSourceChange: (content: string) => void;
}

function EditorArea({ selectedFile, source, isDirty, isCompiling, onSourceChange }: EditorAreaProps) {
  const getFileName = (path: string) => {
    const parts = path.split('/');
    return parts[parts.length - 1];
  };

  return (
    <div className="editor-area">
      <div className="editor-header">
        <span>
          {selectedFile ? getFileName(selectedFile) : 'No file selected'}
          {isDirty && <span className="dirty-indicator"> *</span>}
        </span>
        {isCompiling && <span className="compiling-indicator">Compiling...</span>}
      </div>
      <div className="editor-content">
        <textarea
          value={source}
          onChange={(e) => onSourceChange(e.target.value)}
          placeholder="Select a file to view its source..."
          spellCheck={false}
        />
      </div>
    </div>
  );
}

// Output area component
interface OutputAreaProps {
  activeTab: OutputTab;
  onTabChange: (tab: OutputTab) => void;
  diagnostics: DiagnosticInfo[];
  ast: unknown | null;
  scopes: ScopeInfo[];
  generatedJs: string;
}

function OutputArea({ activeTab, onTabChange, diagnostics, ast, scopes, generatedJs }: OutputAreaProps) {
  return (
    <div className="output-area">
      <div className="output-tabs">
        <button
          className={`output-tab ${activeTab === 'diagnostics' ? 'active' : ''}`}
          onClick={() => onTabChange('diagnostics')}
        >
          Diagnostics ({diagnostics.length})
        </button>
        <button
          className={`output-tab ${activeTab === 'ast' ? 'active' : ''}`}
          onClick={() => onTabChange('ast')}
        >
          AST
        </button>
        <button
          className={`output-tab ${activeTab === 'scope' ? 'active' : ''}`}
          onClick={() => onTabChange('scope')}
        >
          Scope
        </button>
        <button
          className={`output-tab ${activeTab === 'generated' ? 'active' : ''}`}
          onClick={() => onTabChange('generated')}
        >
          Generated JS
        </button>
      </div>
      <div className="output-content">
        {activeTab === 'diagnostics' && (
          <DiagnosticsPanel diagnostics={diagnostics} />
        )}
        {activeTab === 'ast' && (
          <AstPanel ast={ast} />
        )}
        {activeTab === 'scope' && (
          <ScopePanel scopes={scopes} />
        )}
        {activeTab === 'generated' && (
          <GeneratedPanel code={generatedJs} />
        )}
      </div>
    </div>
  );
}

// Diagnostics panel
function DiagnosticsPanel({ diagnostics }: { diagnostics: DiagnosticInfo[] }) {
  if (diagnostics.length === 0) {
    return (
      <div className="empty-state">
        <span className="icon">✓</span>
        <p>No diagnostics</p>
      </div>
    );
  }

  return (
    <div className="diagnostics-list">
      {diagnostics.map((diag, i) => (
        <div
          key={i}
          className={`diagnostic-item ${diag.severity === 'warning' ? 'warning' : ''}`}
        >
          <div className="message">
            {diag.code && <span>[{diag.code}] </span>}
            {diag.message}
          </div>
          {diag.file && (
            <div className="location">
              {diag.file}
              {diag.line && `:${diag.line}`}
              {diag.column && `:${diag.column}`}
            </div>
          )}
        </div>
      ))}
    </div>
  );
}

// AST panel
function AstPanel({ ast }: { ast: unknown | null }) {
  if (!ast) {
    return (
      <div className="empty-state">
        <p>No AST available</p>
      </div>
    );
  }

  return (
    <div className="ast-tree">
      {JSON.stringify(ast, null, 2)}
    </div>
  );
}

// Scope panel
function ScopePanel({ scopes }: { scopes: ScopeInfo[] }) {
  if (scopes.length === 0) {
    return (
      <div className="empty-state">
        <p>No scope information available</p>
      </div>
    );
  }

  return (
    <div className="scope-tree">
      {scopes.map((scope) => (
        <div key={scope.id} className="scope-item">
          <div className="scope-header">
            <span className="scope-kind">{scope.kind}</span>
            {scope.name && <span className="scope-name">{scope.name}</span>}
            <span className="scope-id">#{scope.id}</span>
            {scope.parent !== null && (
              <span className="scope-parent">parent: #{scope.parent}</span>
            )}
          </div>
          {scope.symbols.length > 0 && (
            <div className="scope-symbols">
              {scope.symbols.map((sym) => (
                <div key={sym.id} className="symbol-item">
                  <span className="symbol-kind">{sym.kind}</span>
                  <span className="symbol-name">{sym.name}</span>
                  {sym.body_scope !== null && (
                    <span className="symbol-body">scope: #{sym.body_scope}</span>
                  )}
                  {sym.source_module && (
                    <span className="symbol-source">from {sym.source_module}</span>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>
      ))}
    </div>
  );
}

// Generated JS panel
function GeneratedPanel({ code }: { code: string }) {
  if (!code) {
    return (
      <div className="empty-state">
        <p>No generated code available</p>
        <p style={{ fontSize: '11px', marginTop: '8px' }}>
          Module may have errors
        </p>
      </div>
    );
  }

  return (
    <div className="generated-code">
      {code}
    </div>
  );
}

// Status bar component
interface StatusBarProps {
  status: 'disconnected' | 'connecting' | 'connected' | 'error';
  error: string | null;
  moduleCount: number;
  errorCount: number;
  devMode: DevMode;
  comparison: CompareResponse | null;
  isSavingExpectations: boolean;
  onRefresh: () => void;
  onDevModeChange: (mode: DevMode) => void;
  onSaveExpectations: () => void;
}

function StatusBar({
  status,
  error,
  moduleCount,
  errorCount,
  devMode,
  comparison,
  isSavingExpectations,
  onRefresh,
  onDevModeChange,
  onSaveExpectations,
}: StatusBarProps) {
  const statusClass = status === 'error' ? 'error' : status === 'disconnected' ? 'disconnected' : '';

  return (
    <div className={`status-bar ${statusClass}`}>
      <div className="status-item" onClick={onRefresh} style={{ cursor: 'pointer' }}>
        <span className="status-dot" />
        <span>
          {status === 'connected' && 'Connected'}
          {status === 'connecting' && 'Connecting...'}
          {status === 'disconnected' && 'Disconnected'}
          {status === 'error' && (error || 'Error')}
        </span>
      </div>
      {status === 'connected' && (
        <>
          <div className="status-item">
            <span>{moduleCount} modules</span>
          </div>
          <div className="status-item">
            <span>{errorCount} errors</span>
          </div>
          <div className="status-spacer" />
          <div className="status-item">
            <label className="mode-toggle">
              <input
                type="checkbox"
                checked={devMode === 'compiler-dev'}
                onChange={(e) => onDevModeChange(e.target.checked ? 'compiler-dev' : 'normal')}
              />
              <span>Compiler Dev</span>
            </label>
          </div>
          {devMode === 'compiler-dev' && (
            <>
              {comparison && (
                <div className={`status-item ${comparison.has_differences ? 'diff-warning' : 'diff-ok'}`}>
                  {comparison.has_differences ? (
                    <span>
                      Diff: {!comparison.ast_matches && 'AST '}
                      {!comparison.diagnostics_match && 'Diag '}
                      {!comparison.generated_js_matches && 'JS'}
                    </span>
                  ) : comparison.expected ? (
                    <span>Matches expected</span>
                  ) : (
                    <span>No expectations</span>
                  )}
                </div>
              )}
              <button
                className="save-expectations-btn"
                onClick={onSaveExpectations}
                disabled={isSavingExpectations}
              >
                {isSavingExpectations ? 'Saving...' : 'Save as Expected'}
              </button>
            </>
          )}
        </>
      )}
    </div>
  );
}

export default App;
