//! Function call, dereference, and unary expression generation.

use super::{CodeGenerator, TypeContext};
use decy_hir::{BinaryOperator, HirExpression, HirType};

impl CodeGenerator {
    pub(crate) fn gen_expr_dereference(&self, inner: &HirExpression, ctx: &TypeContext) -> String {
        if let HirExpression::Variable(var_name) = inner {
            if let Some(idx_var) = ctx.get_string_iter_index(var_name) {
                return format!("{}[{}]", var_name, idx_var);
            }

            if let Some(var_type) = ctx.get_type(var_name) {
                if matches!(var_type, HirType::StringReference | HirType::StringLiteral) {
                    return format!("{}.as_bytes()[0] as i32", var_name);
                }
            }
        }

        if let HirExpression::PostIncrement { operand } = inner {
            if let HirExpression::Variable(var_name) = &**operand {
                if let Some(var_type) = ctx.get_type(var_name) {
                    if matches!(var_type, HirType::StringReference | HirType::StringLiteral)
                    {
                        return self.generate_expression_with_context(inner, ctx);
                    }
                }
            }
        }

        let inner_code = self.generate_expression_with_context(inner, ctx);

        let needs_unsafe = match inner {
            HirExpression::Variable(var_name) => ctx.is_pointer(var_name),
            HirExpression::BinaryOp { left, .. } => {
                if let HirExpression::Variable(var_name) = &**left {
                    ctx.is_pointer(var_name)
                } else {
                    false
                }
            }
            _ => false,
        };

        if needs_unsafe {
            return Self::unsafe_block(
                &format!("*{}", inner_code),
                "pointer is valid and properly aligned from caller contract",
            );
        }

        format!("*{}", inner_code)
    }

    pub(crate) fn gen_expr_unary_op(
        &self,
        op: &decy_hir::UnaryOperator,
        operand: &HirExpression,
        ctx: &TypeContext,
    ) -> String {
        use decy_hir::UnaryOperator;
        match op {
            UnaryOperator::PostIncrement => {
                let operand_code = self.generate_expression_with_context(operand, ctx);
                let operand_type = ctx.infer_expression_type(operand);
                if matches!(operand_type, Some(HirType::Pointer(_))) {
                    format!(
                        "{{ let __tmp = {}; {} = {}.wrapping_add(1); __tmp }}",
                        operand_code, operand_code, operand_code
                    )
                } else {
                    format!(
                        "{{ let __tmp = {}; {} += 1; __tmp }}",
                        operand_code, operand_code
                    )
                }
            }
            UnaryOperator::PostDecrement => {
                let operand_code = self.generate_expression_with_context(operand, ctx);
                let operand_type = ctx.infer_expression_type(operand);
                if matches!(operand_type, Some(HirType::Pointer(_))) {
                    format!(
                        "{{ let __tmp = {}; {} = {}.wrapping_sub(1); __tmp }}",
                        operand_code, operand_code, operand_code
                    )
                } else {
                    format!(
                        "{{ let __tmp = {}; {} -= 1; __tmp }}",
                        operand_code, operand_code
                    )
                }
            }
            UnaryOperator::PreIncrement => {
                let operand_code = self.generate_expression_with_context(operand, ctx);
                let operand_type = ctx.infer_expression_type(operand);
                if matches!(operand_type, Some(HirType::Pointer(_))) {
                    format!(
                        "{{ {} = {}.wrapping_add(1); {} }}",
                        operand_code, operand_code, operand_code
                    )
                } else {
                    format!("{{ {} += 1; {} }}", operand_code, operand_code)
                }
            }
            UnaryOperator::PreDecrement => {
                let operand_code = self.generate_expression_with_context(operand, ctx);
                let operand_type = ctx.infer_expression_type(operand);
                if matches!(operand_type, Some(HirType::Pointer(_))) {
                    format!(
                        "{{ {} = {}.wrapping_sub(1); {} }}",
                        operand_code, operand_code, operand_code
                    )
                } else {
                    format!("{{ {} -= 1; {} }}", operand_code, operand_code)
                }
            }
            UnaryOperator::LogicalNot => {
                let operand_code = self.generate_expression_with_context(operand, ctx);
                if Self::is_boolean_expression(operand) {
                    format!("!{}", operand_code)
                } else {
                    format!("({} == 0) as i32", operand_code)
                }
            }
            _ => {
                let op_str = Self::unary_operator_to_string(op);
                let operand_code = self.generate_expression_with_context(operand, ctx);
                format!("{}{}", op_str, operand_code)
            }
        }
    }

