    fn detect_output_params(
        func: Option<&HirFunction>,
    ) -> (std::collections::HashSet<String>, Vec<HirType>, bool) {
        use decy_analyzer::output_params::{OutputParamDetector, ParameterKind};
        let mut skip_output_params = std::collections::HashSet::new();
        let mut output_param_types: Vec<HirType> = Vec::new();
        let mut output_is_fallible = false;

        if let Some(f) = func {
            let output_detector = OutputParamDetector::new();
            let output_params = output_detector.detect(f);

            // Count non-pointer parameters (inputs)
            let input_param_count = f
                .parameters()
                .iter()
                .filter(|p| !matches!(p.param_type(), HirType::Pointer(_)))
                .count();

            // Count potential output params for heuristic
            let output_param_count =
                output_params.iter().filter(|op| op.kind == ParameterKind::Output).count();

            for op in &output_params {
                if op.kind == ParameterKind::Output {
                    // Heuristic: Only treat as output param if:
                    // 1. There are other input parameters (output is derived from inputs)
                    // 2. Or, the name suggests it's an output (result, out, output, ret, etc.)
                    // 3. DECY-085: Or, there are multiple output params (void func with multiple outs)
                    let is_output_name = Self::is_output_param_name(&op.name);

                    if input_param_count > 0 || is_output_name || output_param_count >= 2 {
                        skip_output_params.insert(op.name.clone());
                        output_is_fallible = op.is_fallible;
                        // DECY-085: Collect all output parameter types
                        if let Some(param) = f.parameters().iter().find(|p| p.name() == op.name) {
                            if let HirType::Pointer(inner) = param.param_type() {
                                output_param_types.push((**inner).clone());
                            }
                        }
                    }
                }
            }
        }

        (skip_output_params, output_param_types, output_is_fallible)
    }

    fn is_output_param_name(name: &str) -> bool {
        let name_lower = name.to_lowercase();
        name_lower.contains("result")
            || name_lower.contains("out")
            || name_lower.contains("ret")
            || name_lower == "len"
            || name_lower == "size"
            || name_lower == "x"
            || name_lower == "y"
            || name_lower == "z"
            || name_lower == "w"
            || name_lower == "h"
            || name_lower == "width"
            || name_lower == "height"
            || name_lower == "r"
            || name_lower == "g"
            || name_lower == "b"
            || name_lower == "count"
            || name_lower == "avg"
    }
