use rspack_collections::Identifier;
use rspack_core::{Compilation, RuntimeModule, impl_runtime_module};

#[impl_runtime_module]
#[derive(Debug)]
pub struct GetFullHashRuntimeModule {
  id: Identifier,
}

impl Default for GetFullHashRuntimeModule {
  fn default() -> Self {
    Self::with_default(Identifier::from("webpack/runtime/get_full_hash"))
  }
}

#[async_trait::async_trait]
impl RuntimeModule for GetFullHashRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id.to_string(),
      include_str!("runtime/get_full_hash.ejs").to_string(),
    )]
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    let source = compilation.runtime_template.render(
      &self.id,
      Some(serde_json::json!({
        "_hash": format!("\"{}\"", compilation.get_hash().unwrap_or("XXXX"))
      })),
    )?;

    Ok(source)
  }

  fn full_hash(&self) -> bool {
    true
  }
}
