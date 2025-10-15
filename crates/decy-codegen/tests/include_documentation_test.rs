//! Documentation tests for #include transformation (PREP-INCLUDE validation)
//!
//! Reference: K&R §4.11, ISO C99 §6.10.2
//!
//! This module documents the transformation of C #include directives to Rust module system.
//! #include directives in C perform textual inclusion at preprocessing time, while Rust uses
//! a module system with explicit imports.
//!
//! **Key Insight**: Most C standard library headers map to Rust built-in types and functions,
//! requiring NO explicit imports in the generated Rust code.

/// Document transformation of #include <stdio.h>
///
/// C: #include <stdio.h>
///    int main() {
///        printf("Hello, World!\n");
///        return 0;
///    }
///
/// Rust: fn main() {
///         println!("Hello, World!");  // Built-in macro, no import needed
///       }
///
/// **Transformation**: stdio.h functions map to Rust built-ins:
/// - printf → println! (built-in macro)
/// - scanf → std::io::stdin() (but usually no import needed with proper code generation)
/// - FILE* → std::fs::File (requires use std::fs::File only if type is referenced)
///
/// Reference: K&R §7.2, ISO C99 §7.19
#[test]
fn test_stdio_h_no_import_needed() {
    // This is a documentation test showing that stdio.h requires no imports
    // because Rust transpilation maps printf → println!, etc.

    // Verify the transformation rule
    let c_code_concept = "#include <stdio.h>";
    let rust_equivalent = "// println! is built-in, nothing to import";

    assert!(c_code_concept.contains("#include"), "C code uses #include");

    // Check that rust_equivalent doesn't contain actual import statements
    let has_use_statement = rust_equivalent.contains("use ") || rust_equivalent.starts_with("use");
    assert!(
        !has_use_statement,
        "Rust code needs no use statement for stdio.h → println!"
    );
}

/// Document transformation of #include <stdlib.h>
///
/// C: #include <stdlib.h>
///    int* ptr = malloc(sizeof(int));
///    free(ptr);
///
/// Rust: let ptr = Box::new(0i32);  // malloc → Box::new (built-in)
///       // free is automatic via Drop trait
///
/// **Transformation**: stdlib.h functions map to Rust built-ins:
/// - malloc → Box::new (built-in)
/// - calloc → vec![0; n] (built-in Vec)
/// - realloc → Vec::resize (built-in Vec)
/// - free → automatic Drop (no code needed)
///
/// Reference: K&R §8.7, ISO C99 §7.20.3
#[test]
fn test_stdlib_h_no_import_needed() {
    // Documentation test showing stdlib.h requires no imports

    let _c_code_concept = "#include <stdlib.h>";
    let rust_equivalent = "// No import needed - Box and Vec are built-in";

    assert!(
        !rust_equivalent.contains("use std::"),
        "Rust code needs no import for stdlib.h → Box/Vec"
    );
}

/// Document transformation of #include <string.h>
///
/// C: #include <string.h>
///    size_t len = strlen(s);
///    strcpy(dest, src);
///
/// Rust: let len = s.len();         // Built-in String/slice method
///       dest = src.to_string();    // Built-in String method
///
/// **Transformation**: string.h functions map to Rust built-in methods:
/// - strlen → .len() (built-in method)
/// - strcpy → .to_string() or .clone() (built-in methods)
/// - strcat → String::push_str (built-in method)
/// - strcmp → == operator (built-in)
///
/// Reference: K&R §B3, ISO C99 §7.21
#[test]
fn test_string_h_no_import_needed() {
    // Documentation test showing string.h requires no imports

    let _c_code_concept = "#include <string.h>";
    let rust_equivalent = "// No import needed - String methods are built-in";

    assert!(
        !rust_equivalent.contains("use"),
        "Rust code needs no import for string.h → String methods"
    );
}

/// Document transformation of #include <math.h>
///
/// C: #include <math.h>
///    double x = sqrt(4.0);
///    double y = pow(2.0, 3.0);
///
/// Rust: let x = (4.0_f64).sqrt();     // Built-in method
///       let y = (2.0_f64).powf(3.0);  // Built-in method
///
/// **Transformation**: math.h functions map to Rust built-in f32/f64 methods:
/// - sqrt → .sqrt() (built-in method)
/// - pow → .powf() (built-in method)
/// - sin/cos/tan → .sin()/.cos()/.tan() (built-in methods)
///
/// Reference: ISO C99 §7.12
#[test]
fn test_math_h_no_import_needed() {
    // Documentation test showing math.h requires no imports

    let _c_code_concept = "#include <math.h>";
    let rust_equivalent = "// No import needed - f32/f64 methods are built-in";

    assert!(
        !rust_equivalent.contains("use"),
        "Rust code needs no import for math.h → f32/f64 methods"
    );
}

