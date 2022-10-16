use std::borrow::Cow;

use mail_parser::{parsers::MessageStream, DateTime, Header, HeaderValue};

use crate::{
    compiler::grammar::{
        tests::test_date::{DatePart, TestCurrentDate, TestDate, Zone},
        MatchType,
    },
    Context,
};

impl TestDate {
    pub(crate) fn exec(&self, ctx: &mut Context) -> bool {
        let header_name = ctx.parse_header_name(&self.header_name);

        let result = if let MatchType::Count(rel_match) = &self.match_type {
            let mut date_count = 0;
            ctx.find_headers(
                &[header_name],
                self.index,
                self.mime_anychild,
                |header, _, _| {
                    if ctx.find_dates(header).is_some() {
                        date_count += 1;
                    }
                    false
                },
            );

            let mut result = false;
            for key in &self.key_list {
                if rel_match.cmp_num(date_count as f64, ctx.eval_string(key).as_ref()) {
                    result = true;
                    break;
                }
            }
            result
        } else {
            let key_list = ctx.eval_strings(&self.key_list);
            let mut captured_values = Vec::new();

            let result = ctx.find_headers(
                &[header_name],
                self.index,
                self.mime_anychild,
                |header, _, _| {
                    if let Some(dt) = ctx.find_dates(header) {
                        let date_part = self.date_part.eval(self.zone.eval(dt.as_ref()).as_ref());
                        for key in &key_list {
                            if match &self.match_type {
                                MatchType::Is => self.comparator.is(&date_part, key.as_ref()),
                                MatchType::Contains => {
                                    self.comparator.contains(&date_part, key.as_ref())
                                }
                                MatchType::Value(rel_match) => {
                                    self.comparator
                                        .relational(rel_match, &date_part, key.as_ref())
                                }
                                MatchType::Matches(capture_positions) => self.comparator.matches(
                                    &date_part,
                                    key.as_ref(),
                                    *capture_positions,
                                    &mut captured_values,
                                ),
                                MatchType::Regex(capture_positions) => self.comparator.matches(
                                    &date_part,
                                    key.as_ref(),
                                    *capture_positions,
                                    &mut captured_values,
                                ),
                                MatchType::Count(_) | MatchType::List => false,
                            } {
                                return true;
                            }
                        }
                    }

                    false
                },
            );
            if !captured_values.is_empty() {
                ctx.set_match_variables(captured_values);
            }
            result
        };

        result ^ self.is_not
    }
}

impl TestCurrentDate {
    pub(crate) fn exec(&self, ctx: &mut Context) -> bool {
        let mut result = false;

        if let MatchType::Count(rel_match) = &self.match_type {
            for key in &self.key_list {
                if rel_match.cmp_num(1.0, ctx.eval_string(key).as_ref()) {
                    result = true;
                    break;
                }
            }
        } else {
            let mut captured_values = Vec::new();
            let date_part = self.date_part.eval(
                &(if let Some(zone) = self.zone {
                    DateTime::from_timestamp(ctx.current_time).to_timezone(zone)
                } else {
                    DateTime::from_timestamp(ctx.current_time)
                }),
            );

            for key in &self.key_list {
                let key = ctx.eval_string(key);

                if match &self.match_type {
                    MatchType::Is => self.comparator.is(&date_part, key.as_ref()),
                    MatchType::Contains => self.comparator.contains(&date_part, key.as_ref()),
                    MatchType::Value(rel_match) => {
                        self.comparator
                            .relational(rel_match, &date_part, key.as_ref())
                    }
                    MatchType::Matches(capture_positions) => self.comparator.matches(
                        &date_part,
                        key.as_ref(),
                        *capture_positions,
                        &mut captured_values,
                    ),
                    MatchType::Regex(capture_positions) => self.comparator.matches(
                        &date_part,
                        key.as_ref(),
                        *capture_positions,
                        &mut captured_values,
                    ),
                    MatchType::Count(_) | MatchType::List => false,
                } {
                    result = true;
                    break;
                }
            }

            if !captured_values.is_empty() {
                ctx.set_match_variables(captured_values);
            }
        }

        result ^ self.is_not
    }
}

impl<'x> Context<'x> {
    #[allow(unused_assignments)]
    pub(crate) fn find_dates(&self, header: &'x Header) -> Option<Cow<'x, DateTime>> {
        if let HeaderValue::DateTime(dt) = &header.value {
            if dt.is_valid() {
                return Some(Cow::Borrowed(dt));
            }
        } else if header.offset_end > 0 {
            let bytes = self
                .message
                .raw_message
                .get(header.offset_start..header.offset_end)?;
            if let HeaderValue::DateTime(dt) = MessageStream::new(bytes).parse_date() {
                if dt.is_valid() {
                    return Some(Cow::Owned(dt));
                }
            }
        } else if let HeaderValue::Text(text) = &header.value {
            // Inserted header
            let bytes = format!("{}\n", text).into_bytes();
            if let HeaderValue::DateTime(dt) = MessageStream::new(&bytes).parse_date() {
                if dt.is_valid() {
                    return Some(Cow::Owned(dt));
                }
            }
        }
        None
    }
}

impl DatePart {
    fn eval(&self, dt: &DateTime) -> String {
        match self {
            DatePart::Year => format!("{:04}", dt.year),
            DatePart::Month => format!("{:02}", dt.month),
            DatePart::Day => format!("{:02}", dt.day),
            DatePart::Date => format!("{:04}-{:02}-{:02}", dt.year, dt.month, dt.day,),
            DatePart::Julian => ((dt.julian_day() as f64 - 2400000.5) as i64).to_string(),
            DatePart::Hour => format!("{:02}", dt.hour),
            DatePart::Minute => format!("{:02}", dt.minute),
            DatePart::Second => format!("{:02}", dt.second),
            DatePart::Time => format!("{:02}:{:02}:{:02}", dt.hour, dt.minute, dt.second,),
            DatePart::Iso8601 => dt.to_rfc3339(),
            DatePart::Std11 => dt.to_rfc822(),
            DatePart::Zone => format!(
                "{}{:02}{:02}",
                if dt.tz_before_gmt && (dt.tz_hour > 0 || dt.tz_minute > 0) {
                    "-"
                } else {
                    "+"
                },
                dt.tz_hour,
                dt.tz_minute
            ),
            DatePart::Weekday => dt.day_of_week().to_string(),
        }
    }
}

impl Zone {
    pub(crate) fn eval<'x>(&self, dt: &'x DateTime) -> Cow<'x, DateTime> {
        match self {
            Zone::Time(tz) => Cow::Owned(dt.to_timezone(*tz)),
            Zone::Original => Cow::Borrowed(dt),
            Zone::Local => Cow::Owned(DateTime::from_timestamp(dt.to_timestamp())),
        }
    }
}