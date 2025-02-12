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
        value_name = "SEARCH_TERM",
        help = "Search term in the format 'fieldPath.fieldName:expectedValue'.\n\
                                         - fieldPath: Path to the field, separated by dots (e.g., 'topLevel.nestedField' or just 'field').\n\
                                         - fieldName: Name of the field to search for at the end of the path.\n\
                                         - expectedValue: Value to compare against. The value is compared as a string.\n\
                                         Examples: 'fieldOne.isPresent:true', 'topLevel.nested.value:123'"
    )]
    pub search_term: String,

    #[clap(value_name = "JSON_FILES", num_args = 0.., help = "Paths to JSON files to search within. If provided, srch will search these files instead of stdin or --json-string.")]
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
}