    pub(crate) fn gen_expr_function_call(
        &self,
        function: &str,
        arguments: &[HirExpression],
        ctx: &TypeContext,
        target_type: Option<&HirType>,
    ) -> String {
        match function {
            "strlen" => self.gen_call_strlen(function, arguments, ctx),
            "strcpy" => self.gen_call_strcpy(function, arguments, ctx),
            "malloc" => self.gen_call_malloc(arguments, ctx, target_type),
            "calloc" => self.gen_call_calloc(arguments, ctx, target_type),
            "realloc" => self.gen_call_realloc(arguments, ctx, target_type),
            "free" => self.gen_call_free(arguments, ctx),
            "fopen" => self.gen_call_fopen(arguments, ctx),
            "fclose" => self.gen_call_fclose(arguments, ctx),
            "fgetc" | "getc" => self.gen_call_fgetc(arguments, ctx),
            "fputc" | "putc" => self.gen_call_fputc(arguments, ctx),
            "fprintf" => self.gen_call_fprintf(arguments, ctx),
            "printf" => self.gen_call_printf(arguments, ctx),
            "fread" => self.gen_call_fread(arguments, ctx),
            "fwrite" => self.gen_call_fwrite(arguments, ctx),
            "fputs" => self.gen_call_fputs(arguments, ctx),
            "fork" => "/* fork() transformed to Command API */ 0".to_string(),
            "execl" | "execlp" | "execle" | "execv" | "execvp" | "execve" => {
                self.gen_call_exec(arguments, ctx)
            }
            "waitpid" | "wait3" | "wait4" => {
                "/* waitpid handled by Command API */ child.wait().expect(\"wait failed\")"
                    .to_string()
            }
            "wait" => "child.wait().expect(\"wait failed\")".to_string(),
            "WEXITSTATUS" => self.gen_call_status_macro(arguments, ctx, |s| format!("{}.code().unwrap_or(-1)", s)),
            "WIFEXITED" => self.gen_call_status_macro(arguments, ctx, |s| format!("{}.success()", s)),
            "WIFSIGNALED" => self.gen_call_status_macro(arguments, ctx, |s| format!("{}.signal().is_some()", s)),
            "WTERMSIG" => self.gen_call_status_macro(arguments, ctx, |s| format!("{}.signal().unwrap_or(0)", s)),
            "atoi" => self.gen_call_parse(arguments, ctx, "i32", "0"),
            "atof" => self.gen_call_parse(arguments, ctx, "f64", "0.0"),
            "abs" => {
                if arguments.len() == 1 {
                    let x = self.generate_expression_with_context(&arguments[0], ctx);
                    format!("({}).abs()", x)
                } else {
                    "0 /* abs requires 1 arg */".to_string()
                }
            }
            "exit" => {
                if arguments.len() == 1 {
                    let code = self.generate_expression_with_context(&arguments[0], ctx);
                    format!("std::process::exit({})", code)
                } else {
                    "std::process::exit(1)".to_string()
                }
            }
            "puts" => {
                if arguments.len() == 1 {
                    let s = self.generate_expression_with_context(&arguments[0], ctx);
                    format!("println!(\"{{}}\", {})", s)
                } else {
                    "println!()".to_string()
                }
            }
            "snprintf" => self.gen_call_snprintf(arguments, ctx),
            "sprintf" => self.gen_call_sprintf(arguments, ctx),
            "qsort" => {
                if arguments.len() == 4 {
                    let base = self.generate_expression_with_context(&arguments[0], ctx);
                    let n = self.generate_expression_with_context(&arguments[1], ctx);
                    let cmp = self.generate_expression_with_context(&arguments[3], ctx);
                    format!("{}[..{} as usize].sort_by(|a, b| {}(a, b))", base, n, cmp)
                } else {
                    "/* qsort requires 4 args */".to_string()
                }
            }
            _ => self.gen_call_default(function, arguments, ctx),
        }
    }

