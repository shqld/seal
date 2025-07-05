use super::{fail, pass};

pass!(
	empty_class,
	r#"
        class A {}

        new A() satisfies A;
    "#
);

fail!(
	empty_class_with_shape_mismatch,
	r#"
        class A {}
        class B {}

        new A() satisfies B;
    "#,
	&["Type 'A' is not assignable to type 'B'."]
);

pass!(
	empty_constructor,
	r#"
        class A {
            constructor() {
            }
        }

        new A() satisfies A;
    "#
);

pass!(
	simple_constructor,
	r#"
        class A {
            constructor(n: number) {
            }
        }

        new A(42) satisfies A;
    "#
);

fail!(
	simple_constructor_args_mismatch,
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
	class_property,
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
	property_initializer,
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
	property_initializer_without_type_annotation,
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
	untyped_property,
	r#"
        class A {
            n;
        }
    "#,
	&["Type annotation or initializer is required."]
);

fail!(
	property_initializer_mismatch,
	r#"
        class A {
            n: number = "hello";
        }
    "#,
	&["Type 'string' is not assignable to type 'number'."]
);

pass!(
	class_method,
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
	class_with_multiple_properties,
	r#"
        class Animal {
            name: string = "default";
            age: number = 0;
        }
        
        let animal = new Animal();
        animal satisfies Animal;
        animal.name satisfies string;
        animal.age satisfies number;
    "#
);

pass!(
	class_with_multiple_methods,
	r#"
        class Calculator {
            add(a: number, b: number): number {
                return a + b;
            }
            
            multiply(a: number, b: number): number {
                return a * b;
            }
        }
        
        let calc = new Calculator();
        calc.add satisfies (a: number, b: number) => number;
        calc.multiply satisfies (a: number, b: number) => number;
    "#
);

fail!(
	class_property_access_nonexistent,
	r#"
        class BankAccount {
            balance: number = 1000;
        }
        
        let account = new BankAccount();
        account.pin;
    "#,
	&["Property 'pin' does not exist on type 'BankAccount'."]
);
