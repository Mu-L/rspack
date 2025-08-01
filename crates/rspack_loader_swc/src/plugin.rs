use std::{
  borrow::Cow,
  sync::{Arc, LazyLock},
};

use rspack_core::{
  ApplyContext, BoxLoader, CompilerOptions, Context, ModuleRuleUseLoader,
  NormalModuleFactoryResolveLoader, Plugin, PluginContext, Resolver,
};
use rspack_error::{Result, SerdeResultToRspackResultExt};
use rspack_hook::{plugin, plugin_hook};
use rustc_hash::FxHashMap;
use tokio::sync::RwLock;

use crate::{SWC_LOADER_IDENTIFIER, SwcLoader};

#[plugin]
#[derive(Debug)]
pub struct SwcLoaderPlugin;

impl SwcLoaderPlugin {
  pub fn new() -> Self {
    Self::new_inner()
  }
}

impl Default for SwcLoaderPlugin {
  fn default() -> Self {
    Self::new()
  }
}

impl Plugin for SwcLoaderPlugin {
  fn name(&self) -> &'static str {
    "SwcLoaderPlugin"
  }

  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &CompilerOptions) -> Result<()> {
    ctx
      .context
      .normal_module_factory_hooks
      .resolve_loader
      .tap(resolve_loader::new(self));
    Ok(())
  }
}

type SwcLoaderCache<'a> = LazyLock<RwLock<FxHashMap<(Cow<'a, str>, Cow<'a, str>), Arc<SwcLoader>>>>;
static SWC_LOADER_CACHE: SwcLoaderCache = LazyLock::new(|| RwLock::new(FxHashMap::default()));

#[plugin_hook(NormalModuleFactoryResolveLoader for SwcLoaderPlugin)]
pub(crate) async fn resolve_loader(
  &self,
  _context: &Context,
  _resolver: &Resolver,
  l: &ModuleRuleUseLoader,
) -> Result<Option<BoxLoader>> {
  let loader_request = &l.loader;
  let options = l.options.as_deref().unwrap_or("{}");

  if loader_request.starts_with(SWC_LOADER_IDENTIFIER) {
    if let Some(loader) = SWC_LOADER_CACHE
      .read()
      .await
      .get(&(Cow::Borrowed(loader_request), Cow::Borrowed(options)))
    {
      return Ok(Some(loader.clone()));
    }

    let loader = Arc::new(
      SwcLoader::new(options)
        .to_rspack_result_with_detail(options, "failed to parse builtin:swc-loader options")?
        .with_identifier(loader_request.as_str().into()),
    );

    SWC_LOADER_CACHE.write().await.insert(
      (
        Cow::Owned(loader_request.to_owned()),
        Cow::Owned(options.to_owned()),
      ),
      loader.clone(),
    );
    return Ok(Some(loader));
  }

  Ok(None)
}
