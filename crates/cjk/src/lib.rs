//! Chinese typography data for Wenshu, with no dependencies so any engine can use it.
//!
//! - [`fonts`]: substitutes for Chinese document fonts that are often not installed
//!   (仿宋_GB2312, 方正小标宋简体, 楷体_GB2312…), per platform, open fonts last.
//! - [`punct`]: Chinese punctuation classes (W3C clreq, GB/T 15834), line-start and line-end
//!   prohibitions, and which punctuation can be compressed (标点挤压).

pub mod fonts;
pub mod punct;
