use crate::{
	Ty, TyKind,
	builder::air::{Block, Const, Expr, Function, Module, Stmt},
	kind::FunctionTy,
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
			self.tcx.set_ty(param.name().clone(), ty);
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

		let ty = self.tcx.new_ty(TyKind::Function(FunctionTy {
			params: param_tys,
			ret: ret_ty,
		}));

		self.tcx.set_ty(function.name.clone(), ty);
	}

	pub fn check_block(&self, function: &Function<'tcx>, block: &Block<'tcx>) {
		for stmt in block.stmts() {
			self.check_stmt(function, block, stmt);
		}

		// TODO: unify vars in block
	}

	pub fn check_stmt(&self, function: &Function<'tcx>, _block: &Block<'tcx>, stmt: &Stmt<'tcx>) {
		match stmt {
			Stmt::Let(let_) => {
				let var = let_.var();
				let name = var.name().clone();
				let ty = var.ty();

				if let Some(init) = let_.init() {
					let expected_ty = ty;
					let actual_ty = self.build_expr(init);

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
				let expected_ty = self.tcx.get_ty(assign.left()).unwrap();
				let actual_ty = self.build_expr(assign.right());

				// if no type is specified to the declaration, replace with actual type
				if let TyKind::Infer(_) = expected_ty.kind() {
					self.tcx.set_ty(assign.left().clone(), actual_ty);
					return;
				}

				if !self.satisfies(expected_ty, actual_ty) {
					self.raise_type_error(expected_ty, actual_ty);
				}
			}
			Stmt::Ret(expr) => {
				if let Some(expr) = expr {
					let expected_ty = function.ret.ty();
					let actual_ty = self.build_expr(expr);

					if !self.satisfies(expected_ty, actual_ty) {
						self.raise_type_error(expected_ty, actual_ty);
					}
				} else if !function.ret.ty().is_void() {
					self.add_error("expected return value".to_string());
				}
			}
			Stmt::Expr(expr) => {
				self.build_expr(expr);
			}
			Stmt::Satisfies(expr, ty) => {
				let expected_ty = *ty;
				let actual_ty = self.build_expr(expr);

				if !self.satisfies(expected_ty, actual_ty) {
					self.raise_type_error(expected_ty, actual_ty);
				}
			}
		}
	}

	pub fn build_expr(&self, expr: &Expr) -> Ty<'tcx> {
		match expr {
			Expr::Const(val) => match val {
				Const::Boolean(_) => self.tcx.new_ty(TyKind::Boolean),
				Const::Number(_) => self.tcx.new_ty(TyKind::Number),
				Const::String(_) => self.tcx.new_ty(TyKind::String),
			},
			Expr::Var(name) => {
				if let Some(ty) = self.tcx.get_ty(name) {
					ty
				} else {
					panic!("Type not found: {:?}", name);
				}
			}
		}
	}

	pub fn raise_type_error(&self, expected: Ty<'tcx>, actual: Ty<'tcx>) {
		dbg!(&self.tcx.types);

		self.add_error(format!("expected '{expected}', got '{actual}'"));
	}
}
