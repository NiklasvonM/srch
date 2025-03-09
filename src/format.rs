use crate::parse::SearchResult;

pub struct FormatContext {
    pub field_path_separator: String,
    pub hide_value: bool,
    pub path_output: bool,
}

pub fn format_text_output(
    result: &SearchResult,
    file_path: Option<&str>,
    format_context: &FormatContext,
) -> String {
    if format_context.path_output && file_path.is_some() {
        file_path.unwrap().to_string()
    } else if format_context.hide_value {
        result.json_path.join(&format_context.field_path_separator)
    } else {
        format!(
            "{}: {}",
            result.json_path.join(&format_context.field_path_separator),
            result.value
        )
    }
}
