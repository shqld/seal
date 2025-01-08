use std::collections::{BTreeMap, BTreeSet};

use swc_ecma_ast::{
	TsFnOrConstructorType, TsFnParam, TsKeywordTypeKind, TsLit, TsLitType, TsType, TsTypeLit,
	TsUnionOrIntersectionType,
};

use crate::Ty;

use super::BaseChecker;

impl<'tcx> BaseChecker<'tcx> {
	pub fn build_ts_type(&self, tstype: &TsType) -> Ty<'tcx> {
		match tstype {
			TsType::TsKeywordType(keyword) => match keyword.kind {
				TsKeywordTypeKind::TsNumberKeyword => self.constants.number,
				TsKeywordTypeKind::TsStringKeyword => self.constants.string,
				TsKeywordTypeKind::TsBooleanKeyword => self.constants.boolean,
				TsKeywordTypeKind::TsVoidKeyword => self.constants.void,
				TsKeywordTypeKind::TsNeverKeyword => self.constants.never,
				_ => unimplemented!(),
			},
			TsType::TsFnOrConstructorType(fn_or_constructor) => match fn_or_constructor {
				TsFnOrConstructorType::TsFnType(fn_) => {
					let ret_ty = self.build_ts_type(&fn_.type_ann.type_ann);

					let mut param_tys = vec![];
					for param in &fn_.params {
						let ty = match param {
							TsFnParam::Ident(ident) => {
								self.build_ts_type(&ident.type_ann.as_ref().unwrap().type_ann)
							}
							_ => unimplemented!("{:#?}", param),
						};
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
						.map(|ty| self.build_ts_type(ty))
						.collect::<BTreeSet<_>>(),
				),
				TsUnionOrIntersectionType::TsIntersectionType(_) => unimplemented!(),
			},
			TsType::TsLitType(TsLitType { lit, .. }) => match lit {
				TsLit::Str(str) => self.tcx.new_const_string(str.value.clone()),
				_ => unimplemented!("{:#?}", lit),
			},
			TsType::TsTypeLit(TsTypeLit { members, .. }) => {
				let mut fields = BTreeMap::new();
				for member in members {
					match member {
						swc_ecma_ast::TsTypeElement::TsPropertySignature(prop) => {
							let name = prop.key.as_ident().unwrap().sym.clone();
							let ty = self.build_ts_type(&prop.type_ann.as_ref().unwrap().type_ann);
							fields.insert(name, ty);
						}
						_ => unimplemented!("{:#?}", member),
					}
				}

				self.tcx.new_object(fields)
			}
			_ => unimplemented!("{:#?}", tstype),
		}
	}
}
