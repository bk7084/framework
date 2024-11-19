use rustc_hash::FxHashMap;

/// Simple utility function to preprocess a WGSL shader.
///
/// Currently only simple if condition are supported with the syntax:
/// ```wgsl
/// // #if SOME_FEATURE
/// ... code ...
/// // #fi
/// ```
///
/// This removes all the blank lines and the lines that are not included in the final output.
pub fn preprocess_wgsl(source: &str, conditions: &FxHashMap<&str, bool>) -> String {
    let mut output = String::new();
    let mut include = true; // Whether the current lines should be included
    let mut inside_else = false; // Whether the current block is an else block

    for line in source.lines() {
        if line.is_empty() {
            // Skip empty lines
            continue;
        } else if let Some(condition) = line.strip_prefix("// #if ") {
            // Start of a conditional block
            let condition = condition.trim();
            if let Some(stripped) = condition.strip_prefix('!') {
                include = !*conditions.get(stripped).unwrap_or(&false);
            } else {
                include = *conditions.get(condition).unwrap_or(&false);
            }
            inside_else = false;
        } else if let Some(condition) = line.strip_prefix("// #else") {
            include = !inside_else && !include; // Only include if not already in an else block
            inside_else = true;
        } else if line.contains("// #fi") {
            // End of a conditional block
            include = true;
            inside_else = false;
        } else if include {
            // Include the line if the current block is active
            output.push_str(line);
            output.push('\n');
        }
    }

    output.trim_end().to_string() // Trim trailing whitespace
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE0: &str = r#"
// #if constant_sized_binding_array
@group(1) @binding(0) var textures: array<texture_2d<f32>, 16>;
// #fi

// #if dynamic_binding_array
@group(1) @binding(0) var textures: binding_array<texture_2d<f32>>;
// #fi

// #if use_shadow_maps
@group(2) @binding(0) var smap: texture_depth_2d_array;
@group(2) @binding(1) var smap_sampler: sampler_comparison;
// #fi"#;

    const SOURCE1: &str = r#"
// #if constant_sized_binding_array
@group(1) @binding(0) var textures: array<texture_2d<f32>, 16>;
// #else
@group(1) @binding(0) var textures: array<texture_2d<f32>, 8>; // Default fallback
// #fi

// #if !use_shadow_maps
// Shadow maps are disabled
// #else
@group(2) @binding(0) var smap: texture_depth_2d_array;
@group(2) @binding(1) var smap_sampler: sampler_comparison;
// #fi"#;

    #[test]
    fn test_preprocess_wgsl_0() {
        let mut conditions = FxHashMap::default();
        conditions.insert("constant_sized_binding_array", true);
        conditions.insert("dynamic_binding_array", false);
        conditions.insert("use_shadow_maps", true);

        let result = preprocess_wgsl(SOURCE0, &conditions);
        assert_eq!(
            result,
            r#"@group(1) @binding(0) var textures: array<texture_2d<f32>, 16>;
@group(2) @binding(0) var smap: texture_depth_2d_array;
@group(2) @binding(1) var smap_sampler: sampler_comparison;"#
        );
    }

    #[test]
    fn test_preprocess_wgsl_1() {
        let mut conditions = FxHashMap::default();
        conditions.insert("constant_sized_binding_array", false);
        conditions.insert("dynamic_binding_array", true);
        conditions.insert("use_shadow_maps", false);

        let result = preprocess_wgsl(SOURCE0, &conditions);
        assert_eq!(
            result,
            r#"@group(1) @binding(0) var textures: binding_array<texture_2d<f32>>;"#
        );
    }

    #[test]
    fn test_preprocess_wgsl_2() {
        let mut conditions = FxHashMap::default();
        conditions.insert("constant_sized_binding_array", true);
        conditions.insert("use_shadow_maps", true);

        let result = preprocess_wgsl(SOURCE1, &conditions);
        assert_eq!(
            result,
            r#"@group(1) @binding(0) var textures: array<texture_2d<f32>, 16>;
@group(2) @binding(0) var smap: texture_depth_2d_array;
@group(2) @binding(1) var smap_sampler: sampler_comparison;"#
        );
    }

    #[test]
    fn test_preprocess_wgsl_3() {
        let mut conditions = FxHashMap::default();
        conditions.insert("constant_sized_binding_array", false);
        conditions.insert("use_shadow_maps", false);

        let result = preprocess_wgsl(SOURCE1, &conditions);
        assert_eq!(
            result,
            r#"@group(1) @binding(0) var textures: array<texture_2d<f32>, 8>; // Default fallback
// Shadow maps are disabled"#
        );
    }
}
