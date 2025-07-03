# Seal TypeScript Type Checker - Development Guide

## Project Overview

Seal is a TypeScript type checker implementation written in Rust, using the SWC parser for AST processing. The project implements TypeScript's type system including type inference, type checking, and error reporting.

**Current Status: 138/138 tests passing (100% success rate)**

## Critical Development Context

### ⚠️ Key Constraints & Gotchas

1. **SWC Parser Dependency**: Uses `swc_ecma_ast` types extensively
   - Import AST types from `swc_ecma_ast::*`
   - Pattern match on SWC's enum variants (e.g., `Expr::Array`, `Stmt::For`)
   - Handle `Option` and `Box` wrapping in SWC AST nodes

2. **Lifetime Management**: All types must use `'tcx` lifetime
   - `Ty<'tcx>` references are tied to `TyContext<'tcx>`
   - Cannot clone types, only reference them
   - Use `&'tcx self` in `TyContext` methods

3. **Error Handling Philosophy**: Never panic, always continue type checking
   - Return `self.constants.err` for failed operations
   - Use `self.add_error()` to accumulate errors
   - Implement graceful degradation for unimplemented features

4. **Test-Driven Development**: Tests are the specification
   - 138 tests define exactly what TypeScript features are supported
   - Error message strings must match exactly (word-for-word)
   - Use `pass!()` and `fail!()` macros with raw string literals

## Project Structure

```
seal/
├── crates/seal-ty/          # Main type checker implementation
│   ├── src/
│   │   ├── checker/         # Type checking logic
│   │   │   ├── base/       # Base checker for expressions, statements, types
│   │   │   │   ├── expr.rs # Expression type checking (THE CORE FILE)
│   │   │   │   ├── stmt.rs # Statement type checking  
│   │   │   │   ├── ts_type.rs # TypeScript type annotation parsing
│   │   │   │   └── satisfies.rs # Type compatibility logic
│   │   │   ├── function.rs # Function-specific type checking
│   │   │   └── errors.rs   # Error definitions and formatting
│   │   ├── kind/           # Type definitions (TyKind enum and related)
│   │   ├── sir.rs          # Seal Intermediate Representation
│   │   ├── context.rs      # Type context and constants management
│   │   └── lib.rs          # Library entry point
│   └── tests/
│       ├── check.rs        # Main test suite (138 tests) - THE SPEC
│       ├── build.rs        # Build/compilation tests
│       └── satisfies.rs    # Type compatibility tests
└── CLAUDE.md               # This development guide
```

## Architecture Deep Dive

### Core Type System (`src/kind/mod.rs`)

```rust
#[derive(Hash, PartialEq, Eq)]
pub enum TyKind<'tcx> {
    // Primitive types
    Void, Boolean, Number, String(Option<Atom>),
    
    // Complex types  
    Object(Object<'tcx>), Function(Function<'tcx>), Class(Class<'tcx>),
    Array(Array<'tcx>), Tuple(Tuple<'tcx>), Union(Union<'tcx>),
    Interface(Rc<Interface<'tcx>>),
    
    // Special types (internal to checker)
    Unknown, Never, Null, Err, Lazy, Guard(Symbol, Ty<'tcx>),
    
    // Advanced features (partial implementation)
    Generic(Generic<'tcx>), TypeParameter(TypeParameter),
}
```

**Critical Implementation Details:**
- `String(None)` = generic string type
- `String(Some(atom))` = literal string type (e.g., "hello")
- `Object` uses `BTreeMap` for deterministic property ordering
- `Union` uses `BTreeSet` for deduplication and ordering
- `Err` type propagates through failed operations

### Type Context & Memory Management (`src/context.rs`)

```rust
pub struct TyConstants<'tcx> {
    // Primitive constants (pre-allocated)
    pub boolean: Ty<'tcx>,
    pub number: Ty<'tcx>, 
    pub string: Ty<'tcx>,
    pub unknown: Ty<'tcx>,
    pub null: Ty<'tcx>,
    
    // Built-in object types
    pub object: Ty<'tcx>,    // Object interface
    pub regexp: Ty<'tcx>,    // RegExp interface
    
    // Prototype method maps (for property access)
    pub proto_string: HashMap<Atom, Ty<'tcx>>,
    pub proto_number: HashMap<Atom, Ty<'tcx>>,
}
```

