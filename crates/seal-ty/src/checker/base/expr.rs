use std::collections::{BTreeMap, BTreeSet};

use swc_ecma_ast::{
	AssignExpr, AssignTarget, BinExpr, BinaryOp, BlockStmtOrExpr, Bool, CallExpr, Callee, Expr,
	ExprOrSpread, Lit, MemberExpr, MemberProp, NewExpr, Number, ObjectLit, Pat, Prop, PropOrSpread,
	SimpleAssignTarget, Str, TsSatisfiesExpr, UnaryOp,
};

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
				let binding = match &left {
					AssignTarget::Simple(target) => match &target {
						SimpleAssignTarget::Ident(ident) => ident,
						_ => todo!("{:#?}", target),
					},
					_ => todo!("{:#?}", left),
				};
				let name = Symbol::new(binding.to_id());
				let binding = self.get_binding(&name).unwrap();

				if !binding.is_assignable {
					self.add_error(ErrorKind::CannotAssignToConst(name.clone()));
				}

				let value = self.check_expr(right);

				// TODO: binding is Option<Local>, so we can remove TyKind::Lazy and check if it's None
				if let TyKind::Lazy = binding.ty.kind() {
					// if no type is specified to the declaration, replace with actual type
					self.set_binding(&name, Some(value), value.ty, true);
					return value;
				}

				if !self.satisfies(binding.ty, value.ty) {
					self.raise_type_error(binding.ty, value.ty);
				}

				self.set_binding(&name, Some(value), binding.ty, true);

				value
			}
			Expr::TsSatisfies(TsSatisfiesExpr { expr, type_ann, .. }) => {
				let value = self.check_expr(expr);

				let expected = self.build_ts_type(type_ann);
				let actual = value.ty;

				if !self.satisfies(expected, actual) {
					self.raise_type_error(expected, actual);
				}

				value
			}
			Expr::Lit(lit) => match lit {
				Lit::Bool(Bool { value, .. }) => {
					self.add_local(self.constants.boolean, Value::Bool(*value))
				}
				Lit::Num(Number { value, .. }) => {
					self.add_local(
						self.constants.number,
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
						self.add_error(ErrorKind::UsedBeforeAssigned(name));
						self.add_local(self.constants.err, Value::Err)
					}
				} else {
					self.add_error(ErrorKind::CannotFindName(name));
					self.add_local(self.constants.err, Value::Err)
				}
			}
			Expr::Unary(unary) => {
				let value = self.check_expr(&unary.arg);

				match unary.op {
					UnaryOp::TypeOf => {
						self.add_local(self.constants.type_of, Value::TypeOf(value.id))
					}
					_ => todo!("{:#?}", unary),
				}
			}
			Expr::Bin(BinExpr {
				op, left, right, ..
			}) => {
				let left_ast = left;
				let right_ast = right;
				let left = self.check_expr(left);
				let right = self.check_expr(right);

				match op {
					BinaryOp::EqEqEq => {
						if !self.overlaps(left.ty, right.ty) {
							self.add_error(ErrorKind::NoOverlap(left.ty, right.ty));

							return self.add_local(self.constants.err, Value::Bool(false));
						}

						let value = Value::Eq(left.id, right.id);

						if let Some(narrowed_ty) = self.narrow(left_ast, right_ast) {
							self.add_local(narrowed_ty, value)
						} else {
							self.add_local(self.constants.boolean, Value::Eq(left.id, right.id))
						}
					}
					_ => todo!("{:#?}", op),
				}
			}
			Expr::Member(MemberExpr { obj, prop, .. }) => {
				let key = match &prop {
					MemberProp::Ident(ident) => ident.sym.clone(),
					_ => todo!("{:#?}", prop),
				};

				let obj = self.check_expr(obj);

				match obj.ty.kind() {
					TyKind::Object(obj_ty) => match obj_ty.get_prop(&key) {
						Some(ty) => self.add_local(ty, Value::Member(obj.id, key)),
						None => {
							self.add_error(ErrorKind::PropertyDoesNotExist(obj.ty, key.clone()));
							self.add_local(self.constants.err, Value::Member(obj.id, key))
						}
					},
					TyKind::Interface(interface) => match interface.fields().get(&key) {
						Some(ty) => self.add_local(*ty, Value::Member(obj.id, key)),
						None => {
							self.add_error(ErrorKind::PropertyDoesNotExist(obj.ty, key.clone()));
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

							self.add_error(ErrorKind::PropertyDoesNotExist(*arm, key.clone()));
							return self.add_local(self.constants.err, Value::Member(obj.id, key));
						}

						self.add_local(self.tcx.new_union(prop_arms), Value::Member(obj.id, key))
					}
					TyKind::Number | TyKind::String(_) => {
						let proto = match obj.ty.kind() {
							TyKind::Number => &self.constants.proto_number,
							TyKind::String(_) => &self.constants.proto_string,
							_ => unreachable!(),
						};

						let ty = proto.get(&key).copied().unwrap_or_else(|| {
							self.add_error(ErrorKind::PropertyDoesNotExist(obj.ty, key.clone()));
							self.constants.err
						});

						self.add_local(ty, Value::Member(obj.id, key))
					}
					TyKind::Err => self.add_local(self.constants.err, Value::Member(obj.id, key)),
					_ => {
						// TODO: other error kind? (e.g. "Property access on non-object is not allowed.")
						self.add_error(ErrorKind::PropertyDoesNotExist(obj.ty, key.clone()));
						self.add_local(self.constants.err, Value::Member(obj.id, key))
					}
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
							let binding = self.get_binding(&name).unwrap();

							params.push((name, binding.ty));
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
								self.raise_type_error(expected, ret.ty);
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
							self.add_error(error.kind);
						}

						self.add_local(self.tcx.new_function(result.ty), Value::Closure())
					}
				}
			}
			Expr::New(NewExpr { callee, args, .. }) => {
				let callee = self.check_expr(callee);

				let class = match callee.ty.kind() {
					TyKind::Class(class) => class,
					_ => {
						self.add_error(ErrorKind::NotConstructable);
						return self.add_local(self.constants.err, Value::Err);
					}
				};

				let args = match args {
					Some(args) => args,
					None => {
						self.add_error(ErrorKind::NewOpMissingArgs);
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
						self.add_error(ErrorKind::WrongNumArgs(params.len(), args.len()));
						return instance;
					}

					for ((_, param), arg) in params.iter().zip(args) {
						if !self.satisfies(*param, arg.ty) {
							self.raise_type_error(*param, arg.ty);
						}
					}
				} else if args.len() != 0 {
					// TS(2554)
					self.add_error(ErrorKind::WrongNumArgs(0, args.len()));
				}

				instance
			}
			Expr::Call(CallExpr { callee, args, .. }) => {
				let callee = self.check_expr(match callee {
					Callee::Expr(expr) => expr,
					_ => todo!("{:#?}", callee),
				});

				let function = match callee.ty.kind() {
					TyKind::Function(function) => function,
					_ => {
						self.add_error(ErrorKind::NotCallable(callee.ty));
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
						self.raise_type_error(*param, arg.ty);
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
					// Create a union type of all element types
					let element_types: BTreeSet<_> = elements.iter().map(|e| e.ty).collect();
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
				let parts: Vec<_> = tpl
					.exprs
					.iter()
					.map(|expr| self.check_expr(expr))
					.collect();

				// All interpolated expressions should be convertible to string
				// In TypeScript, this is implicit
				self.add_local(
					self.constants.string,
					Value::Template(parts.into_iter().map(|p| p.id).collect()),
				)
			}
			_ => todo!("{:#?}", expr),
		}
	}
}
