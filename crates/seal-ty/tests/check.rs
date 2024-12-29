use seal_ty::{
	checker::{TypeChecker, parse::parse},
	context::TyContext,
	sema::Sema,
};

fn run(code: &'static str) {
	let result = parse(code).unwrap();

	let ast = result.program;
	let tcx = TyContext::new();
	let sema = Sema::new(&tcx);
	let air = sema.build(&ast);
	let checker = TypeChecker::new(&tcx);

	checker.check(&air);
}

macro_rules! pass {
	($case_name:ident, $code:literal) => {
		#[test]
		fn $case_name() {
			run($code);
		}
	};
}

macro_rules! fail {
	($case_name:ident, $code:literal) => {
		#[should_panic]
		#[test]
		fn $case_name() {
			run($code);
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
	assign_,
	r#"
        let a = 1;
        a = "hello";
    "#
);

fail!(
	assign_to_initialized_var_that_was_originally_uninitialized_,
	r#"
        let a;
        a = 1;
        a = "hello";
    "#
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
    "#
);

fail!(
	function_ret_mismatch_2_,
	r#"
        function f(n: number): number {
            return;
        }
    "#
);

fail!(
	function_ret_mismatch_3_,
	r#"
        function f(n: number): number {
        }
    "#
);

pass!(
	infer_function_param_,
	r#"
        function f(n): number {
            return n;
        }

        f satisfies (n: number) => number;
    "#
);

fail!(
	inferred_function_param_mismatch_,
	r#"
        function f(n): number {
            return n;
        }

        f satisfies (n: string) => number;
    "#
);

pass!(
	infer_function_ret_,
	r#"
        function f(n: number) {
            return n;
        }

        f satisfies (n: number) => number;
    "#
);

fail!(
	infer_function_ret_mismatch_,
	r#"
        function f(n: number) {
            return n;
        }

        f satisfies (n: number) => string;
    "#
);
