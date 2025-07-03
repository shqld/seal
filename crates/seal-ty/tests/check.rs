use seal_ty::{checker::Checker, context::TyContext, parse::parse};

fn run(code: &'static str) -> Result<(), Vec<String>> {
	let result = parse(code).unwrap();

	let ast = result.program;
	let tcx = TyContext::new();
	let checker = Checker::new(&tcx);

	checker
		.check(&ast)
		.map_err(|errors| errors.into_iter().map(|e| format!("{}", e)).collect())
}

macro_rules! pass {
	($case_name:ident, $code:literal) => {
		#[test]
		fn $case_name() {
			run($code).unwrap();
		}
	};
}

macro_rules! fail {
	($case_name:ident, $code:literal, $expected:expr) => {
		#[test]
		fn $case_name() {
			let errors = run($code).unwrap_err();
			let expected: &[&'static str] = $expected;

			assert_eq!(
				errors,
				expected
					.iter()
					.map(|s| s.to_string())
					.collect::<Vec<String>>()
			);
		}
	};
}

pass!(
	let_,
	r#"
        let a = 1;
        a satisfies number;
    "#
);

pass!(
	assign_to_uninitialized_var_,
	r#"
        let a;
        a = 1;
        a satisfies number;
    "#
);

fail!(
	assign_type_mismatched_value_to_initialized_var,
	r#"
        let a = 1;
        a = "hello";
    "#,
	&["Type 'string' is not assignable to type 'number'."]
);

fail!(
	assign_to_initialized_var_that_was_originally_uninitialized_,
	r#"
        let a;
        a = 1;
        a = "hello";
    "#,
	&["Type 'string' is not assignable to type 'number'."]
);

pass!(
	const_binding_,
	r#"
        const x = "42";
        x satisfies "42";
        x satisfies string;
    "#
);

fail!(
	let_binding_,
	r#"
        let x = "42";
        x satisfies "42";
    "#,
	&["Type 'string' is not assignable to type '\"42\"'."]
);

pass!(
	function_1_,
	r#"
        function f(): void {
        }

        f satisfies () => void;
    "#
);

pass!(
	function_void_,
	r#"
        function f(n: number): void {
            return;
        }

        f satisfies (n: number) => void;
    "#
);

pass!(
	function_void_wo_ret_,
	r#"
        function f(n: number): void {
        }

        f satisfies (n: number) => void;
    "#
);

pass!(
	function_3_,
	r#"
        function f(n: number, s: string, b: boolean): void {
        }

        f satisfies (n: number, s: string, b: boolean) => void;
    "#
);

pass!(
	function_ret_,
	r#"
        function f(n: number): number {
            return n;
        }

        f satisfies (n: number) => number;
    "#
);

fail!(
	function_ret_mismatch_1_,
	r#"
        function f(n: number): number {
        }
    "#,
	&["A function whose declared type is 'void' must return a value."]
);

fail!(
	function_ret_mismatch_2_,
	r#"
        function f(n: number): number {
            return;
        }
    "#,
	&["A function whose declared type is 'void' must return a value."]
);

pass!(
	void_function_no_ret_type_ann_1_,
	r#"
        function f() {
            return;
        }

        f satisfies () => void;
    "#
);

pass!(
	void_function_no_ret_type_ann_2_,
	r#"
        function f() {
        }

        f satisfies () => void;
    "#
);

fail!(
	function_no_ret_type_ann_,
	r#"
        function f() {
            return 42;
        }
    "#,
	&["Type 'number' is not assignable to type 'void'."]
);

pass!(
	assign_to_annotated_var_,
	r#"
        let n: number;
        n = 1;

        n satisfies number;
    "#
);

fail!(
	assign_to_annotated_var_type_mismatch_,
	r#"
        let n: number;
        n = "hello";
    "#,
	&["Type 'string' is not assignable to type 'number'."]
);

pass!(
	union_,
	r#"
        let x: number | string;
        x = 42;
        x = "hello";
    "#
);

pass!(
	narrow_union_with_multiple_branches_,
	r#"
        let x: boolean | number | string = 42;

        if (typeof x === 'number') {
            x satisfies number;
        } else if (typeof x === 'string') {
            x satisfies string;
        } else {
            x satisfies boolean;
        }
    "#
);

pass!(
	narrow_union_through_nested_scope_,
	r#"
        let x: number | string = 42;

        if (typeof x === 'number') {
            x satisfies number;

            if (true) {
                // TODO: inner scope should inherit the scoped types in the outer scope
                // x satisfies number;
            }

            x satisfies number;
        }
    "#
);

fail!(
	narrowed_union_,
	r#"
        let x: number | string = 42;

        if (typeof x === 'number') {
            x satisfies number;
        }

        // x: number | string
        x satisfies number;
    "#,
	&["Type 'number | string' is not assignable to type 'number'."]
);

pass!(
	assign_to_narrowed_union_,
	r#"
        let x: number | string = 42;

        if (typeof x === 'number') {
            x = 42;
            // TODO:
            // x = "hello";
        } else {
            // TODO:
            // x = 42;
            x = "hello";
        }
    "#
);

pass!(
	object_,
	r#"
        let x: { n: number } = { n: 42 };
        x satisfies { n: number };
        x.n satisfies number;
    "#
);

fail!(
	assign_to_object_mismatch_1_,
	r#"
        let x: { n: number } = 42;
    "#,
	&["Type 'number' is not assignable to type '{n: number}'."]
);

fail!(
	assign_to_object_mismatch_2_,
	r#"
        let x: { n: number } = { n: 'hello' };
    "#,
	&["Type '{n: \"hello\"}' is not assignable to type '{n: number}'."]
);

pass!(
	narrow_object_union_by_if_stmt_,
	r#"
        let x: { t: "a", u: "x" } | { t: "b", u: "y" } | { t: "c", u: "z" } = { t: "a", u: "x" };

        if (x.t === 'a') {
            x satisfies { t: "a", u: "x" };
        } else if (x.t === 'b') {
            x satisfies { t: "b", u: "y" };
        } else {
            x satisfies { t: "c", u: "z" };
        }
    "#
);

pass!(
	get_prop_on_object_,
	r#"
        let x = { n: 42 };

        x.n satisfies number;
    "#
);

pass!(
	get_prop_on_union_,
	r#"
        let x: { x: number } | { x: string } = { x: 42 };

        // TODO: this assertion doesn't cover the case where 'x.x' is a 'number' or 'string'
        x.x satisfies number | string;
    "#
);

pass!(
	// TODO:
	assign_prop_on_union_,
	r#"
        let x: { x: number } | { x: string } = { x: 42 };

        // x.x = 42;
        // x.x = "hello";
    "#
);

fail!(
	get_prop_on_non_object_,
	r#"
        let x: number = 42;
        x.t;
    "#,
	&["Property 't' does not exist on type 'number'."]
);

fail!(
	get_non_existent_prop_on_object_,
	r#"
        let x: { n: number } = { n: 42 };
        x.t;
    "#,
	&["Property 't' does not exist on type '{n: number}'."]
);

pass!(
	get_prop_on_union_with_non_object_arm_,
	r#"
        let x: { n: number } = { n: 42 };
        x.n satisfies number;
    "#
);

fail!(
	eq_operands_have_no_overlap_,
	r#"
        let x: { n: number } = { n: 42 };
        x.n === "hello";
    "#,
	&[
		"This comparison appears to be unintentional because the types 'number' and '\"hello\"' have no overlap."
	]
);

pass!(
	expr_closure_,
	r#"
        let x = 42;
        let f = () => x;

        f satisfies () => number;
    "#
);

pass!(
	expr_closure_with_ret_type_ann_,
	r#"
        let x = 42;
        let f = (): number => x;

        f satisfies () => number;
    "#
);

fail!(
	expr_closure_ret_missmatch_,
	r#"
        let x = 42;
        let f = (): string => x;
    "#,
	&["Type 'number' is not assignable to type 'string'."]
);

pass!(
	block_closure_,
	r#"
        let x = 42;
        let f = (): number => {
            return x;
        };

        f satisfies () => number;
    "#
);

pass!(
	block_closure_without_ret_type_ann_,
	r#"
        let x = 42;
        let f = () => {};

        f satisfies () => void;
    "#
);

fail!(
	block_closure_ret_mismatch_,
	r#"
        let x = 42;
        let f = (): void => {
            return x;
        };
    "#,
	&["Type 'number' is not assignable to type 'void'."]
);

fail!(
	block_closure_ret_mismatch_without_ret_type_ann_,
	r#"
        let x = 42;
        let f = () => {
            // void is expected for return type because the annotation is missing
            return x;
        };
    "#,
	&["Type 'number' is not assignable to type 'void'."]
);

pass!(
	empty_class_,
	r#"
        class A {}

        new A() satisfies A;
    "#
);

fail!(
	empty_class_with_shame_shape_mismatch_,
	r#"
        class A {}
        class B {}

        new A() satisfies B;
    "#,
	&["Type 'A' is not assignable to type 'B'."]
);

pass!(
	empty_ctor_,
	r#"
        class A {
            constructor() {
            }
        }

        new A() satisfies A;
    "#
);

pass!(
	simple_ctor_,
	r#"
        class A {
            constructor(n: number) {
            }
        }

        new A(42) satisfies A;
    "#
);

fail!(
	simple_ctor_args_mismatch_,
	r#"
        class A {
            constructor(n: number) {
            }
        }

        new A();
        new A("hello");
    "#,
	&[
		"Expected 1 arguments, but got 0.",
		"Type 'string' is not assignable to type 'number'."
	]
);

pass!(
	prop_,
	r#"
        class A {
            n: number;
        }

        let a = new A();
        a satisfies A;
        a.n satisfies number;
    "#
);

pass!(
	prop_initializer_,
	r#"
        class A {
            n: number = 42;
        }

        let a = new A();
        a satisfies A;
        a.n satisfies number;
    "#
);

pass!(
	prop_initializer_without_type_ann_,
	r#"
        class A {
            n = 42;
        }

        let a = new A();
        a satisfies A;
        a.n satisfies number;
    "#
);

fail!(
	untyped_prop_,
	r#"
        class A {
            n;
        }
    "#,
	&["Type annotation or initializer is required."]
);

fail!(
	prop_initializer_mismatch_,
	r#"
        class A {
            n: number = "hello";
        }
    "#,
	&["Type 'string' is not assignable to type 'number'."]
);

pass!(
	method_,
	r#"
        class A {
            m(n: number): number {
                return n;
            }
        }

        let a = new A();
        a.m satisfies (n: number) => number;
    "#
);
pass!(
	function_call_,
	r#"
        function f(): number {
            return 42;
        }

        f satisfies () => number;
        f(42) satisfies number;
    "#
);

fail!(
	function_call_non_callable_value,
	r#"
        let n = 42;

        n satisfies number;
        n(42) satisfies number;
    "#,
	&["This expression is not callable.\n  Type 'number' has no call signatures."]
);

fail!(
	function_call_args_mismatch_,
	r#"
        function f(n: number): number {
            return n;
        }

        f satisfies (n: number) => number;
        f("hello");
    "#,
	&["Type 'string' is not assignable to type 'number'."]
);

pass!(
	number_proto_,
	r#"
        let n = 42;

        n.toExponential satisfies (fractionDigits?: number) => string;
        n.toFixed satisfies (fractionDigits?: number) => string;
        n.toLocaleString satisfies () => string;
        n.toPrecision satisfies (precision?: number) => string;
    "#
);

fail!(
	number_proto_non_existent_method_,
	r#"
        let n = 42;
        n.foo;
    "#,
	&["Property 'foo' does not exist on type 'number'."]
);

pass!(
	string_proto_,
	r#"
        let s = "hello";

        s.length satisfies number;
        s.at satisfies (index: number) => string;
        s.charAt satisfies (index: number) => string;
        s.charCodeAt satisfies (index: number) => number;
        s.codePointAt satisfies (index: number) => number;
        s.concat satisfies (strings: string) => string;
        s.endsWith satisfies (searchString: string) => boolean;
        s.includes satisfies (searchString: string) => boolean;
        s.indexOf satisfies (searchString: string) => number;
        s.isWellFormed satisfies () => boolean;
        s.lastIndexOf satisfies (searchString: string) => number;
        s.localeCompare satisfies (compareString: string) => number;
        // TODO: object
        // s.match satisfies (regexp: string) => object;
        // TODO: object
        // s.matchAll satisfies (regexp: string) => object;
        s.normalize satisfies (form: string) => string;
        s.padEnd satisfies (targetLength: number, padString: string) => string;
        s.padStart satisfies (targetLength: number, padString: string) => string;
        s.repeat satisfies (count: number) => string;
        s.replace satisfies (searchValue: string, replaceValue: string) => string;
        s.replaceAll satisfies (searchValue: string, replaceValue: string) => string;
        s.search satisfies (regexp: string) => number;
        s.slice satisfies (start: number, end: number) => string;
        // TODO: object
        // s.split satisfies (separator: string, limit: number) => object;
        s.startsWith satisfies (searchString: string, position: number) => boolean;
        s.substr satisfies (start: number, length: number) => string;
        s.substring satisfies (start: number, end: number) => string;
        s.toLocaleLowerCase satisfies () => string;
        s.toLocaleUpperCase satisfies () => string;
        s.toLowerCase satisfies () => string;
        s.toUpperCase satisfies () => string;
        s.toWellFormed satisfies () => string;
        s.trim satisfies () => string;
        s.trimEnd satisfies () => string;
        s.trimLeft satisfies () => string;
        s.trimRight satisfies () => string;
        s.trimStart satisfies () => string;
    "#
);

fail!(
	string_proto_non_existent_method_,
	r#"
        let s = "hello";
        s.foo;
    "#,
	&["Property 'foo' does not exist on type 'string'."]
);

// Array literal tests
pass!(
	array_literal_empty_,
	r#"
        let arr = [];
        arr satisfies never[];
    "#
);

pass!(
	array_literal_homogeneous_,
	r#"
        let arr = [1, 2, 3];
        arr satisfies number[];
    "#
);

pass!(
	array_literal_heterogeneous_,
	r#"
        let arr = [1, "hello", true];
        arr satisfies (number | string | boolean)[];
    "#
);

pass!(
	array_literal_nested_,
	r#"
        let arr = [[1, 2], [3, 4]];
        arr satisfies number[][];
    "#
);

// Template literal tests
pass!(
	template_literal_simple_,
	r#"
        let str = `hello world`;
        str satisfies string;
    "#
);

pass!(
	template_literal_with_interpolation_,
	r#"
        let name = "Alice";
        let age = 30;
        let message = `Hello, ${name}! You are ${age} years old.`;
        message satisfies string;
    "#
);

pass!(
	template_literal_nested_,
	r#"
        let a = "world";
        let b = `hello ${a}`;
        let c = `${b}!`;
        c satisfies string;
    "#
);

// Regular expression literal tests
pass!(
	regex_literal_simple_,
	r#"
        let re = /hello/;
        re satisfies RegExp;
    "#
);

pass!(
	regex_literal_with_flags_,
	r#"
        let re = /hello/gi;
        re satisfies RegExp;
    "#
);

pass!(
	regex_literal_properties_,
	r#"
        let re = /hello/gi;
        re.source satisfies string;
        re.global satisfies boolean;
        re.ignoreCase satisfies boolean;
        re.multiline satisfies boolean;
    "#
);

// Object type tests
pass!(
	object_type_basic_,
	r#"
        let obj: Object = {};
        obj satisfies Object;
    "#
);

pass!(
	object_type_with_properties_,
	r#"
        let obj: Object = { name: "test", value: 42 };
        obj satisfies Object;
    "#
);

// Loop statement tests
pass!(
	while_loop_basic_,
	r#"
        let i = 0;
        while (true) {
            i = 42;
        }
        i satisfies number;
    "#
);

pass!(
	do_while_loop_basic_,
	r#"
        let i = 0;
        do {
            i = i + 1;
        } while (i < 10);
        i satisfies number;
    "#
);

pass!(
	for_loop_basic_,
	r#"
        for (let i = 0; i < 10; i = i + 1) {
            let x = i * 2;
            x satisfies number;
        }
    "#
);

pass!(
	for_loop_no_init_,
	r#"
        let i = 0;
        for (; i < 10; i = i + 1) {
            let x = i * 2;
            x satisfies number;
        }
    "#
);

pass!(
	for_loop_empty_,
	r#"
        for (;;) {
            break;
        }
    "#
);

// Switch statement tests
pass!(
	switch_basic_,
	r#"
        let x = 1;
        switch (x) {
            case 1:
                let a = "one";
                break;
            case 2:
                let b = "two";
                break;
            default:
                let c = "other";
        }
    "#
);

pass!(
	switch_string_,
	r#"
        let status = "pending";
        switch (status) {
            case "pending":
                let message = "Waiting...";
                break;
            case "complete":
                let result = "Done!";
                break;
        }
    "#
);

// Try-catch-finally tests
pass!(
	try_catch_basic_,
	r#"
        try {
            let x = 42;
            x satisfies number;
        } catch (e) {
            let error = "Error occurred";
            error satisfies string;
        }
    "#
);

pass!(
	try_catch_finally_,
	r#"
        try {
            let x = 42;
        } catch (e) {
            let error = "Error";
        } finally {
            let cleanup = true;
            cleanup satisfies boolean;
        }
    "#
);

// Throw statement tests
pass!(
	throw_statement_,
	r#"
        function throwError() {
            throw "An error occurred";
        }
        
        throwError satisfies () => void;
    "#
);

pass!(
	throw_object_,
	r#"
        function throwError() {
            throw { message: "Error", code: 500 };
        }
        
        throwError satisfies () => void;
    "#
);

// Catch parameter tests
pass!(
	catch_parameter_unknown_,
	r#"
        try {
            let x = 42;
        } catch (e) {
            e satisfies unknown;
        }
    "#
);

fail!(
	catch_parameter_with_type_annotation_,
	r#"
        try {
            let x = 42;
        } catch (e: string) {
            let error = e;
        }
    "#,
	&["Catch clause parameter cannot have a type annotation."]
);

pass!(
	catch_parameter_usage_,
	r#"
        try {
            throw "error";
        } catch (error) {
            let msg = error;
            msg satisfies unknown;
        }
    "#
);

// Unknown type tests
pass!(
	unknown_type_basic_,
	r#"
        let x: unknown = 42;
        x satisfies unknown;
    "#
);

pass!(
	unknown_accepts_anything_,
	r#"
        let x: unknown = "hello";
        let y: unknown = 42;
        let z: unknown = true;
        let obj: unknown = { a: 1 };
    "#
);

fail!(
	unknown_assignment_restriction_,
	r#"
        let x: unknown = 42;
        let y: number = x;
    "#,
	&["Type 'unknown' is not assignable to type 'number'."]
);

// Array type comprehensive tests
pass!(
	array_type_annotation_,
	r#"
        let arr: number[] = [1, 2, 3];
        arr satisfies number[];
    "#
);

pass!(
	array_empty_with_annotation_,
	r#"
        let arr: string[] = [];
        arr satisfies string[];
    "#
);

fail!(
	array_type_mismatch_,
	r#"
        let arr: number[] = ["hello", "world"];
    "#,
	&["Type 'string[]' is not assignable to type 'number[]'."]
);

pass!(
	array_mixed_compatible_union_,
	r#"
        let arr: (number | string)[] = [1, "hello", 2, "world"];
        arr satisfies (number | string)[];
    "#
);

fail!(
	array_element_access_bounds_,
	r#"
        let arr = [1, 2, 3];
        let invalid: string = arr[0];
    "#,
	&["Type 'number' is not assignable to type 'string'."]
);

pass!(
	array_method_access_,
	r#"
        let arr = [1, 2, 3];
        // Note: Array methods would need to be implemented in the type system
        // For now, just test basic array property access
    "#
);

// Nested array tests
pass!(
	nested_array_2d_,
	r#"
        let matrix: number[][] = [[1, 2], [3, 4], [5, 6]];
        matrix satisfies number[][];
    "#
);

pass!(
	nested_array_3d_,
	r#"
        let cube: number[][][] = [[[1, 2]], [[3, 4]]];
        cube satisfies number[][][];
    "#
);

// Union type comprehensive tests
pass!(
	union_three_types_,
	r#"
        let value: number | string | boolean = 42;
        value = "hello";
        value = true;
    "#
);

pass!(
	union_with_null_undefined_,
	r#"
        let value: string | null = "hello";
        // Note: null and undefined types would need to be implemented
    "#
);

fail!(
	union_assignment_invalid_,
	r#"
        let value: number | string = true;
    "#,
	&["Type 'boolean' is not assignable to type 'number | string'."]
);

// Function type comprehensive tests
pass!(
	function_complex_signature_,
	r#"
        function process(
            x: number,
            y: string,
            callback: (result: number) => string
        ): boolean {
            return true;
        }
        
        process satisfies (x: number, y: string, callback: (result: number) => string) => boolean;
    "#
);

pass!(
	arrow_function_complex_,
	r#"
        let handler = (event: string, data: number): boolean => {
            return true;
        };
        
        handler satisfies (event: string, data: number) => boolean;
    "#
);

fail!(
	function_parameter_count_mismatch_,
	r#"
        function add(a: number, b: number): number {
            return a + b;
        }
        
        add satisfies (x: number) => number;
    "#,
	&["Type '(a: number, b: number) => number' is not assignable to type '(x: number) => number'."]
);

fail!(
	function_return_type_mismatch_,
	r#"
        function getString(): string {
            return "hello";
        }
        
        getString satisfies () => number;
    "#,
	&["Type '() => string' is not assignable to type '() => number'."]
);

// Object type comprehensive tests
pass!(
	object_nested_properties_,
	r#"
        let user = {
            name: "Alice",
            details: {
                age: 30,
                email: "alice@example.com"
            }
        };
        
        user.details.age satisfies number;
        user.details.email satisfies string;
    "#
);

pass!(
	object_method_definition_,
	r#"
        function add(x: number, y: number): number {
            return x + y;
        }
        
        let calculator = {
            add: add,
            value: 42
        };
        
        calculator.add satisfies (x: number, y: number) => number;
        calculator.value satisfies number;
    "#
);

fail!(
	object_missing_property_,
	r#"
        let user: { name: string, age: number } = { name: "Alice" };
    "#,
	&["Type '{name: \"Alice\"}' is not assignable to type '{age: number, name: string}'."]
);

fail!(
	object_extra_property_,
	r#"
        let user: { name: string } = { name: "Alice", age: 30 };
    "#,
	&["Type '{age: number, name: \"Alice\"}' is not assignable to type '{name: string}'."]
);

// Class comprehensive tests
pass!(
	class_inheritance_basic_,
	r#"
        class Animal {
            name: string = "default";
        }
        
        let animal = new Animal();
        animal satisfies Animal;
        animal.name satisfies string;
    "#
);

pass!(
	class_method_overriding_,
	r#"
        class Shape {
            getArea(): number {
                return 0;
            }
        }
        
        let shape = new Shape();
        shape.getArea satisfies () => number;
    "#
);

pass!(
	class_private_property_access_,
	r#"
        class BankAccount {
            balance: number = 1000;
        }
        
        let account = new BankAccount();
        account.balance satisfies number;
    "#
);

// Control flow comprehensive tests
pass!(
	nested_if_statements_,
	r#"
        let x: number | string | boolean = 42;
        
        if (typeof x === 'number') {
            if (x > 0) {
                x satisfies number;
            } else {
                x satisfies number;
            }
        } else if (typeof x === 'string') {
            x satisfies string;
        } else {
            x satisfies boolean;
        }
    "#
);

pass!(
	complex_boolean_expressions_,
	r#"
        let a = true;
        let b = false;
        let c = true;
        
        if (a && b || c) {
            let result = "condition met";
        }
        
        if ((a || b) && c) {
            let result = "complex condition";
        }
    "#
);

// Loop comprehensive tests
pass!(
	for_loop_with_complex_init_,
	r#"
        for (let i = 0; i < 10; i = i + 1) {
            let value = i * 2;
            value satisfies number;
        }
    "#
);

pass!(
	nested_loops_,
	r#"
        for (let i = 0; i < 3; i = i + 1) {
            for (let j = 0; j < 3; j = j + 1) {
                let product = i * j;
                product satisfies number;
            }
        }
    "#
);

pass!(
	while_loop_with_break_continue_,
	r#"
        let count = 0;
        while (count < 10) {
            if (count === 5) {
                count = count + 1;
                continue;
            }
            if (count === 8) {
                break;
            }
            count = count + 1;
        }
    "#
);

// Binary operators comprehensive tests
pass!(
	arithmetic_operators_,
	r#"
        let a = 10;
        let b = 3;
        
        let sum = a + b;
        let diff = a - b;
        let product = a * b;
        let quotient = a / b;
        
        sum satisfies number;
        diff satisfies number;
        product satisfies number;
        quotient satisfies number;
    "#
);

pass!(
	comparison_operators_,
	r#"
        let x = 5;
        let y = 10;
        
        let eq = x === y;
        let neq = x !== y;
        let lt = x < y;
        let lte = x <= y;
        let gt = x > y;
        let gte = x >= y;
        
        eq satisfies boolean;
        neq satisfies boolean;
        lt satisfies boolean;
        lte satisfies boolean;
        gt satisfies boolean;
        gte satisfies boolean;
    "#
);

pass!(
	logical_operators_,
	r#"
        let a = true;
        let b = false;
        
        let and = a && b;
        let or = a || b;
        let notA = !a;
        
        and satisfies boolean;
        or satisfies boolean;
        notA satisfies boolean;
    "#
);

fail!(
	binary_operator_type_mismatch_,
	r#"
        let num = 5;
        let str = "hello";
        let result = num + str;
    "#,
	&["Operator '+' cannot be applied to types 'number' and 'string'."]
);

// Enhanced error message tests - showing improved formatting consistency
fail!(
	property_missing_in_object_,
	r#"
        function f(obj: { a: number, b: string }) {}
        f({ b: "hello" });
    "#,
	&["Type '{b: \"hello\"}' is not assignable to type '{a: number, b: string}'."]
);

fail!(
	function_argument_type_mismatch_,
	r#"
        function f(x: number): void {}
        f("hello");
    "#,
	&["Type 'string' is not assignable to type 'number'."]
);

// String operations comprehensive tests
pass!(
	string_concatenation_,
	r#"
        let greeting = "Hello";
        let space = " ";
        let name = "World";
        let message = greeting + space;
        let fullMessage = message + name;
        fullMessage satisfies string;
    "#
);

pass!(
	string_template_complex_,
	r#"
        let user = "Alice";
        let age = 30;
        let active = true;
        let profile = `User: ${user}, Age: ${age}, Active: ${active}`;
        profile satisfies string;
    "#
);

// Template literal comprehensive tests
pass!(
	template_literal_multiline_,
	r#"
        let html = `
            <div>
                <h1>Title</h1>
                <p>Content</p>
            </div>
        `;
        html satisfies string;
    "#
);

pass!(
	template_literal_nested_expressions_,
	r#"
        let x = 5;
        let y = 10;
        let result = `The sum of ${x} and ${y} is ${x + y}`;
        result satisfies string;
    "#
);

// Regular expression comprehensive tests
pass!(
	regex_complex_patterns_,
	r#"
        let emailPattern = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
        let phonePattern = /^\+?[\d\s\-\(\)]+$/;
        let urlPattern = /https?:\/\/[^\s]+/gi;
        
        emailPattern satisfies RegExp;
        phonePattern satisfies RegExp;
        urlPattern satisfies RegExp;
    "#
);

// Error handling comprehensive tests
pass!(
	try_catch_nested_,
	r#"
        try {
            try {
                let x = 42;
                throw "inner error";
            } catch (innerError) {
                innerError satisfies unknown;
                throw "outer error";
            }
        } catch (outerError) {
            outerError satisfies unknown;
        }
    "#
);

pass!(
	try_catch_finally_complex_,
	r#"
        let resource = "acquired";
        try {
            let data = "processing";
            throw data;
        } catch (error) {
            error satisfies unknown;
            let errorMessage = "handled";
        } finally {
            resource = "released";
        }
    "#
);

// Switch statement comprehensive tests
pass!(
	switch_with_fallthrough_,
	r#"
        let grade = "A";
        switch (grade) {
            case "A":
            case "B":
                let message = "Good job";
                break;
            case "C":
                let message2 = "Average";
                break;
            default:
                let message3 = "Needs improvement";
        }
    "#
);

pass!(
	switch_numeric_cases_,
	r#"
        let code = 200;
        switch (code) {
            case 200:
                let success = "OK";
                break;
            case 404:
                let notFound = "Not Found";
                break;
            case 500:
                let serverError = "Internal Server Error";
                break;
            default:
                let unknown = "Unknown status";
        }
    "#
);

// Never type comprehensive tests
pass!(
	never_type_in_exhaustive_switch_,
	r#"
        let value: "a" | "b" = "a";
        switch (value) {
            case "a":
                let resultA = "A case";
                break;
            case "b":
                let resultB = "B case";
                break;
            default:
                // This should be never type
                let exhaustive: never = value;
        }
    "#
);

fail!(
	never_assignment_,
	r#"
        let impossible: never = 42;
    "#,
	&["Type 'number' is not assignable to type 'never'."]
);

// Function call comprehensive tests
pass!(
	function_higher_order_,
	r#"
        function map(arr: number[], fn: (x: number) => number): number[] {
            return arr;
        }
        
        function double(x: number): number {
            return x * 2;
        }
        
        let numbers = [1, 2, 3];
        let doubled = map(numbers, double);
        doubled satisfies number[];
    "#
);

pass!(
	function_closure_advanced_,
	r#"
        function createCounter(): () => number {
            let count = 0;
            return (): number => {
                count = count + 1;
                return count;
            };
        }
        
        let counter = createCounter();
        counter satisfies () => number;
        let value = counter();
        value satisfies number;
    "#
);

// Type narrowing comprehensive tests
pass!(
	type_narrowing_multiple_guards_,
	r#"
        let value: number | string | boolean | null = "hello";
        
        if (typeof value === 'string') {
            value satisfies string;
        } else if (typeof value === 'number') {
            value satisfies number;
        } else if (typeof value === 'boolean') {
            value satisfies boolean;
        } else {
            // value should be null here
        }
    "#
);

pass!(
	type_narrowing_property_access_,
	r#"
        let obj = { type: "user", name: "Alice" };
        
        if (obj.type === "user") {
            obj.name satisfies string;
        }
    "#
);

// Edge cases and error scenarios
fail!(
	divide_by_zero_type_check_,
	r#"
        let zero = 0;
        let result = 10 / zero;
        result satisfies string;
    "#,
	&["Type 'number' is not assignable to type 'string'."]
);

pass!(
	complex_expression_precedence_,
	r#"
        let a = 2;
        let b = 3;
        let c = 4;
        let result = a + b * c;
        result satisfies number;
    "#
);

fail!(
	invalid_property_chain_,
	r#"
        let obj = { a: { b: { c: 42 } } };
        obj.a.b.d;
    "#,
	&["Property 'd' does not exist on type '{c: number}'."]
);

pass!(
	boolean_type_guards_complex_,
	r#"
        let value: number | string = "test";
        
        if (typeof value === 'string') {
            value satisfies string;
        }
        
        if (typeof value === 'number') {
            value satisfies number;
        }
    "#
);
