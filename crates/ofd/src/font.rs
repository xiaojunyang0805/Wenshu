//! Font subsets for OFD. An OFD reader draws `TextCode` characters through the embedded font's
//! own `cmap` (and `CGTransform` glyph indices where given), so unlike a PDF CID font the subset
//! must work as a font on its own: [`subsetter`] keeps the outlines and metrics but drops `cmap`
//! and `OS/2`, so we add a `cmap` for the characters used and copy `OS/2` from the original.

use std::collections::BTreeMap;

/// A subset font: its bytes and each original glyph id's new id.
pub struct Subset {
    pub data: Vec<u8>,
    pub gids: std::collections::HashMap<u32, u16>,
    /// The new cmap: character → new glyph.
    pub cmap: BTreeMap<u32, u16>,
}

impl Subset {
    pub fn cmap_glyph(&self, c: char) -> Option<u16> {
        self.cmap.get(&u32::from(c)).copied()
    }
}

/// Subset face `index` of `data` to the used glyphs, with `cmap` mapping each character in
/// `chars` (character → original glyph; the first glyph seen for a character) to its new glyph.
pub fn subset(
    data: &[u8],
    index: u32,
    glyphs: &[u32],
    chars: &BTreeMap<char, u32>,
) -> Result<Subset, String> {
    let mut remap = subsetter::GlyphRemapper::new();
    // New ids in character order, so runs of characters get runs of glyphs (short cmap segments).
    for g in chars.values() {
        remap.remap(u16::try_from(*g).map_err(|_| format!("glyph id {g} out of range"))?);
    }
    for g in glyphs {
        if let Ok(g) = u16::try_from(*g) {
            remap.remap(g);
        }
    }
    let sub = subsetter::subset(data, index, &remap).map_err(|e| format!("subset: {e:?}"))?;
    let mut gids = std::collections::HashMap::new();
    for g in glyphs.iter().chain(chars.values()) {
        if let Some(n) = u16::try_from(*g).ok().and_then(|o| remap.get(o)) {
            gids.insert(*g, n);
        }
    }
    let map: BTreeMap<u32, u16> = chars
        .iter()
        .filter_map(|(c, g)| Some((u32::from(*c), *gids.get(g)?)))
        .collect();
    let mut out = tables(&sub, 0).ok_or("subset font unreadable")?;
    out.retain(|(t, _)| t != b"cmap" && t != b"OS/2");
    out.push((*b"cmap", cmap(&map)));
    if let Some(os2) = tables(data, index).and_then(|t| t.into_iter().find(|(t, _)| t == b"OS/2")) {
        out.push(os2);
    }
    let flavor = sub
        .get(..4)
        .and_then(|b| b.try_into().ok())
        .unwrap_or([0, 1, 0, 0]);
    Ok(Subset {
        data: build(flavor, out),
        gids,
        cmap: map,
    })
}

fn u16_at(d: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_be_bytes(
        d.get(at..at.checked_add(2)?)?.try_into().ok()?,
    ))
}

fn u32_at(d: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_be_bytes(
        d.get(at..at.checked_add(4)?)?.try_into().ok()?,
    ))
}

/// The tables of face `index` (a collection's face, or the font itself).
pub(crate) fn tables(data: &[u8], index: u32) -> Option<Vec<([u8; 4], Vec<u8>)>> {
    let mut at = 0usize;
    if data.get(..4)? == b"ttcf" {
        let off = 12usize.checked_add(4usize.checked_mul(usize::try_from(index).ok()?)?)?;
        at = usize::try_from(u32_at(data, off)?).ok()?;
    }
    let n = usize::from(u16_at(data, at.checked_add(4)?)?);
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let rec = at.checked_add(12)?.checked_add(i.checked_mul(16)?)?;
        let tag: [u8; 4] = data.get(rec..rec + 4)?.try_into().ok()?;
        let off = usize::try_from(u32_at(data, rec + 8)?).ok()?;
        let len = usize::try_from(u32_at(data, rec + 12)?).ok()?;
        out.push((tag, data.get(off..off.checked_add(len)?)?.to_vec()));
    }
    Some(out)
}