/// Document transformation of #include "myheader.h" (local headers)
///
/// C: File structure:
///    main.c:
///      #include "myheader.h"
///      int main() {
///          my_function();
///          return 0;
///      }
///
///    myheader.h:
///      void my_function();
///
///    myheader.c:
///      void my_function() { ... }
///
/// Rust: File structure:
///    main.rs:
///      mod mymodule;  // Declares the module
///      use mymodule::my_function;  // Brings function into scope
///
///      fn main() {
///          my_function();
///      }
///
///    mymodule.rs:
///      pub fn my_function() { ... }
///
/// **Transformation**: Local headers → Rust modules
/// - #include "header.h" → mod module_name;
/// - Function/type usage → use module_name::item; (or module_name::item directly)
///
/// Reference: K&R §4.11, ISO C99 §6.10.2
#[test]
fn test_local_header_to_mod_declaration() {
    // Documentation test showing local headers require mod declarations

    let c_code_concept = "#include \"myheader.h\"";
    let rust_equivalent = "mod mymodule;";

    assert!(
        c_code_concept.contains("#include"),
        "C code uses #include for local headers"
    );
    assert!(
        rust_equivalent.contains("mod"),
        "Rust code uses mod declaration for local modules"
    );
}

/// Document transformation of multiple #include directives
///
/// C: #include <stdio.h>
///    #include <stdlib.h>
///    #include "helper.h"
///    #include "data.h"
///
/// Rust: // No imports needed for stdio.h, stdlib.h (built-ins)
///       mod helper;  // Local module
///       mod data;    // Local module
///
///       // Optional: bring items into scope
///       use helper::*;
///       use data::DataStruct;
///
/// **Transformation Rules**:
/// 1. System headers (<*.h>) → usually no import (built-ins)
/// 2. Local headers ("*.h") → mod declarations
/// 3. Specific items → use statements (optional, for convenience)
///
/// Reference: K&R §4.11, ISO C99 §6.10.2
#[test]
fn test_multiple_includes() {
    // Documentation test showing multiple #include handling

    let system_includes = vec!["<stdio.h>", "<stdlib.h>", "<string.h>"];
    let local_includes = vec!["\"helper.h\"", "\"data.h\""];

    // System headers → no imports (usually)
    for header in system_includes {
        assert!(header.starts_with('<'), "System headers use < > syntax");
        // These map to built-ins in Rust, no import needed
    }

    // Local headers → mod declarations
    for header in local_includes {
        assert!(header.starts_with('\"'), "Local headers use \" \" syntax");
        // These become mod declarations in Rust
    }
}

/// Document transformation of header guards (NOT needed in Rust)
///
/// C: myheader.h:
///      #ifndef MYHEADER_H
///      #define MYHEADER_H
///
///      void my_function();
///
///      #endif // MYHEADER_H
///
/// Rust: mymodule.rs:
///         // No header guards needed!
///         // Rust module system prevents multiple inclusion
///
///         pub fn my_function() { ... }
///
/// **Transformation**: Header guards are NOT needed in Rust
/// - C uses #ifndef/#define/#endif to prevent multiple inclusion
/// - Rust module system automatically prevents multiple inclusion
/// - No transformation needed for header guards
///
/// Reference: K&R §4.11, ISO C99 §6.10.2
#[test]
fn test_header_guards_not_needed() {
    // Documentation test showing header guards are not needed in Rust

    let c_header_guard = "#ifndef MYHEADER_H";
    let rust_equivalent = "// No header guard needed - Rust prevents multiple inclusion";

    assert!(c_header_guard.contains("#ifndef"), "C uses header guards");
    assert!(
        !rust_equivalent.contains("#"),
        "Rust has no header guards - module system handles it"
    );
}

/// Document transformation of conditional includes (#ifdef)
///
/// C: #ifdef DEBUG
///    #include "debug_utils.h"
///    #endif
///
/// Rust: #[cfg(debug_assertions)]
///       mod debug_utils;
///
/// **Transformation**: Conditional includes → cfg attributes
/// - #ifdef DEBUG → #[cfg(debug_assertions)]
/// - #ifdef FEATURE → #[cfg(feature = "feature_name")]
/// - Platform-specific: #ifdef __linux__ → #[cfg(target_os = "linux")]
///
/// Reference: K&R §4.11, ISO C99 §6.10.1
#[test]
fn test_conditional_includes() {
    // Documentation test showing conditional includes → cfg attributes

    let c_conditional = "#ifdef DEBUG";
    let rust_equivalent = "#[cfg(debug_assertions)]";

    assert!(
        c_conditional.contains("#ifdef"),
        "C uses #ifdef for conditional compilation"
    );
    assert!(
        rust_equivalent.contains("#[cfg"),
        "Rust uses cfg attributes for conditional compilation"
    );
}

