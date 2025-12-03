// Project state and cache types
//
// Core data structures for tracking compilation state.

use std::collections::{HashMap, HashSet};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::PathBuf;
use std::sync::Arc;

use frel_compiler_core::{
    ast, Diagnostics, ModuleAnalysisResult, ModuleSignature, SignatureRegistry, SignatureResult,
};
use tokio::sync::RwLock;

/// Shared state wrapper for async access
pub type SharedState = Arc<RwLock<ProjectState>>;

/// Main project compilation state
pub struct ProjectState {
    /// Project root directory
    pub root: PathBuf,
    /// Build output directory
    pub build_dir: PathBuf,
    /// Source files: path -> content + hash
    pub sources: HashMap<PathBuf, FileState>,
    /// Module index: file <-> module mapping
    pub module_index: ModuleIndex,
    /// Dependency graph for invalidation
    pub dependencies: DependencyGraph,
    /// Parse cache: path -> AST + diagnostics
    pub parse_cache: HashMap<PathBuf, ParseCacheEntry>,
    /// Signature cache: module -> SignatureResult
    pub signature_cache: HashMap<String, SignatureCacheEntry>,
    /// Analysis cache: module -> AnalysisResult + generated JS
    pub analysis_cache: HashMap<String, AnalysisCacheEntry>,
    /// Current signature registry
    pub registry: SignatureRegistry,
    /// Generation counter for cache invalidation
    pub generation: u64,
    /// Whether initial compilation is complete
    pub initialized: bool,
}

impl ProjectState {
    pub fn new(root: PathBuf, build_dir: PathBuf) -> Self {
        Self {
            root,
            build_dir,
            sources: HashMap::new(),
            module_index: ModuleIndex::new(),
            dependencies: DependencyGraph::new(),
            parse_cache: HashMap::new(),
            signature_cache: HashMap::new(),
            analysis_cache: HashMap::new(),
            registry: SignatureRegistry::new(),
            generation: 0,
            initialized: false,
        }
    }

    /// Get total error count across all modules
    pub fn error_count(&self) -> usize {
        let mut count = 0;
        for entry in self.parse_cache.values() {
            count += entry.diagnostics.error_count();
        }
        for entry in self.signature_cache.values() {
            count += entry.result.diagnostics.error_count();
        }
        for entry in self.analysis_cache.values() {
            count += entry.result.diagnostics.error_count();
        }
        count
    }

    /// Get all modules
    pub fn modules(&self) -> Vec<&str> {
        self.module_index.all_modules()
    }
}

/// State for a single source file
pub struct FileState {
    /// Source text content
    pub content: String,
    /// Content hash for change detection
    pub content_hash: u64,
}

impl FileState {
    pub fn new(content: String) -> Self {
        let content_hash = hash_content(&content);
        Self {
            content,
            content_hash,
        }
    }
}

/// Cached parse result for a file
pub struct ParseCacheEntry {
    /// Parsed AST
    pub file: ast::File,
    /// Parse diagnostics
    pub diagnostics: Diagnostics,
    /// Content hash when parsed
    pub content_hash: u64,
}

/// Cached signature result for a module
pub struct SignatureCacheEntry {
    /// Module signature result
    pub result: SignatureResult,
    /// Hash of exports for change detection
    pub exports_hash: u64,
    /// Generation when cached
    pub generation: u64,
}

/// Cached analysis result for a module
pub struct AnalysisCacheEntry {
    /// Full analysis result
    pub result: ModuleAnalysisResult,
    /// Generated JavaScript code
    pub generated_js: String,
    /// Generation when cached
    pub generation: u64,
}

/// Maps files to modules and vice versa
pub struct ModuleIndex {
    /// File path -> module path
    file_to_module: HashMap<PathBuf, String>,
    /// Module path -> file paths
    module_to_files: HashMap<String, Vec<PathBuf>>,
}

impl ModuleIndex {
    pub fn new() -> Self {
        Self {
            file_to_module: HashMap::new(),
            module_to_files: HashMap::new(),
        }
    }