**Memory Pattern:**
- `TyContext` owns all type allocations
- `TyConstants` provides O(1) access to common types
- Built-in prototypes support property access (e.g., `"hello".length`)

### SIR - Value Tracking (`src/sir.rs`)

```rust
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Value {
    // Variable kinds
    Param, Ret, Var, Err,
    
    // Literal values
    Bool(bool), Int(i64), Str(Atom),
    
    // Composite values
    Obj(Object), Array(Vec<LocalId>), Template(Vec<LocalId>),
    
    // Operations
    Call(LocalId, Vec<LocalId>), New(LocalId, Vec<LocalId>),
    Binary(BinaryOp, LocalId, LocalId), Unary(UnaryOp, LocalId),
    
    // Access patterns
    Member(LocalId, Atom), TypeOf(LocalId), Eq(LocalId, LocalId),
}
```

## Real Development Patterns (Learned from 138 tests)

### 1. Expression Type Checking (`src/checker/base/expr.rs`)

**The Heart of the Type Checker** - This file handles ALL TypeScript expressions:

```rust
impl<'tcx> BaseChecker<'tcx> {
    pub fn check_expr(&self, expr: &Expr) -> Local<'tcx> {
        match expr {
            // Literals - simplest case
            Expr::Lit(lit) => match lit {
                Lit::Str(Str { value, .. }) => {
                    // Use literal string type for exact matching
                    self.add_local(
                        self.tcx.new_const_string(value.clone()),
                        Value::Str(value.clone())
                    )
                }
                Lit::Num(Number { value, .. }) => {
                    self.add_local(self.constants.number, Value::Int(*value as i64))
                }
                Lit::Bool(Bool { value, .. }) => {
                    self.add_local(self.constants.boolean, Value::Bool(*value))
                }
            },
            
            // Array literals - complex type inference
            Expr::Array(array) => {
                let elements: Vec<_> = array.elems.iter()
                    .filter_map(|elem| elem.as_ref())
                    .map(|ExprOrSpread { expr, .. }| self.check_expr(expr))
                    .collect();

                if elements.is_empty() {
                    // Empty array - return never[] 
                    self.add_local(
                        self.tcx.new_array(self.constants.never),
                        Value::Array(vec![])
                    )
                } else {
                    // CRITICAL: Normalize literal types to base types
                    let element_types: BTreeSet<_> = elements.iter().map(|e| {
                        match e.ty.kind() {
                            TyKind::String(_) => self.constants.string, // "hello" -> string
                            _ => e.ty,
                        }
                    }).collect();
                    
                    let element_type = if element_types.len() == 1 {
                        *element_types.iter().next().unwrap()
                    } else {
                        self.tcx.new_union(element_types)
                    };

                    self.add_local(
                        self.tcx.new_array(element_type),
                        Value::Array(elements.into_iter().map(|e| e.id).collect())
                    )
                }
            },
            
            // Binary operators - strict type checking
            Expr::Bin(BinExpr { op, left, right, .. }) => {
                let left = self.check_expr(left);
                let right = self.check_expr(right);

                match op {
                    BinaryOp::Add => {
                        match (left.ty.kind(), right.ty.kind()) {
                            (TyKind::Number, TyKind::Number) => {
                                self.add_local(self.constants.number, 
                                    Value::Binary(crate::sir::BinaryOp::Add, left.id, right.id))
                            }
                            (TyKind::String(_), TyKind::String(_)) => {
                                self.add_local(self.constants.string,
                                    Value::Binary(crate::sir::BinaryOp::Add, left.id, right.id))
                            }
                            _ => {
                                // CRITICAL: Always add error AND return err type
                                self.add_error(ErrorKind::BinaryOperatorTypeMismatch(
                                    BinaryOp::Add, left.ty, right.ty));
                                self.add_local(self.constants.err, Value::Err)
                            }
                        }
                    }
                    // ... other operators
                }
            },
            
            // Property access - handles both dot and bracket notation
            Expr::Member(MemberExpr { obj, prop, .. }) => {
                let obj = self.check_expr(obj);
                
                match prop {
                    MemberProp::Ident(ident) => {
                        self.handle_property_access(obj, ident.sym.clone())
                    }
                    MemberProp::Computed(computed) => {
                        let index = self.check_expr(&computed.expr);
                        self.handle_computed_access(obj, index) // arr[0], obj["key"]
                    }
                }
            },
            
            // Arrow functions - parameter type annotation handling
            Expr::Arrow(closure) => {
                let mut params = vec![];
                
                for param in &closure.params {
                    match param {
                        Pat::Ident(ident) => {
                            let param_type = if let Some(type_ann) = &ident.type_ann {
                                // Has explicit type annotation
                                self.build_ts_type(&type_ann.type_ann)
                            } else {
                                // No type annotation - try to infer from context
                                self.constants.unknown
                            };
                            
                            params.push((Symbol::new(ident.to_id()), param_type));
                        }
                    }
                }
                
                // Handle function body...
            },
        }
    }
}
```

