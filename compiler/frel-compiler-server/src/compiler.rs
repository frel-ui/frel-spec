// Compilation logic
//
// Handles full builds and incremental rebuilds.

use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

use frel_compiler_core::{analyze_module, build_signature, Module};

use crate::state::{
    hash_content, hash_exports, AnalysisCacheEntry, FileState, ParseCacheEntry, ProjectState,
    SignatureCacheEntry,
};

/// Result of a full build
pub struct BuildResult {
    pub duration: Duration,
    pub modules_built: usize,
    pub error_count: usize,
}

/// Result of an incremental rebuild
pub struct IncrementalResult {
    pub duration: Duration,
    pub modules_rebuilt: Vec<String>,
    pub error_count: usize,
}

/// Perform a full build of the project
pub fn full_build(state: &mut ProjectState) -> BuildResult {
    let start = Instant::now();

    // 1. Discover all .frel files
    let files = discover_frel_files(&state.root);

    // 2. Read and parse all files
    for path in &files {
        if let Ok(content) = fs::read_to_string(path) {
            let hash = hash_content(&content);
            state.sources.insert(path.clone(), FileState::new(content.clone()));

            let parse_result = frel_compiler_core::parse_file_with_path(
                &content,
                &path.display().to_string(),
            );

            if let Some(ref file) = parse_result.file {
                state.module_index.update_file(path, &file.module);
                state.parse_cache.insert(
                    path.clone(),
                    ParseCacheEntry {
                        file: file.clone(),
                        diagnostics: parse_result.diagnostics.clone(),
                        content_hash: hash,
                    },
                );

                // Extract imports for dependency graph
                let imports: Vec<String> = file
                    .imports
                    .iter()
                    .map(|imp| imp.module.clone())
                    .collect();
                state.dependencies.update_module_deps(&file.module, &imports);
            }
        }
    }

    // 3. Build signatures for all modules (Phase 1)
    let modules: Vec<String> = state.module_index.all_modules().iter().map(|s| s.to_string()).collect();

    for module_path in &modules {
        if let Some(module_obj) = build_module_object(state, module_path) {
            let result = build_signature(&module_obj);
            let exports_hash = hash_exports(&result.signature);

            state.registry.register(result.signature.clone());
            state.signature_cache.insert(
                module_path.clone(),
                SignatureCacheEntry {
                    result,
                    exports_hash,
                    generation: state.generation,
                },
            );
        }
    }

    // 4. Analyze all modules (Phase 2)
    for module_path in &modules {
        if let Some(module_obj) = build_module_object(state, module_path) {
            let result = analyze_module(&module_obj, &state.registry);

            // Generate JavaScript if no errors
            let generated_js = if !result.diagnostics.has_errors() {
                // Get the first file's AST for codegen
                if let Some(file_path) = state.module_index.files_for_module(module_path).first() {
                    if let Some(cache_entry) = state.parse_cache.get(file_path) {
                        frel_compiler_plugin_javascript::generate(&cache_entry.file)
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            // Write output if we have generated code
            if !generated_js.is_empty() {
                let output_path = module_output_path(&state.build_dir, module_path);
                if let Some(parent) = output_path.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let _ = fs::write(&output_path, &generated_js);
            }

            state.analysis_cache.insert(
                module_path.clone(),
                AnalysisCacheEntry {
                    result,
                    generated_js,
                    generation: state.generation,
                },
            );
        }
    }

    state.initialized = true;
    let error_count = state.error_count();

    BuildResult {
        duration: start.elapsed(),
        modules_built: modules.len(),
        error_count,
    }
}

/// Handle a file change with incremental rebuild
pub fn handle_file_change(state: &mut ProjectState, path: &Path) -> IncrementalResult {
    let start = Instant::now();
    state.generation += 1;

    let mut modules_to_rebuild: HashSet<String> = HashSet::new();

    // 1. Read new content
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => {
            // File deleted - remove from state
            let path_buf = path.to_path_buf();
            if let Some(module) = state.module_index.module_for_file(&path_buf).map(String::from) {
                modules_to_rebuild.insert(module.clone());
                state.dependencies.remove_module(&module);
            }
            state.sources.remove(&path_buf);
            state.parse_cache.remove(&path_buf);
            state.module_index.remove_file(&path_buf);

            return IncrementalResult {
                duration: start.elapsed(),
                modules_rebuilt: modules_to_rebuild.into_iter().collect(),
                error_count: state.error_count(),
            };
        }
    };

    let path_buf = path.to_path_buf();
    let new_hash = hash_content(&content);

    // 2. Quick exit if content unchanged
    if let Some(file_state) = state.sources.get(&path_buf) {
        if file_state.content_hash == new_hash {
            return IncrementalResult {
                duration: start.elapsed(),
                modules_rebuilt: vec![],
                error_count: state.error_count(),
            };
        }
    }

    // 3. Update source state
    state.sources.insert(path_buf.clone(), FileState::new(content.clone()));

    // 4. Parse the changed file
    let parse_result = frel_compiler_core::parse_file_with_path(
        &content,
        &path.display().to_string(),
    );

    let old_module = state.module_index.module_for_file(&path_buf).map(String::from);

    if let Some(ref file) = parse_result.file {
        let new_module = file.module.clone();

        // Update module index
        state.module_index.update_file(&path_buf, &new_module);

        // Update parse cache
        state.parse_cache.insert(
            path_buf.clone(),
            ParseCacheEntry {
                file: file.clone(),
                diagnostics: parse_result.diagnostics.clone(),
                content_hash: new_hash,
            },
        );

        // Update dependency graph
        let imports: Vec<String> = file
            .imports
            .iter()
            .map(|imp| imp.module.clone())
            .collect();
        state.dependencies.update_module_deps(&new_module, &imports);

        // Mark this module for rebuild
        modules_to_rebuild.insert(new_module.clone());

        // If module changed, also rebuild old module
        if let Some(old) = old_module {
            if old != new_module {
                modules_to_rebuild.insert(old);
            }
        }
    }

    // 5. Rebuild signatures for affected modules
    let mut exports_changed: HashSet<String> = HashSet::new();

    for module_path in &modules_to_rebuild {
        if let Some(module_obj) = build_module_object(state, module_path) {
            let result = build_signature(&module_obj);
            let new_exports_hash = hash_exports(&result.signature);

            // Check if exports changed
            if let Some(cached) = state.signature_cache.get(module_path) {
                if cached.exports_hash != new_exports_hash {
                    exports_changed.insert(module_path.clone());
                }
            } else {
                exports_changed.insert(module_path.clone());
            }

            state.registry.register(result.signature.clone());
            state.signature_cache.insert(
                module_path.clone(),
                SignatureCacheEntry {
                    result,
                    exports_hash: new_exports_hash,
                    generation: state.generation,
                },
            );
        }
    }

    // 6. Expand to dependents if exports changed
    for module in &exports_changed {
        let importers = state.dependencies.get_transitive_importers(module);
        modules_to_rebuild.extend(importers);
    }

    // 7. Re-analyze affected modules
    for module_path in &modules_to_rebuild {
        if let Some(module_obj) = build_module_object(state, module_path) {
            let result = analyze_module(&module_obj, &state.registry);

            // Generate JavaScript if no errors
            let generated_js = if !result.diagnostics.has_errors() {
                if let Some(file_path) = state.module_index.files_for_module(module_path).first() {
                    if let Some(cache_entry) = state.parse_cache.get(file_path) {
                        frel_compiler_plugin_javascript::generate(&cache_entry.file)
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            // Write output
            if !generated_js.is_empty() {
                let output_path = module_output_path(&state.build_dir, module_path);
                if let Some(parent) = output_path.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let _ = fs::write(&output_path, &generated_js);
            }

            state.analysis_cache.insert(
                module_path.clone(),
                AnalysisCacheEntry {
                    result,
                    generated_js,
                    generation: state.generation,
                },
            );
        }
    }

    let error_count = state.error_count();

    IncrementalResult {
        duration: start.elapsed(),
        modules_rebuilt: modules_to_rebuild.into_iter().collect(),
        error_count,
    }
}

/// Discover all .frel files in a directory
pub fn discover_frel_files(root: &Path) -> Vec<std::path::PathBuf> {
    let pattern = root.join("**/*.frel");
    let pattern_str = pattern.display().to_string();

    glob::glob(&pattern_str)
        .map(|paths| paths.filter_map(Result::ok).collect())
        .unwrap_or_default()
}

/// Build a Module object from cached ASTs
fn build_module_object(state: &ProjectState, module_path: &str) -> Option<Module> {
    let files: Vec<_> = state
        .module_index
        .files_for_module(module_path)
        .iter()
        .filter_map(|path| state.parse_cache.get(path).map(|e| e.file.clone()))
        .collect();

    if files.is_empty() {
        return None;
    }

    Some(Module::from_files(module_path.to_string(), files))
}

/// Compute output path for a module
fn module_output_path(build_dir: &Path, module_path: &str) -> std::path::PathBuf {
    let parts: Vec<&str> = module_path.split('.').collect();
    let mut path = build_dir.to_path_buf();
    for part in parts {
        path.push(part);
    }
    path.set_extension("js");
    path
}
