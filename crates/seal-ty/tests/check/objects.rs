use super::{fail, pass};

pass!(
	object_literal,
	r#"
        let x: { n: number } = { n: 42 };
        x satisfies { n: number };
        x.n satisfies number;
    "#
);

fail!(
	assign_to_object_mismatch_not_object,
	r#"
        let x: { n: number } = 42;
    "#,
	&["Type 'number' is not assignable to type '{n: number}'."]
);

fail!(
	assign_to_object_mismatch_property_type,
	r#"
        let x: { n: number } = { n: 'hello' };
    "#,
	&["Type '{n: \"hello\"}' is not assignable to type '{n: number}'."]
);

pass!(
	narrow_object_union_by_if_stmt,
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
	get_prop_on_object,
	r#"
        let x = { n: 42 };

        x.n satisfies number;
    "#
);

pass!(
	get_prop_on_union,
	r#"
        let x: { x: number } | { x: string } = { x: 42 };

        // TODO: this assertion doesn't cover the case where 'x.x' is a 'number' or 'string'
        x.x satisfies number | string;
    "#
);

fail!(
	get_prop_on_non_object,
	r#"
        let x: number = 42;
        x.t;
    "#,
	&["Property 't' does not exist on type 'number'."]
);

fail!(
	get_non_existent_prop_on_object,
	r#"
        let x: { n: number } = { n: 42 };
        x.t;
    "#,
	&["Property 't' does not exist on type '{n: number}'."]
);

pass!(
	object_type_basic,
	r#"
        let obj: Object = {};
        obj satisfies Object;
    "#
);

pass!(
	object_type_with_properties,
	r#"
        let obj: Object = { name: "test", value: 42 };
        obj satisfies Object;
    "#
);

pass!(
	object_nested_properties,
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
	object_method_definition,
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
	object_missing_property,
	r#"
        let user: { name: string, age: number } = { name: "Alice" };
    "#,
	&["Type '{name: \"Alice\"}' is not assignable to type '{age: number, name: string}'."]
);

fail!(
	object_extra_property,
	r#"
        let user: { name: string } = { name: "Alice", age: 30 };
    "#,
	&["Type '{age: number, name: \"Alice\"}' is not assignable to type '{name: string}'."]
);

fail!(
	property_missing_in_object,
	r#"
        function f(obj: { a: number, b: string }) {}
        f({ b: "hello" });
    "#,
	&["Type '{b: \"hello\"}' is not assignable to type '{a: number, b: string}'."]
);

fail!(
	invalid_property_chain,
	r#"
        let obj = { a: { b: { c: 42 } } };
        obj.a.b.d;
    "#,
	&["Property 'd' does not exist on type '{c: number}'."]
);
