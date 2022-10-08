use mail_parser::Message;

use crate::{compiler::grammar::test::Test, Context, Event};

use super::RuntimeError;

pub mod comparator;
pub mod glob;
pub mod test_header;
pub mod test_string;

pub(crate) enum TestResult {
    Bool(bool),
    Event { event: Event, is_not: bool },
    Error(RuntimeError),
}

impl Test {
    pub(crate) fn exec(&self, ctx: &mut Context, message: &Message) -> TestResult {
        TestResult::Bool(match &self {
            Test::Address(_) => todo!(),
            Test::Envelope(_) => todo!(),
            Test::Exists(_) => todo!(),
            Test::Header(test) => test.exec(ctx, message),
            Test::Size(_) => todo!(),
            Test::Body(_) => todo!(),
            Test::String(test) => test.exec(ctx),
            Test::Date(_) => todo!(),
            Test::CurrentDate(_) => todo!(),
            Test::Duplicate(_) => todo!(),
            Test::NotifyMethodCapability(_) => todo!(),
            Test::ValidNotifyMethod(_) => todo!(),
            Test::Environment(_) => todo!(),
            Test::ValidExtList(_) => todo!(),
            Test::Ihave(ihave) => {
                ihave
                    .capabilities
                    .iter()
                    .all(|c| ctx.runtime.allowed_capabilities.contains(c))
                    ^ ihave.is_not
            }
            Test::HasFlag(_) => todo!(),
            Test::MailboxExists(test) => {
                return TestResult::Event {
                    event: Event::MailboxExists {
                        names: ctx.eval_strings_owned(&test.mailbox_names),
                    },
                    is_not: test.is_not,
                };
            }
            Test::Metadata(_) => todo!(),
            Test::MetadataExists(_) => todo!(),
            Test::ServerMetadata(_) => todo!(),
            Test::ServerMetadataExists(_) => todo!(),
            Test::MailboxIdExists(_) => todo!(),
            Test::SpamTest(_) => todo!(),
            Test::VirusTest(_) => todo!(),
            Test::SpecialUseExists(_) => todo!(),
            Test::Convert(_) => todo!(),
            Test::True => true,
            Test::False => false,
            Test::Invalid(invalid) => {
                return TestResult::Error(RuntimeError::InvalidInstruction(invalid.clone()))
            }
        })
    }
}