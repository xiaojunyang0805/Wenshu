//! An OFD (GB/T 33190-2016, *电子文件存储与交换格式 版式文档*) writer.
//!
//! The caller builds an [`Ofd`] of [`Page`]s holding text, path and image objects in millimetres
//! (OFD's unit, y down from the page's top-left corner) and calls [`Ofd::write`] for the package
//! (a zip). Text is real text: each [`TextCode`] carries its characters, their glyphs in the font
//! and their positions; fonts are embedded as subsets with a `cmap` for the characters used, and a
//! `CGTransform` is written only where a glyph isn't the one the font's `cmap` gives for its
//! character (ligatures, alternates). Pictures are embedded as given (PNG or JPEG).
//!
//! Every object's `Boundary` is the whole page, so the coordinates in an object (text positions,
//! path points, clip areas) are page coordinates; a `CTM` maps the object's own space into it.
//!
//! The crate knows nothing about layout engines: Wenshu's `wenshu-core` walks WordCraft's page
//! display list into this model.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::io::Write as _;

mod font;

/// Document information (OFD.xml `DocInfo`).
#[derive(Clone, Debug, Default)]
pub struct Meta {
    pub title: String,
    pub author: String,
    pub subject: String,
    pub keywords: Vec<String>,
    pub creator: String,
    pub creator_version: String,
    /// `YYYY-MM-DD`.
    pub creation_date: String,
    /// 32 hex digits identifying the document; derived from the content when empty.
    pub doc_id: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FontId(usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ImageId(usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color(pub u8, pub u8, pub u8);

/// A font face to embed: face `index` of `data` (a font or a collection).
pub struct FontFace<'a> {
    /// `FontName`: what a reader looks for when it can't use the embedded file.
    pub name: String,
    pub family: String,
    pub bold: bool,
    pub italic: bool,
    pub serif: bool,
    pub fixed_width: bool,
    pub data: &'a [u8],
    pub index: u32,
}

/// Picture formats readers take as they are.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageFormat {
    Png,
    Jpeg,
}

impl ImageFormat {
    fn ext(self) -> &'static str {
        match self {
            ImageFormat::Png => "png",
            ImageFormat::Jpeg => "jpg",
        }
    }
}

/// A path segment, in the object's coordinates (mm).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Seg {
    Move(f64, f64),
    Line(f64, f64),
    Quad(f64, f64, f64, f64),
    Cubic(f64, f64, f64, f64, f64, f64),
    Close,
}

/// What every object may carry.
#[derive(Clone, Debug, PartialEq)]
pub struct Common {
    /// Maps the object's own space into page space: `[a b c d e f]`, x' = a·x + c·y + e,
    /// y' = b·x + d·y + f (e and f in mm).
    pub ctm: Option<[f64; 6]>,
    /// Clip areas in page space, all applying (their intersection).
    pub clips: Vec<Vec<Seg>>,
    /// Opacity 0–255.
    pub alpha: u8,
}

impl Default for Common {
    fn default() -> Self {
        Common {
            ctm: None,
            clips: Vec::new(),
            alpha: 255,
        }
    }
}

