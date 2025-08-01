use rayon::prelude::*;
use rspack_collections::{DatabaseItem, UkeyMap};
use rspack_core::{
  ApplyContext, CompilationChunkIds, CompilerOptions, Plugin, PluginContext,
  incremental::IncrementalPasses,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rustc_hash::{FxBuildHasher, FxHashMap};

use crate::id_helpers::{
  assign_deterministic_ids, compare_chunks_natural, get_full_chunk_name, get_used_chunk_ids,
};

#[plugin]
#[derive(Debug, Default)]
pub struct DeterministicChunkIdsPlugin {
  pub delimiter: String,
  pub context: Option<String>,
}

impl DeterministicChunkIdsPlugin {
  pub fn new(delimiter: Option<String>, context: Option<String>) -> Self {
    Self::new_inner(delimiter.unwrap_or_else(|| "~".to_string()), context)
  }
}

#[plugin_hook(CompilationChunkIds for DeterministicChunkIdsPlugin)]
async fn chunk_ids(&self, compilation: &mut rspack_core::Compilation) -> rspack_error::Result<()> {
  if let Some(diagnostic) = compilation.incremental.disable_passes(
    IncrementalPasses::CHUNK_IDS,
    "DeterministicChunkIdsPlugin (optimization.chunkIds = \"deterministic\")",
    "it requires calculating the id of all the chunks, which is a global effect",
  ) {
    if let Some(diagnostic) = diagnostic {
      compilation.push_diagnostic(diagnostic);
    }
    compilation.chunk_ids_artifact.clear();
  }

  let mut used_ids = get_used_chunk_ids(compilation);
  let used_ids_len = used_ids.len();

  let chunk_graph = &compilation.chunk_graph;
  let module_graph = compilation.get_module_graph();
  let module_graph_cache = &compilation.module_graph_cache_artifact;
  let context = self
    .context
    .clone()
    .unwrap_or_else(|| compilation.options.context.as_str().to_string());

  let max_length = 3;
  let expand_factor = 10;
  let salt = 10;

  let chunks = compilation
    .chunk_by_ukey
    .values()
    .filter(|chunk| chunk.id(&compilation.chunk_ids_artifact).is_none())
    .collect::<Vec<_>>();
  let mut chunk_key_to_id =
    FxHashMap::with_capacity_and_hasher(chunks.len(), FxBuildHasher::default());

  let chunk_names = chunks
    .par_iter()
    .map(|chunk| {
      (
        chunk.ukey(),
        get_full_chunk_name(
          chunk,
          chunk_graph,
          &module_graph,
          module_graph_cache,
          &context,
        ),
      )
    })
    .collect::<UkeyMap<_, _>>();

  let mut ordered_chunk_modules_cache = Default::default();

  assign_deterministic_ids(
    chunks,
    |chunk| {
      chunk_names
        .get(&chunk.ukey())
        .expect("should have generated full chunk name")
        .to_string()
    },
    |a, b| {
      compare_chunks_natural(
        chunk_graph,
        &module_graph,
        &compilation.chunk_group_by_ukey,
        &compilation.module_ids_artifact,
        a,
        b,
        &mut ordered_chunk_modules_cache,
      )
    },
    |chunk, id| {
      let size = used_ids.len();
      used_ids.insert(id.to_string());
      if used_ids.len() == size {
        return false;
      }

      chunk_key_to_id.insert(chunk.ukey(), id);
      true
    },
    &[usize::pow(10, max_length)],
    expand_factor,
    used_ids_len,
    salt,
  );

  chunk_key_to_id.into_iter().for_each(|(chunk_ukey, id)| {
    let chunk = compilation.chunk_by_ukey.expect_get_mut(&chunk_ukey);
    chunk.set_id(&mut compilation.chunk_ids_artifact, id.to_string());
  });

  Ok(())
}

impl Plugin for DeterministicChunkIdsPlugin {
  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &CompilerOptions) -> Result<()> {
    ctx
      .context
      .compilation_hooks
      .chunk_ids
      .tap(chunk_ids::new(self));
    Ok(())
  }
}