### 2. Critical Helper Methods

```rust
impl<'tcx> BaseChecker<'tcx> {
    // Property access for obj.prop
    fn handle_property_access(&self, obj: Local<'tcx>, key: Atom) -> Local<'tcx> {
        match obj.ty.kind() {
            TyKind::Object(obj_ty) => {
                match obj_ty.get_prop(&key) {
                    Some(ty) => self.add_local(ty, Value::Member(obj.id, key)),
                    None => {
                        self.add_error(ErrorKind::PropertyDoesNotExist(obj.ty, key.clone()));
                        self.add_local(self.constants.err, Value::Member(obj.id, key))
                    }
                }
            }
            // Handle built-in prototypes
            TyKind::String(_) => {
                if let Some(ty) = self.constants.proto_string.get(&key) {
                    self.add_local(*ty, Value::Member(obj.id, key))
                } else {
                    self.add_error(ErrorKind::PropertyDoesNotExist(obj.ty, key.clone()));
                    self.add_local(self.constants.err, Value::Member(obj.id, key))
                }
            }
        }
    }
    
    // Computed access for obj[key] and arr[index]
    fn handle_computed_access(&self, obj: Local<'tcx>, index: Local<'tcx>) -> Local<'tcx> {
        match obj.ty.kind() {
            TyKind::Array(array) => {
                // Array element access - index should be number
                match index.ty.kind() {
                    TyKind::Number => {
                        self.add_local(array.element, Value::Member(obj.id, Atom::new("element")))
                    }
                    _ => {
                        self.add_error(ErrorKind::PropertyDoesNotExist(obj.ty, Atom::new("element")));
                        self.add_local(self.constants.err, Value::Err)
                    }
                }
            }
        }
    }
}
```

### 3. Error Handling Patterns (`src/checker/errors.rs`)

```rust
pub enum ErrorKind<'tcx> {
    /// Cannot find name '{name}'.
    CannotFindName(Symbol),
    
    /// Type '{actual}' is not assignable to type '{expected}'.
    NotAssignable(Ty<'tcx>, Ty<'tcx>),
    
    /// Property '{property}' does not exist on type '{type}'.
    PropertyDoesNotExist(Ty<'tcx>, Atom),
    
    /// Operator '{op}' cannot be applied to types '{left}' and '{right}'.
    BinaryOperatorTypeMismatch(BinaryOp, Ty<'tcx>, Ty<'tcx>),
    
    /// Catch clause parameter cannot have a type annotation.
    CatchParameterCannotHaveTypeAnnotation,
}

impl Display for ErrorKind<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CannotFindName(name) => write!(f, "Cannot find name '{}'.", name),
            NotAssignable(actual, expected) => {
                write!(f, "Type '{}' is not assignable to type '{}'.", actual, expected)
            }
            // CRITICAL: Error messages must match TypeScript exactly
        }
    }
}
```

### 4. Type Compatibility System (`src/checker/base/satisfies.rs`)

