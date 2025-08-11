use super::{fail, pass};

// === Number Literal Types ===

pass!(
	number_literal_positive,
	r#"
        const x = 42;
        x satisfies 42;
        x satisfies number;
    "#
);

pass!(
	number_literal_negative,
	r#"
        const x = -1;
        x satisfies -1;
        x satisfies number;
    "#
);

pass!(
	number_literal_zero,
	r#"
        const x = 0;
        x satisfies 0;
        x satisfies number;
    "#
);

fail!(
	number_literal_mismatch,
	r#"
        const x = 42;
        x satisfies 43;
    "#,
	&["Type '42' is not assignable to type '43'."]
);

pass!(
	number_literal_let_widening,
	r#"
        let x = 42;
        x satisfies number;
    "#
);

fail!(
	number_literal_let_no_literal,
	r#"
        let x = 42;
        x satisfies 42;
    "#,
	&["Type 'number' is not assignable to type '42'."]
);

// === String Literal Types ===

pass!(
	string_literal_single_quotes,
	r#"
        const x = 'hello';
        x satisfies "hello";
        x satisfies string;
    "#
);

pass!(
	string_literal_double_quotes,
	r#"
        const x = "world";
        x satisfies "world";
        x satisfies string;
    "#
);

pass!(
	string_literal_empty,
	r#"
        const x = "";
        x satisfies "";
        x satisfies string;
    "#
);

fail!(
	string_literal_mismatch,
	r#"
        const x = "hello";
        x satisfies "world";
    "#,
	&["Type '\"hello\"' is not assignable to type '\"world\"'."]
);

pass!(
	string_literal_let_widening,
	r#"
        let x = "hello";
        x satisfies string;
    "#
);

fail!(
	string_literal_let_no_literal,
	r#"
        let x = "hello";
        x satisfies "hello";
    "#,
	&["Type 'string' is not assignable to type '\"hello\"'."]
);

// === Boolean Literal Types ===

pass!(
	boolean_literal_true,
	r#"
        const x = true;
        x satisfies true;
        x satisfies boolean;
    "#
);

pass!(
	boolean_literal_false,
	r#"
        const x = false;
        x satisfies false;
        x satisfies boolean;
    "#
);

fail!(
	boolean_literal_mismatch,
	r#"
        const x = true;
        x satisfies false;
    "#,
	&["Type 'true' is not assignable to type 'false'."]
);

pass!(
	boolean_literal_let_widening,
	r#"
        let x = true;
        x satisfies boolean;
    "#
);

fail!(
	boolean_literal_let_no_literal,
	r#"
        let x = true;
        x satisfies true;
    "#,
	&["Type 'boolean' is not assignable to type 'true'."]
);

// === Literal Union Types ===

pass!(
	literal_union_string,
	r#"
        type Status = "loading" | "success" | "error";
        const status: Status = "loading";
        status satisfies Status;
    "#
);

pass!(
	literal_union_number,
	r#"
        type Level = 1 | 2 | 3 | 4 | 5;
        const level: Level = 3;
        level satisfies 3;
        level satisfies Level;
    "#
);

pass!(
	literal_union_boolean,
	r#"
        type Permission = true | false;
        const perm: Permission = true;
        perm satisfies true;
        perm satisfies Permission;
    "#
);

pass!(
	literal_union_mixed,
	r#"
        type Value = "auto" | 42 | true;
        const val1: Value = "auto";
        const val2: Value = 42;
        const val3: Value = true;
        
        val1 satisfies "auto";
        val2 satisfies 42;
        val3 satisfies true;
    "#
);

fail!(
	literal_union_wrong_value,
	r#"
        type Status = "loading" | "success" | "error";
        const status: Status = "pending";
    "#,
	&["Type '\"pending\"' is not assignable to type '\"loading\" | \"success\" | \"error\"'."]
);

// === Function Parameters and Return Types ===

pass!(
	function_literal_params,
	r#"
        function processStatus(status: "pending" | "done"): boolean {
            return status === "done";
        }
        
        processStatus("pending");
        processStatus("done");
    "#
);

fail!(
	function_literal_param_mismatch,
	r#"
        function processStatus(status: "pending" | "done"): boolean {
            return status === "done";
        }
        
        processStatus("invalid");
    "#,
	&["Type '\"invalid\"' is not assignable to type '\"pending\" | \"done\"'."]
);

pass!(
	function_literal_return,
	r#"
        function getStatus(): "ready" | "busy" {
            return "ready";
        }
        
        const status = getStatus();
        status satisfies "ready" | "busy";
    "#
);

// === Object Properties with Literal Types ===

