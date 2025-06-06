use std::{
  collections::hash_map::Entry,
  hash::Hash,
  ops::{Deref, DerefMut},
  sync::atomic::AtomicU32,
};

use anymap::CloneAny;
use rspack_collections::IdentifierMap;
use rspack_hash::{HashDigest, HashFunction, HashSalt, RspackHash, RspackHashDigest};
use rspack_sources::BoxSource;
use rspack_util::atom::Atom;
use rustc_hash::{FxHashMap as HashMap, FxHashSet};
use serde::Serialize;

use crate::{
  AssetInfo, BindingCell, ChunkInitFragments, ConcatenationScope, ModuleIdentifier, RuntimeGlobals,
  RuntimeSpec, RuntimeSpecMap, SourceType,
};

#[derive(Clone, Debug)]
pub struct CodeGenerationDataUrl {
  inner: String,
}

impl CodeGenerationDataUrl {
  pub fn new(inner: String) -> Self {
    Self { inner }
  }

  pub fn inner(&self) -> &str {
    &self.inner
  }
}

// For performance, mark the js modules containing AUTO_PUBLIC_PATH_PLACEHOLDER
#[derive(Clone, Debug)]
pub struct CodeGenerationPublicPathAutoReplace(pub bool);

#[derive(Clone, Debug)]
pub struct CodeGenerationDataFilename {
  filename: String,
  public_path: String,
}

impl CodeGenerationDataFilename {
  pub fn new(filename: String, public_path: String) -> Self {
    Self {
      filename,
      public_path,
    }
  }

  pub fn filename(&self) -> &str {
    &self.filename
  }

  pub fn public_path(&self) -> &str {
    &self.public_path
  }
}

#[derive(Clone, Debug)]
pub struct CodeGenerationDataAssetInfo {
  inner: AssetInfo,
}

impl CodeGenerationDataAssetInfo {
  pub fn new(inner: AssetInfo) -> Self {
    Self { inner }
  }

  pub fn inner(&self) -> &AssetInfo {
    &self.inner
  }
}

#[derive(Clone, Debug)]
pub struct CodeGenerationDataTopLevelDeclarations {
  inner: FxHashSet<Atom>,
}

impl CodeGenerationDataTopLevelDeclarations {
  pub fn new(inner: FxHashSet<Atom>) -> Self {
    Self { inner }
  }

  pub fn inner(&self) -> &FxHashSet<Atom> {
    &self.inner
  }
}

#[derive(Clone, Debug)]
pub struct CodeGenerationExportsFinalNames {
  inner: HashMap<String, String>,
}

impl CodeGenerationExportsFinalNames {
  pub fn new(inner: HashMap<String, String>) -> Self {
    Self { inner }
  }

  pub fn inner(&self) -> &HashMap<String, String> {
    &self.inner
  }
}

#[derive(Debug, Default, Clone)]
pub struct CodeGenerationData {
  inner: anymap::Map<dyn CloneAny + Send + Sync>,
}

impl Deref for CodeGenerationData {
  type Target = anymap::Map<dyn CloneAny + Send + Sync>;

  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}

impl DerefMut for CodeGenerationData {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.inner
  }
}

#[derive(Debug, Default, Clone)]
pub struct CodeGenerationResult {
  pub inner: BindingCell<HashMap<SourceType, BoxSource>>,
  /// [definition in webpack](https://github.com/webpack/webpack/blob/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/lib/Module.js#L75)
  pub data: CodeGenerationData,
  pub chunk_init_fragments: ChunkInitFragments,
  pub runtime_requirements: RuntimeGlobals,
  pub hash: Option<RspackHashDigest>,
  pub id: CodeGenResultId,
  pub concatenation_scope: Option<ConcatenationScope>,
}

impl CodeGenerationResult {
  pub fn with_javascript(mut self, generation_result: BoxSource) -> Self {
    self.inner.insert(SourceType::JavaScript, generation_result);
    self
  }

  pub fn with_css(mut self, generation_result: BoxSource) -> Self {
    self.inner.insert(SourceType::Css, generation_result);
    self
  }

  pub fn with_asset(mut self, generation_result: BoxSource) -> Self {
    self.inner.insert(SourceType::Asset, generation_result);
    self
  }

  pub fn inner(&self) -> &HashMap<SourceType, BoxSource> {
    &self.inner
  }

  pub fn get(&self, source_type: &SourceType) -> Option<&BoxSource> {
    self.inner.get(source_type)
  }

  pub fn add(&mut self, source_type: SourceType, generation_result: BoxSource) {
    let result = self.inner.insert(source_type, generation_result);
    debug_assert!(result.is_none());
  }