fn checksum(d: &[u8]) -> u32 {
    d.chunks(4).fold(0u32, |s, c| {
        let mut b = [0u8; 4];
        for (i, v) in c.iter().enumerate() {
            if let Some(x) = b.get_mut(i) {
                *x = *v;
            }
        }
        s.wrapping_add(u32::from_be_bytes(b))
    })
}

/// An sfnt from its tables (sorted by tag, 4-byte aligned, `head.checkSumAdjustment` set).
fn build(flavor: [u8; 4], mut tables: Vec<([u8; 4], Vec<u8>)>) -> Vec<u8> {
    tables.sort_by_key(|(t, _)| *t);
    let n = tables.len();
    let n16 = u16::try_from(n).unwrap_or(u16::MAX);
    let mut sel = 0u16;
    while (1u32 << (sel + 1)) <= u32::from(n16) {
        sel += 1;
    }
    let range = (1u16 << sel).saturating_mul(16);
    let mut out = Vec::new();
    out.extend_from_slice(&flavor);
    for v in [
        n16,
        range,
        sel,
        n16.saturating_mul(16).saturating_sub(range),
    ] {
        out.extend_from_slice(&v.to_be_bytes());
    }
    let mut off = 12 + 16 * n;
    let mut head_at = None;
    for (tag, d) in &mut tables {
        if tag == b"head"
            && let Some(adj) = d.get_mut(8..12)
        {
            adj.fill(0);
            head_at = Some(off + 8);
        }
        out.extend_from_slice(tag);
        out.extend_from_slice(&checksum(d).to_be_bytes());
        out.extend_from_slice(&u32::try_from(off).unwrap_or(0).to_be_bytes());
        out.extend_from_slice(&u32::try_from(d.len()).unwrap_or(0).to_be_bytes());
        off += d.len().div_ceil(4) * 4;
    }
    for (_, d) in &tables {
        out.extend_from_slice(d);
        out.resize(out.len().div_ceil(4) * 4, 0);
    }
    if let Some(at) = head_at {
        let adj = 0xB1B0_AFBAu32.wrapping_sub(checksum(&out));
        if let Some(s) = out.get_mut(at..at + 4) {
            s.copy_from_slice(&adj.to_be_bytes());
        }
    }
    out
}

/// A `cmap` table: format 4 (BMP, Windows Unicode) when it fits, and format 12 (full Unicode)
/// when there are characters past the BMP or format 4 would overflow.
pub(crate) fn cmap(map: &BTreeMap<u32, u16>) -> Vec<u8> {
    // Runs of consecutive codes with consecutive glyphs.
    let mut runs: Vec<(u32, u32, u16)> = Vec::new();
    for (&c, &g) in map {
        if let Some((start, end, g0)) = runs.last_mut()
            && *end + 1 == c
            && u32::from(*g0) + (c - *start) == u32::from(g)
        {
            *end = c;
            continue;
        }
        runs.push((c, c, g));
    }
    let bmp: Vec<(u32, u32, u16)> = runs.iter().filter(|r| r.1 <= 0xFFFE).copied().collect();
    let all_bmp = bmp.len() == runs.len();
    let f4 = format4(&bmp);
    let f12 = (!all_bmp || f4.is_none()).then(|| format12(&runs));
    let mut subs: Vec<(u16, Vec<u8>)> = Vec::new();
    if let Some(f) = f4 {
        subs.push((1, f));
    }
    if let Some(f) = f12 {
        subs.push((10, f));
    }
    let mut out = Vec::new();
    out.extend_from_slice(&0u16.to_be_bytes());
    out.extend_from_slice(&u16::try_from(subs.len()).unwrap_or(0).to_be_bytes());
    let mut off = 4 + 8 * subs.len();
    for (enc, d) in &subs {
        out.extend_from_slice(&3u16.to_be_bytes());
        out.extend_from_slice(&enc.to_be_bytes());
        out.extend_from_slice(&u32::try_from(off).unwrap_or(0).to_be_bytes());
        off += d.len();
    }
    for (_, d) in subs {
        out.extend_from_slice(&d);
    }
    out
}