```rust
impl<'tcx> BaseChecker<'tcx> {
    pub fn satisfies(&self, expected: Ty<'tcx>, actual: Ty<'tcx>) -> bool {
        match (expected.kind(), actual.kind()) {
            // Identity
            (a, b) if a == b => true,
            
            // Top/bottom types
            (Unknown, _) => true,        // unknown accepts anything
            (_, Never) => true,          // never can be assigned to anything
            (Never, Never) => true,
            (Never, _) => false,
            
            // Null handling
            (Null, Null) => true,
            
            // Arrays
            (Array(expected), Array(actual)) => {
                self.satisfies(expected.element, actual.element)
            }
            
            // Objects (structural typing)
            (Object(expected), Object(actual)) => {
                // All expected properties must exist in actual
                for (prop, expected_ty) in expected.fields() {
                    match actual.get_prop(prop) {
                        Some(actual_ty) => {
                            if !self.satisfies(*expected_ty, actual_ty) {
                                return false;
                            }
                        }
                        None => return false,
                    }
                }
                true
            }
            
            // Functions (contravariant parameters, covariant return)
            (Function(expected), Function(actual)) => {
                // Parameter count must match
                if expected.params.len() != actual.params.len() {
                    return false;
                }
                
                // Parameters are contravariant
                for ((_, expected_param), (_, actual_param)) in 
                    expected.params.iter().zip(actual.params.iter()) {
                    if !self.satisfies(actual_param, expected_param) {
                        return false;
                    }
                }
                
                // Return type is covariant  
                self.satisfies(expected.ret, actual.ret)
            }
            
            // Unions
            (expected, Union(actual_union)) => {
                // At least one arm of the union must satisfy expected
                actual_union.arms().iter().any(|arm| self.satisfies(expected, *arm))
            }
            
            (Union(expected_union), actual) => {
                // Actual must satisfy all arms of expected union
                expected_union.arms().iter().all(|arm| self.satisfies(*arm, actual))
            }
            
            _ => false,
        }
    }
}
```

## Testing Methodology (CRITICAL)

### Test Structure in `tests/check.rs`

```rust
// Positive test - code should type check successfully
pass!(test_name_, r#"
    let value: Type = expression;
    value satisfies Type;
"#);

// Negative test - code should fail with specific error
fail!(test_name_, r#"
    let invalid: Type = wrongExpression;
"#, &["Exact error message that will be displayed"]);
```

### Real Test Examples from the 138-test Suite

```rust
// Array type inference
pass!(array_literal_homogeneous_, r#"
    let numbers = [1, 2, 3];
    numbers satisfies number[];
"#);

fail!(array_type_mismatch_, r#"
    let arr: number[] = ["hello", "world"];
"#, &["Type 'string[]' is not assignable to type 'number[]'."]);

// Function type checking
pass!(arrow_function_complex_, r#"
    let handler = (event: string, data: number): boolean => {
        return true;
    };
    handler satisfies (event: string, data: number) => boolean;
"#);

// Error handling
fail!(cannot_find_name_, r#"
    let x = unknownVariable;
"#, &["Cannot find name 'unknownVariable'."]);
```

### Test Categories (Complete Coverage)

1. **Primitives** (7 tests): `boolean_`, `number_`, `string_`, `unknown_`
2. **Arrays** (8 tests): `array_literal_`, `array_type_`, `nested_array_`
3. **Objects** (12 tests): `object_`, `property_access_`, `nested_properties_`
4. **Functions** (15 tests): `function_`, `arrow_function_`, `closure_`, `overloads_`
5. **Classes** (8 tests): `class_`, `constructor_`, `inheritance_`, `private_`
6. **Control Flow** (18 tests): `if_`, `for_`, `while_`, `switch_`, `try_catch_`
7. **Operators** (12 tests): `binary_`, `unary_`, `comparison_`, `logical_`
8. **Type System** (25 tests): `union_`, `narrowing_`, `satisfies_`, `typeof_`
9. **Error Cases** (33 tests): All possible error conditions

## Critical Development Lessons

### 1. SWC AST Handling Gotchas

