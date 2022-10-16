use mail_parser::DateTime;

use crate::{
    compiler::grammar::actions::action_redirect::{ByTime, Redirect},
    Action, Context,
};

impl Redirect {
    pub(crate) fn exec(&self, ctx: &mut Context) {
        if let Some(address) = sanitize_address(ctx.eval_string(&self.address).as_ref()) {
            if ctx
                .actions
                .iter()
                .filter(|a| matches!(a, Action::Redirect { .. }))
                .count()
                < ctx.runtime.max_redirects
            {
                if ctx.has_changes {
                    let bytes = ctx.build_message();
                    ctx.actions.push(Action::UpdateMessage { bytes });
                }
                ctx.actions.push(Action::Redirect {
                    address,
                    copy: self.copy,
                    notify: self.notify.clone(),
                    return_of_content: self.return_of_content.clone(),
                    by_time: match &self.by_time {
                        ByTime::Relative {
                            rlimit,
                            mode,
                            trace,
                        } => ByTime::Relative {
                            rlimit: *rlimit,
                            mode: mode.clone(),
                            trace: *trace,
                        },
                        ByTime::Absolute {
                            alimit,
                            mode,
                            trace,
                        } => ByTime::Absolute {
                            alimit: DateTime::parse_rfc3339(ctx.eval_string(alimit).as_ref())
                                .and_then(|d| {
                                    if d.is_valid() {
                                        d.to_timestamp().into()
                                    } else {
                                        None
                                    }
                                })
                                .unwrap_or(0),
                            mode: mode.clone(),
                            trace: *trace,
                        },
                        ByTime::None => ByTime::None,
                    },
                });
            }
        }
    }
}

pub(crate) fn sanitize_address(addr: &str) -> Option<String> {
    let mut result = String::with_capacity(addr.len());
    let mut in_quote = false;
    let mut last_ch = '\n';
    let mut has_at = false;
    let mut has_dot = false;

    for ch in addr.chars() {
        match ch {
            '\"' => {
                if !in_quote {
                    in_quote = true;
                } else if last_ch != '\\' {
                    in_quote = false;
                }
            }
            '@' if !in_quote => {
                if !has_at && !result.is_empty() {
                    has_at = true;
                    result.push(ch);
                } else {
                    return None;
                }
            }
            '.' if !in_quote && has_at && !has_dot => {
                has_dot = true;
                result.push(ch);
            }
            '<' => {
                result.clear();
                has_at = false;
                has_dot = false;
            }
            '>' => (),
            _ => {
                if !ch.is_ascii_whitespace() || in_quote {
                    result.push(ch);
                }
            }
        }
        last_ch = ch;
    }

    if !result.is_empty() && has_at && has_dot {
        Some(result)
    } else {
        None
    }
}