use rspack_collections::Identifier;
use rspack_core::{Compilation, RuntimeModule, impl_runtime_module};

#[impl_runtime_module]
#[derive(Debug)]
pub struct StartupEntrypointRuntimeModule {
  id: Identifier,
  async_chunk_loading: bool,
}

impl StartupEntrypointRuntimeModule {
  pub fn new(async_chunk_loading: bool) -> Self {
    Self::with_default(
      Identifier::from("webpack/runtime/startup_entrypoint"),
      async_chunk_loading,
    )
  }
}

#[async_trait::async_trait]
impl RuntimeModule for StartupEntrypointRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id.to_string(),
      if self.async_chunk_loading {
        include_str!("runtime/startup_entrypoint_with_async.ejs").to_string()
      } else {
        include_str!("runtime/startup_entrypoint.ejs").to_string()
      },
    )]
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    compilation.runtime_template.render(&self.id, None)
  }
}
