//! Chinese punctuation classes after W3C *Requirements for Chinese Text Layout* (clreq) §3.1 and
//! GB/T 15834-2011, for line breaking (避头尾) and punctuation compression (标点挤压).
//!
//! In Simplified Chinese most punctuation occupies a full em but draws in one half of it. With
//! compression on, WPS lets that blank half shrink by up to a tenth of an em when a line is full,
//! and lets a closing mark end a line without room for its blank half (docs/WPS-FINDINGS.md F1).

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

/// How much of `c`'s blank side compression may take on a full line, in ems: a tenth for the
/// closing, pause and stop marks (not ？) and the opening marks, none otherwise. Upstream layout
/// (patch 0005) uses the same marks, measured in WPS.
pub fn squeeze_em(c: char) -> f64 {
    match classify(c) {
        Some(PunctClass::Opening | PunctClass::Closing) => 0.1,
        Some(PunctClass::PauseOrStop) if c != '？' => 0.1,
        _ => 0.0,
    }
}

/// The blank half, in ems, that `c` doesn't need when it ends a line under compression: the
/// closing brackets and the pause and stop marks (not the curly quotes, not ？).
pub fn line_end_blank_em(c: char) -> f64 {
    match classify(c) {
        Some(PunctClass::Closing) if !matches!(c, '”' | '’') => 0.5,
        Some(PunctClass::PauseOrStop) if c != '？' => 0.5,
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
    fn compression_as_wps_measures_it() {
        for c in ['。', '，', '、', '：', '；', '！', '）', '》', '”', '（', '《', '“'] {
            assert_eq!(squeeze_em(c), 0.1, "{c}");
        }
        for c in ['？', '—', '…', '中'] {
            assert_eq!(squeeze_em(c), 0.0, "{c}");
        }
        for c in ['。', '，', '、', '！', '）', '」'] {
            assert_eq!(line_end_blank_em(c), 0.5, "{c}");
        }
        for c in ['”', '’', '？', '（', '中'] {
            assert_eq!(line_end_blank_em(c), 0.0, "{c}");
        }
    }
}