    pub(crate) fn gen_call_strlen(
        &self,
        function: &str,
        arguments: &[HirExpression],
        ctx: &TypeContext,
    ) -> String {
        if arguments.len() == 1 {
            format!(
                "{}.len() as i32",
                self.generate_expression_with_context(&arguments[0], ctx)
            )
        } else {
            let args: Vec<String> = arguments
                .iter()
                .map(|arg| self.generate_expression_with_context(arg, ctx))
                .collect();
            format!("{}({})", function, args.join(", "))
        }
    }

    pub(crate) fn gen_call_strcpy(
        &self,
        function: &str,
        arguments: &[HirExpression],
        ctx: &TypeContext,
    ) -> String {
        if arguments.len() == 2 {
            let src_code =
                self.generate_expression_with_context(&arguments[1], ctx);
            let is_raw_pointer = src_code.contains("(*")
                || src_code.contains(").")
                || src_code.contains("as *");
            if is_raw_pointer {
                format!(
                    "unsafe {{ std::ffi::CStr::from_ptr({} as *const i8).to_str().unwrap_or(\"\").to_string() }}",
                    src_code
                )
            } else {
                format!("{}.to_string()", src_code)
            }
        } else {
            let args: Vec<String> = arguments
                .iter()
                .map(|arg| self.generate_expression_with_context(arg, ctx))
                .collect();
            format!("{}({})", function, args.join(", "))
        }
    }

    pub(crate) fn gen_call_malloc(
        &self,
        arguments: &[HirExpression],
        ctx: &TypeContext,
        target_type: Option<&HirType>,
    ) -> String {
        if arguments.len() == 1 {
            let size_code =
                self.generate_expression_with_context(&arguments[0], ctx);

            if let Some(HirType::Vec(elem_type)) = target_type {
                let elem_type_str = Self::map_type(elem_type);
                let default_val = Self::default_value_for_type(elem_type);
                if let HirExpression::BinaryOp {
                    op: BinaryOperator::Multiply,
                    left,
                    ..
                } = &arguments[0]
                {
                    let count_code =
                        self.generate_expression_with_context(left, ctx);
                    return format!(
                        "vec![{}; ({}) as usize]",
                        default_val, count_code
                    );
                } else {
                    return format!(
                        "Vec::<{}>::with_capacity(({}) as usize)",
                        elem_type_str, size_code
                    );
                }
            }

            if let Some(HirType::Pointer(inner)) = target_type {
                if matches!(inner.as_ref(), HirType::Char) {
                    return format!(
                        "Box::leak(vec![0u8; ({}) as usize].into_boxed_slice()).as_mut_ptr()",
                        size_code
                    );
                }
                if let HirType::Struct(struct_name) = inner.as_ref() {
                    return format!(
                        "Box::into_raw(Box::<{}>::default())",
                        struct_name
                    );
                }
                let elem_type_str = Self::map_type(inner);
                let default_val = Self::default_value_for_type(inner);
                if let HirExpression::BinaryOp {
                    op: BinaryOperator::Multiply,
                    left,
                    ..
                } = &arguments[0]
                {
                    let count_code =
                        self.generate_expression_with_context(left, ctx);
                    return format!(
                        "Box::leak(vec![{}; ({}) as usize].into_boxed_slice()).as_mut_ptr() as *mut {}",
                        default_val, count_code, elem_type_str
                    );
                } else {
                    return format!(
                        "Box::leak(vec![{}; ({}) as usize].into_boxed_slice()).as_mut_ptr() as *mut {}",
                        default_val, size_code, elem_type_str
                    );
                }
            }

            if let HirExpression::BinaryOp {
                op: BinaryOperator::Multiply,
                left,
                ..
            } = &arguments[0]
            {
                let count_code = self.generate_expression_with_context(left, ctx);
                format!("vec![0i32; ({}) as usize]", count_code)
            } else {
                format!("Vec::<u8>::with_capacity(({}) as usize)", size_code)
            }
        } else {
            "Vec::new()".to_string()
        }
    }

    pub(crate) fn gen_call_calloc(
        &self,
        arguments: &[HirExpression],
        ctx: &TypeContext,
        target_type: Option<&HirType>,
    ) -> String {
        if arguments.len() == 2 {
            let count_code =
                self.generate_expression_with_context(&arguments[0], ctx);

            if let Some(HirType::Vec(elem_type)) = target_type {
                let default_val = Self::default_value_for_type(elem_type);
                return format!("vec![{}; {} as usize]", default_val, count_code);
            }

            if let Some(HirType::Pointer(inner)) = target_type {
                let elem_type_str = Self::map_type(inner);
                let default_val = Self::default_value_for_type(inner);
                return format!(
                    "Box::leak(vec![{}; {} as usize].into_boxed_slice()).as_mut_ptr() as *mut {}",
                    default_val, count_code, elem_type_str
                );
            }

            format!("vec![0i32; {} as usize]", count_code)
        } else {
            "Vec::new()".to_string()
        }
    }

