use rspack_collections::Identifier;
use rspack_core::{
  Compilation, Filename, PublicPath, RuntimeGlobals, RuntimeModule, has_hash_placeholder,
  impl_runtime_module,
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct PublicPathRuntimeModule {
  id: Identifier,
  public_path: Box<Filename>,
}

impl PublicPathRuntimeModule {
  pub fn new(public_path: Box<Filename>) -> Self {
    Self::with_default(Identifier::from("webpack/runtime/public_path"), public_path)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for PublicPathRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    Ok(format!(
      "{} = \"{}\";",
      RuntimeGlobals::PUBLIC_PATH.name(),
      &PublicPath::render_filename(compilation, &self.public_path).await
    ))
  }

  // be cacheable only when the template does not contain a hash placeholder
  fn full_hash(&self) -> bool {
    if let Some(template) = self.public_path.template() {
      has_hash_placeholder(template)
    } else {
      true
    }
  }
}
