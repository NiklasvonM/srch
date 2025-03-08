use crate::parse::SearchResult;

pub fn format_text_output(
    result: &SearchResult,
    hide_value: bool,
    path_output: bool,
    file_path: Option<&str>,
) -> String {
    if path_output && file_path.is_some() {
        file_path.unwrap().to_string()
    } else if hide_value {
        result.json_path.clone()
    } else {
        format!("{}: {}", result.json_path, result.value)
    }
}
