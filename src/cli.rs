use clap::Parser;

#[derive(Parser)]
#[clap(
    name = "srch",
    about = "A CLI tool to search for values in JSON from stdin, string, or files.\n\
            Example usage: `srch \"fieldPath.fieldName: true\" example_files/*.json | wc`"
)]
pub struct Cli {
    #[clap(
        short = 'j',
        long = "json-string",
        value_name = "JSON_STRING",
        help = "Provide JSON input as a string directly in the command line."
    )]
    pub json_string: Option<String>,

    #[clap(
        value_name = "SEARCH_PATH",
        help = "Search path in the format 'fieldPath.fieldName'.\n\
                                         - fieldPath: Path to the field, separated by the FIELD_PATH_SEPARATOR (default .) (e.g., 'topLevel.nestedField' or just 'field').\n\
                                         - fieldName: Name of the field to search for at the end of the path."
    )]
    pub search_path: String,

    #[clap(
        value_name = "SEARCH_TERM",
        help = "Regex to compare values against. The values are compared as strings."
    )]
    pub search_term: String,

    #[clap(value_name = "JSON_FILES", num_args = 0.., help = "Paths to JSON files to search within. If provided, srch will search these files instead of stdin or --json-string.\n\
                                                                Example: example_files/*.json")]
    pub json_files: Vec<String>,

    #[clap(
        short = 's',
        long = "single",
        help = "Return only the first match per file."
    )]
    pub single: bool,

    #[clap(
        short = 'p',
        long = "path",
        help = "Output the file path instead of the result path (only for file input)."
    )]
    pub path_output: bool,

    #[clap(
        short = 'f',
        long = "field-path-separator",
        help = "Separator for the field path. Applies both to the input path as well as the output paths.",
        default_value = "."
    )]
    pub field_path_separator: String,

    #[clap(
        short = 'd',
        long = "hide-value",
        help = "If provided, the values found are not printed."
    )]
    pub hide_value: bool,
}
