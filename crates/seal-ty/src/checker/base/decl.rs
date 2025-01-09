use swc_ecma_ast::{Decl, Pat, VarDeclKind};

use crate::{TyKind, symbol::Symbol};

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
			Decl::Fn(_) => {
				unreachable!("Function declaration must be handled in module or function context");
			}
			_ => unimplemented!("{:#?}", decl),
		}
	}
}
