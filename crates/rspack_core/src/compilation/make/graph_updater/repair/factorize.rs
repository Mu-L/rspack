use std::sync::Arc;

use rspack_error::Diagnostic;
use rspack_sources::BoxSource;

use super::{TaskContext, add::AddTask};
use crate::{
  BoxDependency, CompilationId, CompilerId, CompilerOptions, Context, ExportsInfoData,
  FactorizeInfo, ModuleFactory, ModuleFactoryCreateData, ModuleFactoryResult, ModuleIdentifier,
  ModuleLayer, ModuleProfile, Resolve, ResolverFactory,
  module_graph::ModuleGraphModule,
  utils::task_loop::{Task, TaskResult, TaskType},
};

#[derive(Debug)]
pub struct FactorizeTask {
  pub compiler_id: CompilerId,
  pub compilation_id: CompilationId,
  pub module_factory: Arc<dyn ModuleFactory>,
  pub original_module_identifier: Option<ModuleIdentifier>,
  pub original_module_source: Option<BoxSource>,
  pub original_module_context: Option<Box<Context>>,
  pub issuer: Option<Box<str>>,
  pub issuer_layer: Option<ModuleLayer>,
  pub dependencies: Vec<BoxDependency>,
  pub resolve_options: Option<Arc<Resolve>>,
  pub options: Arc<CompilerOptions>,
  pub current_profile: Option<Box<ModuleProfile>>,
  pub resolver_factory: Arc<ResolverFactory>,
}

#[async_trait::async_trait]
impl Task<TaskContext> for FactorizeTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Background
  }
  async fn background_run(self: Box<Self>) -> TaskResult<TaskContext> {
    if let Some(current_profile) = &self.current_profile {
      current_profile.mark_factory_start();
    }
    let dependency = &self.dependencies[0];

    let context = if let Some(context) = dependency.get_context()
      && !context.is_empty()
    {
      context
    } else if let Some(context) = &self.original_module_context
      && !context.is_empty()
    {
      context
    } else {
      &self.options.context
    }
    .clone();

    let issuer_layer = dependency
      .get_layer()
      .or(self.issuer_layer.as_ref())
      .cloned();

    let request = self.dependencies[0]
      .as_module_dependency()
      .map(|d| d.request().to_string())
      .or_else(|| {
        self.dependencies[0]
          .as_context_dependency()
          .map(|d| d.request().to_string())
      })
      .unwrap_or_default();
    // Error and result are not mutually exclusive in webpack module factorization.
    // Rspack puts results that need to be shared in both error and ok in [ModuleFactoryCreateData].
    let mut create_data = ModuleFactoryCreateData {
      compiler_id: self.compiler_id,
      compilation_id: self.compilation_id,
      resolve_options: self.resolve_options,
      options: self.options.clone(),
      context,
      request,
      dependencies: self.dependencies,
      issuer: self.issuer,
      issuer_identifier: self.original_module_identifier,
      issuer_layer,
      resolver_factory: self.resolver_factory,

      file_dependencies: Default::default(),
      missing_dependencies: Default::default(),
      context_dependencies: Default::default(),
      diagnostics: Default::default(),
    };
    let factory_result = match self.module_factory.create(&mut create_data).await {
      Ok(result) => Some(result),
      Err(mut e) => {
        // Wrap source code if available
        if let Some(s) = self.original_module_source {
          let has_source_code = e.source_code().is_some();
          if !has_source_code {
            e = e.with_source_code(s.source().to_string());
          }
        }
        // Bail out if `options.bail` set to `true`,
        // which means 'Fail out on the first error instead of tolerating it.'
        if self.options.bail {
          return Err(e);
        }
        create_data.diagnostics.insert(
          0,
          Into::<Diagnostic>::into(e).with_loc(create_data.dependencies[0].loc()),
        );
        None
      }
    };

    if let Some(current_profile) = &self.current_profile {
      current_profile.mark_factory_end();
    }

    let factorize_info = FactorizeInfo::new(
      create_data.diagnostics,
      create_data
        .dependencies
        .iter()
        .map(|dep| *dep.id())
        .collect(),
      create_data.file_dependencies,
      create_data.context_dependencies,
      create_data.missing_dependencies,
    );
    let exports_info = ExportsInfoData::default();
    Ok(vec![Box::new(FactorizeResultTask {
      original_module_identifier: self.original_module_identifier,
      factory_result,
      dependencies: create_data.dependencies,
      current_profile: self.current_profile,
      exports_info_related: exports_info,
      factorize_info,
    })])
  }
}

#[derive(Debug)]
pub struct FactorizeResultTask {
  //  pub dependency: DependencyId,
  pub original_module_identifier: Option<ModuleIdentifier>,
  /// Result will be available if [crate::ModuleFactory::create] returns `Ok`.
  pub factory_result: Option<ModuleFactoryResult>,
  pub dependencies: Vec<BoxDependency>,
  pub current_profile: Option<Box<ModuleProfile>>,
  pub exports_info_related: ExportsInfoData,
  pub factorize_info: FactorizeInfo,
}

#[async_trait::async_trait]
impl Task<TaskContext> for FactorizeResultTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Main
  }
  async fn main_run(self: Box<Self>, context: &mut TaskContext) -> TaskResult<TaskContext> {
    let FactorizeResultTask {
      original_module_identifier,
      factory_result,
      mut dependencies,
      current_profile,
      exports_info_related,
      mut factorize_info,
    } = *self;

    let artifact = &mut context.artifact;
    if !factorize_info.diagnostics().is_empty() {
      artifact
        .file_dependencies
        .add_batch_file(&factorize_info.file_dependencies());
      artifact
        .context_dependencies
        .add_batch_file(&factorize_info.context_dependencies());
      artifact
        .missing_dependencies
        .add_batch_file(&factorize_info.missing_dependencies());
      artifact
        .make_failed_dependencies
        .insert(*dependencies[0].id());
    }
    // write factorize_info to dependencies[0] and set success factorize_info to others
    for dep in &mut dependencies {
      let dep_factorize_info = if let Some(d) = dep.as_context_dependency_mut() {
        d.factorize_info_mut()
      } else if let Some(d) = dep.as_module_dependency_mut() {
        d.factorize_info_mut()
      } else {
        unreachable!("only module dependency and context dependency can factorize")
      };
      *dep_factorize_info = std::mem::take(&mut factorize_info);
    }

    let module_graph = &mut TaskContext::get_module_graph_mut(&mut artifact.module_graph_partial);
    let Some(factory_result) = factory_result else {
      let dep = &dependencies[0];
      tracing::trace!("Module created with failure, but without bailout: {dep:?}");
      // sync dependencies to mg
      for dep in dependencies {
        module_graph.add_dependency(dep)
      }
      return Ok(vec![]);
    };

    let Some(module) = factory_result.module else {
      let dep = &dependencies[0];
      tracing::trace!("Module ignored: {dep:?}");
      // sync dependencies to mg
      for dep in dependencies {
        module_graph.add_dependency(dep)
      }
      return Ok(vec![]);
    };
    let module_identifier = module.identifier();
    let mut mgm = ModuleGraphModule::new(module.identifier(), exports_info_related.id());
    mgm.set_issuer_if_unset(original_module_identifier);

    module_graph.set_exports_info(exports_info_related.id(), exports_info_related);
    tracing::trace!("Module created: {}", &module_identifier);

    Ok(vec![Box::new(AddTask {
      original_module_identifier,
      module,
      module_graph_module: Box::new(mgm),
      dependencies,
      current_profile,
    })])
  }
}
