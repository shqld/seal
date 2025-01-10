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
	union_to_same_union,
	r#"
        let x: number | string = 42;
        x satisfies number | string;
    "#
);

pass!(
	element_to_union,
	r#"
        let x: number = 42;
        x satisfies number | string;
    "#
);

pass!(
	union_to_super_union,
	r#"
        let x: number | string = 42;
        x satisfies boolean | number | string;
    "#
);

fail!(
	union_to_sub_union,
	r#"
        let x: boolean | number | string = 42;
        x satisfies number | string;
    "#,
	&["Type 'boolean | number | string' is not assignable to type 'number | string'."]
);

pass!(
	const_string_to_string,
	r#"
        let x: "hello" = "hello";
        x satisfies string;
    "#
);

pass!(
	const_string_to_same_const_string,
	r#"
        let x: "hello" = "hello";
        x satisfies "hello";
    "#
);

fail!(
	const_string_to_other_const_string,
	r#"
        let x: "hello" = "hello";
        "hello" satisfies "world";
    "#,
	&["Type '\"hello\"' is not assignable to type '\"world\"'."]
);

pass!(
	string_to_string,
	r#"
        let x: string = "hello";
        x satisfies string;
    "#
);

fail!(
	string_to_const_string,
	r#"
        let x: string = "hello";
        x satisfies "hello";
    "#,
	&["Type 'string' is not assignable to type '\"hello\"'."]
);

pass!(
	function,
	r#"
        function f(x: number): string {
            return "hello";
        }
        f satisfies (x: number) => string;
    "#
);

fail!(
	function_wrong_length_of_params,
	r#"
        function f(x: number, y: number): string {
            return "hello";
        }
        f satisfies (x: number) => string;
    "#,
	&["Type '(x: number, y: number) => string' is not assignable to type '(x: number) => string'."]
);

fail!(
	function_wrong_params,
	r#"
        function f(x: number): string {
            return "hello";
        }
        f satisfies (x: string) => string;
    "#,
	&["Type '(x: number) => string' is not assignable to type '(x: string) => string'."]
);

fail!(
	function_wrong_ret,
	r#"
        function f(x: number): number {
            return 42;
        }
        f satisfies (x: number) => string;
    "#,
	&["Type '(x: number) => number' is not assignable to type '(x: number) => string'."]
);
