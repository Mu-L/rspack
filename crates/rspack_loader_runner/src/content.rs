use std::{
  fmt::Debug,
  path::{Path, PathBuf},
  sync::Arc,
};

use anymap::CloneAny;
use once_cell::sync::OnceCell;
use rspack_cacheable::{
  cacheable,
  with::{AsInner, AsOption, AsPreset, AsString},
};
use rspack_error::{Error, Result, ToStringResultToRspackResultExt};
use rspack_paths::Utf8PathBuf;
use rustc_hash::FxHashMap;

use crate::{Scheme, get_scheme};

#[derive(Clone, PartialEq, Eq)]
pub enum Content {
  String(String),
  Buffer(Vec<u8>),
}

impl Content {
  pub fn try_into_string(self) -> Result<String> {
    match self {
      Content::String(s) => Ok(s),
      Content::Buffer(b) => String::from_utf8(b).to_rspack_result(),
    }
  }

  pub fn into_string_lossy(self) -> String {
    match self {
      Content::String(s) => s,
      Content::Buffer(b) => String::from_utf8_lossy(&b).into_owned(),
    }
  }

  pub fn as_bytes(&self) -> &[u8] {
    match self {
      Content::String(s) => s.as_bytes(),
      Content::Buffer(b) => b,
    }
  }

  pub fn into_bytes(self) -> Vec<u8> {
    match self {
      Content::String(s) => s.into_bytes(),
      Content::Buffer(b) => b,
    }
  }

  pub fn is_buffer(&self) -> bool {
    matches!(self, Content::Buffer(..))
  }

  pub fn is_string(&self) -> bool {
    matches!(self, Content::String(..))
  }
}

impl TryFrom<Content> for String {
  type Error = Error;

  fn try_from(content: Content) -> Result<Self> {
    content.try_into_string()
  }
}

impl From<Content> for Vec<u8> {
  fn from(content: Content) -> Self {
    content.into_bytes()
  }
}

impl From<String> for Content {
  fn from(s: String) -> Self {
    Self::String(s)
  }
}

impl From<Vec<u8>> for Content {
  fn from(buf: Vec<u8>) -> Self {
    Self::Buffer(buf)
  }
}

impl Debug for Content {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut content = f.debug_struct("Content");

    let s = match self {
      Self::String(s) => s.to_string(),
      Self::Buffer(b) => String::from_utf8_lossy(b).to_string(),
    };

    let ty = match self {
      Self::String(_) => "String",
      Self::Buffer(_) => "Buffer",
    };

    // Safe string truncation to approximately 20 characters
    let truncated = if s.len() <= 20 {
      s.as_str()
    } else {
      // Find the last character boundary at or before position 20
      let mut end = 20;
      while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
      }
      &s[0..end]
    };

    content.field(ty, &truncated.to_owned()).finish()
  }
}

#[cacheable]
#[derive(Debug, Clone)]
pub struct ResourceData {
  /// Resource with absolute path, query and fragment
  pub resource: String,
  /// Absolute resource path only
  #[cacheable(with=AsOption<AsPreset>)]
  pub resource_path: Option<Utf8PathBuf>,
  /// Resource query with `?` prefix
  pub resource_query: Option<String>,
  /// Resource fragment with `#` prefix
  pub resource_fragment: Option<String>,
  pub resource_description: Option<DescriptionData>,
  pub mimetype: Option<String>,
  pub parameters: Option<String>,
  pub encoding: Option<String>,
  pub encoded_content: Option<String>,
  pub context: Option<String>,
  #[cacheable(with=AsInner)]
  pub(crate) scheme: OnceCell<Scheme>,
}

impl ResourceData {
  pub fn new(resource: String) -> Self {
    Self {
      resource,
      resource_path: None,
      resource_query: None,
      resource_fragment: None,
      resource_description: None,
      mimetype: None,
      parameters: None,
      encoding: None,
      encoded_content: None,
      scheme: OnceCell::new(),
      context: None,
    }
  }
  pub fn set_context(&mut self, context: Option<String>) {
    self.context = context;
  }
  pub fn get_context(&self) -> Option<&String> {
    self.context.as_ref()
  }

