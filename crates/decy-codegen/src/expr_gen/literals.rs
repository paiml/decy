//! Literal and variable expression generation.

use super::{escape_rust_keyword, CodeGenerator, TypeContext};
use decy_hir::{HirExpression, HirType};

impl CodeGenerator {
    pub(crate) fn gen_expr_int_literal(&self, val: i32, target_type: Option<&HirType>) -> String {
        if val == 0 {
            if let Some(HirType::Option(_)) = target_type {
                return "None".to_string();
            }
            if let Some(HirType::Pointer(_)) = target_type {
                return "std::ptr::null_mut()".to_string();
            }
        }
        val.to_string()
    }

    pub(crate) fn gen_expr_float_literal(&self, val: &str, target_type: Option<&HirType>) -> String {
        let val_stripped = val.trim_end_matches(['f', 'F', 'l', 'L']);
        match target_type {
            Some(HirType::Float) => format!("{}f32", val_stripped),
            Some(HirType::Double) => format!("{}f64", val_stripped),
            _ => {
                if val_stripped.contains('.')
                    || val_stripped.contains('e')
                    || val_stripped.contains('E')
                {
                    format!("{}f64", val_stripped)
                } else {
                    format!("{}.0f64", val_stripped)
                }
            }
        }
    }

    pub(crate) fn gen_expr_address_of(
        &self,
        inner: &HirExpression,
        ctx: &TypeContext,
        target_type: Option<&HirType>,
    ) -> String {
        if let Some(HirType::Pointer(ptr_inner)) = target_type {
            let inner_code = self.generate_expression_with_context(inner, ctx);
            let ptr_type = Self::map_type(&HirType::Pointer(ptr_inner.clone()));
            return format!("&mut {} as {}", inner_code, ptr_type);
        }
        let inner_code = self.generate_expression_with_context(inner, ctx);
        if matches!(*inner, HirExpression::Dereference(_)) {
            format!("&({})", inner_code)
        } else {
            format!("&{}", inner_code)
        }
    }

    pub(crate) fn gen_expr_unary_address_of(
        &self,
        operand: &HirExpression,
        ctx: &TypeContext,
        target_type: Option<&HirType>,
    ) -> String {
        if let Some(HirType::Pointer(ptr_inner)) = target_type {
            let inner_code = self.generate_expression_with_context(operand, ctx);
            let ptr_type = Self::map_type(&HirType::Pointer(ptr_inner.clone()));
            return format!("&mut {} as {}", inner_code, ptr_type);
        }
        let inner_code = self.generate_expression_with_context(operand, ctx);
        format!("&{}", inner_code)
    }

    pub(crate) fn gen_expr_unary_logical_not(
        &self,
        operand: &HirExpression,
        ctx: &TypeContext,
        target_type: Option<&HirType>,
    ) -> String {
        let inner_code = self.generate_expression_with_context(operand, ctx);
        let inner_parens = if matches!(*operand, HirExpression::BinaryOp { .. }) {
            format!("({})", inner_code)
        } else {
            inner_code.clone()
        };
        if let Some(HirType::Int) = target_type {
            if Self::is_boolean_expression(operand) {
                return format!("(!{}) as i32", inner_parens);
            } else {
                return format!("({} == 0) as i32", inner_code);
            }
        }
        if Self::is_boolean_expression(operand) {
            format!("!{}", inner_parens)
        } else {
            format!("({} == 0)", inner_code)
        }
    }

    pub(crate) fn gen_expr_string_literal(&self, s: &str, target_type: Option<&HirType>) -> String {
        if let Some(HirType::Pointer(inner)) = target_type {
            if matches!(inner.as_ref(), HirType::Char) {
                let escaped: String = s
                    .chars()
                    .map(|c| match c {
                        '"' => "\\\"".to_string(),
                        '\\' => "\\\\".to_string(),
                        c => c.to_string(),
                    })
                    .collect();
                return format!("b\"{}\\0\".as_ptr() as *mut u8", escaped);
            }
        }
        format!("\"{}\"", s)
    }

    pub(crate) fn gen_expr_char_literal(c: i8) -> String {
        let val = c as u8;
        if val == 0 {
            "0u8".to_string()
        } else if val.is_ascii_graphic() || val == b' ' {
            format!("b'{}'", val as char)
        } else {
            format!("{}u8", val)
        }
    }

