use swc_ecma_ast::{
	AssignTarget, Decl, Expr, ExprStmt, FnDecl, Lit, ModuleItem, Pat, Program, ReturnStmt,
	SimpleAssignTarget, Stmt, TsSatisfiesExpr, VarDeclKind,
};

use super::{
	Sema,
	air::{self},
};

impl<'tcx> Sema<'tcx> {
	pub fn build(self, ast: &Program) -> air::Module<'tcx> {
		let stmts = match &ast {
			Program::Script(script) => &script.body,
			Program::Module(module) => &module
				.body
				.iter()
				.filter_map(|item| match item {
					ModuleItem::Stmt(stmt) => Some(stmt.clone()),
					_ => None,
				})
				.collect::<Vec<_>>(),
		};

		// TODO:
		for stmt in stmts {
			self.build_stmt(stmt);
		}

		self.finish_function();

		self.module.into_inner()
	}

	fn build_stmt(&self, stmt: &Stmt) {
		match stmt {
			Stmt::Decl(decl) => self.build_decl(decl),
			Stmt::Expr(ExprStmt { expr, .. }) => {
				let expr = self.build_expr(expr);

				match expr {
					air::Expr::Var(_) => {}
					_ => {
						self.add_expr_stmt(expr);
					}
				}
			}
			Stmt::Return(ReturnStmt { arg, .. }) => {
				if self.is_current_function_main() {
					panic!("Cannot return on main");
				}

				if let Some(arg) = arg {
					let expr = self.build_expr(arg);

					self.add_assign_stmt(self.get_current_function_ret().var().clone(), expr);
				}

				self.finish_block(Some(air::Term::Return));
			}
			_ => unimplemented!("{:#?}", stmt),
		}
	}

	fn build_decl(&self, decl: &Decl) {
		match decl {
			Decl::Var(var) => {
				let _is_const = match var.kind {
					VarDeclKind::Var => panic!("Var is not supported"),
					VarDeclKind::Const => true,
					VarDeclKind::Let => false,
				};

				for var_declarator in &var.decls {
					if let Some(init) = &var_declarator.init {
						let expr = self.build_expr(init);
						let binding = match &var_declarator.name {
							Pat::Ident(ident) => ident,
							_ => unimplemented!("{:#?}", var_declarator.name),
						};
						let id = binding.to_id();
						// let ty = binding
						// 	.type_ann
						// 	.as_ref()
						// 	.map(|type_ann| self.ty_builder.build_tstype(&type_ann.type_ann));

						self.add_assign_stmt(air::Var { id }, expr);
					}
				}
			}
			Decl::Fn(fn_decl) => {
				self.build_fn_decl(fn_decl);
			}
			_ => unimplemented!("{:#?}", decl),
		}
	}

	fn build_fn_decl(&self, fn_decl: &FnDecl) {
		let FnDecl {
			ident, function, ..
		} = fn_decl;

		let function_id = ident.to_id();

		let params = {
			let mut params = vec![];

			for param in &function.params {
				match &param.pat {
					Pat::Ident(ident) => {
						let id = ident.to_id();

						let ty = match &ident.type_ann {
							Some(type_ann) => self.ty_builder.build_tstype(&type_ann.type_ann),
							None => {
								panic!("Param type annotation is required");
							}
						};

						params.push(air::TypedVar::new(air::Var { id }, ty));
					}
					_ => unimplemented!("{:#?}", param),
				}
			}

			params
		};

		let ret = {
			let ty = match &function.return_type {
				Some(type_ann) => self.ty_builder.build_tstype(&type_ann.type_ann),
				None => {
					// NOTE: seal does't infer the return type
					self.tcx.new_ty(crate::TyKind::Void)
				}
			};

			air::TypedVar::new(air::Var::new_ret(&function_id), ty)
		};

		self.start_function(air::Function {
			id: function_id.clone(),
			params,
			ret,
			body: vec![self.new_block()],
		});

		let body = match &function.body {
			Some(body) => body,
			None => panic!("Function body is required"),
		};

		for stmt in &body.stmts {
			self.build_stmt(stmt);
		}

		self.finish_function();
	}

	fn build_expr(&self, expr: &Expr) -> air::Expr {
		match expr {
			Expr::Assign(assign) => {
				let binding = match &assign.left {
					AssignTarget::Simple(target) => match &target {
						SimpleAssignTarget::Ident(ident) => ident,
						_ => unimplemented!("{:#?}", target),
					},
					_ => unimplemented!("{:#?}", assign.left),
				};
				let id = binding.to_id();
				// let ty = binding
				// 	.type_ann
				// 	.as_ref()
				// 	.map(|type_ann| self.ty_builder.build_tstype(&type_ann.type_ann));
				let var = air::Var { id };

				self.add_assign_stmt(var.clone(), self.build_expr(&assign.right));

				air::Expr::Var(var)
			}
			Expr::TsSatisfies(TsSatisfiesExpr { expr, type_ann, .. }) => {
				let expr = self.build_expr(expr);

				self.add_satisfies_stmt(
					// TODO:
					expr.clone(),
					self.ty_builder.build_tstype(type_ann),
				);

				expr
			}
			Expr::Lit(lit) => air::Expr::Const(match lit {
				Lit::Bool(val) => air::Const::Boolean(val.value),
				Lit::Num(val) => air::Const::Number(val.value),
				Lit::Str(val) => air::Const::String(val.value.clone()),
				_ => unimplemented!(),
			}),
			Expr::Ident(ident) => {
				let id = ident.to_id();
				let var = air::Var { id };

				air::Expr::Var(var)
			}
			_ => unimplemented!("{:#?}", expr),
		}
	}
}
