use swc_ecma_ast::{TsFnOrConstructorType, TsKeywordTypeKind, TsType};

use crate::{Ty, TyKind, context::TyContext, kind::FunctionTy};

pub struct TypeBuilder<'tcx> {
	tcx: &'tcx TyContext<'tcx>,
}

impl<'tcx> TypeBuilder<'tcx> {
	pub fn new(tcx: &'tcx TyContext<'tcx>) -> TypeBuilder<'tcx> {
		TypeBuilder { tcx }
	}

	pub fn build_tstype(&self, tstype: &TsType) -> Ty<'tcx> {
		self.tcx.new_ty(match tstype {
			TsType::TsKeywordType(keyword) => match keyword.kind {
				TsKeywordTypeKind::TsNumberKeyword => TyKind::Number,
				TsKeywordTypeKind::TsStringKeyword => TyKind::String,
				TsKeywordTypeKind::TsBooleanKeyword => TyKind::Boolean,
				TsKeywordTypeKind::TsVoidKeyword => TyKind::Void,
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

					TyKind::Function(FunctionTy {
						params: param_tys,
						ret: ret_ty,
					})
				}
				_ => unimplemented!(),
			},
			_ => unimplemented!("{:#?}", tstype),
		})
	}
}