/// One run of characters: `glyphs` either one per character, in order, or (a ligature, a
/// cluster) any number for the whole run. `dx` / `dy` are the steps from each character's
/// position to the next (one fewer than the characters; `dy` empty when all on one line).
#[derive(Clone, Debug, PartialEq, Default)]
pub struct TextCode {
    pub x: f64,
    pub y: f64,
    pub text: String,
    pub glyphs: Vec<u32>,
    pub dx: Vec<f64>,
    pub dy: Vec<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Text {
    pub common: Common,
    pub font: FontId,
    /// Em size in mm.
    pub size: f64,
    pub fill: Option<Color>,
    /// Outline colour and width (mm): synthetic bold.
    pub stroke: Option<(Color, f64)>,
    /// Horizontal glyph scale (1 = none).
    pub hscale: f64,
    pub codes: Vec<TextCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Cap {
    #[default]
    Butt,
    Round,
    Square,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Join {
    #[default]
    Miter,
    Round,
    Bevel,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Stroke {
    pub color: Color,
    /// mm.
    pub width: f64,
    pub dash: Vec<f64>,
    pub cap: Cap,
    pub join: Join,
}

/// A fill: a colour, or a gradient between colour stops (offsets 0–1).
#[derive(Clone, Debug, PartialEq)]
pub enum Fill {
    Solid(Color),
    Axial {
        start: (f64, f64),
        end: (f64, f64),
        stops: Vec<(f64, Color)>,
    },
    Radial {
        center: (f64, f64),
        radius: f64,
        stops: Vec<(f64, Color)>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Path {
    pub common: Common,
    pub segs: Vec<Seg>,
    pub fill: Option<Fill>,
    pub stroke: Option<Stroke>,
    pub even_odd: bool,
}

/// A picture: `common.ctm` maps the unit square (0,0)–(1,1), the picture's top-left to its
/// bottom-right, onto the page.
#[derive(Clone, Debug, PartialEq)]
pub struct Image {
    pub common: Common,
    pub image: ImageId,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Object {
    Text(Text),
    Path(Path),
    Image(Image),
}

/// A page: its size in mm and its objects, in painting order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Page {
    pub width: f64,
    pub height: f64,
    pub objects: Vec<Object>,
}

/// A document being built.
pub struct Ofd<'a> {
    pub meta: Meta,
    pub pages: Vec<Page>,
    fonts: Vec<FontFace<'a>>,
    images: Vec<(Vec<u8>, ImageFormat)>,
}

/// Numbers as OFD writes them: up to 3 decimals (a micrometre), no trailing zeros.
fn num(v: f64) -> String {
    if !v.is_finite() {
        return "0".to_string();
    }
    let r = (v * 1000.0).round() / 1000.0;
    let mut s = format!("{r:.3}");
    while s.ends_with('0') {
        s.pop();
    }
    if s.ends_with('.') {
        s.pop();
    }
    if s == "-0" { "0".to_string() } else { s }
}

/// XML text and attribute escaping; characters XML 1.0 can't hold become spaces.
fn esc(s: &str) -> String {
    let mut o = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => o.push_str("&amp;"),
            '<' => o.push_str("&lt;"),
            '>' => o.push_str("&gt;"),
            '"' => o.push_str("&quot;"),
            '\t' | '\n' | '\r' => o.push(' '),
            c if (c as u32) < 0x20 || c == '\u{FFFE}' || c == '\u{FFFF}' => o.push(' '),
            c => o.push(c),
        }
    }
    o
}

/// A list of steps, runs of equal steps written OFD's short way (`g 3 5.25`).
fn deltas(v: &[f64]) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut i = 0;
    while let Some(&d) = v.get(i) {
        let s = num(d);
        let mut n = 1;
        while v.get(i + n).is_some_and(|&e| num(e) == s) {
            n += 1;
        }
        if n >= 3 {
            out.push(format!("g {n} {s}"));
        } else {
            for _ in 0..n {
                out.push(s.clone());
            }
        }
        i += n;
    }
    out.join(" ")
}

fn path_data(segs: &[Seg]) -> String {
    let mut s = String::new();
    for g in segs {
        if !s.is_empty() {
            s.push(' ');
        }
        let _ = match *g {
            Seg::Move(x, y) => write!(s, "M {} {}", num(x), num(y)),
            Seg::Line(x, y) => write!(s, "L {} {}", num(x), num(y)),
            Seg::Quad(a, b, x, y) => write!(s, "Q {} {} {} {}", num(a), num(b), num(x), num(y)),
            Seg::Cubic(a, b, c, d, x, y) => write!(
                s,
                "B {} {} {} {} {} {}",
                num(a),
                num(b),
                num(c),
                num(d),
                num(x),
                num(y)
            ),
            Seg::Close => write!(s, "C"),
        };
    }
    s
}

fn color(c: Color) -> String {
    format!("{} {} {}", c.0, c.1, c.2)
}

const NS: &str = "http://www.ofdspec.org/2016";

impl<'a> Ofd<'a> {
    pub fn new(meta: Meta) -> Self {
        Ofd {
            meta,
            pages: Vec::new(),
            fonts: Vec::new(),
            images: Vec::new(),
        }
    }

    /// Registers a font face; the caller keeps one id per face.
    pub fn add_font(&mut self, face: FontFace<'a>) -> FontId {
        self.fonts.push(face);
        FontId(self.fonts.len() - 1)
    }

    pub fn add_image(&mut self, bytes: Vec<u8>, format: ImageFormat) -> ImageId {
        self.images.push((bytes, format));
        ImageId(self.images.len() - 1)
    }

    /// The OFD package.
    pub fn write(&self) -> Result<Vec<u8>, String> {
        if self.pages.is_empty() {
            return Err("no pages".to_string());
        }
        // IDs: 1 the colour space, then fonts, pictures, and per page the page, layer, objects.
        let mut next = 2u64;
        let font_ids: Vec<u64> = self.fonts.iter().map(|_| post_inc(&mut next)).collect();
        let image_ids: Vec<u64> = self.images.iter().map(|_| post_inc(&mut next)).collect();

        // What each font draws: characters with their glyph (the first seen), and every glyph.
        let mut chars: Vec<BTreeMap<char, u32>> = vec![BTreeMap::new(); self.fonts.len()];
        let mut glyphs: Vec<BTreeSet<u32>> = vec![BTreeSet::new(); self.fonts.len()];
        for o in self.pages.iter().flat_map(|p| &p.objects) {
            if let Object::Text(t) = o
                && let (Some(cm), Some(gs)) = (chars.get_mut(t.font.0), glyphs.get_mut(t.font.0))
            {
                for c in &t.codes {
                    gs.extend(c.glyphs.iter().copied());
                    if c.glyphs.len() == c.text.chars().count() {
                        for (ch, g) in c.text.chars().zip(&c.glyphs) {
                            cm.entry(ch).or_insert(*g);
                        }
                    }
                }
            }
        }
        let mut files: Vec<(String, Vec<u8>)> = Vec::new();
        let mut subsets: Vec<Option<font::Subset>> = Vec::new();
        let mut public = String::new();
        let _ = write!(
            public,
            r#"<?xml version="1.0" encoding="UTF-8"?><ofd:Res xmlns:ofd="{NS}" BaseLoc="Res"><ofd:ColorSpaces><ofd:ColorSpace ID="1" Type="RGB" BitsPerComponent="8"/></ofd:ColorSpaces>"#
        );
        if !self.fonts.is_empty() {
            public.push_str("<ofd:Fonts>");
        }
        for (i, f) in self.fonts.iter().enumerate() {
            let id = font_ids.get(i).copied().unwrap_or(0);
            let (Some(cm), Some(gs)) = (chars.get(i), glyphs.get(i)) else {
                continue;
            };
            let gl: Vec<u32> = gs.iter().copied().collect();
            let sub = if gl.is_empty() {
                None
            } else {
                match font::subset(f.data, f.index, &gl, cm) {
                    Ok(s) => Some(s),
                    Err(e) => {
                        log::warn!(
                            "OFD: font {} not embedded ({e}); readers use their own",
                            f.name
                        );
                        None
                    }
                }
            };
            let _ = write!(
                public,
                r#"<ofd:Font ID="{id}" FontName="{}" FamilyName="{}""#,
                esc(&f.name),
                esc(&f.family)
            );
            for (on, attr) in [
                (f.bold, "Bold"),
                (f.italic, "Italic"),
                (f.serif, "Serif"),
                (f.fixed_width, "FixedWidth"),
            ] {
                if on {
                    let _ = write!(public, r#" {attr}="true""#);
                }
            }
            match &sub {
                Some(s) => {
                    let name = format!("font_{id}.ttf");
                    let _ = write!(public, "><ofd:FontFile>{name}</ofd:FontFile></ofd:Font>");
                    files.push((format!("Doc_0/Res/{name}"), s.data.clone()));
                }
                None => public.push_str("/>"),
            }
            subsets.push(sub);
        }
        if !self.fonts.is_empty() {
            public.push_str("</ofd:Fonts>");
        }
        public.push_str("</ofd:Res>");

        let mut docres = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?><ofd:Res xmlns:ofd="{NS}" BaseLoc="Res">"#
        );
        if !self.images.is_empty() {
            docres.push_str("<ofd:MultiMedias>");
            for (i, (bytes, fmt)) in self.images.iter().enumerate() {
                let id = image_ids.get(i).copied().unwrap_or(0);
                let name = format!("image_{id}.{}", fmt.ext());
                let _ = write!(
                    docres,
                    r#"<ofd:MultiMedia ID="{id}" Type="Image"><ofd:MediaFile>{name}</ofd:MediaFile></ofd:MultiMedia>"#
                );
                files.push((format!("Doc_0/Res/{name}"), bytes.clone()));
            }
            docres.push_str("</ofd:MultiMedias>");
        }
        docres.push_str("</ofd:Res>");

        let mut page_list = String::new();
        let mut page_files = Vec::new();
        for (pi, p) in self.pages.iter().enumerate() {
            let pid = post_inc(&mut next);
            let _ = write!(
                page_list,
                r#"<ofd:Page ID="{pid}" BaseLoc="Pages/Page_{pi}/Content.xml"/>"#
            );
            let content = self.page_xml(p, &mut next, &font_ids, &image_ids, &subsets);
            page_files.push((
                format!("Doc_0/Pages/Page_{pi}/Content.xml"),
                content.into_bytes(),
            ));
        }
        let first = self
            .pages
            .first()
            .map_or((210.0, 297.0), |p| (p.width, p.height));
        let max_id = next - 1;
        let document = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?><ofd:Document xmlns:ofd="{NS}"><ofd:CommonData><ofd:MaxUnitID>{max_id}</ofd:MaxUnitID><ofd:PageArea><ofd:PhysicalBox>0 0 {} {}</ofd:PhysicalBox></ofd:PageArea><ofd:PublicRes>PublicRes.xml</ofd:PublicRes><ofd:DocumentRes>DocumentRes.xml</ofd:DocumentRes><ofd:DefaultCS>1</ofd:DefaultCS></ofd:CommonData><ofd:Pages>{page_list}</ofd:Pages></ofd:Document>"#,
            num(first.0),
            num(first.1)
        );
        let ofd = self.ofd_xml(&page_files);

        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        let opts = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        let mut put = |name: &str, data: &[u8]| -> Result<(), String> {
            zip.start_file(name, opts).map_err(|e| e.to_string())?;
            zip.write_all(data).map_err(|e| e.to_string())
        };
        put("OFD.xml", ofd.as_bytes())?;
        put("Doc_0/Document.xml", document.as_bytes())?;
        put("Doc_0/PublicRes.xml", public.as_bytes())?;
        put("Doc_0/DocumentRes.xml", docres.as_bytes())?;
        for (n, d) in page_files.iter().chain(&files) {
            put(n, d)?;
        }
        zip.finish()
            .map(std::io::Cursor::into_inner)
            .map_err(|e| e.to_string())
    }

    fn ofd_xml(&self, pages: &[(String, Vec<u8>)]) -> String {
        let m = &self.meta;
        let id = if m.doc_id.len() == 32 && m.doc_id.chars().all(|c| c.is_ascii_hexdigit()) {
            m.doc_id.clone()
        } else {
            // FNV-1a over the title and the pages' content, twice with different seeds.
            let mut h = [0xcbf2_9ce4_8422_2325u64, 0x8422_2325_cbf2_9ce4u64];
            for (k, hk) in h.iter_mut().enumerate() {
                for b in m
                    .title
                    .bytes()
                    .chain(m.creation_date.bytes())
                    .chain(pages.iter().flat_map(|p| p.1.iter().copied()))
                    .chain([k as u8])
                {
                    *hk = (*hk ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3);
                }
            }
            format!("{:016x}{:016x}", h[0], h[1])
        };
        let mut info = format!("<ofd:DocID>{id}</ofd:DocID>");
        let opt = |info: &mut String, tag: &str, v: &str| {
            if !v.is_empty() {
                let _ = write!(info, "<ofd:{tag}>{}</ofd:{tag}>", esc(v));
            }
        };
        opt(&mut info, "Title", &m.title);
        opt(&mut info, "Author", &m.author);
        opt(&mut info, "Subject", &m.subject);
        opt(&mut info, "CreationDate", &m.creation_date);
        if !m.keywords.is_empty() {
            info.push_str("<ofd:Keywords>");
            for k in &m.keywords {
                let _ = write!(info, "<ofd:Keyword>{}</ofd:Keyword>", esc(k));
            }
            info.push_str("</ofd:Keywords>");
        }
        opt(&mut info, "Creator", &m.creator);
        opt(&mut info, "CreatorVersion", &m.creator_version);
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?><ofd:OFD xmlns:ofd="{NS}" Version="1.0" DocType="OFD"><ofd:DocBody><ofd:DocInfo>{info}</ofd:DocInfo><ofd:DocRoot>Doc_0/Document.xml</ofd:DocRoot></ofd:DocBody></ofd:OFD>"#
        )
    }

