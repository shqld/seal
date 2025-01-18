use std::collections::{BTreeMap, BTreeSet};

use swc_ecma_ast::{
	TsEntityName, TsFnOrConstructorType, TsFnParam, TsKeywordTypeKind, TsLit, TsLitType, TsType,
	TsTypeLit, TsTypeRef, TsUnionOrIntersectionType,
};

use crate::{Ty, TyKind, symbol::Symbol};

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
				_ => todo!("{:#?}", keyword),
			},
			TsType::TsFnOrConstructorType(fn_or_constructor) => match fn_or_constructor {
				TsFnOrConstructorType::TsFnType(fn_) => {
					let ret = self.build_ts_type(&fn_.type_ann.type_ann);

					let mut params = vec![];
					for param in &fn_.params {
						match param {
							TsFnParam::Ident(ident) => {
								let name = Symbol::new(ident.to_id());
								let ty =
									self.build_ts_type(&ident.type_ann.as_ref().unwrap().type_ann);

								params.push((name, ty));
							}
							_ => todo!("{:#?}", param),
						};
					}

					self.tcx.new_function(crate::kind::Function { params, ret })
				}
				_ => todo!("{:#?}", fn_or_constructor),
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
				_ => todo!("{:#?}", lit),
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
						_ => todo!("{:#?}", member),
					}
				}

				self.tcx.new_object(crate::kind::Object { fields })
			}
			TsType::TsTypeRef(TsTypeRef { type_name, .. }) => {
				let name = Symbol::new(match type_name {
					TsEntityName::Ident(ident) => ident.to_id(),
					TsEntityName::TsQualifiedName(_) => unimplemented!(),
				});
				let binding = self.get_binding(&name).unwrap();

				match binding.ty.kind() {
					TyKind::Class(class) => self.tcx.new_interface(class.interface()),
					_ => binding.ty,
				}
			}
			_ => todo!("{:#?}", tstype),
		}
	}
}
