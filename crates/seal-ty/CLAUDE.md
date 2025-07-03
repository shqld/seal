# Seal-Ty Crate - Practical Development Guide

## Quick Status Check

**Current: 138/138 tests passing ✅**

```bash
# Verify everything works
cargo test
# Should output: test result: ok. 138 passed; 0 failed
```

## Daily Development Workflow

### 1. Essential Commands

```bash
# Primary development cycle
cargo test test_name_ -- --nocapture    # Debug specific failing test
cargo test array_                       # Test a feature category  
cargo test                              # Full suite before commit
cargo check                             # Quick compilation check

# When something breaks
RUST_BACKTRACE=1 cargo test failing_test_ -- --nocapture
```

### 2. File Priority Map

```
HIGH PRIORITY (touch these often):
├── tests/check.rs           # THE SPECIFICATION - 138 tests define what works
├── src/checker/base/expr.rs # THE CORE - all expression type checking
└── src/checker/errors.rs    # Error messages (must match tests exactly)

MEDIUM PRIORITY (modify for new features):
├── src/kind/mod.rs          # Add new types here
├── src/checker/base/stmt.rs # Statement type checking
├── src/checker/base/satisfies.rs # Type compatibility rules
└── src/context.rs           # Add new built-in types

LOW PRIORITY (rarely change):
├── src/sir.rs               # Value representation
├── src/checker/base/ts_type.rs # Type annotation parsing
└── src/lib.rs               # Exports
```

## Critical Patterns (Copy These)

### 1. Adding New Expression Support

```rust
// In src/checker/base/expr.rs, add to check_expr() match:
Expr::NewSyntax(new_syntax) => {
    // 1. Check sub-expressions first
    let operand = self.check_expr(&new_syntax.operand);
    
    // 2. Type check and infer result type
    let result_type = match operand.ty.kind() {
        TyKind::ValidType => self.constants.expected_result,
        _ => {
            // 3. Always add error + return err type for failures
            self.add_error(ErrorKind::TypeMismatch(operand.ty));
            self.constants.err
        }
    };
    
    // 4. Return local with computed type
    self.add_local(result_type, Value::NewValue(operand.id))
}
```

### 2. SWC AST Pattern Matching (CRITICAL)

```rust
// ✅ CORRECT - Handle SWC's wrapping patterns
match expr {
    Expr::Member(MemberExpr { obj, prop, .. }) => {
        // obj is &Box<Expr> - access directly
        let obj_result = self.check_expr(obj);
        
        match prop {
            MemberProp::Ident(ident) => {
                // ident.sym is the property name
                self.handle_property_access(obj_result, ident.sym.clone())
            }
            MemberProp::Computed(computed) => {
                // computed.expr is &Box<Expr> - need &
                let index = self.check_expr(&computed.expr);
                self.handle_computed_access(obj_result, index)
            }
        }
    }
    
    Expr::Call(CallExpr { callee, args, .. }) => {
        // callee is &Callee enum, not &Expr
        let callee_expr = match callee {
            Callee::Expr(expr) => expr, // This is &Box<Expr>
            _ => todo!("Super/Import calls not implemented"),
        };
        
        let func = self.check_expr(callee_expr);
        let arg_results: Vec<_> = args.iter().map(|arg| {
            self.check_expr(&arg.expr) // arg.expr is Box<Expr>
        }).collect();
        
        // ... function call logic
    }
}
```

### 3. Type Inference Best Practices

```rust
// ✅ CORRECT - Normalize literal types in collections
let element_types: BTreeSet<_> = elements.iter().map(|e| {
    match e.ty.kind() {
        TyKind::String(_) => self.constants.string,   // "hello" -> string
        TyKind::Number => self.constants.number,      // Keep as number
        _ => e.ty,
    }
}).collect();

// ✅ CORRECT - Handle union creation
let result_type = if element_types.len() == 1 {
    *element_types.iter().next().unwrap()
} else {
    self.tcx.new_union(element_types)
};
```

### 4. Error Handling Pattern