    fn page_xml(
        &self,
        p: &Page,
        next: &mut u64,
        font_ids: &[u64],
        image_ids: &[u64],
        subsets: &[Option<font::Subset>],
    ) -> String {
        let bx = format!("0 0 {} {}", num(p.width), num(p.height));
        let mut s = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?><ofd:Page xmlns:ofd="{NS}"><ofd:Area><ofd:PhysicalBox>{bx}</ofd:PhysicalBox></ofd:Area><ofd:Content>"#
        );
        let _ = write!(s, r#"<ofd:Layer ID="{}" Type="Body">"#, post_inc(next));
        for o in &p.objects {
            let id = post_inc(next);
            match o {
                Object::Text(t) => {
                    let Some(fid) = font_ids.get(t.font.0) else {
                        continue;
                    };
                    let _ = write!(
                        s,
                        r#"<ofd:TextObject ID="{id}" Boundary="{bx}" Font="{fid}" Size="{}""#,
                        num(t.size)
                    );
                    common_attrs(&mut s, &t.common);
                    if let Some((_, w)) = t.stroke {
                        let _ = write!(s, r#" Stroke="true" LineWidth="{}""#, num(w));
                    }
                    if t.fill.is_none() {
                        s.push_str(r#" Fill="false""#);
                    }
                    if (t.hscale - 1.0).abs() > 1e-4 && t.hscale.is_finite() && t.hscale > 0.0 {
                        let _ = write!(s, r#" HScale="{}""#, num(t.hscale));
                    }
                    s.push('>');
                    clips(&mut s, &t.common, &bx);
                    if let Some(c) = t.fill {
                        let _ = write!(s, r#"<ofd:FillColor Value="{}"/>"#, color(c));
                    }
                    if let Some((c, _)) = t.stroke {
                        let _ = write!(s, r#"<ofd:StrokeColor Value="{}"/>"#, color(c));
                    }
                    let sub = subsets.get(t.font.0).and_then(Option::as_ref);
                    for c in &t.codes {
                        if let Some(sub) = sub {
                            cg_transforms(&mut s, c, sub);
                        }
                        let _ = write!(s, r#"<ofd:TextCode X="{}" Y="{}""#, num(c.x), num(c.y));
                        if !c.dx.is_empty() {
                            let _ = write!(s, r#" DeltaX="{}""#, deltas(&c.dx));
                        }
                        if c.dy.iter().any(|d| d.abs() > 5e-4) {
                            let _ = write!(s, r#" DeltaY="{}""#, deltas(&c.dy));
                        }
                        let _ = write!(s, ">{}</ofd:TextCode>", esc(&c.text));
                    }
                    s.push_str("</ofd:TextObject>");
                }
                Object::Path(pa) => {
                    let _ = write!(s, r#"<ofd:PathObject ID="{id}" Boundary="{bx}""#);
                    common_attrs(&mut s, &pa.common);
                    match &pa.stroke {
                        Some(st) => {
                            let _ = write!(s, r#" LineWidth="{}""#, num(st.width));
                            if st.cap != Cap::Butt {
                                let _ = write!(
                                    s,
                                    r#" Cap="{}""#,
                                    if st.cap == Cap::Round {
                                        "Round"
                                    } else {
                                        "Square"
                                    }
                                );
                            }
                            if st.join != Join::Miter {
                                let _ = write!(
                                    s,
                                    r#" Join="{}""#,
                                    if st.join == Join::Round {
                                        "Round"
                                    } else {
                                        "Bevel"
                                    }
                                );
                            }
                            if !st.dash.is_empty() {
                                let d: Vec<String> = st.dash.iter().map(|v| num(*v)).collect();
                                let _ = write!(s, r#" DashPattern="{}""#, d.join(" "));
                            }
                        }
                        None => s.push_str(r#" Stroke="false""#),
                    }
                    if pa.fill.is_some() {
                        s.push_str(r#" Fill="true""#);
                        if pa.even_odd {
                            s.push_str(r#" Rule="Even-Odd""#);
                        }
                    }
                    s.push('>');
                    clips(&mut s, &pa.common, &bx);
                    if let Some(st) = &pa.stroke {
                        let _ = write!(s, r#"<ofd:StrokeColor Value="{}"/>"#, color(st.color));
                    }
                    if let Some(f) = &pa.fill {
                        fill_xml(&mut s, f);
                    }
                    let _ = write!(
                        s,
                        "<ofd:AbbreviatedData>{}</ofd:AbbreviatedData></ofd:PathObject>",
                        path_data(&pa.segs)
                    );
                }
                Object::Image(im) => {
                    let Some(rid) = image_ids.get(im.image.0) else {
                        continue;
                    };
                    let _ = write!(
                        s,
                        r#"<ofd:ImageObject ID="{id}" Boundary="{bx}" ResourceID="{rid}""#
                    );
                    let mut c = im.common.clone();
                    c.ctm.get_or_insert([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
                    common_attrs(&mut s, &c);
                    s.push('>');
                    clips(&mut s, &im.common, &bx);
                    s.push_str("</ofd:ImageObject>");
                }
            }
        }
        s.push_str("</ofd:Layer></ofd:Content></ofd:Page>");
        s
    }
}

fn post_inc(n: &mut u64) -> u64 {
    let v = *n;
    *n += 1;
    v
}

fn common_attrs(s: &mut String, c: &Common) {
    if let Some(m) = c.ctm {
        let v: Vec<String> = m
            .iter()
            .map(|x| format!("{}", (x * 1e6).round() / 1e6 + 0.0))
            .collect();
        let _ = write!(s, r#" CTM="{}""#, v.join(" "));
    }
    if c.alpha < 255 {
        let _ = write!(s, r#" Alpha="{}""#, c.alpha);
    }
}

/// Clip areas. An area's path is painted like any path (stroked, not filled, by default), and
/// readers clip to what it paints (WPS does): a clip to the area's inside must say so. Areas are
/// in the object's own space, before its CTM (WPS reads them so), so page-space areas go through
/// the inverse of the object's CTM.
fn clips(s: &mut String, c: &Common, bx: &str) {
    if c.clips.is_empty() {
        return;
    }
    let inverse = match c.ctm {
        Some(m) => match invert(m) {
            Some(i) => Some(i),
            None => return,
        },
        None => None,
    };
    s.push_str("<ofd:Clips>");
    for page_area in &c.clips {
        let mapped;
        let area = match inverse {
            Some(i) => {
                mapped = map_segs(page_area, i);
                &mapped
            }
            None => page_area,
        };
        let _ = write!(
            s,
            r#"<ofd:Clip><ofd:Area><ofd:Path Boundary="{bx}" Stroke="false" Fill="true"><ofd:AbbreviatedData>{}</ofd:AbbreviatedData></ofd:Path></ofd:Area></ofd:Clip>"#,
            path_data(area)
        );
    }
    s.push_str("</ofd:Clips>");
}

/// The inverse of `[a b c d e f]`, None when it is singular.
fn invert([a, b, c, d, e, f]: [f64; 6]) -> Option<[f64; 6]> {
    let det = a * d - b * c;
    if !det.is_finite() || det.abs() < 1e-12 {
        return None;
    }
    let (ia, ib, ic, id) = (d / det, -b / det, -c / det, a / det);
    Some([ia, ib, ic, id, -(ia * e + ic * f), -(ib * e + id * f)])
}

fn map_segs(segs: &[Seg], [a, b, c, d, e, f]: [f64; 6]) -> Vec<Seg> {
    let p = |x: f64, y: f64| (a * x + c * y + e, b * x + d * y + f);
    segs.iter()
        .map(|g| match *g {
            Seg::Move(x, y) => {
                let (x, y) = p(x, y);
                Seg::Move(x, y)
            }
            Seg::Line(x, y) => {
                let (x, y) = p(x, y);
                Seg::Line(x, y)
            }
            Seg::Quad(x1, y1, x, y) => {
                let ((x1, y1), (x, y)) = (p(x1, y1), p(x, y));
                Seg::Quad(x1, y1, x, y)
            }
            Seg::Cubic(x1, y1, x2, y2, x, y) => {
                let ((x1, y1), (x2, y2), (x, y)) = (p(x1, y1), p(x2, y2), p(x, y));
                Seg::Cubic(x1, y1, x2, y2, x, y)
            }
            Seg::Close => Seg::Close,
        })
        .collect()
}

fn fill_xml(s: &mut String, f: &Fill) {
    let segments = |s: &mut String, stops: &[(f64, Color)]| {
        for (o, c) in stops {
            let _ = write!(
                s,
                r#"<ofd:Segment Position="{}"><ofd:Color Value="{}"/></ofd:Segment>"#,
                num(o.clamp(0.0, 1.0)),
                color(*c)
            );
        }
    };
    match f {
        Fill::Solid(c) => {
            let _ = write!(s, r#"<ofd:FillColor Value="{}"/>"#, color(*c));
        }
        Fill::Axial { start, end, stops } => {
            let _ = write!(
                s,
                r#"<ofd:FillColor><ofd:AxialShd StartPoint="{} {}" EndPoint="{} {}" Extend="3" MapType="Direct">"#,
                num(start.0),
                num(start.1),
                num(end.0),
                num(end.1)
            );
            segments(s, stops);
            s.push_str("</ofd:AxialShd></ofd:FillColor>");
        }
        Fill::Radial {
            center,
            radius,
            stops,
        } => {
            let (x, y) = (num(center.0), num(center.1));
            let _ = write!(
                s,
                r#"<ofd:FillColor><ofd:RadialShd StartPoint="{x} {y}" EndPoint="{x} {y}" StartRadius="0" EndRadius="{}" Extend="3" MapType="Direct">"#,
                num(*radius)
            );
            segments(s, stops);
            s.push_str("</ofd:RadialShd></ofd:FillColor>");
        }
    }
}

/// `CGTransform`s for the characters of `c` whose glyph isn't the one the subset's cmap gives:
/// the whole run when glyphs and characters don't pair up, else each such character.
fn cg_transforms(s: &mut String, c: &TextCode, sub: &font::Subset) {
    let n = c.text.chars().count();
    let new = |g: &u32| sub.gids.get(g).copied();
    if c.glyphs.len() != n {
        let gl: Option<Vec<String>> = c
            .glyphs
            .iter()
            .map(|g| new(g).map(|v| v.to_string()))
            .collect();
        if let (Some(gl), false) = (gl, c.glyphs.is_empty() || n == 0) {
            let _ = write!(
                s,
                r#"<ofd:CGTransform CodePosition="0" CodeCount="{n}" GlyphCount="{}"><ofd:Glyphs>{}</ofd:Glyphs></ofd:CGTransform>"#,
                gl.len(),
                gl.join(" ")
            );
        }
        return;
    }
    // The cmap's glyph for a character is the first one recorded for it (see `write`), so a later
    // different glyph for the same character needs its own transform.
    for (i, (ch, g)) in c.text.chars().zip(&c.glyphs).enumerate() {
        let cmap_glyph = sub.cmap_glyph(ch);
        if let Some(ng) = new(g)
            && cmap_glyph != Some(ng)
        {
            let _ = write!(
                s,
                r#"<ofd:CGTransform CodePosition="{i}" CodeCount="1" GlyphCount="1"><ofd:Glyphs>{ng}</ofd:Glyphs></ofd:CGTransform>"#
            );
        }
    }
}

#[cfg(test)]
mod tests;
