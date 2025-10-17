//! Documentation tests for function pointer transformations from C to Rust
//!
//! This test suite documents how C function pointers transform to Rust function pointers (fn types).
//!
//! ## C99 Standard References
//! - ISO C99 §6.7.5.3: Function declarators (including prototypes)
//! - ISO C99 §6.3.2.1: Lvalues, arrays, and function designators
//! - K&R C 2nd Ed §5.11: Pointers to Functions
//!
//! ## Key Transformations
//!
//! ### Simple Function Pointers
//! ```c
//! int (*fp)(int, int);              // C function pointer
//! ```
//! ```rust
//! let fp: fn(i32, i32) -> i32;      // Rust function pointer
//! ```
//!
//! ### Function Pointer as Parameter
//! ```c
//! void process(int (*callback)(int));
//! ```
//! ```rust
//! fn process(callback: fn(i32) -> i32)
//! ```
//!
//! ### Function Pointer in Typedef
//! ```c
//! typedef int (*Callback)(int);
//! ```
//! ```rust
//! type Callback = fn(i32) -> i32;
//! ```
//!
//! ### Function Pointer in Struct
//! ```c
//! struct Handler {
//!     int (*process)(int);
//! };
//! ```
//! ```rust
//! struct Handler {
//!     process: fn(i32) -> i32,
//! }
//! ```
//!
//! ## Safety Considerations
//!
//! - **Rust fn pointers are SAFE**: Unlike C, Rust fn pointers cannot be null
//! - **Type safety**: Rust enforces function signature matching at compile time
//! - **Option<fn>**: Use Option for nullable function pointers
//! - **Closures**: Rust closures (Fn, FnMut, FnOnce) are more powerful but different from fn
//! - **No casting**: Rust doesn't allow casting between incompatible function types
//!
//! ## Common Patterns
//!
//! 1. **Callbacks**: Function pointers for event handlers
//! 2. **Strategy pattern**: Different algorithms via function pointers
//! 3. **Virtual dispatch**: Function pointer tables (poor man's vtable)
//! 4. **Plugin systems**: Loading functions dynamically (requires unsafe)
//!
//! ## Differences from C
//!
//! - Rust fn pointers are NOT nullable (use Option<fn> instead)
//! - Rust has no implicit function-to-pointer decay (use & for function references)
//! - Rust separates fn pointers from closures (Fn traits)
//! - Rust requires explicit function pointer types (no void*)

#[cfg(test)]
mod function_pointer_documentation_tests {
    /// Document transformation of simple function pointer declaration
    ///
    /// C: `int (*fp)(int, int);`
    /// Rust: `let fp: fn(i32, i32) -> i32;`
    ///
    /// Reference: K&R §5.11, ISO C99 §6.7.5.3
    #[test]
    fn test_simple_function_pointer() {
        let c_code = "int (*fp)(int, int);";
        let rust_equivalent = "let fp: fn(i32, i32) -> i32;";

        // Demonstrate usage
        fn add(a: i32, b: i32) -> i32 {
            a + b
        }

        let fp: fn(i32, i32) -> i32 = add;
        let result = fp(10, 20);
        assert_eq!(result, 30, "Function pointer works");

        assert!(c_code.contains("int (*fp)"));
        assert!(rust_equivalent.contains("fn(i32, i32) -> i32"));
    }

    /// Document function pointer assignment and invocation
    ///
    /// C: `fp = &function; result = (*fp)(arg);`
    /// Rust: `fp = function; result = fp(arg);`
    ///
    /// Reference: K&R §5.11
    #[test]
    fn test_function_pointer_assignment_and_call() {
        fn square(x: i32) -> i32 {
            x * x
        }

        fn cube(x: i32) -> i32 {
            x * x * x
        }

        // C would use: int (*fp)(int) = &square;
        let mut fp: fn(i32) -> i32 = square;
        assert_eq!(fp(5), 25, "Square function");

        // Reassign to different function
        fp = cube;
        assert_eq!(fp(3), 27, "Cube function");

        // Rust doesn't require * to dereference fn pointers
        let result = fp(2); // Not (*fp)(2)
        assert_eq!(result, 8);
    }

