use clap::Parser;

#[derive(Parser)]
#[clap(
    name = "srch",
    about = "A CLI tool to search for values in JSON from stdin, string, or files.\n\
            Examples:
                srch fieldOne.index 2 example_files/*.json\t# Search for index 2 under fieldOne
                srch index \"[0-2]\" example_files/*.json\t\t# Regex range search on 'index' field
                srch isPresent true example_files/*.json -s\t# First match for 'isPresent: true'
                srch key value -j '{\"key\": \"value\"}'\t\t# Search string input
                cat data.json | srch name \"Max\"\t\t\t# Search stdin input
                srch key_nested_value \"test\" data.json -f \"_\"\t# Custom separator
                srch index \"[0-9]\" data.json --hide-values\t# Show paths only"
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

#[cfg(test)]
mod tests {
    use crate::cli::Cli;
    use clap::Parser;

    #[test]
    fn test_default_values() {
        let args = Cli::parse_from(&["srch", "field.name", "search"]);
        assert_eq!(args.json_string, None);
        assert_eq!(args.search_path, "field.name");
        assert_eq!(args.search_term, "search");
        assert_eq!(args.json_files, Vec::<String>::new());
        assert_eq!(args.single, false);
        assert_eq!(args.path_output, false);
        assert_eq!(args.field_path_separator, ".");
        assert_eq!(args.hide_value, false);
    }

    #[test]
    fn test_short_arguments() {
        let args = Cli::parse_from(&[
            "srch",
            "-j",
            "{\"key\": \"value\"}",
            "field.name",
            "search",
            "-s",
            "-p",
            "-f",
            "_",
            "-d",
        ]);
        assert_eq!(args.json_string, Some("{\"key\": \"value\"}".to_string()));
        assert_eq!(args.search_path, "field.name");
        assert_eq!(args.search_term, "search");
        assert_eq!(args.json_files, Vec::<String>::new());
        assert_eq!(args.single, true);
        assert_eq!(args.path_output, true);
        assert_eq!(args.field_path_separator, "_");
        assert_eq!(args.hide_value, true);
    }

    #[test]
    fn test_long_arguments() {
        let args = Cli::parse_from(&[
            "srch",
            "--json-string",
            "{\"key\": \"value\"}",
            "field.name",
            "search",
            "--single",
            "--path",
            "--field-path-separator",
            "_",
            "--hide-value",
        ]);
        assert_eq!(args.json_string, Some("{\"key\": \"value\"}".to_string()));
        assert_eq!(args.search_path, "field.name");
        assert_eq!(args.search_term, "search");
        assert_eq!(args.json_files, Vec::<String>::new());
        assert_eq!(args.single, true);
        assert_eq!(args.path_output, true);
        assert_eq!(args.field_path_separator, "_");
        assert_eq!(args.hide_value, true);
    }

    #[test]
    fn test_json_files_argument() {
        let args = Cli::parse_from(&["srch", "field.name", "search", "file1.json", "file2.json"]);
        assert_eq!(
            args.json_files,
            vec!["file1.json".to_string(), "file2.json".to_string()]
        );
    }
}
