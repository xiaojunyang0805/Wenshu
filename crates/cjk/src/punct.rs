//! Chinese punctuation classes after W3C *Requirements for Chinese Text Layout* (clreq) §3.1 and
//! GB/T 15834-2011, for line breaking (避头尾) and punctuation compression (标点挤压).
//!
//! In Simplified Chinese most punctuation occupies a full em but draws in one half of it; the
//! other half is blank and may be removed where two such marks meet, or at a line end.

/// A punctuation mark's layout class.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PunctClass {
    /// Opening brackets and quotes (（ 【 《 “ ‘ 「 『 〔 〖): ink on the right half.
    Opening,
    /// Closing brackets and quotes (） 】 》 ” ’ 」 』 〕 〗): ink on the left half.
    Closing,
    /// Pause and stop marks (、 ， 。 ； ： ？ ！): ink on the left half in Simplified Chinese.
    PauseOrStop,
    /// Middle dots (· ・): centred, half blank on each side.
    Middle,
    /// Dashes and ellipses (—— ……): two ems that must not be split.
    Inseparable,
}

/// The class of `c`, or `None` when it isn't Chinese punctuation.
pub fn classify(c: char) -> Option<PunctClass> {
    Some(match c {
        '（' | '【' | '《' | '〈' | '“' | '‘' | '「' | '『' | '〔' | '〖' | '［' | '｛' | '〘'
        | '〚' => PunctClass::Opening,
        '）' | '】' | '》' | '〉' | '”' | '’' | '」' | '』' | '〕' | '〗' | '］' | '｝' | '〙'
        | '〛' => PunctClass::Closing,
        '、' | '，' | '。' | '；' | '：' | '？' | '！' | '．' => PunctClass::PauseOrStop,
        '·' | '・' | '･' => PunctClass::Middle,
        '—' | '…' | '⋯' => PunctClass::Inseparable,
        _ => return None,
    })
}

/// A line may not start with `c` (行首禁则): closing marks, pause and stop marks, middle dots.
/// Dashes and ellipses may start a line; they only must not be split between two lines
/// (GB/T 15834-2011 §5.1.6, §5.1.7), which [`no_break_between`] covers.
pub fn no_line_start(c: char) -> bool {
    matches!(
        classify(c),
        Some(PunctClass::Closing | PunctClass::PauseOrStop | PunctClass::Middle)
    ) || matches!(c, '％' | '‰' | '℃' | '々' | '〻')
}

/// A line may not break between `a` and `b`: the two halves of a dash (——) or an ellipsis
/// (……).
pub fn no_break_between(a: char, b: char) -> bool {
    a == b && classify(a) == Some(PunctClass::Inseparable)
}

/// A line may not end with `c` (行尾禁则): opening marks and currency signs.
pub fn no_line_end(c: char) -> bool {
    classify(c) == Some(PunctClass::Opening) || matches!(c, '￥' | '＄' | '￡' | '¥')
}

/// The blank half of `c` that compression may remove, in ems: 0.5 for opening, closing, pause
/// and stop marks, 0.25 a side for middle dots, none otherwise.
pub fn compressible_em(c: char) -> f64 {
    match classify(c) {
        Some(PunctClass::Opening | PunctClass::Closing | PunctClass::PauseOrStop) => 0.5,
        Some(PunctClass::Middle) => 0.25,
        _ => 0.0,
    }
}

/// Space, in ems, that compression removes between adjacent marks `a` then `b` (clreq §3.1.6):
/// a closing or stop mark followed by any opening, closing or stop mark (`”“` `。”` `）、`), or
/// two opening marks, keep only one blank half between them. Upstream layout (patch 0005) uses
/// the same pairs but leaves out ； ： ？ ！, which sit centred in Japanese fonts.
pub fn adjacent_compression_em(a: char, b: char) -> f64 {
    use PunctClass::{Closing, Opening, PauseOrStop};
    match (classify(a), classify(b)) {
        (Some(Closing | PauseOrStop), Some(Opening | Closing | PauseOrStop))
        | (Some(Opening), Some(Opening)) => 0.5,
        _ => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_start_rules() {
        for c in ['，', '。', '、', '）', '》', '”', '’', '！', '？', '·'] {
            assert!(no_line_start(c), "{c}");
        }
        for c in ['中', 'A', '（', '“', '《', '—', '…'] {
            assert!(!no_line_start(c), "{c}");
        }
        assert!(no_break_between('—', '—') && no_break_between('…', '…'));
        assert!(!no_break_between('—', '…') && !no_break_between('中', '中'));
    }

    #[test]
    fn line_end_rules() {
        for c in ['（', '“', '《', '【', '￥'] {
            assert!(no_line_end(c), "{c}");
        }
        assert!(!no_line_end('）'));
    }

    #[test]
    fn compression_pairs() {
        assert_eq!(adjacent_compression_em('，', '“'), 0.5);
        assert_eq!(adjacent_compression_em('）', '。'), 0.5);
        assert_eq!(adjacent_compression_em('《', '“'), 0.5);
        assert_eq!(adjacent_compression_em('。', '”'), 0.5);
        assert_eq!(adjacent_compression_em('（', '）'), 0.0);
        assert_eq!(adjacent_compression_em('中', '“'), 0.0);
        assert_eq!(compressible_em('·'), 0.25);
    }
}
