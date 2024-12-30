#![expect(clippy::missing_docs_in_private_items)]

pub mod login;
pub mod register;
pub mod whoami;

#[cfg(test)]
mod test;