    /// Update index when a file is parsed
    pub fn update_file(&mut self, path: &PathBuf, module_path: &str) {
        // Remove from old module if it moved
        if let Some(old_module) = self.file_to_module.get(path) {
            if old_module != module_path {
                if let Some(files) = self.module_to_files.get_mut(old_module) {
                    files.retain(|p| p != path);
                }
            }
        }

        self.file_to_module
            .insert(path.clone(), module_path.to_owned());
        self.module_to_files
            .entry(module_path.to_owned())
            .or_default()
            .push(path.clone());
    }

    /// Remove a file from the index
    pub fn remove_file(&mut self, path: &PathBuf) {
        if let Some(module) = self.file_to_module.remove(path) {
            if let Some(files) = self.module_to_files.get_mut(&module) {
                files.retain(|p| p != path);
                if files.is_empty() {
                    self.module_to_files.remove(&module);
                }
            }
        }
    }

    /// Get all files that contribute to a module
    pub fn files_for_module(&self, module_path: &str) -> &[PathBuf] {
        self.module_to_files
            .get(module_path)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Get module for a file
    pub fn module_for_file(&self, path: &PathBuf) -> Option<&str> {
        self.file_to_module.get(path).map(|s| s.as_str())
    }

    /// Get all module paths
    pub fn all_modules(&self) -> Vec<&str> {
        self.module_to_files.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for ModuleIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// Tracks import relationships for invalidation propagation
pub struct DependencyGraph {
    /// Module -> modules that import from it (reverse deps)
    importers: HashMap<String, HashSet<String>>,
    /// Module -> modules it imports from (forward deps)
    imports: HashMap<String, HashSet<String>>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            importers: HashMap::new(),
            imports: HashMap::new(),
        }
    }

    /// Update dependencies for a module based on its imports
    pub fn update_module_deps(&mut self, module: &str, import_modules: &[String]) {
        // Clear old forward deps
        if let Some(old_imports) = self.imports.get(module) {
            for old_import in old_imports.clone() {
                if let Some(set) = self.importers.get_mut(&old_import) {
                    set.remove(module);
                }
            }
        }

        // Set new forward deps
        let import_set: HashSet<String> = import_modules.iter().cloned().collect();
        self.imports.insert(module.to_owned(), import_set);

        // Update reverse deps
        for import_module in import_modules {
            self.importers
                .entry(import_module.clone())
                .or_default()
                .insert(module.to_owned());
        }
    }

    /// Get all modules that would be affected if `module`'s exports change
    pub fn get_transitive_importers(&self, module: &str) -> HashSet<String> {
        let mut affected = HashSet::new();
        let mut queue = vec![module.to_owned()];

        while let Some(current) = queue.pop() {
            if let Some(importers) = self.importers.get(&current) {
                for importer in importers {
                    if affected.insert(importer.clone()) {
                        queue.push(importer.clone());
                    }
                }
            }
        }
        affected
    }

    /// Get direct imports of a module
    pub fn get_imports(&self, module: &str) -> HashSet<String> {
        self.imports
            .get(module)
            .cloned()
            .unwrap_or_else(HashSet::new)
    }

    /// Remove a module from the graph
    pub fn remove_module(&mut self, module: &str) {
        // Remove from importers of modules this one imports
        if let Some(imports) = self.imports.remove(module) {
            for imported in imports {
                if let Some(set) = self.importers.get_mut(&imported) {
                    set.remove(module);
                }
            }
        }
        // Remove this module's importers entry
        self.importers.remove(module);
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute a hash of content for change detection
pub fn hash_content(content: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}

/// Compute a hash of a module's exports for invalidation detection
pub fn hash_exports(signature: &ModuleSignature) -> u64 {
    let mut hasher = DefaultHasher::new();
    for export in &signature.exports {
        export.name.hash(&mut hasher);
        // Hash the kind discriminant
        std::mem::discriminant(&export.kind).hash(&mut hasher);
    }
    hasher.finish()
}