    /// Document function pointer as function parameter
    ///
    /// C: `void apply(int (*f)(int), int x) { ... }`
    /// Rust: `fn apply(f: fn(i32) -> i32, x: i32) { ... }`
    ///
    /// Reference: K&R §5.11, ISO C99 §6.9.1
    #[test]
    fn test_function_pointer_as_parameter() {
        let c_code = "void apply(int (*f)(int), int x);";
        let rust_equivalent = "fn apply(f: fn(i32) -> i32, x: i32) -> i32;";

        fn apply(f: fn(i32) -> i32, x: i32) -> i32 {
            f(x)
        }

        fn double(x: i32) -> i32 {
            x * 2
        }

        let result = apply(double, 21);
        assert_eq!(result, 42, "Callback invoked correctly");

        assert!(c_code.contains("int (*f)(int)"));
        assert!(rust_equivalent.contains("fn(i32) -> i32"));
    }

    /// Document array of function pointers
    ///
    /// C: `int (*operations[4])(int, int);`
    /// Rust: `let operations: [fn(i32, i32) -> i32; 4];`
    ///
    /// Reference: K&R §5.11, §5.12
    #[test]
    fn test_function_pointer_array() {
        let c_code = "int (*operations[4])(int, int);";
        let rust_equivalent = "let operations: [fn(i32, i32) -> i32; 4];";

        fn add(a: i32, b: i32) -> i32 {
            a + b
        }
        fn sub(a: i32, b: i32) -> i32 {
            a - b
        }
        fn mul(a: i32, b: i32) -> i32 {
            a * b
        }
        fn div(a: i32, b: i32) -> i32 {
            if b != 0 {
                a / b
            } else {
                0
            }
        }

        let operations: [fn(i32, i32) -> i32; 4] = [add, sub, mul, div];

        assert_eq!(operations[0](10, 5), 15, "Add");
        assert_eq!(operations[1](10, 5), 5, "Subtract");
        assert_eq!(operations[2](10, 5), 50, "Multiply");
        assert_eq!(operations[3](10, 5), 2, "Divide");

        assert!(c_code.contains("(*operations[4])"));
        assert!(rust_equivalent.contains("[fn(i32, i32) -> i32; 4]"));
    }

    /// Document function pointer in struct (callback pattern)
    ///
    /// C: `struct Handler { int (*callback)(int); };`
    /// Rust: `struct Handler { callback: fn(i32) -> i32 }`
    ///
    /// Reference: K&R §6.1, §6.7
    #[test]
    fn test_function_pointer_in_struct() {
        let c_code = r#"
struct Handler {
    int (*callback)(int);
};
"#;
        let rust_equivalent = r#"
struct Handler {
    callback: fn(i32) -> i32,
}
"#;

        struct Handler {
            callback: fn(i32) -> i32,
        }

        fn process(x: i32) -> i32 {
            x + 100
        }

        let handler = Handler { callback: process };
        let result = (handler.callback)(42);
        assert_eq!(result, 142, "Struct callback works");

        assert!(c_code.contains("int (*callback)(int)"));
        assert!(rust_equivalent.contains("callback: fn(i32) -> i32"));
    }

    /// Document typedef for function pointer
    ///
    /// C: `typedef int (*BinaryOp)(int, int);`
    /// Rust: `type BinaryOp = fn(i32, i32) -> i32;`
    ///
    /// Reference: K&R §6.7, ISO C99 §6.7.7
    #[test]
    fn test_function_pointer_typedef() {
        let c_code = "typedef int (*BinaryOp)(int, int);";
        let rust_equivalent = "type BinaryOp = fn(i32, i32) -> i32;";

        type BinaryOp = fn(i32, i32) -> i32;

        fn max(a: i32, b: i32) -> i32 {
            if a > b {
                a
            } else {
                b
            }
        }

        let op: BinaryOp = max;
        assert_eq!(op(10, 20), 20, "Typedef works");

        assert!(c_code.contains("typedef int (*BinaryOp)"));
        assert!(rust_equivalent.contains("type BinaryOp"));
    }

