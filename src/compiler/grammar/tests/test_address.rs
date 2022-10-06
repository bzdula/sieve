use serde::{Deserialize, Serialize};

use crate::compiler::{
    grammar::{command::CompilerState, test::Test, Comparator},
    lexer::{string::StringItem, word::Word, Token},
    CompileError,
};

use crate::compiler::grammar::{AddressPart, MatchType};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct TestAddress {
    pub header_list: Vec<StringItem>,
    pub key_list: Vec<StringItem>,
    pub address_part: AddressPart,
    pub match_type: MatchType,
    pub comparator: Comparator,
    pub index: Option<u16>,
    pub index_last: bool,

    pub mime: bool,
    pub mime_anychild: bool,

    pub list: bool,
}

impl<'x> CompilerState<'x> {
    pub(crate) fn parse_test_address(&mut self) -> Result<Test, CompileError> {
        let mut address_part = AddressPart::All;
        let mut match_type = MatchType::Is;
        let mut comparator = Comparator::AsciiCaseMap;
        let mut header_list = None;
        let key_list;
        let mut index = None;
        let mut index_last = false;

        let mut mime = false;
        let mut mime_anychild = false;

        let mut list = false;

        loop {
            let token_info = self.tokens.unwrap_next()?;
            match token_info.token {
                Token::Tag(
                    word @ (Word::LocalPart | Word::Domain | Word::All | Word::User | Word::Detail),
                ) => {
                    address_part = word.into();
                }
                Token::Tag(
                    word @ (Word::Is
                    | Word::Contains
                    | Word::Matches
                    | Word::Value
                    | Word::Count
                    | Word::Regex),
                ) => {
                    match_type = self.parse_match_type(word)?;
                }
                Token::Tag(Word::Comparator) => {
                    comparator = self.parse_comparator()?;
                }
                Token::Tag(Word::Index) => {
                    index = (self.tokens.expect_number(u16::MAX as usize)? as u16).into();
                }
                Token::Tag(Word::Last) => {
                    index_last = true;
                }
                Token::Tag(Word::List) => {
                    list = true;
                }
                Token::Tag(Word::Mime) => {
                    mime = true;
                }
                Token::Tag(Word::AnyChild) => {
                    mime_anychild = true;
                }
                _ => {
                    if header_list.is_none() {
                        header_list = self.parse_strings_token(token_info)?.into();
                    } else {
                        key_list = self.parse_strings_token(token_info)?;
                        break;
                    }
                }
            }
        }

        Ok(Test::Address(TestAddress {
            header_list: header_list.unwrap(),
            key_list,
            address_part,
            match_type,
            comparator,
            index,
            index_last,
            mime,
            mime_anychild,
            list,
        }))
    }
}

impl From<Word> for AddressPart {
    fn from(word: Word) -> Self {
        match word {
            Word::LocalPart => AddressPart::LocalPart,
            Word::Domain => AddressPart::Domain,
            Word::All => AddressPart::All,
            Word::User => AddressPart::User,
            Word::Detail => AddressPart::Detail,
            _ => unreachable!(),
        }
    }
}
