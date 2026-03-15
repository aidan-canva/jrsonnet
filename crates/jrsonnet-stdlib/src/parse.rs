use jrsonnet_evaluator::{function::builtin, runtime_error, IStr, Result, Val};

#[builtin]
pub fn builtin_parse_json(str: IStr) -> Result<Val> {
	let value: Val =
		serde_json::from_str(&str).map_err(|e| runtime_error!("failed to parse json: {e}"))?;
	Ok(value)
}

/// Detect whether the input is a YAML stream (contains `---` document markers).
fn is_yaml_stream(s: &str) -> bool {
	for line in s.lines() {
		let trimmed = line.trim_end();
		if trimmed == "---" || trimmed.starts_with("--- ") {
			return true;
		}
	}
	false
}

#[builtin]
pub fn builtin_parse_yaml(str: IStr) -> Result<Val> {
	let is_stream = is_yaml_stream(&str);
	let out = serde_saphyr::from_multiple_with_options::<Val>(
		&str,
		serde_saphyr::Options {
			// Golang/C++ compat
			legacy_octal_numbers: true,
			// Disable budget limits - we trust the YAML input
			budget: None,
			// Many Helm Charts contain duplicate keys - take last key rather than erroring
			duplicate_keys: serde_saphyr::DuplicateKeyPolicy::LastWins,
			..Default::default()
		},
	)
	.map_err(|e| runtime_error!("failed to parse yaml: {e}"))?;

	// go-jsonnet compatibility:
	// - If input is a YAML stream (has --- markers), always return an array
	// - If input is a single document (no --- markers), return the value directly
	Ok(if is_stream {
		Val::Arr(out.into())
	} else if out.is_empty() {
		Val::Null
	} else if out.len() == 1 {
		out.into_iter().next().unwrap()
	} else {
		Val::Arr(out.into())
	})
}