    pub(crate) fn gen_call_realloc(
        &self,
        arguments: &[HirExpression],
        ctx: &TypeContext,
        target_type: Option<&HirType>,
    ) -> String {
        if arguments.len() == 2 {
            let ptr_code =
                self.generate_expression_with_context(&arguments[0], ctx);
            let size_code =
                self.generate_expression_with_context(&arguments[1], ctx);
            let realloc_call =
                format!("realloc({} as *mut (), {})", ptr_code, size_code);

            if let Some(HirType::Pointer(inner)) = target_type {
                let target_ptr_type =
                    Self::map_type(&HirType::Pointer(inner.clone()));
                format!("{} as {}", realloc_call, target_ptr_type)
            } else {
                realloc_call
            }
        } else {
            "std::ptr::null_mut()".to_string()
        }
    }

    pub(crate) fn gen_call_free(&self, arguments: &[HirExpression], ctx: &TypeContext) -> String {
        if arguments.len() == 1 {
            let ptr_code =
                self.generate_expression_with_context(&arguments[0], ctx);
            format!("drop({})", ptr_code)
        } else {
            "/* free() */".to_string()
        }
    }

    pub(crate) fn gen_call_fopen(&self, arguments: &[HirExpression], ctx: &TypeContext) -> String {
        if arguments.len() == 2 {
            let filename =
                self.generate_expression_with_context(&arguments[0], ctx);
            let mode = self.generate_expression_with_context(&arguments[1], ctx);
            if mode.contains('w') || mode.contains('a') {
                format!("std::fs::File::create({}).ok()", filename)
            } else {
                format!("std::fs::File::open({}).ok()", filename)
            }
        } else {
            "None /* fopen requires 2 args */".to_string()
        }
    }

    pub(crate) fn gen_call_fclose(&self, arguments: &[HirExpression], ctx: &TypeContext) -> String {
        if arguments.len() == 1 {
            let file_code =
                self.generate_expression_with_context(&arguments[0], ctx);
            format!("drop({})", file_code)
        } else {
            "/* fclose() */".to_string()
        }
    }

    pub(crate) fn gen_call_fgetc(&self, arguments: &[HirExpression], ctx: &TypeContext) -> String {
        if arguments.len() == 1 {
            let file_code =
                self.generate_expression_with_context(&arguments[0], ctx);
            format!(
                "{{ use std::io::Read; let mut buf = [0u8; 1]; {}.read(&mut buf).map(|_| buf[0] as i32).unwrap_or(-1) }}",
                file_code
            )
        } else {
            "-1 /* fgetc requires 1 arg */".to_string()
        }
    }

    pub(crate) fn gen_call_fputc(&self, arguments: &[HirExpression], ctx: &TypeContext) -> String {
        if arguments.len() == 2 {
            let char_code =
                self.generate_expression_with_context(&arguments[0], ctx);
            let file_code =
                self.generate_expression_with_context(&arguments[1], ctx);
            format!(
                "{{ use std::io::Write; {}.write(&[{} as u8]).map(|_| {} as i32).unwrap_or(-1) }}",
                file_code, char_code, char_code
            )
        } else {
            "-1 /* fputc requires 2 args */".to_string()
        }
    }

    pub(crate) fn gen_call_fprintf(&self, arguments: &[HirExpression], ctx: &TypeContext) -> String {
        if arguments.len() >= 2 {
            let file_code =
                self.generate_expression_with_context(&arguments[0], ctx);
            let fmt = self.generate_expression_with_context(&arguments[1], ctx);
            let rust_fmt = Self::convert_c_format_to_rust(&fmt);
            if arguments.len() == 2 {
                format!(
                    "{{ use std::io::Write; write!({}, {}).map(|_| 0).unwrap_or(-1) }}",
                    file_code, rust_fmt
                )
            } else {
                let s_positions = Self::find_string_format_positions(&fmt);
                let args: Vec<String> = arguments[2..]
                    .iter()
                    .enumerate()
                    .map(|(i, a)| {
                        let arg_code = self.generate_expression_with_context(a, ctx);
                        if s_positions.contains(&i) {
                            format!("unsafe {{ std::ffi::CStr::from_ptr({} as *const i8).to_str().unwrap_or(\"\") }}", arg_code)
                        } else {
                            arg_code
                        }
                    })
                    .collect();
                format!(
                    "{{ use std::io::Write; write!({}, {}, {}).map(|_| 0).unwrap_or(-1) }}",
                    file_code, rust_fmt, args.join(", ")
                )
            }
        } else {
            "-1 /* fprintf requires 2+ args */".to_string()
        }
    }

