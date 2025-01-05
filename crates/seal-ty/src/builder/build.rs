use std::collections::BTreeSet;

use swc_ecma_ast::{
	AssignTarget, BinaryOp, Decl, Expr, ExprStmt, FnDecl, IfStmt, Lit, ModuleItem, Pat, Program,
	ReturnStmt, SimpleAssignTarget, Stmt, TsFnOrConstructorType, TsKeywordTypeKind, TsLit,
	TsLitType, TsSatisfiesExpr, TsType, TsUnionOrIntersectionType, UnaryOp, VarDeclKind,
};

use crate::Ty;

use super::{
	Sema,
	sir::{self},
};

impl<'tcx> Sema<'tcx> {
	pub fn build(self, ast: &Program) -> sir::Module<'tcx> {
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
					sir::Expr::Var(_) => {}
					_ => {
						self.add_expr_stmt(expr);
					}
				}
			}
			Stmt::Return(ReturnStmt { arg, .. }) => {
				if self.is_current_function_main() {
					panic!("Cannot return on main");
				}

				let expr = arg.as_ref().map(|arg| self.build_expr(arg));

				self.add_ret_stmt(expr);
				self.finish_block(sir::Term::Return);
			}
			Stmt::If(IfStmt {
				test, cons, alt, ..
			}) => {
				let test = self.build_expr(test);
				let cons_block = self.new_block();
				let continue_block = self.new_block();
				let alt = alt.as_ref().map(|alt| (alt, self.new_block()));

				if let Some((_, alt_block)) = &alt {
					self.finish_block(sir::Term::Switch(test, cons_block.id(), alt_block.id()));
				} else {
					self.finish_block(sir::Term::Switch(
						test,
						cons_block.id(),
						continue_block.id(),
					));
				}

				self.add_block(cons_block);
				self.build_stmt(cons);

				if let Some((alt, alt_block)) = alt {
					self.add_block(alt_block);
					self.build_stmt(alt);
				}

				self.add_block(continue_block);
			}
			Stmt::Block(block) => {
				for stmt in &block.stmts {
					self.build_stmt(stmt);
				}
			}
			_ => unimplemented!("{:#?}", stmt),
		}
	}

	fn build_decl(&self, decl: &Decl) {
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
					let name = sir::Symbol::new(binding.to_id());
					let ty = binding
						.type_ann
						.as_ref()
						.map(|type_ann| self.build_tstype(&type_ann.type_ann))
						.unwrap_or_else(|| self.tcx.new_infer_ty());

					let init = match &var_declarator.init {
						Some(init) => Some(self.build_expr(init)),
						None => {
							if is_const {
								panic!("Const variable must be initialized");
							}
							None
						}
					};

					self.add_var_entry(name.clone(), !is_const);
					self.add_let_stmt(name, ty, init);
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

		let function_name = sir::Symbol::new(ident.to_id());

		let params = {
			let mut params = vec![];

			for param in &function.params {
				match &param.pat {
					Pat::Ident(ident) => {
						let name = sir::Symbol::new(ident.to_id());

						let ty = match &ident.type_ann {
							Some(type_ann) => self.build_tstype(&type_ann.type_ann),
							None => {
								panic!("Param type annotation is required");
							}
						};

						self.add_var_entry(name.clone(), false);
						params.push(sir::TypedVar::new(name, ty));
					}
					_ => unimplemented!("{:#?}", param),
				}
			}

			params
		};

		let ret = {
			let ty = match &function.return_type {
				Some(type_ann) => self.build_tstype(&type_ann.type_ann),
				None => {
					// NOTE: seal does't infer the return type
					self.tcx.new_ty(crate::TyKind::Void)
				}
			};

			sir::TypedVar::new(sir::Symbol::new_ret(), ty)
		};

		self.start_function(&function_name, params, ret);

		let body = match &function.body {
			Some(body) => body,
			None => panic!("Function body is required"),
		};

		for stmt in &body.stmts {
			self.build_stmt(stmt);
		}

		self.finish_function();
	}

	fn build_expr(&self, expr: &Expr) -> sir::Expr {
		match expr {
			Expr::Assign(assign) => {
				let binding = match &assign.left {
					AssignTarget::Simple(target) => match &target {
						SimpleAssignTarget::Ident(ident) => ident,
						_ => unimplemented!("{:#?}", target),
					},
					_ => unimplemented!("{:#?}", assign.left),
				};
				let name = sir::Symbol::new(binding.to_id());

				if !self.is_var_can_be_assigned(&name) {
					panic!("Cannot assign to immutable variable");
				}

				self.add_assign_stmt(name.clone(), self.build_expr(&assign.right));

				sir::Expr::Var(name)
			}
			Expr::TsSatisfies(TsSatisfiesExpr { expr, type_ann, .. }) => {
				let expr = self.build_expr(expr);

				self.add_satisfies_stmt(
					// TODO:
					expr.clone(),
					self.build_tstype(type_ann),
				);

				expr
			}
			Expr::Lit(lit) => sir::Expr::Const(match lit {
				Lit::Bool(val) => sir::Const::Boolean(val.value),
				Lit::Num(val) => sir::Const::Number(val.value),
				Lit::Str(val) => sir::Const::String(val.value.clone()),
				_ => unimplemented!(),
			}),
			Expr::Ident(ident) => {
				let name = sir::Symbol::new(ident.to_id());

				sir::Expr::Var(name)
			}
			Expr::Unary(unary) => {
				let expr = self.build_expr(&unary.arg);

				match unary.op {
					UnaryOp::TypeOf => sir::Expr::TypeOf(Box::new(expr)),
					_ => unimplemented!("{:#?}", unary),
				}
			}
			Expr::Bin(bin) => {
				let left = self.build_expr(&bin.left);
				let right = self.build_expr(&bin.right);

				match bin.op {
					BinaryOp::EqEqEq => sir::Expr::Eq(Box::new(left), Box::new(right)),
					_ => unimplemented!("{:#?}", bin),
				}
			}
			_ => unimplemented!("{:#?}", expr),
		}
	}

	pub fn build_tstype(&self, tstype: &TsType) -> Ty<'tcx> {
		match tstype {
			TsType::TsKeywordType(keyword) => match keyword.kind {
				TsKeywordTypeKind::TsNumberKeyword => self.tcx.new_number(),
				TsKeywordTypeKind::TsStringKeyword => self.tcx.new_string(),
				TsKeywordTypeKind::TsBooleanKeyword => self.tcx.new_boolean(),
				TsKeywordTypeKind::TsVoidKeyword => self.tcx.new_void(),
				_ => unimplemented!(),
			},
			TsType::TsFnOrConstructorType(fn_or_constructor) => match fn_or_constructor {
				TsFnOrConstructorType::TsFnType(fn_) => {
					let ret_ty = self.build_tstype(&fn_.type_ann.type_ann);

					let mut param_tys = vec![];
					for param in &fn_.params {
						let ty = self.build_tstype(
							// TODO:
							&param.clone().expect_ident().type_ann.unwrap().type_ann,
						);
						param_tys.push(ty);
					}

					self.tcx.new_function(param_tys, ret_ty)
				}
				_ => unimplemented!(),
			},
			TsType::TsUnionOrIntersectionType(ty) => match ty {
				TsUnionOrIntersectionType::TsUnionType(ty) => self.tcx.new_union(
					ty.types
						.iter()
						.map(|ty| self.build_tstype(ty))
						.collect::<BTreeSet<_>>(),
				),
				TsUnionOrIntersectionType::TsIntersectionType(_) => unimplemented!(),
			},
			TsType::TsLitType(TsLitType { lit, .. }) => match lit {
				TsLit::Str(str) => self.tcx.new_const_string(str.value.clone()),
				_ => unimplemented!(),
			},
			_ => unimplemented!("{:#?}", tstype),
		}
	}
}
