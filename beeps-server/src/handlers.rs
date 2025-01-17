#![expect(clippy::missing_docs_in_private_items)]

pub mod documents;
pub mod health;
pub mod login;
pub mod register;
pub mod whoami;

#[cfg(test)]
mod test;
