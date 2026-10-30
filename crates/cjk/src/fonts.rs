//! Substitutes for Chinese document fonts.
//!
//! Government and enterprise documents name fonts that ship only with particular systems or
//! commercial packs: 仿宋_GB2312 and 楷体_GB2312 (Windows XP era), 方正小标宋简体 (Founder), 华文
//! (STXihei…, macOS Office). When one is missing, the substitute must keep its *design class*
//! (FangSong, Kai, Song, Hei): drawing a 公文 body in a sans face is the most visible defect a
//! Chinese reader sees. Each list goes Windows names, macOS names, then open fonts (Fandol,
//! LXGW WenKai, Noto/Source Han), and the engine takes the first installed one.

/// FangSong (仿宋) designs: the 公文 body font.
pub const FANGSONG: &[&str] = &[
    "FangSong",
    "仿宋",
    "STFangsong",
    "华文仿宋",
    "FandolFang",
    "Noto Serif CJK SC",
    "Source Han Serif SC",
    "Noto Serif SC",
    "SimSun",
];
/// Kai (楷体) designs: 公文 second-level headings, signatures.
pub const KAITI: &[&str] = &[
    "KaiTi",
    "楷体",
    "STKaiti",
    "华文楷体",
    "Kaiti SC",
    "LXGW WenKai",
    "FandolKai",
    "Noto Serif CJK SC",
    "Source Han Serif SC",
    "Noto Serif SC",
    "SimSun",
];
/// Song / Ming (宋体) designs: the default body font of Chinese Word documents.
pub const SONG: &[&str] = &[
    "SimSun",
    "宋体",
    "NSimSun",
    "新宋体",
    "Songti SC",
    "STSong",
    "华文宋体",
    "FandolSong",
    "Noto Serif CJK SC",
    "Source Han Serif SC",
    "Noto Serif SC",
];
/// Heavy display Song (小标宋, 中宋): 公文 titles and the issuing-organ mark.
pub const XIAOBIAOSONG: &[&str] = &[
    "FZXiaoBiaoSong-B05S",
    "方正小标宋简体",
    "STZhongsong",
    "华文中宋",
    "Songti SC",
    "FandolSong",
    "Noto Serif CJK SC",
    "Source Han Serif SC",
    "Noto Serif SC",
    "SimSun",
];
/// Hei (黑体) designs: 公文 first-level headings.
pub const HEI: &[&str] = &[
    "SimHei",
    "黑体",
    "STHeiti",
    "华文黑体",
    "Heiti SC",
    "PingFang SC",
    "FandolHei",
    "Noto Sans CJK SC",
    "Source Han Sans SC",
    "Noto Sans SC",
    "Microsoft YaHei",
];
/// Modern UI sans (微软雅黑, 等线, 苹方).
pub const YAHEI: &[&str] = &[
    "Microsoft YaHei",
    "微软雅黑",
    "DengXian",
    "等线",
    "PingFang SC",
    "Noto Sans CJK SC",
    "Source Han Sans SC",
    "Noto Sans SC",
    "SimHei",
];

/// Substitutes for `family`, best first, or `None` when it isn't a Chinese document font this
/// table knows. Matching ignores case, spaces and the `_GB2312` / `_GBK` / `简体` / `-GB` suffixes.
pub fn substitutes(family: &str) -> Option<&'static [&'static str]> {
    let key = normalize(family);
    let list = match key.as_str() {
        "仿宋" | "fangsong" | "方正仿宋" | "fzfangsong" | "fzfangsong-z02s" | "fzfangsong-z02"
        | "华文仿宋" | "stfangsong" | "仿宋体" => FANGSONG,
        "楷体" | "kaiti" | "方正楷体" | "fzkai" | "fzkai-z03s" | "fzkai-z03" | "华文楷体"
        | "stkaiti" | "楷体字" => KAITI,
        "宋体" | "simsun" | "新宋体" | "nsimsun" | "华文宋体" | "stsong" | "方正书宋"
        | "fzshusong" | "fzshusong-z01s" => SONG,
        "方正小标宋"
        | "fzxiaobiaosong"
        | "fzxiaobiaosong-b05s"
        | "fzxiaobiaosong-b05"
        | "小标宋"
        | "华文中宋"
        | "stzhongsong"
        | "方正大标宋"
        | "fzdabiaosong-b06s" => XIAOBIAOSONG,
        "黑体" | "simhei" | "方正黑体" | "fzhei" | "fzhei-b01s" | "华文黑体" | "stheiti"
        | "华文细黑" | "stxihei" => HEI,
        "微软雅黑" | "microsoftyahei" | "等线" | "dengxian" | "苹方" | "pingfangsc" => {
            YAHEI
        }
        _ => return None,
    };
    Some(list)
}

/// Lower-case, without spaces or the encoding suffixes Chinese font names carry.
fn normalize(family: &str) -> String {
    let mut key: String = family
        .trim()
        .chars()
        .filter(|c| !c.is_whitespace())
        .flat_map(char::to_lowercase)
        .collect();
    for suffix in [
        "_gb2312", "_gbk", "-gb2312", "gb2312", "_gb18030", "简体", "-gb", "_gb",
    ] {
        if let Some(stripped) = key.strip_suffix(suffix) {
            key = stripped.to_string();
            break;
        }
    }
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gongwen_fonts_keep_their_design_class() {
        assert_eq!(substitutes("仿宋_GB2312"), Some(FANGSONG));
        assert_eq!(substitutes("FangSong_GB2312"), Some(FANGSONG));
        assert_eq!(substitutes("方正仿宋_GBK"), Some(FANGSONG));
        assert_eq!(substitutes("楷体_GB2312"), Some(KAITI));
        assert_eq!(substitutes("方正小标宋简体"), Some(XIAOBIAOSONG));
        assert_eq!(substitutes("FZXiaoBiaoSong-B05S"), Some(XIAOBIAOSONG));
        assert_eq!(substitutes("黑体"), Some(HEI));
        assert_eq!(substitutes(" Microsoft YaHei "), Some(YAHEI));
    }

    #[test]
    fn other_fonts_fall_through() {
        assert_eq!(substitutes("Calibri"), None);
        assert_eq!(substitutes(""), None);
        assert_eq!(substitutes("Noto Serif CJK SC"), None);
    }

    #[test]
    fn serif_designs_never_fall_back_to_sans() {
        for list in [FANGSONG, KAITI, SONG, XIAOBIAOSONG] {
            assert!(
                list.iter()
                    .all(|f| !f.contains("Sans") && !f.contains("Hei") && !f.contains("YaHei")),
                "{list:?}"
            );
        }
    }
}
