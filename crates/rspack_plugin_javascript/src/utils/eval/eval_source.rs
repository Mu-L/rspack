use std::{fmt::Display, sync::Arc};

use rspack_error::{TraceableError, miette::Severity};
use serde_json::json;
use swc_core::{
  common::{FileName, Spanned},
  ecma::{
    ast::EsVersion,
    parser::{EsSyntax, Syntax, parse_file_as_expr},
  },
};

use super::BasicEvaluatedExpression;
use crate::visitors::JavascriptParser;

#[inline]
pub fn eval_source<T: Display>(
  parser: &mut JavascriptParser,
  source: String,
  error_title: T,
) -> Option<BasicEvaluatedExpression<'static>> {
  let cm: Arc<swc_core::common::SourceMap> = Default::default();
  let fm = cm.new_source_file(Arc::new(FileName::Anon), source);
  let result = parse_file_as_expr(
    &fm,
    Syntax::Es(EsSyntax::default()),
    EsVersion::EsNext,
    None,
    &mut vec![],
  );
  match result {
    Err(err) => {
      let span = err.span();
      // Push the error to diagnostics
      parser.warning_diagnostics.push(Box::new(
        TraceableError::from_source_file(
          &fm,
          span.lo.0.saturating_sub(1) as usize,
          span.hi.0.saturating_sub(1) as usize,
          format!("{error_title} warning"),
          format!("failed to parse {}", json!(fm.src.as_str())),
        )
        .with_severity(Severity::Warning),
      ));
      None
    }
    Ok(expr) => BasicEvaluatedExpression::with_owned_expression(*expr, |expr| {
      Some(parser.evaluate_expression(expr))
    }),
  }
}