    pub(crate) fn gen_call_printf(&self, arguments: &[HirExpression], ctx: &TypeContext) -> String {
        if !arguments.is_empty() {
            let fmt = self.generate_expression_with_context(&arguments[0], ctx);
            let rust_fmt = Self::convert_c_format_to_rust(&fmt);
            if arguments.len() == 1 {
                format!("print!({})", rust_fmt)
            } else {
                let s_positions = Self::find_string_format_positions(&fmt);
                let args: Vec<String> = arguments[1..]
                    .iter()
                    .enumerate()
                    .map(|(i, a)| {
                        let arg_code =
                            self.generate_expression_with_context(a, ctx);
                        if s_positions.contains(&i) && !Self::is_string_ternary(a) {
                            let arg_type = ctx.infer_expression_type(a);
                            let is_raw_pointer =
                                matches!(arg_type, Some(HirType::Pointer(_)));
                            let is_function_call =
                                matches!(a, HirExpression::FunctionCall { .. });
                            if is_raw_pointer || is_function_call {
                                Self::wrap_raw_ptr_with_cstr(&arg_code)
                            } else {
                                Self::wrap_with_cstr(&arg_code)
                            }
                        } else {
                            arg_code
                        }
                    })
                    .collect();
                format!("print!({}, {})", rust_fmt, args.join(", "))
            }
        } else {
            "print!(\"\")".to_string()
        }
    }

    pub(crate) fn gen_call_fread(&self, arguments: &[HirExpression], ctx: &TypeContext) -> String {
        if arguments.len() == 4 {
            let buf_code =
                self.generate_expression_with_context(&arguments[0], ctx);
            let file_code =
                self.generate_expression_with_context(&arguments[3], ctx);
            format!(
                "{{ use std::io::Read; {}.read(&mut {}).unwrap_or(0) }}",
                file_code, buf_code
            )
        } else {
            "0 /* fread requires 4 args */".to_string()
        }
    }

    pub(crate) fn gen_call_fwrite(&self, arguments: &[HirExpression], ctx: &TypeContext) -> String {
        if arguments.len() == 4 {
            let data_code =
                self.generate_expression_with_context(&arguments[0], ctx);
            let file_code =
                self.generate_expression_with_context(&arguments[3], ctx);
            format!(
                "{{ use std::io::Write; {}.write(&{}).unwrap_or(0) }}",
                file_code, data_code
            )
        } else {
            "0 /* fwrite requires 4 args */".to_string()
        }
    }

    pub(crate) fn gen_call_fputs(&self, arguments: &[HirExpression], ctx: &TypeContext) -> String {
        if arguments.len() == 2 {
            let str_code =
                self.generate_expression_with_context(&arguments[0], ctx);
            let file_code =
                self.generate_expression_with_context(&arguments[1], ctx);
            format!(
                "{{ use std::io::Write; {}.write_all({}.as_bytes()).map(|_| 0).unwrap_or(-1) }}",
                file_code, str_code
            )
        } else {
            "-1 /* fputs requires 2 args */".to_string()
        }
    }

    pub(crate) fn gen_call_exec(&self, arguments: &[HirExpression], ctx: &TypeContext) -> String {
        if !arguments.is_empty() {
            let cmd = self.generate_expression_with_context(&arguments[0], ctx);
            let args: Vec<String> = arguments
                .iter()
                .skip(2)
                .filter(|a| !matches!(a, HirExpression::NullLiteral))
                .map(|a| self.generate_expression_with_context(a, ctx))
                .collect();
            if args.is_empty() {
                format!(
                    "{{ use std::process::Command; Command::new({}).status().expect(\"command failed\"); }}",
                    cmd
                )
            } else {
                let arg_chain: String =
                    args.iter().map(|a| format!(".arg({})", a)).collect();
                format!(
                    "{{ use std::process::Command; Command::new({}){}.status().expect(\"command failed\"); }}",
                    cmd, arg_chain
                )
            }
        } else {
            "/* exec requires args */".to_string()
        }
    }

