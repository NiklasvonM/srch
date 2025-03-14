pub fn parse_search_path<'a>(
    search_path: &'a str,
    field_path_separator: &'a str,
) -> Result<(Vec<&'a str>, &'a str), String> {
    if let Some((field_path_str, field_name)) = search_path.rsplit_once(field_path_separator) {
        if !field_name.is_empty() {
            let field_path_parts: Vec<&str> = field_path_str.split(field_path_separator).collect();
            Ok((field_path_parts, field_name))
        } else {
            Err("Invalid search term format. Field name or expected value is empty.".to_string())
        }
    } else {
        // Handle case where there's no dot in path, e.g., "field:value" - fieldPath is empty
        let field_name = search_path;
        if !field_name.is_empty() {
            Ok((vec![], field_name)) // Empty field_path_parts when no path
        } else {
            Err("Invalid search term format. Field name or expected value is empty.".to_string())
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ComparisonOperator {
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Equal,
}

impl ComparisonOperator {
    fn from_str(op_str: &str) -> Option<Self> {
        match op_str {
            "<" => Some(ComparisonOperator::LessThan),
            "<=" => Some(ComparisonOperator::LessThanOrEqual),
            ">" => Some(ComparisonOperator::GreaterThan),
            ">=" => Some(ComparisonOperator::GreaterThanOrEqual),
            "==" => Some(ComparisonOperator::Equal),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum NumericSearchTerm {
    SingleComparison(ComparisonOperator, f64),
    RangeComparison(ComparisonOperator, f64, ComparisonOperator, f64),
}

impl NumericSearchTerm {
    pub fn from_search_term(search_term: &str) -> Option<Self> {
        // Try to parse as range first
        if let Some(range_term) = Self::parse_as_range(search_term) {
            return Some(range_term);
        }

        // Then try as single comparison
        if let Some(single_term) = Self::parse_as_single(search_term) {
            return Some(single_term);
        }

        None
    }

    fn parse_as_single(search_term: &str) -> Option<Self> {
        let ops = ["<=", ">=", "<", ">", "=="];
        for op_str in ops {
            if let Some(num_str) = search_term.strip_prefix(op_str) {
                if let Ok(num_value) = num_str.parse::<f64>() {
                    if let Some(operator) = ComparisonOperator::from_str(op_str) {
                        return Some(NumericSearchTerm::SingleComparison(operator, num_value));
                    }
                }
            }
        }
        None
    }

    fn parse_as_range(search_term: &str) -> Option<Self> {
        let ops = ["<=", ">=", "<", ">"];
        for op1_str in &ops {
            if let Some(rest1) = search_term.strip_prefix(op1_str) {
                for op2_str in &ops {
                    if let Some(num_str1_end_op2) = rest1.find(op2_str) {
                        let num_str1 = &rest1[..num_str1_end_op2];
                        let rest2 = &rest1[num_str1_end_op2..];
                        let num_str2 = &rest2[op2_str.len()..];

                        if let (Ok(num1), Ok(num2)) =
                            (num_str1.parse::<f64>(), num_str2.parse::<f64>())
                        {
                            if let (Some(op1), Some(op2)) = (
                                ComparisonOperator::from_str(op1_str),
                                ComparisonOperator::from_str(op2_str),
                            ) {
                                return Some(NumericSearchTerm::RangeComparison(
                                    op1, num1, op2, num2,
                                ));
                            }
                        }
                    }
                }
            }
        }
        None
    }

    fn compare_single(&self, json_num: f64) -> bool {
        match self {
            NumericSearchTerm::SingleComparison(op, target_num) => match op {
                ComparisonOperator::GreaterThan => json_num > *target_num,
                ComparisonOperator::LessThan => json_num < *target_num,
                ComparisonOperator::GreaterThanOrEqual => json_num >= *target_num,
                ComparisonOperator::LessThanOrEqual => json_num <= *target_num,
                ComparisonOperator::Equal => json_num == *target_num,
            },
            _ => false,
        }
    }

    fn compare_range(&self, json_num: f64) -> bool {
        match self {
            NumericSearchTerm::RangeComparison(op1, num1, op2, num2) => {
                NumericSearchTerm::SingleComparison(op1.clone(), *num1).compare_single(json_num)
                    && NumericSearchTerm::SingleComparison(op2.clone(), *num2)
                        .compare_single(json_num)
            }
            _ => false,
        }
    }

    pub fn matches(&self, json_num: f64) -> bool {
        match self {
            NumericSearchTerm::SingleComparison(_, _) => self.compare_single(json_num),
            NumericSearchTerm::RangeComparison(_, _, _, _) => self.compare_range(json_num),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_search_path_valid_with_path() {
        let search_path = "a.b.c.field";
        let field_path_separator = ".";
        let result = parse_search_path(search_path, field_path_separator);
        assert_eq!(result, Ok((vec!["a", "b", "c"], "field")));
    }

    #[test]
    fn test_parse_search_path_valid_without_path() {
        let search_path = "field";
        let field_path_separator = ".";
        let result = parse_search_path(search_path, field_path_separator);
        assert_eq!(result, Ok((vec![], "field")));
    }

    #[test]
    fn test_parse_search_path_valid_with_different_separator() {
        let search_path = "a/b/c/field";
        let field_path_separator = "/";
        let result = parse_search_path(search_path, field_path_separator);
        assert_eq!(result, Ok((vec!["a", "b", "c"], "field")));
    }

    #[test]
    fn test_parse_search_path_empty_field_name() {
        let search_path = "a.b.c.";
        let field_path_separator = ".";
        let result = parse_search_path(search_path, field_path_separator);
        assert_eq!(
            result,
            Err("Invalid search term format. Field name or expected value is empty.".to_string())
        );
    }

    #[test]
    fn test_parse_search_path_empty_search_path() {
        let search_path = "";
        let field_path_separator = ".";
        let result = parse_search_path(search_path, field_path_separator);
        assert_eq!(
            result,
            Err("Invalid search term format. Field name or expected value is empty.".to_string())
        );
    }

    #[test]
    fn test_parse_search_path_only_separator() {
        let search_path = ".";
        let field_path_separator = ".";
        let result = parse_search_path(search_path, field_path_separator);
        assert_eq!(
            result,
            Err("Invalid search term format. Field name or expected value is empty.".to_string())
        );
    }

    #[test]
    fn test_parse_search_path_multiple_separators_no_field_name() {
        let search_path = "a.b.c..";
        let field_path_separator = ".";
        let result = parse_search_path(search_path, field_path_separator);
        assert_eq!(
            result,
            Err("Invalid search term format. Field name or expected value is empty.".to_string())
        );
    }

    #[test]
    fn test_parse_search_path_field_name_with_separator_char() {
        let search_path = "a.b.c.field";
        let field_path_separator = ".";
        let result = parse_search_path(search_path, field_path_separator);
        assert_eq!(result, Ok((vec!["a", "b", "c"], "field")));
    }

    #[test]
    fn test_parse_numeric_search_term_valid() {
        assert_eq!(
            NumericSearchTerm::from_search_term("<=10"),
            Some(NumericSearchTerm::SingleComparison(
                ComparisonOperator::LessThanOrEqual,
                10.0
            ))
        );
        assert_eq!(
            NumericSearchTerm::from_search_term(">=20"),
            Some(NumericSearchTerm::SingleComparison(
                ComparisonOperator::GreaterThanOrEqual,
                20.0
            ))
        );
        assert_eq!(
            NumericSearchTerm::from_search_term("<5"),
            Some(NumericSearchTerm::SingleComparison(
                ComparisonOperator::LessThan,
                5.0
            ))
        );
        assert_eq!(
            NumericSearchTerm::from_search_term(">25"),
            Some(NumericSearchTerm::SingleComparison(
                ComparisonOperator::GreaterThan,
                25.0
            ))
        );
    }

    #[test]
    fn test_parse_numeric_search_term_invalid() {
        assert_eq!(NumericSearchTerm::from_search_term("!=10"), None);
        assert_eq!(NumericSearchTerm::from_search_term("~10"), None);
        assert_eq!(NumericSearchTerm::from_search_term("=10"), None);
        assert_eq!(NumericSearchTerm::from_search_term("10<"), None);
        assert_eq!(NumericSearchTerm::from_search_term("10>"), None);
        assert_eq!(NumericSearchTerm::from_search_term("10<="), None);
        assert_eq!(NumericSearchTerm::from_search_term("10>="), None);
    }

    #[test]
    fn test_parse_numeric_search_term_no_operator() {
        assert_eq!(NumericSearchTerm::from_search_term("10"), None);
        assert_eq!(NumericSearchTerm::from_search_term("abc"), None);
        assert_eq!(NumericSearchTerm::from_search_term(""), None);
    }

    #[test]
    fn test_parse_numeric_range_term_valid() {
        assert_eq!(
            NumericSearchTerm::from_search_term(">10<20"),
            Some(NumericSearchTerm::RangeComparison(
                ComparisonOperator::GreaterThan,
                10.0,
                ComparisonOperator::LessThan,
                20.0
            ))
        );
        assert_eq!(
            NumericSearchTerm::from_search_term(">=5<=15"),
            Some(NumericSearchTerm::RangeComparison(
                ComparisonOperator::GreaterThanOrEqual,
                5.0,
                ComparisonOperator::LessThanOrEqual,
                15.0
            ))
        );
        assert_eq!(
            NumericSearchTerm::from_search_term("<=25>=1"),
            Some(NumericSearchTerm::RangeComparison(
                ComparisonOperator::LessThanOrEqual,
                25.0,
                ComparisonOperator::GreaterThanOrEqual,
                1.0
            ))
        );
        assert_eq!(
            NumericSearchTerm::from_search_term(">=1<=25"),
            Some(NumericSearchTerm::RangeComparison(
                ComparisonOperator::GreaterThanOrEqual,
                1.0,
                ComparisonOperator::LessThanOrEqual,
                25.0
            ))
        );
    }

    #[test]
    fn test_parse_numeric_range_term_invalid() {
        assert_eq!(NumericSearchTerm::from_search_term(">10-20"), None);
        assert_eq!(NumericSearchTerm::from_search_term("10"), None);
        assert_eq!(NumericSearchTerm::from_search_term("><1020"), None);
        assert_eq!(NumericSearchTerm::from_search_term("1020<>"), None);
        assert_eq!(NumericSearchTerm::from_search_term("=10<20"), None);
        assert_eq!(NumericSearchTerm::from_search_term(">10=20"), None);
    }

    #[test]
    fn test_parse_numeric_range_term_single_number_search() {
        assert_eq!(NumericSearchTerm::from_search_term("10"), None);
        assert_eq!(NumericSearchTerm::from_search_term("abc"), None);
    }

    #[test]
    fn test_parse_numeric_range_term_empty() {
        assert_eq!(NumericSearchTerm::from_search_term(""), None);
    }

    #[test]
    fn test_parse_numeric_range_term_operators_only() {
        assert_eq!(NumericSearchTerm::from_search_term("><"), None);
        assert_eq!(NumericSearchTerm::from_search_term("<>"), None);
        assert_eq!(NumericSearchTerm::from_search_term(">="), None);
        assert_eq!(NumericSearchTerm::from_search_term("<="), None);
    }
}
