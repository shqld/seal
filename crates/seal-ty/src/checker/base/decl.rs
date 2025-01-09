use swc_ecma_ast::{ClassDecl, Decl, FnDecl, Pat, VarDeclKind};

use crate::{
	TyKind,
	checker::{class::ClassChecker, function::FunctionChecker},
	symbol::Symbol,
};

use super::BaseChecker;

impl BaseChecker<'_> {
	pub fn check_decl(&self, decl: &Decl) {
		match decl {
			Decl::Var(var) => {
				let is_const = match var.kind {
					VarDeclKind::Var => panic!("Var is not supported"),
					VarDeclKind::Const => true,
					VarDeclKind::Let => false,
				};

				for var_declarator in &var.decls {
					let binding = match &var_declarator.name {
						Pat::Ident(ident) => ident,
						_ => unimplemented!("{:#?}", var_declarator.name),
					};
					let ty = binding
						.type_ann
						.as_ref()
						.map(|type_ann| self.build_ts_type(&type_ann.type_ann))
						.unwrap_or_else(|| self.constants.lazy);

					let binding = Symbol::new(binding.to_id());

					if let Some(init) = &var_declarator.init {
						let expected = ty;
						let actual = self.check_expr(init);

						// if no type is specified to the declaration, replace with actual type
						if let TyKind::Lazy = ty.kind() {
							self.add_var(&binding, actual, !is_const);
							return;
						}

						if !self.satisfies(expected, actual) {
							self.raise_type_error(expected, actual);
						}
					} else if is_const {
						panic!("Const variable must be initialized");
					};

					self.add_var(&binding, ty, !is_const);
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
									panic!("Param type annotation is required");
								}
							};

							params.push((name, ty));
						}
						_ => unimplemented!("{:#?}", param),
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
				let function = checker.check_function(function);

				if let Err(errors) = checker.into_result() {
					for error in errors {
						self.add_error(error);
					}
				};

				self.add_var(&name, self.tcx.new_function(function), false);
			}
			Decl::Class(ClassDecl { ident, class, .. }) => {
				let name = Symbol::new(ident.to_id());

				let checker = ClassChecker::new(self.tcx, &name);
				let class = checker.check_class(class);

				if let Err(errors) = checker.into_result() {
					for error in errors {
						self.add_error(error);
					}
				};

				self.add_var(&name, self.tcx.new_class(class), false);
			}
			_ => unimplemented!("{:#?}", decl),
		}
	}
}
