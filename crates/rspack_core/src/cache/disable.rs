use rspack_error::Result;

use super::Cache;
use crate::make::MakeArtifact;

/// Disable cache implementation
///
/// Disable cache will clean the corresponding artifact before target step run.
#[derive(Debug)]
pub struct DisableCache;

#[async_trait::async_trait]
impl Cache for DisableCache {
  async fn before_make(&mut self, make_artifact: &mut MakeArtifact) -> Result<()> {
    *make_artifact = Default::default();
    Ok(())
  }
}
