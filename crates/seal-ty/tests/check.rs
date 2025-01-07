use seal_ty::{
	checker::{TypeChecker, parse::parse},
	context::TyContext,
};

fn run(code: &'static str) -> Result<(), Vec<String>> {
	let result = parse(code).unwrap();

	let ast = result.program;
	let tcx = TyContext::new();
	let checker = TypeChecker::new(&tcx);

	checker.check(&ast)
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
	&["expected 'number', got 'string'"]
);

fail!(
	assign_to_initialized_var_that_was_originally_uninitialized_,
	r#"
        let a;
        a = 1;
        a = "hello";
    "#,
	&["expected 'number', got 'string'"]
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
	&["function does not return"]
);

fail!(
	function_ret_mismatch_2_,
	r#"
        function f(n: number): number {
            return;
        }
    "#,
	&["expected return value"]
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
	&["expected 'void', got 'number'"]
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
	&["expected 'number', got 'string'"]
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
	narrow_primitive_union_by_if_stmt_,
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

fail!(
	narrowed_union_,
	r#"
        let x: number | string = 42;

        if (typeof x === 'number') {
            x satisfies number;
        }

        // x: number | string
        x satisfies string;
    "#,
	&["expected 'string', got 'number | string'"]
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
	&["expected '{n: number}', got 'number'"]
);

fail!(
	assign_to_object_mismatch_2_,
	r#"
        let x: { n: number } = { n: 'hello' };
    "#,
	&["expected '{n: number}', got '{n: \"hello\"}'"]
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
	&["Property 't' does not exist on type 'number'",]
);

fail!(
	get_non_existent_prop_on_object_,
	r#"
        let x: { n: number } = { n: 42 };
        x.t;
    "#,
	&["Property 't' does not exist on type '{n: number}'"]
);

fail!(
	get_prop_on_union_with_non_object_arm_,
	r#"
        let x: number | { n: number } = 42;
        x.n;
    "#,
	&["Property 'n' does not exist on type 'number'"]
);

fail!(
	eq_operands_have_no_overlap_,
	r#"
        let x: { n: number } = { n: 42 };
        x.n === "hello";
    "#,
	&[
		"This comparison appears to be unintentional because the types 'number' and '\"hello\"' have no overlap"
	]
);
