//! The page specification of GB/T 9704-2012, in twentieths of a point (twips) as OOXML uses.
//!
//! §5.2: A4 paper; top margin (天头) 37 mm ± 1, inner margin (订口) 28 mm ± 1; type area (版心)
//! 156 mm × 225 mm. §5.2.2: 22 lines a page, 28 characters a line, body in 3号 (16 pt) FangSong.

/// Twips per millimetre (1440 per inch).
pub const TWIPS_PER_MM: f64 = 1440.0 / 25.4;

pub const fn mm(v: u32) -> u32 {
    // Rounded: 37 mm is 2097.6 twips.
    ((v as f64) * TWIPS_PER_MM + 0.5) as u32
}

/// A4 page.
pub const PAGE_W: u32 = 11906;
pub const PAGE_H: u32 = 16838;
/// Margins: top 37 mm; left (订口) 28 mm; type area 156 × 225 mm sets the other two.
pub const MARGIN_TOP: u32 = mm(37);
pub const MARGIN_LEFT: u32 = mm(28);
pub const TYPE_W: u32 = mm(156);
pub const TYPE_H: u32 = mm(225);
pub const MARGIN_RIGHT: u32 = PAGE_W - MARGIN_LEFT - TYPE_W;
pub const MARGIN_BOTTOM: u32 = PAGE_H - MARGIN_TOP - TYPE_H;
/// Tolerance on margins (§5.2.1: ± 1 mm).
pub const MARGIN_TOLERANCE: u32 = mm(1);

pub const LINES_PER_PAGE: u32 = 22;
pub const CHARS_PER_LINE: u32 = 28;
/// Line pitch: the type height over 22 lines.
pub const LINE_PITCH: u32 = TYPE_H / LINES_PER_PAGE;

/// Font sizes in half-points (OOXML `w:sz`): 初号 42 pt … 4号 14 pt.
pub const SIZE_2HAO: u32 = 44; // 22 pt: title
pub const SIZE_3HAO: u32 = 32; // 16 pt: body
pub const SIZE_4HAO: u32 = 28; // 14 pt: page numbers
pub const SIZE_ORGAN_MARK: u32 = 72; // 36 pt: issuing-organ mark (common practice; §7.2.3 leaves it to the organ)

/// Fonts as documents name them, so files open the same way in WPS and Word.
pub const FONT_BODY: &str = "仿宋_GB2312";
pub const FONT_TITLE: &str = "方正小标宋简体";
pub const FONT_H1: &str = "黑体";
pub const FONT_H2: &str = "楷体_GB2312";
pub const FONT_PAGE_NUMBER: &str = "宋体";

/// Red for the organ mark and the separator line (§7.2.3, §7.2.6).
pub const RED: &str = "FF0000";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_geometry_matches_the_standard() {
        assert_eq!(MARGIN_TOP, 2098);
        assert_eq!(MARGIN_LEFT, 1587);
        // 26 mm right, ~35 mm bottom on A4.
        assert!(
            (MARGIN_RIGHT as i64 - mm(26) as i64).abs() <= 2,
            "{MARGIN_RIGHT}"
        );
        assert!(
            (MARGIN_BOTTOM as i64 - mm(35) as i64).abs() <= 3,
            "{MARGIN_BOTTOM}"
        );
        assert_eq!(LINE_PITCH, 579);
    }
}
