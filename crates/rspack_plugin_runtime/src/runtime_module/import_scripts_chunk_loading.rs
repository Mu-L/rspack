use rspack_collections::{DatabaseItem, Identifier};
use rspack_core::{
  BooleanMatcher, Chunk, ChunkUkey, Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleStage,
  compile_boolean_matcher, impl_runtime_module,
};

use super::{
  generate_javascript_hmr_runtime,
  utils::{chunk_has_js, get_output_dir},
};
use crate::{
  get_chunk_runtime_requirements,
  runtime_module::utils::{get_initial_chunk_ids, stringify_chunks},
};

#[impl_runtime_module]
#[derive(Debug, Default)]
pub struct ImportScriptsChunkLoadingRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
  with_create_script_url: bool,
}

impl ImportScriptsChunkLoadingRuntimeModule {
  pub fn new(with_create_script_url: bool) -> Self {
    Self::with_default(
      Identifier::from("webpack/runtime/import_scripts_chunk_loading"),
      None,
      with_create_script_url,
    )
  }

  async fn generate_base_uri(
    &self,
    chunk: &Chunk,
    compilation: &Compilation,
  ) -> rspack_error::Result<String> {
    let base_uri = if let Some(base_uri) = chunk
      .get_entry_options(&compilation.chunk_group_by_ukey)
      .and_then(|options| options.base_uri.as_ref())
      .and_then(|base_uri| serde_json::to_string(base_uri).ok())
    {
      base_uri
    } else {
      let root_output_dir = get_output_dir(chunk, compilation, false).await?;
      format!(
        "self.location + {}",
        serde_json::to_string(&if root_output_dir.is_empty() {
          "".to_string()
        } else {
          format!("/../{root_output_dir}")
        })
        .expect("should able to be serde_json::to_string")
      )
    };
    Ok(format!("{} = {};\n", RuntimeGlobals::BASE_URI, base_uri))
  }

  fn template_id(&self, id: TemplateId) -> String {
    let base_id = self.id.as_str();

    match id {
      TemplateId::Raw => base_id.to_string(),
      TemplateId::WithHmr => format!("{base_id}_with_hmr"),
      TemplateId::WithHmrManifest => format!("{base_id}_with_hmr_manifest"),
    }
  }
}

enum TemplateId {
  Raw,
  WithHmr,
  WithHmrManifest,
}

#[async_trait::async_trait]
impl RuntimeModule for ImportScriptsChunkLoadingRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![
      (
        self.template_id(TemplateId::Raw),
        include_str!("runtime/import_scripts_chunk_loading.ejs").to_string(),
      ),
      (
        self.template_id(TemplateId::WithHmr),
        include_str!("runtime/import_scripts_chunk_loading_with_hmr.ejs").to_string(),
      ),
      (
        self.template_id(TemplateId::WithHmrManifest),
        include_str!("runtime/import_scripts_chunk_loading_with_hmr_manifest.ejs").to_string(),
      ),
    ]
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    let chunk = compilation
      .chunk_by_ukey
      .expect_get(&self.chunk.expect("The chunk should be attached."));

    let runtime_requirements = get_chunk_runtime_requirements(compilation, &chunk.ukey());
    let initial_chunks = get_initial_chunk_ids(self.chunk, compilation, chunk_has_js);

    let with_base_uri = runtime_requirements.contains(RuntimeGlobals::BASE_URI);
    let with_hmr = runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS);
    let with_hmr_manifest = runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_MANIFEST);
    let with_loading = runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS);
    let with_callback = runtime_requirements.contains(RuntimeGlobals::CHUNK_CALLBACK);

    let condition_map =
      compilation
        .chunk_graph
        .get_chunk_condition_map(&chunk.ukey(), compilation, chunk_has_js);
    let has_js_matcher = compile_boolean_matcher(&condition_map);

    let mut source = String::default();

    if with_base_uri {
      source.push_str(&self.generate_base_uri(chunk, compilation).await?);
    }

    // object to store loaded chunks
    // "1" means "already loaded"
    if with_hmr {
      let state_expression = format!("{}_importScripts", RuntimeGlobals::HMR_RUNTIME_STATE_PREFIX);
      source.push_str(&format!(
        "var installedChunks = {} = {} || {};\n",
        state_expression,
        state_expression,
        &stringify_chunks(&initial_chunks, 1)
      ));
    } else {
      source.push_str(&format!(
        "var installedChunks = {};\n",
        &stringify_chunks(&initial_chunks, 1)
      ));
    }

    if with_loading || with_callback {
      let chunk_loading_global_expr = format!(
        "{}[\"{}\"]",
        &compilation.options.output.global_object, &compilation.options.output.chunk_loading_global
      );

      let body = if matches!(has_js_matcher, BooleanMatcher::Condition(false)) {
        "installedChunks[chunkId] = 1;".to_string()
      } else {
        let url = if self.with_create_script_url {
          format!(
            "{}({} + {}(chunkId))",
            RuntimeGlobals::CREATE_SCRIPT_URL,
            RuntimeGlobals::PUBLIC_PATH,
            RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME
          )
        } else {
          format!(
            "{} + {}(chunkId)",
            RuntimeGlobals::PUBLIC_PATH,
            RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME
          )
        };
        format!(
          r#"
          // "1" is the signal for "already loaded
          if (!installedChunks[chunkId]) {{
            if ({}) {{
              importScripts({});
            }}
          }}
          "#,
          &has_js_matcher.render("chunkId"),
          url
        )
      };

      let render_source = compilation.runtime_template.render(
        &self.template_id(TemplateId::Raw),
        Some(serde_json::json!({
          "_body": body,
          "_chunk_loading_global_expr": chunk_loading_global_expr,
          "_with_loading": with_loading,
        })),
      )?;

      // If chunkId not corresponding chunkName will skip load it.
      source.push_str(&render_source);
    }

    if with_hmr {
      let url = if self.with_create_script_url {
        format!(
          "{}({} + {}(chunkId))",
          RuntimeGlobals::CREATE_SCRIPT_URL,
          RuntimeGlobals::PUBLIC_PATH,
          RuntimeGlobals::GET_CHUNK_UPDATE_SCRIPT_FILENAME
        )
      } else {
        format!(
          "{} + {}(chunkId)",
          RuntimeGlobals::PUBLIC_PATH,
          RuntimeGlobals::GET_CHUNK_UPDATE_SCRIPT_FILENAME
        )
      };

      let source_with_hmr = compilation.runtime_template.render(&self.template_id(TemplateId::WithHmr), Some(serde_json::json!({
        "_url": &url,
        "_global_object": &compilation.options.output.global_object.as_str(),
        "_hot_update_global": &serde_json::to_string(&compilation.options.output.hot_update_global).expect("failed to serde_json::to_string(hot_update_global)"),
      })))?;

      source.push_str(&source_with_hmr);
      source.push_str(&generate_javascript_hmr_runtime("importScripts"));
    }

    if with_hmr_manifest {
      // TODO: import_scripts_chunk_loading_with_hmr_manifest same as jsonp_chunk_loading_with_hmr_manifest
      let source_with_hmr_manifest = compilation
        .runtime_template
        .render(&self.template_id(TemplateId::WithHmrManifest), None)?;

      source.push_str(&source_with_hmr_manifest);
    }

    Ok(source)
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Attach
  }
}