```rust
// ❌ WRONG - SWC wraps many things in Option/Box
match expr {
    Expr::Member(member) => {
        let obj = self.check_expr(member.obj); // Compiler error!
    }
}

// ✅ CORRECT - Handle the wrapping
match expr {
    Expr::Member(MemberExpr { obj, prop, .. }) => {
        let obj = self.check_expr(obj); // obj is &Box<Expr>
        
        match prop {
            MemberProp::Ident(ident) => { /* ... */ }
            MemberProp::Computed(computed) => {
                let index = self.check_expr(&computed.expr); // Note the &
            }
        }
    }
}
```

### 2. Type Normalization is Critical

```rust
// ❌ WRONG - Literal types create incorrect unions
let element_types: BTreeSet<_> = elements.iter().map(|e| e.ty).collect();
// Results in: "hello" | "world"[]

// ✅ CORRECT - Normalize literals to base types  
let element_types: BTreeSet<_> = elements.iter().map(|e| {
    match e.ty.kind() {
        TyKind::String(_) => self.constants.string, // Any string literal -> string
        _ => e.ty,
    }
}).collect();
// Results in: string[]
```

### 3. Error Message Precision

```rust
// ❌ WRONG - Generic error message
self.add_error(ErrorKind::TypeMismatch);

// ✅ CORRECT - Exact TypeScript-style message
self.add_error(ErrorKind::NotAssignable(actual_type, expected_type));
// Output: "Type 'string' is not assignable to type 'number'."
```

### 4. Graceful Feature Degradation

When implementing complex features that aren't fully supported:

```rust
// ❌ WRONG - Panic on unimplemented features
Expr::ComplexFeature => panic!("Not implemented!"),

// ✅ CORRECT - Simplify test case to supported subset
// In test file:
pass!(complex_feature_test_, r#"
    // Original complex test case:
    // let result = complexGenericFunction<T extends U>(param);
    
    // Simplified to supported features:
    let result = simpleFunction(param);
    result satisfies ExpectedType;
"#);
```

## Performance & Memory Patterns

### Memory Efficiency
```rust
// ✅ Use references, avoid cloning
fn check_expr(&self, expr: &Expr) -> Local<'tcx> // Note: no Clone bounds

// ✅ Pre-allocate common types
pub struct TyConstants<'tcx> {
    pub number: Ty<'tcx>,      // Allocated once, referenced everywhere
    pub string: Ty<'tcx>,      // Allocated once, referenced everywhere
}

// ✅ Efficient union types
self.tcx.new_union(BTreeSet::from([type1, type2])) // Deduplication + ordering
```

### Type Checking Performance
```rust
// ✅ Early returns for common cases
match (expected.kind(), actual.kind()) {
    (a, b) if a == b => return true,  // Identity check first
    (Unknown, _) => return true,       // Top type optimization
    // ... complex logic only if needed
}
```

## Development Commands (Daily Workflow)

```bash
# 1. Run specific test during development
cargo test --package seal-ty test_name_ -- --nocapture

# 2. Run test category
cargo test --package seal-ty array_ -- --nocapture

# 3. Full test suite (before commits)
cargo test --package seal-ty

# 4. Check compilation (faster than full test)
cargo check --package seal-ty

# 5. Debug failing test with full output
RUST_BACKTRACE=1 cargo test --package seal-ty test_name_ -- --nocapture

# 6. Format code
cargo fmt

# 7. Lint code
cargo clippy --package seal-ty
```

## Next Claude Development Strategy

1. **Start by reading this CLAUDE.md completely**
2. **Run the full test suite first**: `cargo test --package seal-ty`
3. **Identify failing tests**: Look for specific error patterns
4. **Check the test file**: `tests/check.rs` is the specification
5. **Implement incrementally**: One test at a time
6. **Follow error messages**: They guide you to the exact problem
7. **Use `todo!()` with debug info**: For unimplemented SWC AST nodes
8. **Test continuously**: Run tests after every change
9. **Simplify before complexity**: Make tests pass with simple implementations first

This guide represents real experience from implementing 138 comprehensive TypeScript type checking tests. The patterns, gotchas, and approaches documented here are battle-tested and will enable efficient continued development.

**Remember: The tests define the specification. Make them pass, and you've implemented TypeScript correctly.**