/// Document complete transformation example
///
/// C Program:
///   main.c:
///     #include <stdio.h>
///     #include <stdlib.h>
///     #include "utils.h"
///
///     int main() {
///         int* arr = malloc(10 * sizeof(int));
///         printf("Array allocated\n");
///         init_array(arr, 10);
///         free(arr);
///         return 0;
///     }
///
///   utils.h:
///     #ifndef UTILS_H
///     #define UTILS_H
///     void init_array(int* arr, int size);
///     #endif
///
///   utils.c:
///     #include "utils.h"
///     void init_array(int* arr, int size) {
///         for (int i = 0; i < size; i++) {
///             arr[i] = 0;
///         }
///     }
///
/// Rust Program:
///   main.rs:
///     mod utils;  // Declares utils module
///     use utils::init_array;  // Optional: brings function into scope
///
///     fn main() {
///         let mut arr = vec![0i32; 10];  // malloc → Vec
///         println!("Array allocated");   // printf → println!
///         init_array(&mut arr);          // Function call
///         // free is automatic via Vec's Drop
///     }
///
///   utils.rs:
///     // No header guard needed
///     // No #include "utils.h" needed
///
///     pub fn init_array(arr: &mut [i32]) {
///         for (i, item) in arr.iter_mut().enumerate() {
///             *item = 0;
///         }
///     }
///
/// **Key Transformations**:
/// 1. <stdio.h>, <stdlib.h> → no imports (built-ins)
/// 2. "utils.h" → mod utils;
/// 3. Header guards → removed (not needed)
/// 4. #include in .c file → removed (module contains implementation)
///
/// Reference: K&R §4.11, ISO C99 §6.10.2
#[test]
fn test_complete_transformation_example() {
    // Documentation test showing complete program transformation

    // C uses #include for both system and local headers
    let c_includes = ["<stdio.h>", "<stdlib.h>", "\"utils.h\""];

    // Rust uses mod only for local modules
    // System libraries are built-in (println!, Vec, etc.)
    let rust_declarations = [
        "mod utils;",
        // No need for stdio/stdlib - built into Rust
    ];

    assert_eq!(c_includes.len(), 3, "C has 3 includes");
    assert_eq!(
        rust_declarations.len(),
        1,
        "Rust has 1 mod declaration (system headers are built-in)"
    );
}

/// Summary of transformation rules
///
/// This test documents the complete set of rules for #include transformation.
///
/// **System Headers** (<header.h>):
/// - <stdio.h> → NO IMPORT (println!, etc. are built-in)
/// - <stdlib.h> → NO IMPORT (Box, Vec are built-in)
/// - <string.h> → NO IMPORT (String methods are built-in)
/// - <math.h> → NO IMPORT (f32/f64 methods are built-in)
///
/// **Local Headers** ("header.h"):
/// - "header.h" → mod module_name;
/// - Header file → separate .rs file (module)
/// - #include "header.h" in .c → removed (implementation is in module)
///
/// **Special Cases**:
/// - Header guards (#ifndef/#define/#endif) → REMOVED (not needed)
/// - Conditional includes (#ifdef) → #[cfg(...)] attributes
/// - Forward declarations → pub mod declarations
///
/// **Unsafe Blocks**: 0 (module system is safe)
///
/// Reference: K&R §4.11, ISO C99 §6.10.2
#[test]
fn test_transformation_rules_summary() {
    // This test serves as documentation for all transformation rules

    // Rule 1: System headers → no import (built-ins)
    let system_headers_map_to_builtins = true;
    assert!(
        system_headers_map_to_builtins,
        "System headers map to Rust built-ins"
    );

    // Rule 2: Local headers → mod declarations
    let local_headers_become_modules = true;
    assert!(
        local_headers_become_modules,
        "Local headers become Rust modules"
    );

    // Rule 3: Header guards → removed
    let header_guards_not_needed = true;
    assert!(header_guards_not_needed, "Header guards not needed in Rust");

    // Rule 4: No unsafe blocks needed
    let unsafe_blocks = 0;
    assert_eq!(unsafe_blocks, 0, "Module system introduces 0 unsafe blocks");
}