```rust
// ✅ CORRECT - Always continue type checking after errors
match some_operation() {
    Ok(result) => result,
    Err(_) => {
        self.add_error(ErrorKind::SpecificError(details));
        return self.add_local(self.constants.err, Value::Err);
    }
}

// ✅ CORRECT - Check if binding exists
let binding = if let Some(binding) = self.get_binding(&name) {
    binding
} else {
    self.add_error(ErrorKind::CannotFindName(name));
    return self.add_local(self.constants.err, Value::Err);
};
```

## Test-Driven Development (TDD)

### Test Structure (Copy This Format)

```rust
// In tests/check.rs:

// Positive test - should pass type checking
pass!(descriptive_test_name_, r#"
    let variable: Type = expression;
    variable satisfies Type;
    
    // More complex example:
    let arr = [1, 2, 3];
    arr satisfies number[];
    arr[0] satisfies number;
"#);

// Negative test - should fail with specific error
fail!(error_test_name_, r#"
    let invalid: number = "string";
"#, &["Type 'string' is not assignable to type 'number'."]);
//    ^^^^^ EXACT error message - must match word for word
```

### Test Categories by Feature

1. **Basic Types**: `boolean_`, `number_`, `string_`, `void_`, `unknown_`, `never_`
2. **Collections**: `array_literal_`, `array_type_`, `tuple_`, `nested_array_`
3. **Objects**: `object_literal_`, `property_access_`, `nested_properties_`, `method_`
4. **Functions**: `function_decl_`, `arrow_function_`, `call_`, `closure_`, `higher_order_`
5. **Classes**: `class_basic_`, `constructor_`, `inheritance_`, `method_override_`
6. **Control Flow**: `if_stmt_`, `for_loop_`, `while_loop_`, `switch_`, `try_catch_`
7. **Operators**: `binary_ops_`, `unary_ops_`, `comparison_`, `logical_`, `typeof_`
8. **Type System**: `union_`, `type_narrowing_`, `satisfies_`, `type_guards_`

### TDD Workflow

```bash
# 1. Write failing test
cargo test new_feature_test_ -- --nocapture
# Should fail with "not yet implemented" or compilation error

# 2. Implement minimal functionality to make test pass
# Edit src/checker/base/expr.rs or appropriate file

# 3. Verify test passes
cargo test new_feature_test_ -- --nocapture

# 4. Run full suite to ensure no regressions
cargo test

# 5. Add edge cases and error scenarios
```

## Common Debugging Scenarios

### Scenario 1: "not yet implemented" Panic
```
thread 'test_name_' panicked at src/checker/base/expr.rs:XXX:YY:
not yet implemented: SomeASTNode { ... }
```

**Solution**: Add match arm in `check_expr()` for the missing AST node.

### Scenario 2: "called Option::unwrap() on None"
```
thread 'test_name_' panicked at src/checker/base/expr.rs:XXX:YY:
called `Option::unwrap()` on a `None` value
```

**Solution**: Replace `.unwrap()` with proper error handling:
```rust
// ❌ BAD
let binding = self.get_binding(&name).unwrap();

// ✅ GOOD  
let binding = if let Some(binding) = self.get_binding(&name) {
    binding
} else {
    self.add_error(ErrorKind::CannotFindName(name));
    return self.add_local(self.constants.err, Value::Err);
};
```

### Scenario 3: Test assertion failure
```
assertion `left == right` failed
  left: ["Type 'ActualType' is not assignable to type 'ExpectedType'."]
 right: ["Expected error message"]
```

**Solution**: Error message mismatch. Check error formatting in `src/checker/errors.rs`.

### Scenario 4: Missing property on built-in types
```
Property 'length' does not exist on type 'string'
```

**Solution**: Add to built-in prototypes in `src/context.rs`:
```rust
pub proto_string: HashMap<Atom, Ty<'tcx>>, // Add missing methods here
```

## Type System Architecture (Quick Reference)