    /// Document NULL function pointer → Option<fn>
    ///
    /// C: `int (*fp)(int) = NULL; if (fp) fp(x);`
    /// Rust: `let fp: Option<fn(i32) -> i32> = None; if let Some(f) = fp { f(x); }`
    ///
    /// Reference: K&R §5.11, ISO C99 §7.17
    #[test]
    fn test_nullable_function_pointer() {
        let c_code = r#"
int (*fp)(int) = NULL;
if (fp != NULL) {
    result = fp(x);
}
"#;
        let rust_equivalent = r#"
let fp: Option<fn(i32) -> i32> = None;
if let Some(f) = fp {
    result = f(x);
}
"#;

        fn increment(x: i32) -> i32 {
            x + 1
        }

        // NULL pointer
        let fp: Option<fn(i32) -> i32> = None;
        assert!(fp.is_none(), "NULL function pointer");

        // Non-NULL pointer
        let fp = Some(increment as fn(i32) -> i32);
        if let Some(f) = fp {
            assert_eq!(f(41), 42, "Function pointer called");
        }

        // Using map for optional call
        let fp = Some(increment as fn(i32) -> i32);
        let result = fp.map(|f| f(99)).unwrap_or(0);
        assert_eq!(result, 100, "Optional function pointer with map");

        assert!(c_code.contains("NULL"));
        assert!(rust_equivalent.contains("Option<fn"));
    }

    /// Document function returning function pointer
    ///
    /// C: `int (*get_operation(char op))(int, int);`
    /// Rust: `fn get_operation(op: char) -> fn(i32, i32) -> i32`
    ///
    /// Reference: K&R §5.12
    #[test]
    fn test_function_returning_function_pointer() {
        let c_code = "int (*get_operation(char op))(int, int);";
        let rust_equivalent = "fn get_operation(op: char) -> fn(i32, i32) -> i32;";

        fn add(a: i32, b: i32) -> i32 {
            a + b
        }
        fn mul(a: i32, b: i32) -> i32 {
            a * b
        }

        fn get_operation(op: char) -> fn(i32, i32) -> i32 {
            match op {
                '+' => add,
                '*' => mul,
                _ => add, // default
            }
        }

        let add_op = get_operation('+');
        assert_eq!(add_op(10, 5), 15, "Addition operation");

        let mul_op = get_operation('*');
        assert_eq!(mul_op(10, 5), 50, "Multiplication operation");

        assert!(c_code.contains("(*get_operation"));
        assert!(rust_equivalent.contains("-> fn(i32, i32) -> i32"));
    }

    /// Document function pointer with no parameters
    ///
    /// C: `void (*fp)(void);`
    /// Rust: `let fp: fn();`
    ///
    /// Reference: ISO C99 §6.7.5.3
    #[test]
    fn test_function_pointer_no_params() {
        let c_code = "void (*fp)(void);";
        let rust_equivalent = "let fp: fn();";

        static mut CALLED: bool = false;

        fn callback() {
            unsafe {
                CALLED = true;
            }
        }

        let fp: fn() = callback;
        fp();

        unsafe {
            assert!(CALLED, "Function called");
        }

        assert!(c_code.contains("void (*fp)(void)"));
        assert!(rust_equivalent.contains("fn()"));
    }

    /// Document function pointer table (virtual dispatch pattern)
    ///
    /// C: Function pointer table for polymorphism
    /// Rust: Same pattern, or use trait objects for true polymorphism
    ///
    /// Reference: Common C pattern for OOP
    #[test]
    fn test_function_pointer_table_vtable() {
        let c_code = r#"
struct Shape {
    int (*area)(struct Shape*);
    int (*perimeter)(struct Shape*);
};
"#;
        let rust_equivalent = r#"
struct Shape {
    area: fn(&Shape) -> i32,
    perimeter: fn(&Shape) -> i32,
}
"#;

        #[derive(Clone)]
        struct Shape {
            width: i32,
            height: i32,
            area: fn(&Shape) -> i32,
            perimeter: fn(&Shape) -> i32,
        }

        fn rectangle_area(s: &Shape) -> i32 {
            s.width * s.height
        }

        fn rectangle_perimeter(s: &Shape) -> i32 {
            2 * (s.width + s.height)
        }

        let rect = Shape {
            width: 10,
            height: 5,
            area: rectangle_area,
            perimeter: rectangle_perimeter,
        };

        assert_eq!((rect.area)(&rect), 50, "Area via function pointer");
        assert_eq!(
            (rect.perimeter)(&rect),
            30,
            "Perimeter via function pointer"
        );

        assert!(c_code.contains("int (*area)"));
        assert!(rust_equivalent.contains("area: fn(&Shape) -> i32"));
    }

