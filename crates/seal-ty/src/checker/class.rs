use std::collections::HashMap;
use std::rc::Rc;
use std::{collections::BTreeMap, ops::Deref};

use swc_ecma_ast::{Class, ClassMember, Constructor, ParamOrTsParamProp, Pat, PropName, Stmt};

use super::base::BaseChecker;
use super::errors::{Error, ErrorKind};
use super::function::FunctionChecker;

use crate::sir::{self, Value};
use crate::{context::TyContext, symbol::Symbol};

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
	pub fn new(tcx: &'tcx TyContext<'tcx>, name: &Symbol) -> ClassChecker<'tcx> {
		let base = BaseChecker::new(tcx);

		ClassChecker {
			base,
			name: name.clone(),
		}
	}

	pub fn check_class(self, class: &Class) -> ClassCheckerResult<'tcx> {
		let mut ctor = None;
		let mut field_tys = BTreeMap::new();
		let mut methods = vec![];

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
								self.raise_type_error(ty, init.ty);
							}
							ty
						}
						(Some(ty), None) => ty,
						(None, Some(init)) => init.ty,
						(None, None) => {
							self.add_error(ErrorKind::ClassPropMissingTypeAnnOrInit);
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
										self.add_error(ErrorKind::ParamMissingTypeAnn);
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
						self.add_error(error.kind);
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
			ty: crate::kind::Class::new(
				ctor_ty,
				Rc::new(crate::kind::Interface::new(self.name.clone(), field_tys)),
			),
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
								self.add_error(ErrorKind::ParamMissingTypeAnn);
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
				self.add_error(ErrorKind::MissingBody);
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
				Stmt::Return(_) => {
					checker.add_error(ErrorKind::ClassCtorWithReturn);
				}
				_ => checker.check_stmt(stmt),
			}
		}

		for error in checker.errors.into_inner() {
			self.add_error(error.kind);
		}

		(
			crate::kind::Function::new(params, self.constants.void),
			sir::Func {
				locals: checker.locals.into_inner(),
			},
		)
	}
}