  pub fn get_scheme(&self) -> &Scheme {
    self.scheme.get_or_init(|| get_scheme(&self.resource))
  }

  pub fn set_resource(&mut self, v: String) {
    self.resource = v;
  }

  pub fn path<P: Into<Utf8PathBuf>>(mut self, v: P) -> Self {
    self.resource_path = Some(v.into());
    self
  }

  pub fn set_path<P: Into<Utf8PathBuf>>(&mut self, v: P) {
    self.resource_path = Some(v.into());
  }

  pub fn set_path_optional<P: Into<Utf8PathBuf>>(&mut self, v: Option<P>) {
    self.resource_path = v.map(Into::into);
  }

  pub fn query(mut self, v: String) -> Self {
    self.resource_query = Some(v);
    self
  }

  pub fn set_query(&mut self, v: String) {
    self.resource_query = Some(v);
  }

  pub fn query_optional(mut self, v: Option<String>) -> Self {
    self.resource_query = v;
    self
  }

  pub fn set_query_optional(&mut self, v: Option<String>) {
    self.resource_query = v;
  }

  pub fn fragment(mut self, v: String) -> Self {
    self.resource_fragment = Some(v);
    self
  }

  pub fn set_fragment(&mut self, v: String) {
    self.resource_fragment = Some(v);
  }

  pub fn fragment_optional(mut self, v: Option<String>) -> Self {
    self.resource_fragment = v;
    self
  }

  pub fn set_fragment_optional(&mut self, v: Option<String>) {
    self.resource_fragment = v;
  }

  pub fn description(mut self, v: DescriptionData) -> Self {
    self.resource_description = Some(v);
    self
  }

  pub fn description_optional(mut self, v: Option<DescriptionData>) -> Self {
    self.resource_description = v;
    self
  }

  pub fn mimetype(mut self, v: String) -> Self {
    self.mimetype = Some(v);
    self
  }

  pub fn set_mimetype(&mut self, v: String) {
    self.mimetype = Some(v);
  }

  pub fn parameters(mut self, v: String) -> Self {
    self.parameters = Some(v);
    self
  }

  pub fn set_parameters(&mut self, v: String) {
    self.parameters = Some(v);
  }

  pub fn encoding(mut self, v: String) -> Self {
    self.encoding = Some(v);
    self
  }

  pub fn set_encoding(&mut self, v: String) {
    self.encoding = Some(v);
  }

  pub fn encoded_content(mut self, v: String) -> Self {
    self.encoded_content = Some(v);
    self
  }

  pub fn set_encoded_content(&mut self, v: String) {
    self.encoded_content = Some(v);
  }
}

/// Used for [Rule.descriptionData](https://rspack.rs/config/module.html#ruledescriptiondata) and
/// package.json.sideEffects in tree shaking.
#[cacheable]
#[derive(Debug, Clone)]
pub struct DescriptionData {
  /// Path to package.json
  #[cacheable(with=AsString)]
  path: PathBuf,

  /// Raw package.json
  #[cacheable(with=AsInner<AsPreset>)]
  json: Arc<serde_json::Value>,
}

impl DescriptionData {
  pub fn new(path: PathBuf, json: Arc<serde_json::Value>) -> Self {
    Self { path, json }
  }

  pub fn path(&self) -> &Path {
    &self.path
  }

  pub fn json(&self) -> &serde_json::Value {
    self.json.as_ref()
  }

  pub fn into_parts(self) -> (PathBuf, Arc<serde_json::Value>) {
    (self.path, self.json)
  }
}

pub type AdditionalData = anymap::Map<dyn CloneAny + Send + Sync>;
pub type ParseMeta = FxHashMap<String, Box<dyn CloneAny + Send + Sync>>;
