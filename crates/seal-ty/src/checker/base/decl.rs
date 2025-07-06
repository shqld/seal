use swc_ecma_ast::{ClassDecl, Decl, FnDecl, Pat, VarDeclKind};
use swc_common::Spanned;

use crate::{
	TyKind,
	checker::{class::ClassChecker, errors::ErrorKind, function::FunctionChecker},
	sir::{Def, Value},
	symbol::Symbol,
};

use super::BaseChecker;

impl BaseChecker<'_> {
	pub fn check_decl(&self, decl: &Decl) {
		match decl {
			Decl::Var(var) => {
				let is_const = match var.kind {
					VarDeclKind::Var => {
						self.add_error_with_span(ErrorKind::Var, var.span);

						// NOTE: 'var' is not allowed, but check the following declarators
						//       to provide feedbacks on where the variable is referenced
						true
					}
					VarDeclKind::Const => true,
					VarDeclKind::Let => false,
				};

				for var_declarator in &var.decls {
					let binding = match &var_declarator.name {
						Pat::Ident(ident) => ident,
						_ => todo!("{:#?}", var_declarator.name),
					};

					let name = Symbol::new(binding.to_id());

					if let VarDeclKind::Var = var.kind {
						self.set_binding(
							&name,
							None,
							// NOTE: to make every reference to the 'var' variable an error, set this type to never
							self.constants.never,
							false,
						);
						continue;
					}

					let binding_ty = binding
						.type_ann
						.as_ref()
						.map(|type_ann| self.build_ts_type(&type_ann.type_ann))
						.unwrap_or_else(|| self.constants.lazy);

					if let Some(init) = &var_declarator.init {
						let actual = self.check_expr(init);

						// TODO: binding is Option<Local>, so we can remove TyKind::Lazy and check if it's None
						if let TyKind::Lazy = binding_ty.kind() {
							// if no type is specified to the declaration, replace with actual type
							self.set_binding(
								&name,
								Some(actual),
								match is_const {
									true => actual.ty,
									false => self.widen(actual.ty),
								},
								!is_const,
							);
							return;
						}

						if !self.satisfies(binding_ty, actual.ty) {
							self.raise_type_error(binding_ty, actual.ty, init.span());
						}

						self.set_binding(&name, Some(actual), binding_ty, !is_const);
					} else {
						if is_const {
							self.add_error_with_span(ErrorKind::ConstMissingInit, var_declarator.span);
						}

						self.set_binding(&name, None, binding_ty, !is_const);
					}
				}
			}
			Decl::Fn(FnDecl {
				ident, function, ..
			}) => {
				let name = Symbol::new(ident.to_id());

				let mut params = vec![];
				for param in &function.params {
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

				let ret = match &function.return_type {
					Some(type_ann) => self.build_ts_type(&type_ann.type_ann),
					None => {
						// NOTE: seal does't infer the return type
						self.constants.void
					}
				};

				let checker = FunctionChecker::new(self.tcx, params, ret);
				let result = checker.check_function(function);

				for error in result.errors {
					self.add_error_with_span(error.kind, error.span);
				}

				let ty = self.tcx.new_function(result.ty);

				self.set_binding(
					&name,
					Some(self.add_local(ty, Value::Ref(self.tcx.add_def(Def::Func(result.def))))),
					ty,
					false,
				);
			}
			Decl::Class(ClassDecl { ident, class, .. }) => {
				let name = Symbol::new(ident.to_id());

				let checker = ClassChecker::new_with_parent(self, &name);
				let result = checker.check_class(class);

				for error in result.errors {
					self.add_error_with_span(error.kind, error.span);
				}

				let ty = self.tcx.new_class(result.ty);

				self.set_binding(
					&name,
					Some(self.add_local(ty, Value::Ref(self.tcx.add_def(Def::Class(result.def))))),
					ty,
					false,
				);
			}
			_ => todo!("{:#?}", decl),
		}
	}
}