    /// Document function pointer vs closure distinction
    ///
    /// C: Only has function pointers
    /// Rust: Has both fn pointers and closures (Fn, FnMut, FnOnce traits)
    ///
    /// Reference: Rust Book ch. 13.1
    #[test]
    fn test_function_pointer_vs_closure() {
        let c_note = "C has only function pointers (no closures with captured environment)";
        let rust_note =
            "Rust has fn pointers (no capture) and closures (Fn/FnMut/FnOnce with capture)";

        // Function pointer (no capture)
        fn add_ten(x: i32) -> i32 {
            x + 10
        }
        let fp: fn(i32) -> i32 = add_ten;
        assert_eq!(fp(5), 15);

        // Closure with capture (NOT a function pointer)
        let offset = 10;
        let closure = |x: i32| x + offset;
        assert_eq!(closure(5), 15);

        // Function pointers can be coerced from non-capturing closures
        let non_capturing_closure = |x: i32| x * 2;
        let fp2: fn(i32) -> i32 = non_capturing_closure;
        assert_eq!(fp2(21), 42);

        // Capturing closures CANNOT be coerced to fn pointers
        // let offset = 5;
        // let capturing = |x: i32| x + offset;
        // let fp3: fn(i32) -> i32 = capturing; // ERROR: cannot coerce

        assert!(c_note.contains("only function pointers"));
        assert!(rust_note.contains("fn pointers"));
    }

    /// Document function pointer type casting safety
    ///
    /// C: Allows dangerous function pointer casts (undefined behavior)
    /// Rust: Prohibits incompatible function pointer casts (compile error)
    ///
    /// Reference: ISO C99 §6.3.2.3 (undefined behavior), Rust type safety
    #[test]
    fn test_function_pointer_cast_safety() {
        let c_code = r#"
// C allows this (undefined behavior!)
int (*fp1)(int) = some_func;
void (*fp2)(void) = (void (*)(void))fp1;  // DANGEROUS
"#;
        let rust_note = "Rust prevents incompatible function pointer casts at compile time";

        fn int_func(x: i32) -> i32 {
            x + 1
        }

        let _fp1: fn(i32) -> i32 = int_func;

        // This would NOT compile in Rust:
        // let fp2: fn() = _fp1 as fn();  // ERROR: cannot cast

        // Rust requires exact signature matching
        let fp_correct: fn(i32) -> i32 = int_func;
        assert_eq!(fp_correct(41), 42);

        assert!(c_code.contains("DANGEROUS"));
        assert!(rust_note.contains("compile time"));
    }

    /// Document comparison and equality of function pointers
    ///
    /// C: Can compare function pointers with == and !=
    /// Rust: Can compare function pointers (derive PartialEq)
    ///
    /// Reference: ISO C99 §6.5.9
    #[test]
    #[allow(ambiguous_wide_pointer_comparisons)]
    #[allow(unpredictable_function_pointer_comparisons)]
    fn test_function_pointer_comparison() {
        let c_code = "if (fp1 == fp2) { ... }";
        let rust_equivalent = "if fp1 == fp2 { ... }";

        fn func_a(x: i32) -> i32 {
            x + 1
        }
        fn func_b(x: i32) -> i32 {
            x + 2
        }

        let fp1: fn(i32) -> i32 = func_a;
        let fp2: fn(i32) -> i32 = func_a;
        let fp3: fn(i32) -> i32 = func_b;

        assert_eq!(fp1, fp2, "Same function pointers are equal");
        assert_ne!(fp1, fp3, "Different function pointers are not equal");

        assert!(c_code.contains("fp1 == fp2"));
        assert!(rust_equivalent.contains("fp1 == fp2"));
    }

    /// Document function pointer with multiple parameters
    ///
    /// C: `int (*fp)(int, float, char*);`
    /// Rust: `let fp: fn(i32, f32, &str) -> i32;`
    ///
    /// Reference: ISO C99 §6.7.5.3
    #[test]
    fn test_function_pointer_multiple_params() {
        let c_code = "int (*fp)(int, float, char*);";
        let rust_equivalent = "let fp: fn(i32, f32, &str) -> i32;";

        fn complex_func(a: i32, b: f32, s: &str) -> i32 {
            a + b as i32 + s.len() as i32
        }

        let fp: fn(i32, f32, &str) -> i32 = complex_func;
        let result = fp(10, 3.0, "hello");
        assert_eq!(result, 18, "Multiple parameters work"); // 10 + 3 + 5

        assert!(c_code.contains("int, float, char*"));
        assert!(rust_equivalent.contains("i32, f32, &str"));
    }

