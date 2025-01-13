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

fail!(
	get_prop_on_union_with_non_object_arm_,
	r#"
        let x: number | { n: number } = 42;
        x.n;
    "#,
	&["Property 'n' does not exist on type 'number'."]
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
	&["This expression is not callable.\nType 'number' has no call signatures."]
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
	&["Property 'foo' does not exist on type '\"hello\"'."]
);