    pub(crate) fn gen_expr_variable_pointer_target(
        escaped_name: &str,
        name: &str,
        ptr_inner: &Box<HirType>,
        ctx: &TypeContext,
    ) -> Option<String> {
        let var_type = ctx.get_type(name)?;
        if matches!(var_type, HirType::Box(_)) {
            return Some(format!("Box::into_raw({})", escaped_name));
        }
        match var_type {
            HirType::Reference { inner, mutable } => {
                let element_type_match = match inner.as_ref() {
                    HirType::Array { element_type, .. } => {
                        Some((element_type.as_ref(), *mutable))
                    }
                    HirType::Vec(elem_type) => Some((elem_type.as_ref(), *mutable)),
                    _ => None,
                };

                if let Some((elem_type, is_mutable)) = element_type_match {
                    if elem_type == ptr_inner.as_ref() {
                        if is_mutable {
                            return Some(format!("{}.as_mut_ptr()", escaped_name));
                        } else {
                            let ptr_type = Self::map_type(&HirType::Pointer(
                                ptr_inner.clone(),
                            ));
                            return Some(format!(
                                "{}.as_ptr() as {}",
                                escaped_name, ptr_type
                            ));
                        }
                    }
                } else if inner.as_ref() == ptr_inner.as_ref() {
                    if *mutable {
                        return Some(format!("{} as *mut _", escaped_name));
                    } else {
                        return Some(format!("{} as *const _ as *mut _", escaped_name));
                    }
                }
            }
            HirType::Vec(elem_type) => {
                if elem_type.as_ref() == ptr_inner.as_ref() {
                    return Some(format!("{}.as_mut_ptr()", escaped_name));
                }
            }
            HirType::Array { element_type, .. } => {
                if element_type.as_ref() == ptr_inner.as_ref() {
                    return Some(format!("{}.as_mut_ptr()", escaped_name));
                }
                if matches!(ptr_inner.as_ref(), HirType::Void) {
                    return Some(format!("{}.as_mut_ptr() as *mut ()", escaped_name));
                }
            }
            HirType::Pointer(_var_inner) => {
                return Some(escaped_name.to_string());
            }
            _ => {}
        }
        None
    }

    pub(crate) fn gen_expr_variable_numeric_coercion(
        escaped_name: &str,
        name: &str,
        target: &HirType,
        ctx: &TypeContext,
    ) -> Option<String> {
        let var_type = ctx.get_type(name)?;
        let cast_suffix = if matches!(var_type, HirType::Int | HirType::UnsignedInt) {
            match target {
                HirType::Float => Some("f32"),
                HirType::Double => Some("f64"),
                _ => None,
            }
        } else if matches!(var_type, HirType::Float | HirType::Double) {
            match target {
                HirType::Int => Some("i32"),
                HirType::UnsignedInt => Some("u32"),
                _ => None,
            }
        } else if matches!(var_type, HirType::Char) && matches!(target, HirType::Int) {
            Some("i32")
        } else {
            None
        };

        cast_suffix.map(|suffix| {
            let code = format!("{} as {}", escaped_name, suffix);
            if ctx.is_global(name) {
                format!("unsafe {{ {} }}", code)
            } else {
                code
            }
        })
    }

    pub(crate) fn gen_expr_variable(
        &self,
        name: &str,
        ctx: &TypeContext,
        target_type: Option<&HirType>,
    ) -> String {
        match name {
            "stderr" => return "std::io::stderr()".to_string(),
            "stdin" => return "std::io::stdin()".to_string(),
            "stdout" => return "std::io::stdout()".to_string(),
            "errno" => return "unsafe { ERRNO }".to_string(),
            "ERANGE" => return "34i32".to_string(),
            "EINVAL" => return "22i32".to_string(),
            "ENOENT" => return "2i32".to_string(),
            "EACCES" => return "13i32".to_string(),
            _ => {}
        }
        let escaped_name = escape_rust_keyword(name);
        let escaped_name =
            ctx.get_renamed_local(&escaped_name).cloned().unwrap_or(escaped_name);
        if let Some(HirType::Vec(_)) = target_type {
            return escaped_name;
        }
        if let Some(HirType::Pointer(ptr_inner)) = target_type {
            if let Some(result) =
                Self::gen_expr_variable_pointer_target(&escaped_name, name, ptr_inner, ctx)
            {
                return result;
            }
        }

        if let Some(HirType::Char) = target_type {
            if let Some(var_type) = ctx.get_type(name) {
                if matches!(var_type, HirType::Int) {
                    return format!("{} as u8", escaped_name);
                }
            }
        }

        if let Some(target) = target_type {
            if let Some(result) =
                Self::gen_expr_variable_numeric_coercion(&escaped_name, name, target, ctx)
            {
                return result;
            }
        }

        if ctx.is_global(name) {
            format!("unsafe {{ {} }}", escaped_name)
        } else {
            escaped_name
        }
    }
}
