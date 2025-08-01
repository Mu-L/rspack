use rspack_core::{
  ContextMode, ContextOptions, DependencyCategory, SpanExt, try_convert_str_to_context_mode,
};
use rspack_regex::RspackRegex;
use swc_core::{common::Spanned, ecma::ast::CallExpr};

use super::JavascriptParserPlugin;
use crate::{
  dependency::RequireContextDependency,
  visitors::{JavascriptParser, clean_regexp_in_context_module, expr_matcher::is_require_context},
};

pub struct RequireContextDependencyParserPlugin;

const DEFAULT_REGEXP_STR: &str = r"^\.\/.*$";

impl JavascriptParserPlugin for RequireContextDependencyParserPlugin {
  fn call(&self, parser: &mut JavascriptParser, expr: &CallExpr, _name: &str) -> Option<bool> {
    if expr
      .callee
      .as_expr()
      .is_none_or(|expr| !is_require_context(&**expr))
    {
      return None;
    }

    let mode = if expr.args.len() == 4 {
      let mode_expr = parser.evaluate_expression(&expr.args[3].expr);
      if !mode_expr.is_string() {
        // FIXME: return `None` in webpack
        ContextMode::Sync
      } else if let Some(mode_expr) = try_convert_str_to_context_mode(mode_expr.string()) {
        mode_expr
      } else {
        ContextMode::Sync
      }
    } else {
      ContextMode::Sync
    };

    let (reg_exp, reg_exp_span) = if expr.args.len() >= 3 {
      let reg_exp_expr = parser.evaluate_expression(&expr.args[2].expr);
      let reg_exp = if !reg_exp_expr.is_regexp() {
        // FIXME: return `None` in webpack
        RspackRegex::new(DEFAULT_REGEXP_STR).expect("reg should success")
      } else {
        let (expr, flags) = reg_exp_expr.regexp();
        RspackRegex::with_flags(expr.as_str(), flags.as_str()).expect("reg should success")
      };
      (reg_exp, Some(expr.args[2].expr.span().into()))
    } else {
      (
        RspackRegex::new(DEFAULT_REGEXP_STR).expect("reg should success"),
        None,
      )
    };

    let recursive = if expr.args.len() >= 2 {
      let recursive_expr = parser.evaluate_expression(&expr.args[1].expr);
      if !recursive_expr.is_bool() {
        // FIXME: return `None` in webpack
        true
      } else {
        recursive_expr.bool()
      }
    } else {
      true
    };

    if let Some(arg) = expr.args.first() {
      let request_expr = parser.evaluate_expression(&arg.expr);
      if !request_expr.is_string() {
        return None;
      }

      let reg_exp = clean_regexp_in_context_module(reg_exp, reg_exp_span, parser);
      parser
        .dependencies
        .push(Box::new(RequireContextDependency::new(
          ContextOptions {
            mode,
            recursive,
            reg_exp,
            include: None,
            exclude: None,
            category: DependencyCategory::CommonJS,
            request: request_expr.string().to_string(),
            context: request_expr.string().to_string(),
            namespace_object: rspack_core::ContextNameSpaceObject::Unset,
            group_options: None,
            replaces: Vec::new(),
            start: expr.span().real_lo(),
            end: expr.span().real_hi(),
            referenced_exports: None,
            attributes: None,
          },
          expr.span.into(),
          parser.in_try,
        )));
      return Some(true);
    }

    None
  }
}