    pub(crate) fn gen_call_status_macro(
        &self,
        arguments: &[HirExpression],
        ctx: &TypeContext,
        fmt_fn: impl Fn(String) -> String,
    ) -> String {
        if !arguments.is_empty() {
            let status_var =
                self.generate_expression_with_context(&arguments[0], ctx);
            fmt_fn(status_var)
        } else {
            "/* macro requires status arg */".to_string()
        }
    }

    pub(crate) fn gen_call_parse(
        &self,
        arguments: &[HirExpression],
        ctx: &TypeContext,
        rust_type: &str,
        default: &str,
    ) -> String {
        if arguments.len() == 1 {
            let s = self.generate_expression_with_context(&arguments[0], ctx);
            format!("{}.parse::<{}>().unwrap_or({})", s, rust_type, default)
        } else {
            format!("{} /* parse requires 1 arg */", default)
        }
    }

    pub(crate) fn gen_call_snprintf(&self, arguments: &[HirExpression], ctx: &TypeContext) -> String {
        if arguments.len() >= 3 {
            let fmt = self.generate_expression_with_context(&arguments[2], ctx);
            let rust_fmt = Self::convert_c_format_to_rust(&fmt);
            if arguments.len() == 3 {
                format!("format!({})", rust_fmt)
            } else {
                let args: Vec<String> = arguments[3..]
                    .iter()
                    .map(|a| self.generate_expression_with_context(a, ctx))
                    .collect();
                format!("format!({}, {})", rust_fmt, args.join(", "))
            }
        } else {
            "String::new() /* snprintf requires 3+ args */".to_string()
        }
    }

    pub(crate) fn gen_call_sprintf(&self, arguments: &[HirExpression], ctx: &TypeContext) -> String {
        if arguments.len() >= 2 {
            let fmt = self.generate_expression_with_context(&arguments[1], ctx);
            let rust_fmt = Self::convert_c_format_to_rust(&fmt);
            if arguments.len() == 2 {
                format!("format!({})", rust_fmt)
            } else {
                let args: Vec<String> = arguments[2..]
                    .iter()
                    .map(|a| self.generate_expression_with_context(a, ctx))
                    .collect();
                format!("format!({}, {})", rust_fmt, args.join(", "))
            }
        } else {
            "String::new() /* sprintf requires 2+ args */".to_string()
        }
    }

