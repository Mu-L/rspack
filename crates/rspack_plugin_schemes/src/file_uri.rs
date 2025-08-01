use rspack_core::{
  ApplyContext, CompilerOptions, ModuleFactoryCreateData, NormalModuleFactoryResolveForScheme,
  Plugin, PluginContext, ResourceData, Scheme,
};
use rspack_error::{Result, ToStringResultToRspackResultExt, error};
use rspack_hook::{plugin, plugin_hook};
use rspack_paths::AssertUtf8;
use url::Url;

#[plugin]
#[derive(Debug, Default)]
pub struct FileUriPlugin;

#[plugin_hook(NormalModuleFactoryResolveForScheme for FileUriPlugin)]
async fn normal_module_factory_resolve_for_scheme(
  &self,
  _data: &mut ModuleFactoryCreateData,
  resource_data: &mut ResourceData,
  scheme: &Scheme,
) -> Result<Option<bool>> {
  if scheme.is_file() {
    let url = Url::parse(&resource_data.resource).to_rspack_result()?;
    let path = url
      .to_file_path()
      .map_err(|_| error!("Failed to get file path of {url}"))?
      .assert_utf8();
    let query = url.query().map(|q| format!("?{q}"));
    let fragment = url.fragment().map(|f| format!("#{f}"));
    let new_resource_data = ResourceData::new(format!(
      "{}{}{}",
      path,
      query.as_deref().unwrap_or(""),
      fragment.as_deref().unwrap_or("")
    ))
    .path(path)
    .query_optional(query)
    .fragment_optional(fragment);
    *resource_data = new_resource_data;
    return Ok(Some(true));
  }
  Ok(None)
}

#[async_trait::async_trait]
impl Plugin for FileUriPlugin {
  fn name(&self) -> &'static str {
    "rspack.FileUriPlugin"
  }

  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &CompilerOptions) -> Result<()> {
    ctx
      .context
      .normal_module_factory_hooks
      .resolve_for_scheme
      .tap(normal_module_factory_resolve_for_scheme::new(self));
    Ok(())
  }
}
