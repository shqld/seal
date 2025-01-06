use std::collections::BTreeSet;

use crate::{
	Ty, TyKind,
	builder::sir::{Block, Expr},
};

use super::TypeChecker;

impl<'tcx> TypeChecker<'tcx> {
	pub fn narrow(&self, block: &Block<'tcx>, left: &Expr, right: &Expr) -> Option<Ty<'tcx>> {
		match (left, right) {
			(Expr::TypeOf(arg), value) | (value, Expr::TypeOf(arg)) => {
				if let Expr::Var(name) = arg.as_ref() {
					let value_ty = self.build_expr(block, value);

					// TODO: seal should allow only const string for rhs of Eq(TypeOf) in Sir?
					if let TyKind::String(Some(value)) = value_ty.kind() {
						let narrowed_ty = match value.as_str() {
							"boolean" => Some(self.constants.boolean),
							"number" => Some(self.constants.number),
							"string" => Some(self.constants.string),
							_ => None,
						};

						if let Some(narrowed_ty) = narrowed_ty {
							return Some(self.tcx.new_guard(name.clone(), narrowed_ty));
						}
					}
				}
			}
			(Expr::Member(obj, prop), value) | (value, Expr::Member(obj, prop)) => {
				let obj_ty = self.build_expr(block, obj);

				if let Expr::Var(name) = obj.as_ref() {
					if let TyKind::Union(uni) = obj_ty.kind() {
						let value_ty = self.build_expr(block, value);

						let narrowed_arms: BTreeSet<_> = uni
							.arms()
							.iter()
							.filter_map(|&arm| {
								if let TyKind::Object(obj) = arm.kind() {
									obj.fields()
										.get(prop)
										.filter(|&&prop_ty| prop_ty == value_ty)
										.map(|_| arm)
								} else {
									None
								}
							})
							.collect();

						let narrowed_ty = self.tcx.new_union(narrowed_arms);

						return Some(self.tcx.new_guard(name.clone(), narrowed_ty));
					}
				}
			}
			_ => {}
		};

		None
	}
}
