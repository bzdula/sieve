use crate::compiler::grammar::{Comparator, MatchType, RelationalMatch};

use super::glob::{glob_match, glob_match_with_values};

impl MatchType {
    pub(crate) fn match_value(
        &self,
        haystack: &str,
        needle: &str,
        comparator: &Comparator,
        matched_values: &mut Vec<(usize, String)>,
    ) -> bool {
        match self {
            MatchType::Is => match comparator {
                Comparator::Octet => haystack == needle,
                Comparator::AsciiNumeric => {
                    if let (Ok(haystack), Ok(needle)) =
                        (haystack.parse::<f64>(), needle.parse::<f64>())
                    {
                        haystack == needle
                    } else {
                        false
                    }
                }
                _ => haystack.to_lowercase() == needle.to_lowercase(),
            },
            MatchType::Contains => match comparator {
                Comparator::Octet => haystack.contains(needle),
                _ => haystack.to_lowercase().contains(&needle.to_lowercase()),
            },
            MatchType::Value(value) => match comparator {
                Comparator::Octet => value.cmp(haystack, needle.as_ref()),
                Comparator::AsciiNumeric => {
                    if let (Ok(haystack), Ok(needle)) =
                        (haystack.parse::<f64>(), needle.parse::<f64>())
                    {
                        value.cmp(&haystack, &needle)
                    } else {
                        false
                    }
                }
                _ => value.cmp(&haystack.to_lowercase(), &needle.to_lowercase()),
            },
            MatchType::Matches(positions) => {
                let to_lower = matches!(comparator, Comparator::AsciiCaseMap);
                if *positions == 0 {
                    glob_match(haystack, needle, to_lower)
                } else {
                    glob_match_with_values(haystack, needle, to_lower, *positions, matched_values)
                }
            }
            MatchType::Regex(vars) => false,
            MatchType::Count(_) | MatchType::List => false,
        }
    }
}

impl RelationalMatch {
    pub(crate) fn cmp_num(&self, num: f64, value: &str) -> bool {
        if let Ok(value) = value.parse::<f64>() {
            self.cmp(&num, &value)
        } else {
            false
        }
    }

    pub(crate) fn cmp<T>(&self, haystack: &T, needle: &T) -> bool
    where
        T: PartialOrd + ?Sized,
    {
        match self {
            RelationalMatch::Gt => haystack.gt(needle),
            RelationalMatch::Ge => haystack.ge(needle),
            RelationalMatch::Lt => haystack.lt(needle),
            RelationalMatch::Le => haystack.le(needle),
            RelationalMatch::Eq => haystack.eq(needle),
            RelationalMatch::Ne => haystack.ne(needle),
        }
    }
}
