use std::collections::BTreeSet;

use swc_ecma_ast::{Expr, MemberExpr, UnaryExpr, UnaryOp};

use crate::{Ty, TyKind, symbol::Symbol};

use super::BaseChecker;

impl<'tcx> BaseChecker<'tcx> {
	pub fn narrow(&self, left: &Expr, right: &Expr) -> Option<Ty<'tcx>> {
		match (left, right) {
			(
				Expr::Unary(UnaryExpr {
					op: UnaryOp::TypeOf,
					arg,
					..
				}),
				value,
			)
			| (
				value,
				Expr::Unary(UnaryExpr {
					op: UnaryOp::TypeOf,
					arg,
					..
				}),
			) => {
				if let Expr::Ident(ident) = arg.as_ref() {
					let name = Symbol::new(ident.to_id());
					let value = self.check_expr(value);

					// TODO: seal should allow only const string for rhs of Eq(TypeOf) in Sir?
					if let TyKind::String(Some(value)) = value.ty.kind() {
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
			(Expr::Member(MemberExpr { obj, prop, .. }), value)
			| (value, Expr::Member(MemberExpr { obj, prop, .. })) => {
				if let Expr::Ident(ident) = obj.as_ref() {
					let name = Symbol::new(ident.to_id());
					let obj = self.check_expr(obj);
					let key = &prop.as_ident().unwrap().sym;

					if let TyKind::Union(uni) = obj.ty.kind() {
						let value = self.check_expr(value);

						let narrowed_arms: BTreeSet<_> = uni
							.arms()
							.iter()
							.filter_map(|&arm| {
								if let TyKind::Object(obj) = arm.kind() {
									obj.get_prop(key)
										.filter(|&prop_ty| prop_ty == value.ty)
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