pass!(
	object_literal_properties,
	r#"
        const config = {
            mode: "development" as const,
            port: 3000 as const,
            secure: true as const
        };
        
        config.mode satisfies "development";
        config.port satisfies 3000;
        config.secure satisfies true;
    "#
);

pass!(
	object_type_literal_properties,
	r#"
        type Config = {
            mode: "development" | "production";
            port: 3000 | 8080;
            secure: boolean;
        };
        
        const config: Config = {
            mode: "development",
            port: 3000,
            secure: true
        };
        
        config satisfies Config;
    "#
);

fail!(
	object_literal_property_mismatch,
	r#"
        type Config = {
            mode: "development" | "production";
        };
        
        const config: Config = {
            mode: "staging"
        };
    "#,
	&[
		"Type '{mode: \"staging\"}' is not assignable to type '{mode: \"development\" | \"production\"}'."
	]
);

// === Array Elements with Literal Types ===

pass!(
	array_literal_elements,
	r#"
        const statuses = ["pending", "done"];
        statuses satisfies string[];
    "#
);

fail!(
	array_literal_elements_fail,
	r#"
        const statuses = ["pending", "done"];
        statuses satisfies readonly ["pending", "done"];
    "#,
	&["Type 'string[]' is not assignable to type '[\"pending\", \"done\"]'."]
);

pass!(
	array_literal_elements_as_const,
	r#"
        const statuses = ["pending", "done"] as const;
        statuses satisfies readonly ["pending", "done"];
    "#
);

pass!(
	array_literal_union_elements,
	r#"
        type Status = "pending" | "done" | "error";
        const statuses: Status[] = ["pending", "done"];
        statuses[0] satisfies Status;
    "#
);

// === Conditional Type Narrowing with Literals ===

pass!(
	literal_type_narrowing,
	r#"
        type Status = "loading" | "success" | "error";
        let status: Status = "loading";
        
        if (status === "success") {
            status satisfies "success";
        }
    "#
);

pass!(
	literal_type_narrowing_else,
	r#"
        type Status = "loading" | "success" | "error";
        let status: Status = "loading";
        
        if (status === "success") {
            status satisfies "success";
        } else {
            status satisfies "loading" | "error";
        }
    "#
);

// === Switch Statements with Literal Types ===

pass!(
	switch_literal_exhaustive,
	r#"
        type Direction = "up" | "down" | "left" | "right";
        const direction: Direction = "up";
        
        switch (direction) {
            case "up":
                let upResult = "going up";
                break;
            case "down":
                let downResult = "going down";
                break;
            case "left":
                let leftResult = "going left";
                break;
            case "right":
                let rightResult = "going right";
                break;
        }
    "#
);

// === Complex Literal Type Scenarios ===

pass!(
	literal_type_complex_scenario,
	r#"
        type EventType = "click" | "hover" | "focus";
        type Priority = 1 | 2 | 3;
        
        interface Event {
            type: EventType;
            priority: Priority;
            enabled: boolean;
        }
        
        const event: Event = {
            type: "click",
            priority: 1,
            enabled: true
        };
        
        event.type satisfies "click";
        event.priority satisfies 1;
        event.enabled satisfies boolean;
    "#
);

pass!(
	literal_type_assignment_widening,
	r#"
        let mode = "development";  // let widens to string
        const env = "production";  // const preserves literal
        
        mode satisfies string;
        env satisfies "production";
    "#
);

// === Edge Cases ===

pass!(
	literal_type_null_undefined,
	r#"
        const nullValue = null;
        const undefinedValue = undefined;
        
        nullValue satisfies null;
        undefinedValue satisfies undefined;
    "#
);

pass!(
	literal_type_number_operations,
	r#"
        const a = 5;
        const b = 10;
        const sum = a + b;  // Should be number, not literal
        
        sum satisfies number;
    "#
);

fail!(
	literal_type_number_operations_not_literal,
	r#"
        const a = 5;
        const b = 10;
        const sum = a + b;
        
        sum satisfies 15;  // Operation results are not literal types
    "#,
	&["Type 'number' is not assignable to type '15'."]
);

pass!(
	literal_type_string_concatenation,
	r#"
        const greeting = "Hello";
        const name = "World";
        const message = greeting + " " + name;  // Should be string, not literal
        
        message satisfies string;
    "#
);

fail!(
	literal_type_string_concatenation_not_literal,
	r#"
        const greeting = "Hello";
        const name = "World";
        const message = greeting + " " + name;
        
        message satisfies "Hello World";  // Concatenation results are not literal types
    "#,
	&["Type 'string' is not assignable to type '\"Hello World\"'."]
);
