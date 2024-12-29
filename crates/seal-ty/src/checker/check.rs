use crate::{
	Ty, TyKind,
	kind::FunctionTy,
	sema::air::{Assign, Block, Const, Expr, Function, Module, Stmt, Var},
};

use super::TypeChecker;

impl<'tcx> TypeChecker<'tcx> {
	pub fn check(&'tcx self, module: &Module<'tcx>) {
		for function in &module.functions {
			self.check_function(function);
		}
	}

	pub fn check_function(&'tcx self, function: &Function<'tcx>) {
		self.enter_function(function);

		for param in &function.params {
			self.tcx.set_ty(param.id.clone(), param.ty);
		}

		self.set_ret_ty(function.ret_ty);

		for block in &function.body {
			self.check_block(block);
		}

		if !function.ret_ty.is_void() {
			// TODO: 'has_returned' flag
			let has_assigned_to_ret = function.body.iter().any(|block| {
				block
					.stmts
					.iter()
					.any(|stmt| matches!(stmt, Stmt::Assign(Assign { var: Var::Ret, .. })))
			});

			if !has_assigned_to_ret {
				panic!("Function does not return");
			}
		}

		let ty = self.tcx.new_ty(TyKind::Function(FunctionTy {
			params: function.params.iter().map(|param| param.ty).collect(),
			ret: function.ret_ty,
		}));

		self.tcx.set_ty(function.id.clone(), ty);
	}

	pub fn check_block(&'tcx self, block: &Block<'tcx>) {
		for stmt in &block.stmts {
			self.check_stmt(stmt);
		}
	}

	pub fn check_stmt(&'tcx self, stmt: &Stmt<'tcx>) {
		match stmt {
			Stmt::Assign(Assign { var, expr }) => {
				let actual_ty = self.build_expr(expr);

				let expected_ty = match var {
					Var::Id(id) => match self.tcx.get_ty(id) {
						Some(ty) => ty,
						None => {
							self.tcx.set_ty(id.clone(), actual_ty);
							return;
						}
					},
					Var::Ret => {
						let ty = self.get_ret_ty();

						if let TyKind::Infer(id) = ty.kind() {
							self.tcx.infer.add_constraint(*id, actual_ty);
							// TODO: unify when function scope ends
							self.tcx.infer.unify(*id, actual_ty);

							return;
						}

						ty
					}
				};

				if !self.satisfies(expected_ty, actual_ty) {
					panic!("Type mismatch");
				}
			}
			Stmt::Expr(expr) => {
				self.build_expr(expr);
			}
			Stmt::Satisfies(expr, ty) => {
				let expected_ty = *ty;
				let actual_ty = self.build_expr(expr);

				if !self.satisfies(expected_ty, actual_ty) {
					panic!("Type mismatch");
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
			Expr::Var(var) => match var {
				Var::Id(id) => {
					if let Some(ty) = self.tcx.get_ty(id) {
						ty
					} else {
						panic!("Type not found: {:?}", id);
					}
				}
				Var::Ret => self.get_ret_ty(),
			},
		}
	}
}
