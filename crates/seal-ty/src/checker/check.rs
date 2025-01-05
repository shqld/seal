use crate::{
	Ty, TyKind,
	builder::sir::{Block, Const, Expr, Function, Module, Stmt, Term},
};

use super::TypeChecker;

impl<'tcx> TypeChecker<'tcx> {
	pub fn check(self, module: &Module<'tcx>) -> Result<(), Vec<String>> {
		dbg!(&module);

		for function in &module.functions {
			self.check_function(function);
		}

		if self.errors.borrow().is_empty() {
			Ok(())
		} else {
			Err(self.errors.into_inner())
		}
	}

	pub fn check_function(&self, function: &Function<'tcx>) {
		let mut param_tys = vec![];
		for param in &function.params {
			let ty = param.ty();
			param_tys.push(ty);
			self.tcx.set_ty(param.name(), ty);
		}

		for block in &function.body {
			self.check_block(function, block);
		}

		let ret_ty = function.ret.ty();
		if !ret_ty.is_void() {
			let has_assigned_to_ret = function
				.body
				.iter()
				.flat_map(|block| block.stmts())
				.any(|stmt| matches!(stmt, Stmt::Ret(_)));

			if !has_assigned_to_ret {
				self.add_error("function does not return".to_string());
			}
		}

		let ty = self.tcx.new_function(param_tys, ret_ty);

		self.tcx.set_ty(&function.name, ty);
	}

	pub fn check_block(&self, function: &Function<'tcx>, block: &Block<'tcx>) {
		dbg!(&block);

		for stmt in block.stmts() {
			self.check_stmt(function, block, stmt);
		}

		if let Term::Switch(Expr::Eq(left, right), cons_block_id, alt_block_id) =
			block.term().unwrap()
		{
			match (left.as_ref(), right.as_ref()) {
				(Expr::TypeOf(arg), cmp) | (cmp, Expr::TypeOf(arg)) => {
					if let Expr::Var(name) = arg.as_ref() {
						let cmp_ty = self.build_expr(block, cmp);

						// TODO: seal should allow only const string for rhs of Eq(TypeOf) in Sir?
						if let TyKind::String(Some(value)) = cmp_ty.kind() {
							let narrowed_ty = match value.as_str() {
								"boolean" => Some(self.tcx.new_boolean()),
								"number" => Some(self.tcx.new_number()),
								"string" => Some(self.tcx.new_string()),
								_ => None,
							};

							if let Some(narrowed_ty) = narrowed_ty {
								self.tcx.override_ty(name, *cons_block_id, narrowed_ty);

								let var_ty = self.tcx.get_ty(name, block.id()).unwrap();

								if let TyKind::Union(uni) = var_ty.kind() {
									self.tcx.override_ty(
										name,
										*alt_block_id,
										self.tcx.new_excluded_union(uni, narrowed_ty),
									);
								}
							}
						}
					}
				}
				_ => {}
			}
		}

		// TODO: unify vars in block
	}

	pub fn check_stmt(&self, function: &Function<'tcx>, block: &Block<'tcx>, stmt: &Stmt<'tcx>) {
		match stmt {
			Stmt::Let(let_) => {
				let var = let_.var();
				let name = var.name();
				let ty = var.ty();

				if let Some(init) = let_.init() {
					let expected_ty = ty;
					let actual_ty = self.build_expr(block, init);

					// if no type is specified to the declaration, replace with actual type
					if let TyKind::Infer(_) = ty.kind() {
						self.tcx.set_ty(name, actual_ty);
						return;
					}

					if !self.satisfies(expected_ty, actual_ty) {
						self.raise_type_error(expected_ty, actual_ty);
					}
				}

				self.tcx.set_ty(name, ty);
			}
			Stmt::Assign(assign) => {
				let expected_ty = self.tcx.get_ty(assign.left(), block.id()).unwrap();
				let actual_ty = self.build_expr(block, assign.right());

				// if no type is specified to the declaration, replace with actual type
				if let TyKind::Infer(_) = expected_ty.kind() {
					self.tcx.set_ty(assign.left(), actual_ty);
					return;
				}

				if !self.satisfies(expected_ty, actual_ty) {
					self.raise_type_error(expected_ty, actual_ty);
				}
			}
			Stmt::Ret(expr) => {
				if let Some(expr) = expr {
					let expected_ty = function.ret.ty();
					let actual_ty = self.build_expr(block, expr);

					if !self.satisfies(expected_ty, actual_ty) {
						self.raise_type_error(expected_ty, actual_ty);
					}
				} else if !function.ret.ty().is_void() {
					self.add_error("expected return value".to_string());
				}
			}
			Stmt::Expr(expr) => {
				self.build_expr(block, expr);
			}
			Stmt::Satisfies(expr, ty) => {
				let expected_ty = *ty;
				let actual_ty = self.build_expr(block, expr);

				if !self.satisfies(expected_ty, actual_ty) {
					self.raise_type_error(expected_ty, actual_ty);
				}
			}
		}
	}

	pub fn build_expr(&self, block: &Block<'tcx>, expr: &Expr) -> Ty<'tcx> {
		match expr {
			Expr::Const(value) => match value {
				Const::Boolean(_) => self.tcx.new_ty(TyKind::Boolean),
				Const::Number(_) => self.tcx.new_ty(TyKind::Number),
				Const::String(value) => self.tcx.new_const_string(value.clone()),
			},
			Expr::Var(name) => {
				if let Some(ty) = self.tcx.get_ty(name, block.id()) {
					ty
				} else {
					panic!("Type not found: {:?}", name);
				}
			}
			Expr::TypeOf(_) => self.constants.type_of,
			Expr::Eq(_, _) => self.tcx.new_ty(TyKind::Boolean),
		}
	}

	pub fn raise_type_error(&self, expected: Ty<'tcx>, actual: Ty<'tcx>) {
		use TyKind::*;

		if matches!(actual.kind(), String(Some(_))) && !matches!(expected.kind(), String(_)) {
			self.add_error(format!(
				"expected '{expected}', got '{}'",
				TyKind::String(None)
			));
		} else {
			self.add_error(format!("expected '{expected}', got '{actual}'"));
		}
	}
}
