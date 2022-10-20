use serde::{Deserialize, Serialize};

use crate::compiler::{
    grammar::{
        instruction::{CompilerState, Instruction},
        Capability,
    },
    lexer::{string::StringItem, word::Word, Token},
    CompileError,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Redirect {
    pub copy: bool,
    pub address: StringItem,
    pub notify: Notify,
    pub return_of_content: Ret,
    pub by_time: ByTime<StringItem>,
    pub list: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum NotifyItem {
    Success,
    Failure,
    Delay,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Notify {
    Never,
    Items(Vec<NotifyItem>),
    Default,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Ret {
    Full,
    Hdrs,
    Default,
}

/*

   Usage:   redirect [:bytimerelative <rlimit: number> /
                      :bytimeabsolute <alimit:string>
                      [:bymode "notify"|"return"] [:bytrace]]
                     <address: string>

*/

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ByTime<T> {
    Relative {
        rlimit: u64,
        mode: ByMode,
        trace: bool,
    },
    Absolute {
        alimit: T,
        mode: ByMode,
        trace: bool,
    },
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ByMode {
    Notify,
    Return,
    Default,
}

impl<'x> CompilerState<'x> {
    pub(crate) fn parse_redirect(&mut self) -> Result<(), CompileError> {
        let address;
        let mut copy = false;
        let mut ret = Ret::Default;
        let mut notify = Notify::Default;
        let mut list = false;
        let mut by_mode = ByMode::Default;
        let mut by_trace = false;
        let mut by_rlimit = None;
        let mut by_alimit = None;

        loop {
            let token_info = self.tokens.unwrap_next()?;
            match token_info.token {
                Token::Tag(Word::Copy) => {
                    self.validate_argument(
                        1,
                        Capability::Copy.into(),
                        token_info.line_num,
                        token_info.line_pos,
                    )?;
                    copy = true;
                }
                Token::Tag(Word::List) => {
                    self.validate_argument(
                        2,
                        Capability::ExtLists.into(),
                        token_info.line_num,
                        token_info.line_pos,
                    )?;
                    list = true;
                }
                Token::Tag(Word::ByTrace) => {
                    self.validate_argument(
                        3,
                        Capability::RedirectDeliverBy.into(),
                        token_info.line_num,
                        token_info.line_pos,
                    )?;
                    by_trace = true;
                }
                Token::Tag(Word::ByMode) => {
                    self.validate_argument(
                        4,
                        Capability::RedirectDeliverBy.into(),
                        token_info.line_num,
                        token_info.line_pos,
                    )?;
                    let by_mode_ = self.tokens.expect_static_string()?;
                    if by_mode_.eq_ignore_ascii_case(b"notify") {
                        by_mode = ByMode::Notify;
                    } else if by_mode_.eq_ignore_ascii_case(b"return") {
                        by_mode = ByMode::Return;
                    } else {
                        return Err(token_info.expected("\"notify\" or \"return\""));
                    }
                }
                Token::Tag(Word::ByTimeRelative) => {
                    self.validate_argument(
                        5,
                        Capability::RedirectDeliverBy.into(),
                        token_info.line_num,
                        token_info.line_pos,
                    )?;
                    by_rlimit = (self.tokens.expect_number(u64::MAX as usize)? as u64).into();
                }
                Token::Tag(Word::ByTimeAbsolute) => {
                    self.validate_argument(
                        5,
                        Capability::RedirectDeliverBy.into(),
                        token_info.line_num,
                        token_info.line_pos,
                    )?;
                    by_alimit = self.parse_string()?.into();
                }
                Token::Tag(Word::Ret) => {
                    self.validate_argument(
                        6,
                        Capability::RedirectDsn.into(),
                        token_info.line_num,
                        token_info.line_pos,
                    )?;
                    let ret_ = self.tokens.expect_static_string()?;
                    if ret_.eq_ignore_ascii_case(b"full") {
                        ret = Ret::Full;
                    } else if ret_.eq_ignore_ascii_case(b"hdrs") {
                        ret = Ret::Hdrs;
                    } else {
                        return Err(token_info.expected("\"FULL\" or \"HDRS\""));
                    }
                }
                Token::Tag(Word::Notify) => {
                    self.validate_argument(
                        7,
                        Capability::RedirectDsn.into(),
                        token_info.line_num,
                        token_info.line_pos,
                    )?;
                    let notify_ = self.tokens.expect_static_string()?;
                    if notify_.eq_ignore_ascii_case(b"never") {
                        notify = Notify::Never;
                    } else {
                        let mut items = Vec::new();
                        for item in String::from_utf8_lossy(&notify_).split(',') {
                            let item = item.trim();
                            if item.eq_ignore_ascii_case("success") {
                                items.push(NotifyItem::Success);
                            } else if item.eq_ignore_ascii_case("failure") {
                                items.push(NotifyItem::Failure);
                            } else if item.eq_ignore_ascii_case("delay") {
                                items.push(NotifyItem::Delay);
                            }
                        }
                        if !items.is_empty() {
                            notify = Notify::Items(items);
                        } else {
                            return Err(
                                token_info.expected("\"NEVER\" or \"SUCCESS, FAILURE, DELAY, ..\"")
                            );
                        }
                    }
                }
                _ => {
                    address = self.parse_string_token(token_info)?;
                    break;
                }
            }
        }

        self.instructions.push(Instruction::Redirect(Redirect {
            address,
            copy,
            notify,
            return_of_content: ret,
            by_time: if let Some(alimit) = by_alimit {
                ByTime::Absolute {
                    alimit,
                    mode: by_mode,
                    trace: by_trace,
                }
            } else if let Some(rlimit) = by_rlimit {
                ByTime::Relative {
                    rlimit,
                    mode: by_mode,
                    trace: by_trace,
                }
            } else {
                ByTime::None
            },
            list,
        }));
        Ok(())
    }
}
