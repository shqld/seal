use swc_ecma_ast::{ClassDecl, Decl, FnDecl, Pat, VarDeclKind};

use crate::{
	TyKind,
	checker::{class::ClassChecker, errors::ErrorKind, function::FunctionChecker},
	symbol::Symbol,
};

use super::BaseChecker;

impl BaseChecker<'_> {
	pub fn check_decl(&self, decl: &Decl) {
		match decl {
			Decl::Var(var) => {
				let is_const = match var.kind {
					VarDeclKind::Var => {
						self.add_error(ErrorKind::Var);

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
						self.add_var(
							&name,
							// NOTE: to make every reference to the 'var' variable an error, set this type to never
							self.constants.never,
							false,
						);
						continue;
					}

					let ty = binding
						.type_ann
						.as_ref()
						.map(|type_ann| self.build_ts_type(&type_ann.type_ann))
						.unwrap_or_else(|| self.constants.lazy);

					if let Some(init) = &var_declarator.init {
						let expected = ty;
						let actual = self.check_expr(init);

						// if no type is specified to the declaration, replace with actual type
						if let TyKind::Lazy = ty.kind() {
							self.add_var(&name, actual, !is_const);
							return;
						}

						if !self.satisfies(expected, actual) {
							self.raise_type_error(expected, actual);
						}
					} else if is_const {
						self.add_error(ErrorKind::ConstMissingInit);
					};

					self.add_var(&name, ty, !is_const);
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
									self.add_error(ErrorKind::ParamMissingTypeAnn);
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
					self.add_error(error.kind);
				}

				self.add_var(&name, self.tcx.new_function(result.ty), false);
			}
			Decl::Class(ClassDecl { ident, class, .. }) => {
				let name = Symbol::new(ident.to_id());

				let checker = ClassChecker::new(self.tcx, &name);
				let result = checker.check_class(class);

				for error in result.errors {
					self.add_error(error.kind);
				}

				self.add_var(&name, self.tcx.new_class(result.ty), false);
			}
			_ => todo!("{:#?}", decl),
		}
	}
}
