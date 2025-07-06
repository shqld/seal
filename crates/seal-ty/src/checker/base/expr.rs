use std::collections::{BTreeMap, BTreeSet};

use swc_ecma_ast::{
	AssignExpr, AssignTarget, BinExpr, BinaryOp, BlockStmtOrExpr, Bool, CallExpr, Callee, Expr,
	ExprOrSpread, Lit, MemberExpr, MemberProp, NewExpr, Number, ObjectLit, Pat, Prop, PropOrSpread,
	SeqExpr, SimpleAssignTarget, Str, TsSatisfiesExpr, UnaryOp,
};
use swc_common::Spanned;

use crate::{
	TyKind,
	checker::{errors::ErrorKind, function::FunctionChecker},
	sir::{Local, Value},
	symbol::Symbol,
};

use super::BaseChecker;

impl<'tcx> BaseChecker<'tcx> {
	pub fn check_expr(&self, expr: &Expr) -> Local<'tcx> {
		match expr {
			Expr::Assign(AssignExpr { left, right, .. }) => {
				let (binding_ident, binding_span) = match &left {
					AssignTarget::Simple(target) => match &target {
						SimpleAssignTarget::Ident(ident) => (ident, ident.span),
						_ => todo!("{:#?}", target),
					},
					_ => todo!("{:#?}", left),
				};
				let name = Symbol::new(binding_ident.to_id());
				let binding = if let Some(binding) = self.get_binding(&name) {
					binding
				} else {
					// If binding doesn't exist, add an error and return
					self.add_error_with_span(ErrorKind::CannotFindName(name), binding_span);
					return self.add_local(self.constants.err, Value::Err);
				};

				if !binding.is_assignable {
					self.add_error_with_span(
						ErrorKind::CannotAssignToConst(name.clone()),
						binding_span,
					);
				}

				let value = self.check_expr(right);

				// TODO: binding is Option<Local>, so we can remove TyKind::Lazy and check if it's None
				if let TyKind::Lazy = binding.ty.kind() {
					// if no type is specified to the declaration, replace with actual type
					// For let variables, widen literal types to base types
					let inferred_type = if binding.is_assignable {
						// let variable - widen literal types (42 -> number, true -> boolean)
						self.widen(value.ty)
					} else {
						// const variable - keep literal types
						value.ty
					};
					self.set_binding(&name, Some(value), inferred_type, true);
					return value;
				}

				if !self.satisfies(binding.ty, value.ty) {
					self.raise_type_error(binding.ty, value.ty, right.span());
				}

				self.set_binding(&name, Some(value), binding.ty, true);

				value
			}
			Expr::TsSatisfies(TsSatisfiesExpr { expr, type_ann, .. }) => {
				let value = self.check_expr(expr);

				let expected = self.build_ts_type(type_ann);
				let actual = value.ty;

				if !self.satisfies(expected, actual) {
					self.raise_type_error(expected, actual, expr.span());
				}

				value
			}
			Expr::Lit(lit) => match lit {
				Lit::Bool(Bool { value, .. }) => {
					self.add_local(self.tcx.new_const_boolean(*value), Value::Bool(*value))
				}
				Lit::Num(Number { value, .. }) => {
					self.add_local(
						self.tcx.new_const_number(*value as i64),
						// TODO: float
						Value::Int(*value as i64),
					)
				}
				Lit::Str(Str { value, .. }) => self.add_local(
					self.tcx.new_const_string(value.clone()),
					Value::Str(value.clone()),
				),
				Lit::Regex(regex) => {
					// Regular expressions are represented as RegExp objects
					self.add_local(self.constants.regexp, Value::Regex(regex.exp.clone()))
				}
				_ => todo!("{:#?}", lit),
			},
			Expr::Ident(ident) => {
				let name = Symbol::new(ident.to_id());

				if let Some(binding) = self.get_binding(&name) {
					// TODO: if in closure, this should be Value::Var
					if let Some(current) = binding.current {
						Local {
							id: current.id,
							// NOTE: we could use current.ty here, but it would make the code that is in progress harder to write (and TypeScript uses binding.ty for 'let' bindings, too)
							ty: binding.ty,
						}
					} else {
						self.add_error_with_span(ErrorKind::UsedBeforeAssigned(name), ident.span);
						self.add_local(self.constants.err, Value::Err)
					}
				} else {
					self.add_error_with_span(ErrorKind::CannotFindName(name), ident.span);
					self.add_local(self.constants.err, Value::Err)
				}
			}
			Expr::Unary(unary) => {
				let value = self.check_expr(&unary.arg);

				match unary.op {
					UnaryOp::TypeOf => {
						self.add_local(self.constants.type_of, Value::TypeOf(value.id))
					}
					UnaryOp::Bang => {
						// Logical NOT operator - always returns boolean
						self.add_local(
							self.constants.boolean,
							Value::Unary(crate::sir::UnaryOp::Not, value.id),
						)
					}
					UnaryOp::Plus => {
						// Unary plus - converts to number
						self.add_local(
							self.constants.number,
							Value::Unary(crate::sir::UnaryOp::Plus, value.id),
						)
					}
					UnaryOp::Minus => {
						// Unary minus - converts to number
						self.add_local(
							self.constants.number,
							Value::Unary(crate::sir::UnaryOp::Minus, value.id),
						)
					}
					_ => todo!("{:#?}", unary),
				}
			}
			Expr::Bin(BinExpr {
				op,
				left,
				right,
				span,
				..
			}) => {
				let left_ast = left;
				let right_ast = right;
				let left = self.check_expr(left);
				let right = self.check_expr(right);

				match op {
					BinaryOp::EqEqEq => {
						if !self.overlaps(left.ty, right.ty) {
							self.add_error_with_span(
								ErrorKind::NoOverlap(left.ty, right.ty),
								*span,
							);

							return self.add_local(self.constants.err, Value::Bool(false));
						}

						let value = Value::Eq(left.id, right.id);

						if let Some(narrowed_ty) = self.narrow(left_ast, right_ast) {
							self.add_local(narrowed_ty, value)
						} else {
							self.add_local(self.constants.boolean, Value::Eq(left.id, right.id))
						}
					}
					BinaryOp::NotEqEq => {
						// !== operator - strict inequality
						if !self.overlaps(left.ty, right.ty) {
							self.add_error_with_span(
								ErrorKind::NoOverlap(left.ty, right.ty),
								*span,
							);
							return self.add_local(self.constants.err, Value::Bool(true));
						}
						self.add_local(
							self.constants.boolean,
							Value::Binary(crate::sir::BinaryOp::And, left.id, right.id),
						)
					}
					// Arithmetic operators
					BinaryOp::Add => match (left.ty.kind(), right.ty.kind()) {
						(TyKind::Number(_), TyKind::Number(_)) => self.add_local(
							self.constants.number,
							Value::Binary(crate::sir::BinaryOp::Add, left.id, right.id),
						),
						(TyKind::String(_), TyKind::String(_)) => self.add_local(
							self.constants.string,
							Value::Binary(crate::sir::BinaryOp::Add, left.id, right.id),
						),
						_ => {
							self.add_error_with_span(
								ErrorKind::BinaryOperatorTypeMismatch(
									BinaryOp::Add,
									left.ty,
									right.ty,
								),
								*span,
							);
							self.add_local(self.constants.err, Value::Err)
						}
					},
					BinaryOp::Sub => match (left.ty.kind(), right.ty.kind()) {
						(TyKind::Number(_), TyKind::Number(_)) => self.add_local(
							self.constants.number,
							Value::Binary(crate::sir::BinaryOp::Sub, left.id, right.id),
						),
						_ => {
							self.add_error_with_span(
								ErrorKind::BinaryOperatorTypeMismatch(
									BinaryOp::Sub,
									left.ty,
									right.ty,
								),
								*span,
							);
							self.add_local(self.constants.err, Value::Err)
						}
					},
					BinaryOp::Mul => match (left.ty.kind(), right.ty.kind()) {
						(TyKind::Number(_), TyKind::Number(_)) => self.add_local(
							self.constants.number,
							Value::Binary(crate::sir::BinaryOp::Mul, left.id, right.id),
						),
						_ => {
							self.add_error_with_span(
								ErrorKind::BinaryOperatorTypeMismatch(
									BinaryOp::Mul,
									left.ty,
									right.ty,
								),
								*span,
							);
							self.add_local(self.constants.err, Value::Err)
						}
					},
					BinaryOp::Div => match (left.ty.kind(), right.ty.kind()) {
						(TyKind::Number(_), TyKind::Number(_)) => self.add_local(
							self.constants.number,
							Value::Binary(crate::sir::BinaryOp::Div, left.id, right.id),
						),
						_ => {
							self.add_error_with_span(
								ErrorKind::BinaryOperatorTypeMismatch(
									BinaryOp::Div,
									left.ty,
									right.ty,
								),
								*span,
							);
							self.add_local(self.constants.err, Value::Err)
						}
					},
					// Comparison operators
					BinaryOp::Lt => match (left.ty.kind(), right.ty.kind()) {
						(TyKind::Number(_), TyKind::Number(_)) => self.add_local(
							self.constants.boolean,
							Value::Binary(crate::sir::BinaryOp::Lt, left.id, right.id),
						),
						(TyKind::String(_), TyKind::String(_)) => self.add_local(
							self.constants.boolean,
							Value::Binary(crate::sir::BinaryOp::Lt, left.id, right.id),
						),
						_ => {
							self.add_error_with_span(
								ErrorKind::BinaryOperatorTypeMismatch(
									BinaryOp::Lt,
									left.ty,
									right.ty,
								),
								*span,
							);
							self.add_local(self.constants.err, Value::Err)
						}
					},
					BinaryOp::LtEq => match (left.ty.kind(), right.ty.kind()) {
						(TyKind::Number(_), TyKind::Number(_)) => self.add_local(
							self.constants.boolean,
							Value::Binary(crate::sir::BinaryOp::LtEq, left.id, right.id),
						),
						(TyKind::String(_), TyKind::String(_)) => self.add_local(
							self.constants.boolean,
							Value::Binary(crate::sir::BinaryOp::LtEq, left.id, right.id),
						),
						_ => {
							self.add_error_with_span(
								ErrorKind::BinaryOperatorTypeMismatch(
									BinaryOp::LtEq,
									left.ty,
									right.ty,
								),
								*span,
							);
							self.add_local(self.constants.err, Value::Err)
						}
					},
					BinaryOp::Gt => match (left.ty.kind(), right.ty.kind()) {
						(TyKind::Number(_), TyKind::Number(_)) => self.add_local(
							self.constants.boolean,
							Value::Binary(crate::sir::BinaryOp::Gt, left.id, right.id),
						),
						(TyKind::String(_), TyKind::String(_)) => self.add_local(
							self.constants.boolean,
							Value::Binary(crate::sir::BinaryOp::Gt, left.id, right.id),
						),
						_ => {
							self.add_error_with_span(
								ErrorKind::BinaryOperatorTypeMismatch(
									BinaryOp::Gt,
									left.ty,
									right.ty,
								),
								*span,
							);
							self.add_local(self.constants.err, Value::Err)
						}
					},
					BinaryOp::GtEq => match (left.ty.kind(), right.ty.kind()) {
						(TyKind::Number(_), TyKind::Number(_)) => self.add_local(
							self.constants.boolean,
							Value::Binary(crate::sir::BinaryOp::GtEq, left.id, right.id),
						),
						(TyKind::String(_), TyKind::String(_)) => self.add_local(
							self.constants.boolean,
							Value::Binary(crate::sir::BinaryOp::GtEq, left.id, right.id),
						),
						_ => {
							self.add_error_with_span(
								ErrorKind::BinaryOperatorTypeMismatch(
									BinaryOp::GtEq,
									left.ty,
									right.ty,
								),
								*span,
							);
							self.add_local(self.constants.err, Value::Err)
						}
					},
					// Logical operators
					BinaryOp::LogicalAnd => self.add_local(
						self.constants.boolean,
						Value::Binary(crate::sir::BinaryOp::And, left.id, right.id),
					),
					BinaryOp::LogicalOr => self.add_local(
						self.constants.boolean,
						Value::Binary(crate::sir::BinaryOp::Or, left.id, right.id),
					),
					_ => todo!("{:#?}", op),
				}
			}
			Expr::Member(MemberExpr {
				obj, prop, span, ..
			}) => {
				let obj = self.check_expr(obj);

				match &prop {
					MemberProp::Ident(ident) => {
						let key = ident.sym.clone();
						self.handle_property_access(obj, key, *span)
					}
					MemberProp::Computed(computed) => {
						// Handle computed property access like arr[0] or obj["key"]
						let index = self.check_expr(&computed.expr);
						self.handle_computed_access(obj, index, *span)
					}
					_ => todo!("{:#?}", prop),
				}
			}
			Expr::Object(ObjectLit { props, .. }) => {
				let mut obj_ty = crate::kind::Object::new(BTreeMap::new());
				let mut obj = crate::sir::Object::new();

				for prop in props {
					match prop {
						PropOrSpread::Prop(prop) => match prop.as_ref() {
							Prop::KeyValue(kv) => {
								let key = kv.key.as_ident().unwrap().sym.clone();
								let value = self.check_expr(&kv.value);

								obj_ty.fields.insert(key.clone(), value.ty);
								obj.fields.push((key, value.id));
							}
							_ => todo!("{:#?}", prop),
						},
						_ => todo!("{:#?}", prop),
					}
				}

				self.add_local(self.tcx.new_object(obj_ty), Value::Obj(obj))
			}
			Expr::Arrow(closure) => {
				let mut params = vec![];

				for param in &closure.params {
					match param {
						Pat::Ident(ident) => {
							let name = Symbol::new(ident.to_id());
							let param_type = if let Some(type_ann) = &ident.type_ann {
								self.build_ts_type(&type_ann.type_ann)
							} else {
								// If no type annotation, try to get from bindings (for closure context)
								if let Some(binding) = self.get_binding(&name) {
									binding.ty
								} else {
									// If no binding found, use unknown type
									self.constants.unknown
								}
							};

							params.push((name, param_type));
						}
						_ => todo!("{:#?}", param),
					};
				}

				match closure.body.as_ref() {
					BlockStmtOrExpr::Expr(body) => {
						// TODO: use FunctionChecker or BaseChecker
						let ret = self.check_expr(body);

						if let Some(return_type) = &closure.return_type {
							let expected = self.build_ts_type(&return_type.type_ann);

							if !self.satisfies(expected, ret.ty) {
								self.raise_type_error(expected, ret.ty, body.span());
							}
						}

						self.add_local(
							self.tcx.new_function(crate::kind::Function {
								params,
								ret: ret.ty,
							}),
							Value::Closure(),
						)
					}
					BlockStmtOrExpr::BlockStmt(body) => {
						let ret = match &closure.return_type {
							Some(type_ann) => self.build_ts_type(&type_ann.type_ann),
							None => self.constants.void,
						};

						let checker = FunctionChecker::new(self.tcx, params, ret);

						for (name, var) in self.bindings.borrow().iter() {
							let param = checker.add_local(var.ty, Value::Param);
							checker.set_binding(name, Some(param), var.ty, true);
						}

						let result = checker.check_body(body);

						for error in result.errors {
							self.add_error_with_span(error.kind, error.span);
						}

						self.add_local(self.tcx.new_function(result.ty), Value::Closure())
					}
				}
			}
			Expr::New(NewExpr {
				callee, args, span, ..
			}) => {
				let callee = self.check_expr(callee);

				let class = match callee.ty.kind() {
					TyKind::Class(class) => class,
					_ => {
						self.add_error_with_span(ErrorKind::NotConstructable, *span);
						return self.add_local(self.constants.err, Value::Err);
					}
				};

				let args = match args {
					Some(args) => args,
					None => {
						self.add_error_with_span(ErrorKind::NewOpMissingArgs, *span);
						return self.add_local(self.constants.err, Value::Err);
					}
				};

				let args = args.iter().map(|ExprOrSpread { expr, spread }| {
					if spread.is_some() {
						todo!()
					}

					self.check_expr(expr)
				});

				let instance = self.add_local(
					self.tcx.new_interface(class.interface().clone()),
					Value::New(callee.id, args.clone().map(|arg| arg.id).collect()),
				);

				if let Some(ctor) = class.ctor() {
					let params = &ctor.params;

					if params.len() != args.len() {
						self.add_error_with_span(
							ErrorKind::WrongNumArgs(params.len(), args.len()),
							*span,
						);
						return instance;
					}

					for ((_, param), arg) in params.iter().zip(args) {
						if !self.satisfies(*param, arg.ty) {
							self.raise_type_error(*param, arg.ty, *span);
						}
					}
				} else if args.len() != 0 {
					// TS(2554)
					self.add_error_with_span(ErrorKind::WrongNumArgs(0, args.len()), *span);
				}

				instance
			}
			Expr::Call(CallExpr {
				callee, args, span, ..
			}) => {
				let callee = self.check_expr(match callee {
					Callee::Expr(expr) => expr,
					_ => todo!("{:#?}", callee),
				});

				let function = match callee.ty.kind() {
					TyKind::Function(function) => function,
					_ => {
						self.add_error_with_span(ErrorKind::NotCallable(callee.ty), *span);
						return self.add_local(self.constants.err, Value::Err);
					}
				};

				let args = args.iter().map(|ExprOrSpread { expr, spread }| {
					if spread.is_some() {
						todo!()
					}

					self.check_expr(expr)
				});

				for ((_, param), arg) in function.params.iter().zip(args.clone()) {
					if !self.satisfies(*param, arg.ty) {
						self.raise_type_error(*param, arg.ty, *span);
					}
				}

				self.add_local(
					function.ret,
					Value::Call(callee.id, args.map(|arg| arg.id).collect()),
				)
			}
			Expr::Array(array) => {
				let elements: Vec<_> = array
					.elems
					.iter()
					.filter_map(|elem| elem.as_ref())
					.map(|ExprOrSpread { expr, spread }| {
						if spread.is_some() {
							todo!("spread in array literal")
						}
						self.check_expr(expr)
					})
					.collect();

				if elements.is_empty() {
					// Empty array - we'll need a way to handle generic arrays
					// For now, return an array of 'never' type
					self.add_local(
						self.tcx.new_array(self.constants.never),
						Value::Array(vec![]),
					)
				} else {
					// Create a union type of all element types, normalizing literal types
					let element_types: BTreeSet<_> = elements
						.iter()
						.map(|e| {
							// Normalize literal types to their base types for array inference
							match e.ty.kind() {
								TyKind::String(_) => self.constants.string,
								TyKind::Number(_) => self.constants.number,
								TyKind::Boolean(_) => self.constants.boolean,
								_ => e.ty,
							}
						})
						.collect();

					let element_type = if element_types.len() == 1 {
						*element_types.iter().next().unwrap()
					} else {
						self.tcx.new_union(element_types)
					};

					self.add_local(
						self.tcx.new_array(element_type),
						Value::Array(elements.into_iter().map(|e| e.id).collect()),
					)
				}
			}
			Expr::Tpl(tpl) => {
				// Template literals always result in string type
				// We could track the literal value for const strings, but for now just return string
				let parts: Vec<_> = tpl.exprs.iter().map(|expr| self.check_expr(expr)).collect();

				// All interpolated expressions should be convertible to string
				// In TypeScript, this is implicit
				self.add_local(
					self.constants.string,
					Value::Template(parts.into_iter().map(|p| p.id).collect()),
				)
			}
			Expr::Seq(SeqExpr { exprs, .. }) => {
				// Sequence expression (comma operator): evaluate all expressions, return the last one
				let mut result = None;
				for expr in exprs {
					result = Some(self.check_expr(expr));
				}
				result.unwrap_or_else(|| self.add_local(self.constants.void, Value::Err))
			}
			Expr::Paren(paren) => {
				// Parenthesized expression - just evaluate the inner expression
				self.check_expr(&paren.expr)
			}
			_ => todo!("{:#?}", expr),
		}
	}

	fn handle_property_access(
		&self,
		obj: crate::sir::Local<'tcx>,
		key: swc_atoms::Atom,
		span: swc_common::Span,
	) -> crate::sir::Local<'tcx> {
		match obj.ty.kind() {
			TyKind::Object(obj_ty) => match obj_ty.get_prop(&key) {
				Some(ty) => self.add_local(ty, Value::Member(obj.id, key)),
				None => {
					self.add_error_with_span(
						ErrorKind::PropertyDoesNotExist(obj.ty, key.clone()),
						span,
					);
					self.add_local(self.constants.err, Value::Member(obj.id, key))
				}
			},
			TyKind::Interface(interface) => match interface.fields().get(&key) {
				Some(ty) => self.add_local(*ty, Value::Member(obj.id, key)),
				None => {
					self.add_error_with_span(
						ErrorKind::PropertyDoesNotExist(obj.ty, key.clone()),
						span,
					);
					self.add_local(self.constants.err, Value::Member(obj.id, key))
				}
			},
			TyKind::Union(uni) => {
				let mut prop_arms = BTreeSet::new();

				for arm in uni.arms() {
					if let TyKind::Object(obj) = arm.kind() {
						if let Some(prop) = obj.get_prop(&key) {
							prop_arms.insert(prop);
							continue;
						}
					}

					self.add_error_with_span(
						ErrorKind::PropertyDoesNotExist(obj.ty, key.clone()),
						span,
					);
					return self.add_local(self.constants.err, Value::Member(obj.id, key));
				}

				self.add_local(self.tcx.new_union(prop_arms), Value::Member(obj.id, key))
			}
			_ => {
				// Check for primitive methods
				let (name, props) = if let TyKind::Number(_) = obj.ty.kind() {
					("number", &self.constants.proto_number)
				} else if let TyKind::String(_) = obj.ty.kind() {
					("string", &self.constants.proto_string)
				} else {
					self.add_error_with_span(
						ErrorKind::PropertyDoesNotExist(obj.ty, key.clone()),
						span,
					);
					return self.add_local(self.constants.err, Value::Member(obj.id, key));
				};

				if let Some(ty) = props.get(&key) {
					self.add_local(*ty, Value::Member(obj.id, key))
				} else {
					self.add_error_with_span(
						ErrorKind::PropertyDoesNotExist(
							match name {
								"number" => self.constants.number,
								"string" => self.constants.string,
								_ => unreachable!(),
							},
							key.clone(),
						),
						span,
					);
					self.add_local(self.constants.err, Value::Member(obj.id, key))
				}
			}
		}
	}

	fn handle_computed_access(
		&self,
		obj: crate::sir::Local<'tcx>,
		index: crate::sir::Local<'tcx>,
		span: swc_common::Span,
	) -> crate::sir::Local<'tcx> {
		match obj.ty.kind() {
			TyKind::Array(array) => {
				// For arrays, index should be number and we return element type
				match index.ty.kind() {
					TyKind::Number(_) => self.add_local(
						array.element,
						Value::Member(obj.id, swc_atoms::Atom::new("element")),
					),
					_ => {
						self.add_error_with_span(
							ErrorKind::PropertyDoesNotExist(
								obj.ty,
								swc_atoms::Atom::new("element"),
							),
							span,
						);
						self.add_local(self.constants.err, Value::Err)
					}
				}
			}
			TyKind::Object(_) => {
				// For objects, index should be string
				match index.ty.kind() {
					TyKind::String(Some(key)) => {
						// We know the exact key
						self.handle_property_access(obj, key.clone(), span)
					}
					TyKind::String(None) => {
						// Dynamic string key - we can't statically determine the type
						// In a more sophisticated implementation, we'd use index signatures
						self.add_local(
							self.constants.unknown,
							Value::Member(obj.id, swc_atoms::Atom::new("computed")),
						)
					}
					_ => {
						self.add_error_with_span(
							ErrorKind::PropertyDoesNotExist(
								obj.ty,
								swc_atoms::Atom::new("computed"),
							),
							span,
						);
						self.add_local(self.constants.err, Value::Err)
					}
				}
			}
			_ => {
				self.add_error_with_span(
					ErrorKind::PropertyDoesNotExist(obj.ty, swc_atoms::Atom::new("computed")),
					span,
				);
				self.add_local(self.constants.err, Value::Err)
			}
		}
	}
}