  pub fn set_hash(
    &mut self,
    hash_function: &HashFunction,
    hash_digest: &HashDigest,
    hash_salt: &HashSalt,
  ) {
    let mut hasher = RspackHash::with_salt(hash_function, hash_salt);
    for (source_type, source) in self.inner.as_ref() {
      source_type.hash(&mut hasher);
      source.hash(&mut hasher);
    }
    self.chunk_init_fragments.hash(&mut hasher);
    self.runtime_requirements.hash(&mut hasher);
    self.hash = Some(hasher.digest(hash_digest));
  }
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub struct CodeGenResultId(u32);

impl Default for CodeGenResultId {
  fn default() -> Self {
    Self(CODE_GEN_RESULT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
  }
}

pub static CODE_GEN_RESULT_ID: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Default, Clone)]
pub struct CodeGenerationResults {
  module_generation_result_map: HashMap<CodeGenResultId, BindingCell<CodeGenerationResult>>,
  map: IdentifierMap<RuntimeSpecMap<CodeGenResultId>>,
}

impl CodeGenerationResults {
  pub fn is_empty(&self) -> bool {
    self.module_generation_result_map.is_empty() && self.map.is_empty()
  }

  pub fn insert(
    &mut self,
    module_identifier: ModuleIdentifier,
    codegen_res: CodeGenerationResult,
    runtimes: impl IntoIterator<Item = RuntimeSpec>,
  ) {
    let codegen_res_id = codegen_res.id;
    self
      .module_generation_result_map
      .insert(codegen_res_id, BindingCell::from(codegen_res));
    for runtime in runtimes {
      self.add(module_identifier, runtime, codegen_res_id);
    }
  }

  pub fn remove(&mut self, module_identifier: &ModuleIdentifier) -> Option<()> {
    let runtime_map = self.map.remove(module_identifier)?;
    for result in runtime_map.values() {
      self.module_generation_result_map.remove(result)?;
    }
    Some(())
  }

  pub fn get(
    &self,
    module_identifier: &ModuleIdentifier,
    runtime: Option<&RuntimeSpec>,
  ) -> &BindingCell<CodeGenerationResult> {
    if let Some(entry) = self.map.get(module_identifier) {
      if let Some(runtime) = runtime {
        entry
          .get(runtime)
          .and_then(|m| {
            self.module_generation_result_map.get(m)
          })
          .unwrap_or_else(|| {
            panic!(
              "Failed to code generation result for {module_identifier} with runtime {runtime:?} \n {entry:?}"
            )
          })
      } else {
        if entry.size() > 1 {
          let mut values = entry.values();
          let results: FxHashSet<_> = entry.values().collect();
          if results.len() > 1 {
            panic!(
              "No unique code generation entry for unspecified runtime for {module_identifier} ",
            );
          }

          return values
            .next()
            .and_then(|m| self.module_generation_result_map.get(m))
            .unwrap_or_else(|| panic!("Expected value exists"));
        }

        entry
          .values()
          .next()
          .and_then(|m| self.module_generation_result_map.get(m))
          .unwrap_or_else(|| panic!("Expected value exists"))
      }
    } else {
      panic!(
        "No code generation entry for {} (existing entries: {:?})",
        module_identifier,
        self.map.keys().collect::<Vec<_>>()
      )
    }
  }

  pub fn add(
    &mut self,
    module_identifier: ModuleIdentifier,
    runtime: RuntimeSpec,
    result: CodeGenResultId,
  ) {
    match self.map.entry(module_identifier) {
      Entry::Occupied(mut record) => {
        record.get_mut().set(runtime, result);
      }
      Entry::Vacant(record) => {
        let mut spec_map = RuntimeSpecMap::default();
        spec_map.set(runtime, result);
        record.insert(spec_map);
      }
    };
  }

  pub fn get_runtime_requirements(
    &self,
    module_identifier: &ModuleIdentifier,
    runtime: Option<&RuntimeSpec>,
  ) -> RuntimeGlobals {
    self.get(module_identifier, runtime).runtime_requirements
  }

  pub fn get_hash(
    &self,
    module_identifier: &ModuleIdentifier,
    runtime: Option<&RuntimeSpec>,
  ) -> Option<&RspackHashDigest> {
    let code_generation_result = self.get(module_identifier, runtime);

    code_generation_result.hash.as_ref()
  }

  pub fn inner(
    &self,
  ) -> (
    &IdentifierMap<RuntimeSpecMap<CodeGenResultId>>,
    &HashMap<CodeGenResultId, BindingCell<CodeGenerationResult>>,
  ) {
    (&self.map, &self.module_generation_result_map)
  }
}

#[derive(Debug)]
pub struct CodeGenerationJob {
  pub module: ModuleIdentifier,
  pub hash: RspackHashDigest,
  pub runtime: RuntimeSpec,
  pub runtimes: Vec<RuntimeSpec>,
}
