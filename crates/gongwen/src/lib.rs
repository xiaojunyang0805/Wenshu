//! GB/T 9704-2012 *党政机关公文格式* (layout of official documents of Party and government
//! organs): the page specification ([`spec`]), a DOCX generator ([`generate`]) and a checker
//! that reports where a DOCX departs from the standard ([`check`]).
//!
//! Both work on OOXML directly, so they don't depend on any layout engine.

pub mod check;
pub mod generate;
pub mod spec;
pub mod templates;

pub use check::{Finding, check_docx};
pub use generate::{Gongwen, write_docx};
