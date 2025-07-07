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
				TsKeywordTypeKind::TsUnknownKeyword => self.constants.unknown,
				TsKeywordTypeKind::TsNullKeyword => self.constants.null,
				TsKeywordTypeKind::TsUndefinedKeyword => self.constants.undefined,
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
				TsLit::Number(num) => {
					if num.value.fract() != 0.0 {
						self.add_error_with_span(
							crate::checker::errors::ErrorKind::InvalidNumberLiteral(num.value),
							num.span,
						);
						self.constants.number
					} else {
						self.tcx.new_const_number(num.value as i64)
					}
				}
				TsLit::Bool(bool) => self.tcx.new_const_boolean(bool.value),
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
			TsType::TsTypeRef(TsTypeRef {
				type_name, span, ..
			}) => {
				let name = Symbol::new(match type_name {
					TsEntityName::Ident(ident) => ident.to_id(),
					TsEntityName::TsQualifiedName(_) => unimplemented!(),
				});

				if let Some(binding) = self.get_binding(&name) {
					match binding.ty.kind() {
						TyKind::Class(class) => self.tcx.new_interface(class.interface()),
						_ => binding.ty,
					}
				} else {
					// Handle built-in types
					match name.name().as_ref() {
						"Object" => self.constants.object,
						"RegExp" => self.constants.regexp,
						_ => {
							self.add_error_with_span(
								crate::checker::errors::ErrorKind::CannotFindName(name),
								*span,
							);
							self.constants.err
						}
					}
				}
			}
			TsType::TsArrayType(array) => {
				let element = self.build_ts_type(&array.elem_type);
				self.tcx.new_array(element)
			}
			TsType::TsParenthesizedType(paren) => self.build_ts_type(&paren.type_ann),
			_ => todo!("{:#?}", tstype),
		}
	}
}