### Core Types (src/kind/mod.rs)
```rust
TyKind::Void           // void
TyKind::Boolean        // boolean  
TyKind::Number         // number
TyKind::String(None)   // string (generic)
TyKind::String(Some(atom)) // "literal string"
TyKind::Array(Array)   // T[]
TyKind::Object(Object) // { prop: Type }
TyKind::Function(Function) // (params) => ReturnType
TyKind::Union(Union)   // Type1 | Type2
TyKind::Unknown        // unknown (top type)
TyKind::Never          // never (bottom type)
TyKind::Err            // <err> (error propagation)
```

### Built-in Constants (src/context.rs)
```rust
self.constants.boolean  // Pre-allocated boolean type
self.constants.number   // Pre-allocated number type
self.constants.string   // Pre-allocated string type
self.constants.err      // Use for failed operations
self.constants.unknown  // Use for untyped/any values
```

### Memory Management
- All types have lifetime `'tcx` tied to `TyContext`
- Never clone types - always use references
- Use `self.tcx.new_*()` methods to create new types
- Use `self.constants.*` for common types

## Performance Tips

### Fast Compilation
```bash
cargo check              # Fastest - just check compilation
cargo test --lib         # Skip integration tests  
cargo test test_name_    # Run single test
```

### Efficient Type Checking
```rust
// ✅ Fast path for identity
if expected == actual { return true; }

// ✅ Early returns for special cases
match (expected.kind(), actual.kind()) {
    (Unknown, _) => return true,
    (_, Never) => return true,
    // ... expensive logic only if needed
}

// ✅ Use BTreeSet for unions (automatic deduplication)
self.tcx.new_union(BTreeSet::from([type1, type2]))
```

## Integration with SWC

### Key SWC Types You'll Use
```rust
use swc_ecma_ast::{
    Expr, Stmt, Pat, TsType,           // Core AST nodes
    BinExpr, CallExpr, MemberExpr,     // Expression types
    IfStmt, ForStmt, WhileStmt,        // Statement types  
    BinaryOp, UnaryOp,                 // Operators
    Ident, Atom,                       // Identifiers and strings
};
```

### SWC Pattern Matching
```rust
// Expressions
match expr {
    Expr::Ident(ident) => { /* ident.sym is Atom */ }
    Expr::Lit(lit) => { /* lit is Lit enum */ }
    Expr::Bin(BinExpr { op, left, right, .. }) => { /* binary ops */ }
    Expr::Call(CallExpr { callee, args, .. }) => { /* function calls */ }
    Expr::Member(MemberExpr { obj, prop, .. }) => { /* property access */ }
}

// Statements  
match stmt {
    Stmt::Expr(ExprStmt { expr, .. }) => { /* expression statement */ }
    Stmt::Decl(decl) => { /* declarations */ }
    Stmt::If(IfStmt { test, cons, alt, .. }) => { /* if statements */ }
    Stmt::For(ForStmt { init, test, update, body, .. }) => { /* for loops */ }
}
```

## Emergency Debugging

When completely stuck:

```bash
# 1. Check what the test expects
cat tests/check.rs | grep -A 10 "failing_test_name"

# 2. See full error with stack trace  
RUST_BACKTRACE=full cargo test failing_test_ -- --nocapture

# 3. Add debug prints (temporarily)
println!("DEBUG: type = {:?}", some_type);
dbg!(&expr);

# 4. Check if similar tests exist and pass
cargo test similar_feature_ -- --nocapture

# 5. Simplify the test case to minimal reproduction
# Edit tests/check.rs to remove complexity
```

Remember: **The 138 tests are the complete specification**. If they pass, the TypeScript type checker is correct. Focus on making tests pass rather than implementing theoretical completeness.

## Next Steps for New Features

1. **Find a failing or missing test** - What TypeScript feature isn't covered?
2. **Write a simple test first** - Use `pass!()` macro with basic case
3. **Run the test** - It should fail with clear error message
4. **Implement in expr.rs** - Add pattern match for new SWC AST node  
5. **Handle errors gracefully** - Always return `self.constants.err` on failure
6. **Verify test passes** - Single test should now work
7. **Add edge cases** - More complex tests with error scenarios
8. **Run full suite** - Ensure no regressions in 138 tests

This guide captures the real, practical experience of implementing a complete TypeScript type checker. Follow these patterns and the 138 tests will guide you to a correct implementation.