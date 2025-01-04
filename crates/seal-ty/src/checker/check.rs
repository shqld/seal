use crate::{
	Ty, TyKind,
	kind::FunctionTy,
	sema::air::{Block, Const, Expr, Function, Module, Stmt, Symbol},
};

use super::TypeChecker;

impl<'tcx> TypeChecker<'tcx> {
	pub fn check(&'tcx self, module: &Module<'tcx>) {
		dbg!(module);
		for function in &module.functions {
			self.check_function(function);
		}
	}

	pub fn check_function(&'tcx self, function: &Function<'tcx>) {
		let mut param_tys = vec![];
		for param in &function.params {
			let ty = param.ty();
			param_tys.push(ty);
			self.tcx.set_ty(param.name().clone(), ty);
		}

		let ret_ty = function.ret.ty();
		self.tcx.set_ty(function.ret.name().clone(), ret_ty);

		for block in &function.body {
			self.check_block(function, block);
		}

		let ret_ty = self.tcx.get_ty(&Symbol::new_ret(&function.name)).unwrap();
		if !ret_ty.is_void() {
			// TODO: 'has_returned' flag
			let has_assigned_to_ret = function.body.iter().any(|block| {
				block.stmts().iter().any(|stmt| match stmt {
					Stmt::Assign(assign) => assign.left().is_ret(),
					_ => false,
				})
			});

			if !has_assigned_to_ret {
				panic!("Function does not return");
			}
		}

		let ty = self.tcx.new_ty(TyKind::Function(FunctionTy {
			params: param_tys,
			ret: ret_ty,
		}));

		self.tcx.set_ty(function.name.clone(), ty);
	}

	pub fn check_block(&'tcx self, _function: &Function<'tcx>, block: &Block<'tcx>) {
		for stmt in block.stmts() {
			self.check_stmt(stmt);
		}

		// TODO: unify vars in block
	}

	pub fn check_stmt(&'tcx self, stmt: &Stmt<'tcx>) {
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

	pub fn build_expr(&'tcx self, expr: &Expr) -> Ty<'tcx> {
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

		panic!("Type mismatch: expected {expected}, got {actual}");
	}
}