    pub(crate) fn gen_call_default(
        &self,
        function: &str,
        arguments: &[HirExpression],
        ctx: &TypeContext,
    ) -> String {
        let slice_mappings = ctx.get_slice_func_len_indices(function);
        let len_indices_to_skip: std::collections::HashSet<usize> = slice_mappings
            .map(|mappings| mappings.iter().map(|(_, len_idx)| *len_idx).collect())
            .unwrap_or_default();
        let array_indices: std::collections::HashSet<usize> = slice_mappings
            .map(|mappings| mappings.iter().map(|(arr_idx, _)| *arr_idx).collect())
            .unwrap_or_default();

        let args: Vec<String> = arguments
            .iter()
            .enumerate()
            .filter_map(|(i, arg)| {
                if len_indices_to_skip.contains(&i) {
                    return None;
                }

                if array_indices.contains(&i) {
                    let arg_code = self.generate_expression_with_context(arg, ctx);
                    return Some(format!("&{}", arg_code));
                }

                let is_address_of = matches!(arg, HirExpression::AddressOf(_))
                    || matches!(
                        arg,
                        HirExpression::UnaryOp {
                            op: decy_hir::UnaryOperator::AddressOf,
                            ..
                        }
                    );

                if is_address_of {
                    let inner = match arg {
                        HirExpression::AddressOf(inner) => inner.as_ref(),
                        HirExpression::UnaryOp { operand, .. } => operand.as_ref(),
                        _ => unreachable!(),
                    };

                    let expects_mut = ctx
                        .get_function_param_type(function, i)
                        .map(|t| {
                            matches!(t, HirType::Reference { mutable: true, .. })
                        })
                        .unwrap_or(true);

                    let inner_code =
                        self.generate_expression_with_context(inner, ctx);
                    if expects_mut {
                        Some(format!("&mut {}", inner_code))
                    } else {
                        Some(format!("&{}", inner_code))
                    }
                } else {
                    if let Some(string_iter_params) =
                        ctx.get_string_iter_func(function)
                    {
                        if let Some((_, is_mutable)) =
                            string_iter_params.iter().find(|(idx, _)| *idx == i)
                        {
                            if let HirExpression::Variable(var_name) = arg {
                                let var_type = ctx.get_type(var_name);
                                if matches!(var_type, Some(HirType::Array { .. })) {
                                    if *is_mutable {
                                        return Some(format!("&mut {}", var_name));
                                    } else {
                                        return Some(format!("&{}", var_name));
                                    }
                                }
                            }
                            if let HirExpression::StringLiteral(s) = arg {
                                return Some(format!("b\"{}\"", s));
                            }
                            if let HirExpression::AddressOf(inner) = arg {
                                let inner_code = self
                                    .generate_expression_with_context(inner, ctx);
                                if *is_mutable {
                                    return Some(format!("&mut {}", inner_code));
                                } else {
                                    return Some(format!("&{}", inner_code));
                                }
                            }
                        }
                    }

                    let param_type = ctx.get_function_param_type(function, i);
                    let is_raw_pointer_param = param_type
                        .map(|t| matches!(t, HirType::Pointer(_)))
                        .unwrap_or(false);

                    if is_raw_pointer_param {
                        if let HirExpression::Variable(var_name) = arg {
                            let var_type = ctx.get_type(var_name);
                            if matches!(var_type, Some(HirType::Array { .. })) {
                                return Some(format!("{}.as_mut_ptr()", var_name));
                            }
                        }
                        if let HirExpression::StringLiteral(s) = arg {
                            return Some(format!("\"{}\".as_ptr() as *mut u8", s));
                        }
                    }

                    let is_ref_param = param_type
                        .map(|t| matches!(t, HirType::Reference { .. }))
                        .unwrap_or(false);
                    if is_ref_param {
                        if let HirExpression::Variable(var_name) = arg {
                            let var_type = ctx.get_type(var_name);
                            if matches!(var_type, Some(HirType::Pointer(_))) {
                                return Some(Self::unsafe_block(
                                    &format!("&mut *{}", var_name),
                                    "pointer is non-null and valid for the duration of the call",
                                ));
                            }
                        }
                    }

                    let is_slice_param = param_type
                        .map(|t| matches!(t, HirType::Array { size: None, .. }))
                        .unwrap_or(false);
                    if is_slice_param {
                        if let HirExpression::Variable(var_name) = arg {
                            let var_type = ctx.get_type(var_name);
                            if matches!(var_type, Some(HirType::Array { size: Some(_), .. })) {
                                return Some(format!("&mut {}", var_name));
                            }
                        }
                    }

                    let is_int_param = param_type
                        .map(|t| matches!(t, HirType::Int))
                        .unwrap_or(false);
                    if is_int_param {
                        if let HirExpression::CharLiteral(c) = arg {
                            return Some(format!("{}i32", *c as i32));
                        }
                    }

                    let is_string_param = param_type
                        .map(|t| matches!(t, HirType::StringReference | HirType::StringLiteral))
                        .unwrap_or(false);
                    let is_string_func = matches!(
                        function,
                        "strcmp" | "strncmp" | "strchr" | "strrchr" | "strstr" | "strlen"
                    );
                    if is_string_param || is_string_func {
                        if let HirExpression::PointerFieldAccess { pointer, field } = arg {
                            let ptr_code = self.generate_expression_with_context(pointer, ctx);
                            return Some(Self::unsafe_block(
                                &format!("std::ffi::CStr::from_ptr((*{}).{} as *const i8).to_str().unwrap_or(\"\")", ptr_code, field),
                                "string pointer is null-terminated and valid",
                            ));
                        }
                    }

                    Some(self.generate_expression_with_context(arg, ctx))
                }
            })
            .collect();
        let safe_function = match function {
            "write" => "c_write",
            "read" => "c_read",
            "type" => "c_type",
            "match" => "c_match",
            "self" => "c_self",
            "in" => "c_in",
            _ => function,
        };
        format!("{}({})", safe_function, args.join(", "))
    }
}
