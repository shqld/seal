use super::{fail, pass};

pass!(
	function_basic,
	r#"
        function f(): void {
        }

        f satisfies () => void;
    "#
);

pass!(
	function_void,
	r#"
        function f(n: number): void {
            return;
        }

        f satisfies (n: number) => void;
    "#
);

pass!(
	function_void_without_return,
	r#"
        function f(n: number): void {
        }

        f satisfies (n: number) => void;
    "#
);

pass!(
	function_multiple_params,
	r#"
        function f(n: number, s: string, b: boolean): void {
        }

        f satisfies (n: number, s: string, b: boolean) => void;
    "#
);

pass!(
	function_return_value,
	r#"
        function f(n: number): number {
            return n;
        }

        f satisfies (n: number) => number;
    "#
);

fail!(
	function_return_mismatch_no_return,
	r#"
        function f(n: number): number {
        }
    "#,
	&["A function whose declared type is 'void' must return a value."]
);

fail!(
	function_return_mismatch_void_return,
	r#"
        function f(n: number): number {
            return;
        }
    "#,
	&["A function whose declared type is 'void' must return a value."]
);

pass!(
	void_function_no_return_type_annotation_with_return,
	r#"
        function f() {
            return;
        }

        f satisfies () => void;
    "#
);

pass!(
	void_function_no_return_type_annotation_without_return,
	r#"
        function f() {
        }

        f satisfies () => void;
    "#
);

fail!(
	function_no_return_type_annotation_with_value,
	r#"
        function f() {
            return 42;
        }
    "#,
	&["Type 'number' is not assignable to type 'void'."]
);

pass!(
	arrow_function_expression,
	r#"
        let x = 42;
        let f = () => x;

        f satisfies () => number;
    "#
);

pass!(
	arrow_function_with_return_type_annotation,
	r#"
        let x = 42;
        let f = (): number => x;

        f satisfies () => number;
    "#
);

fail!(
	arrow_function_return_mismatch,
	r#"
        let x = 42;
        let f = (): string => x;
    "#,
	&["Type 'number' is not assignable to type 'string'."]
);

pass!(
	arrow_function_block,
	r#"
        let x = 42;
        let f = (): number => {
            return x;
        };

        f satisfies () => number;
    "#
);

pass!(
	arrow_function_block_without_return_type_annotation,
	r#"
        let x = 42;
        let f = () => {};

        f satisfies () => void;
    "#
);

fail!(
	arrow_function_block_return_mismatch,
	r#"
        let x = 42;
        let f = (): void => {
            return x;
        };
    "#,
	&["Type 'number' is not assignable to type 'void'."]
);

fail!(
	arrow_function_block_return_mismatch_without_annotation,
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
	function_call,
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
	function_call_args_mismatch,
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
	function_complex_signature,
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
	arrow_function_complex,
	r#"
        let handler = (event: string, data: number): boolean => {
            return true;
        };
        
        handler satisfies (event: string, data: number) => boolean;
    "#
);

fail!(
	function_parameter_count_mismatch,
	r#"
        function add(a: number, b: number): number {
            return a + b;
        }
        
        add satisfies (x: number) => number;
    "#,
	&["Type '(a: number, b: number) => number' is not assignable to type '(x: number) => number'."]
);

fail!(
	function_return_type_mismatch,
	r#"
        function getString(): string {
            return "hello";
        }
        
        getString satisfies () => number;
    "#,
	&["Type '() => string' is not assignable to type '() => number'."]
);

fail!(
	function_argument_type_mismatch,
	r#"
        function f(x: number): void {}
        f("hello");
    "#,
	&["Type 'string' is not assignable to type 'number'."]
);

pass!(
	function_higher_order,
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
	function_closure_advanced,
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