    /// Document higher-order functions (functions taking function pointers)
    ///
    /// C: map, filter, reduce patterns with function pointers
    /// Rust: Same patterns, idiomatic with iterators
    ///
    /// Reference: Common functional programming pattern
    #[test]
    fn test_higher_order_functions() {
        let c_code = r#"
void map(int* arr, int len, int (*f)(int)) {
    for (int i = 0; i < len; i++) {
        arr[i] = f(arr[i]);
    }
}
"#;
        let rust_equivalent = "fn map(arr: &mut [i32], f: fn(i32) -> i32)";

        fn map(arr: &mut [i32], f: fn(i32) -> i32) {
            for item in arr.iter_mut() {
                *item = f(*item);
            }
        }

        fn double(x: i32) -> i32 {
            x * 2
        }

        let mut data = [1, 2, 3, 4, 5];
        map(&mut data, double);
        assert_eq!(data, [2, 4, 6, 8, 10], "Map function works");

        // More idiomatic Rust would use iterators:
        let data2: Vec<i32> = [1, 2, 3, 4, 5].iter().map(|&x| double(x)).collect();
        assert_eq!(data2, vec![2, 4, 6, 8, 10]);

        assert!(c_code.contains("int (*f)(int)"));
        assert!(rust_equivalent.contains("fn(i32) -> i32"));
    }

    /// Document function pointer initialization in struct literals
    ///
    /// C: `struct Handler h = { .callback = my_func };`
    /// Rust: `let h = Handler { callback: my_func };`
    ///
    /// Reference: ISO C99 §6.7.8 (designated initializers)
    #[test]
    fn test_function_pointer_struct_initialization() {
        let c_code = "struct Handler h = { .callback = process };";
        let rust_equivalent = "let h = Handler { callback: process };";

        struct Handler {
            callback: fn(i32) -> i32,
        }

        fn process(x: i32) -> i32 {
            x * 3
        }

        let h = Handler { callback: process };
        assert_eq!((h.callback)(7), 21, "Struct initialization works");

        assert!(c_code.contains(".callback = process"));
        assert!(rust_equivalent.contains("callback: process"));
    }

    /// Document transformation rules summary and edge cases
    ///
    /// This test documents all the transformation rules and important differences
    /// between C function pointers and Rust fn types.
    #[test]
    #[allow(ambiguous_wide_pointer_comparisons)]
    #[allow(unpredictable_function_pointer_comparisons)]
    fn test_function_pointer_transformation_rules_summary() {
        let c_summary = r#"
C Function Pointer Rules:
1. Syntax: ReturnType (*name)(ParamTypes)
2. Can be NULL (requires null check)
3. Implicit function-to-pointer decay
4. Can cast between function pointer types (unsafe)
5. No distinction between function pointers and closures
"#;

        let rust_summary = r#"
Rust Function Pointer Rules:
1. Syntax: fn(ParamTypes) -> ReturnType
2. NOT nullable (use Option<fn> for nullable)
3. No implicit decay (use & for function references)
4. Type-safe (no unsafe casts allowed)
5. Separate types: fn pointers vs Fn/FnMut/FnOnce closures
6. Function pointers implement Copy, PartialEq, Eq
"#;

        // Rule 1: Syntax
        let _fp1: fn(i32) -> i32;
        // C would be: int (*_fp1)(int);

        // Rule 2: Non-nullable (use Option for nullable)
        let fp2: Option<fn(i32) -> i32> = None;
        assert!(fp2.is_none());

        // Rule 6: Function pointers are Copy
        fn sample(x: i32) -> i32 {
            x
        }
        let fp3: fn(i32) -> i32 = sample;
        let fp4 = fp3; // Copy, not move
        let _result1 = fp3(1); // Still usable after "copy"
        let _result2 = fp4(2);

        // Rule 6: Function pointers are PartialEq
        assert_eq!(fp3, fp4);

        assert!(c_summary.contains("Can be NULL"));
        assert!(rust_summary.contains("NOT nullable"));
        assert!(rust_summary.contains("Type-safe"));
    }
}
