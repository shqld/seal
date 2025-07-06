use std::collections::HashMap;
use std::rc::Rc;
use std::{collections::BTreeMap, ops::Deref};

use swc_ecma_ast::{Class, ClassMember, Constructor, ParamOrTsParamProp, Pat, PropName, Stmt};
use swc_common::Spanned;

use super::base::BaseChecker;
use super::errors::{Error, ErrorKind};
use super::function::FunctionChecker;

use crate::sir::{self, Value};
use crate::symbol::Symbol;

pub struct ClassCheckerResult<'tcx> {
	pub ty: crate::kind::Class<'tcx>,
	pub def: sir::Class,
	pub errors: Vec<Error<'tcx>>,
}

#[derive(Debug)]
pub struct ClassChecker<'tcx> {
	// TODO: replace with more basic checker (only with tcx, errors, and constants)
	base: BaseChecker<'tcx>,
	name: Symbol,
}

impl<'tcx> Deref for ClassChecker<'tcx> {
	type Target = BaseChecker<'tcx>;

	fn deref(&self) -> &Self::Target {
		&self.base
	}
}

impl<'tcx> ClassChecker<'tcx> {
	pub fn new_with_parent(parent: &BaseChecker<'tcx>, name: &Symbol) -> ClassChecker<'tcx> {
		let base = parent.new_scoped_checker();

		ClassChecker {
			base,
			name: name.clone(),
		}
	}

	pub fn check_class(self, class: &Class) -> ClassCheckerResult<'tcx> {
		// Check for parent class
		let parent: Option<crate::Ty<'tcx>> = if let Some(super_class) = &class.super_class {
			// Evaluate the super class expression
			let parent_local = self.check_expr(super_class);
			// The parent should be a class type
			match parent_local.ty.kind() {
				crate::TyKind::Class(_) => Some(parent_local.ty),
				_ => {
					self.add_error_with_span(ErrorKind::ExtendsNonClass(parent_local.ty), super_class.span());
					None
				}
			}
		} else {
			None
		};

		let mut ctor = None;
		let mut field_tys = BTreeMap::new();
		let mut methods = vec![];

		// If there's a parent class, inherit its fields
		if let Some(parent_ty) = parent {
			if let crate::TyKind::Class(parent_class) = parent_ty.kind() {
				// Copy parent fields to child
				for (key, ty) in parent_class.fields() {
					field_tys.insert(key.clone(), *ty);
				}
			}
		}

		for member in &class.body {
			match member {
				ClassMember::Constructor(consructor) => {
					ctor = Some(self.check_constructor(consructor));
				}
				ClassMember::ClassProp(prop) => {
					let key = match &prop.key {
						PropName::Ident(ident) => ident.sym.clone(),
						_ => todo!("{:#?}", prop.key),
					};

					let ty = prop
						.type_ann
						.as_ref()
						.map(|type_ann| self.build_ts_type(&type_ann.type_ann));
					let init = prop.value.as_ref().map(|value| self.check_expr(value));

					let ty = match (ty, init) {
						(Some(ty), Some(init)) => {
							if !self.satisfies(ty, init.ty) {
								self.raise_type_error(ty, init.ty, prop.span);
							}
							ty
						}
						(Some(ty), None) => ty,
						(None, Some(init)) => init.ty,
						(None, None) => {
							self.add_error_with_span(ErrorKind::ClassPropMissingTypeAnnOrInit, prop.span);
							self.constants.err
						}
					};

					field_tys.insert(key, ty);
				}
				ClassMember::Method(method) => {
					let key = match &method.key {
						PropName::Ident(ident) => ident.sym.clone(),
						_ => todo!("{:#?}", method.key),
					};

					let mut params = vec![];
					for param in &method.function.params {
						match &param.pat {
							Pat::Ident(ident) => {
								let name = Symbol::new(ident.to_id());

								let ty = match &ident.type_ann {
									Some(type_ann) => self.build_ts_type(&type_ann.type_ann),
									None => {
										self.add_error_with_span(ErrorKind::ParamMissingTypeAnn, ident.span);
										self.constants.err
									}
								};

								params.push((name, ty));
							}
							_ => todo!("{:#?}", param),
						}
					}

					let ret = match &method.function.return_type {
						Some(type_ann) => self.build_ts_type(&type_ann.type_ann),
						None => {
							// NOTE: seal does't infer the return type
							self.constants.void
						}
					};

					let checker = FunctionChecker::new(self.tcx, params, ret);
					let result = checker.check_function(&method.function);

					for error in result.errors {
						self.add_error_with_span(error.kind, error.span);
					}

					field_tys.insert(key, self.tcx.new_function(result.ty));
					methods.push(result.def);
				}
				_ => todo!("{:#?}", member),
			}
		}

		let (ctor_ty, ctor) = match ctor {
			Some((ctor_ty, ctor)) => (Some(ctor_ty), Some(ctor)),
			None => (None, None),
		};

		ClassCheckerResult {
			ty: if let Some(parent_ty) = parent {
				crate::kind::Class::new_with_parent(
					ctor_ty,
					Rc::new(crate::kind::Interface::new(self.name.clone(), field_tys)),
					parent_ty,
				)
			} else {
				crate::kind::Class::new(
					ctor_ty,
					Rc::new(crate::kind::Interface::new(self.name.clone(), field_tys)),
				)
			},
			def: sir::Class { ctor, methods },
			errors: self.base.errors.into_inner(),
		}
	}

	pub fn check_constructor(
		&self,
		consructor: &Constructor,
	) -> (crate::kind::Function<'tcx>, sir::Func) {
		let mut params = vec![];
		for param in &consructor.params {
			match param {
				ParamOrTsParamProp::Param(param) => match &param.pat {
					Pat::Ident(ident) => {
						let name = Symbol::new(ident.to_id());

						let ty = match &ident.type_ann {
							Some(type_ann) => self.build_ts_type(&type_ann.type_ann),
							None => {
								self.add_error_with_span(ErrorKind::ParamMissingTypeAnn, ident.span);
								self.constants.err
							}
						};

						params.push((name, ty));
					}
					_ => todo!("{:#?}", param),
				},
				_ => todo!("{:#?}", param),
			}
		}

		// NOTE: Constructor cannot have return stmt in seal, so we should not use FunctionChecker
		let checker = BaseChecker::new(self.tcx);

		for (name, ty) in &params {
			let param = checker.add_local(*ty, Value::Param);
			checker.set_binding(name, Some(param), *ty, false);
		}

		let body = match &consructor.body {
			Some(body) => body,
			_ => {
				self.add_error_with_span(ErrorKind::MissingBody, consructor.span);
				return (
					crate::kind::Function::new(params, self.constants.void),
					sir::Func {
						locals: HashMap::new(),
					},
				);
			}
		};

		for stmt in &body.stmts {
			match stmt {
				Stmt::Return(ret_stmt) => {
					checker.add_error_with_span(ErrorKind::ClassCtorWithReturn, ret_stmt.span);
				}
				_ => checker.check_stmt(stmt),
			}
		}

		for error in checker.errors.into_inner() {
			self.add_error_with_span(error.kind, error.span);
		}

		(
			crate::kind::Function::new(params, self.constants.void),
			sir::Func {
				locals: checker.locals.into_inner(),
			},
		)
	}
}