fn format4(runs: &[(u32, u32, u16)]) -> Option<Vec<u8>> {
    let mut segs: Vec<(u16, u16, u16)> = runs
        .iter()
        .filter_map(|&(s, e, g)| {
            let s = u16::try_from(s).ok()?;
            let e = u16::try_from(e).ok()?;
            Some((s, e, g.wrapping_sub(s)))
        })
        .collect();
    segs.push((0xFFFF, 0xFFFF, 1));
    let n = segs.len();
    let len = 16 + 8 * n;
    let len16 = u16::try_from(len).ok()?;
    let n2 = u16::try_from(n * 2).ok()?;
    let mut sel = 0u16;
    while (1usize << (sel + 1)) <= n {
        sel += 1;
    }
    let range = (1u16 << sel) * 2;
    let mut d = Vec::with_capacity(len);
    for v in [4u16, len16, 0, n2, range, sel, n2.saturating_sub(range)] {
        d.extend_from_slice(&v.to_be_bytes());
    }
    for s in &segs {
        d.extend_from_slice(&s.1.to_be_bytes());
    }
    d.extend_from_slice(&0u16.to_be_bytes());
    for s in &segs {
        d.extend_from_slice(&s.0.to_be_bytes());
    }
    for s in &segs {
        d.extend_from_slice(&s.2.to_be_bytes());
    }
    for _ in &segs {
        d.extend_from_slice(&0u16.to_be_bytes());
    }
    Some(d)
}

fn format12(runs: &[(u32, u32, u16)]) -> Vec<u8> {
    let mut d = Vec::with_capacity(16 + 12 * runs.len());
    d.extend_from_slice(&12u16.to_be_bytes());
    d.extend_from_slice(&0u16.to_be_bytes());
    d.extend_from_slice(
        &u32::try_from(16 + 12 * runs.len())
            .unwrap_or(0)
            .to_be_bytes(),
    );
    d.extend_from_slice(&0u32.to_be_bytes());
    d.extend_from_slice(&u32::try_from(runs.len()).unwrap_or(0).to_be_bytes());
    for &(s, e, g) in runs {
        d.extend_from_slice(&s.to_be_bytes());
        d.extend_from_slice(&e.to_be_bytes());
        d.extend_from_slice(&u32::from(g).to_be_bytes());
    }
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Looks a character up in a format 4 subtable, as a reader does.
    fn lookup4(t: &[u8], c: u16) -> Option<u16> {
        let off = usize::try_from(u32_at(t, 8)?).ok()?;
        let n = usize::from(u16_at(t, off + 6)? / 2);
        for i in 0..n {
            let end = u16_at(t, off + 14 + 2 * i)?;
            if c <= end {
                let start = u16_at(t, off + 16 + 2 * n + 2 * i)?;
                let delta = u16_at(t, off + 16 + 4 * n + 2 * i)?;
                return (c >= start).then(|| c.wrapping_add(delta));
            }
        }
        None
    }

    #[test]
    fn cmap_maps_runs_and_singles() {
        let map: BTreeMap<u32, u16> = [(0x41, 1), (0x42, 2), (0x43, 3), (0x4E2D, 4), (0x6587, 9)]
            .into_iter()
            .collect();
        let t = cmap(&map);
        assert_eq!(lookup4(&t, 0x41), Some(1));
        assert_eq!(lookup4(&t, 0x43), Some(3));
        assert_eq!(lookup4(&t, 0x4E2D), Some(4));
        assert_eq!(lookup4(&t, 0x6587), Some(9));
        assert_eq!(lookup4(&t, 0x44), None);
    }

    #[test]
    fn cmap_adds_format12_past_the_bmp() {
        let map: BTreeMap<u32, u16> = [(0x41, 1), (0x2_0000, 2)].into_iter().collect();
        let t = cmap(&map);
        assert_eq!(u16_at(&t, 2), Some(2));
    }
}
