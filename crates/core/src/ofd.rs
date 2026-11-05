//! OFD export: each laid-out page's display list (the one WordCraft's PDF export and renderer
//! draw) written through `wenshu-ofd`. Text stays text in embedded font subsets; rules, fills,
//! shapes and paths are vector paths; pictures are embedded (PNG, or JPEG as it is); metafile
//! pictures are drawn as their paths, bitmaps and text.
//!
//! Coordinates: the layout works in points, y down from the page's top-left; OFD in millimetres,
//! y down. Paths and clips are transformed into page millimetres here; text and pictures keep
//! their own space and carry a `CTM` when turned, rotated, slanted or stretched.

use std::collections::HashMap;
use std::sync::Arc;

use kurbo::{Affine, BezPath, PathEl, Point};
use wenshu_ofd as ofd;
use wordcraft_doc::para::ShapeKind;
use wordcraft_doc::{Document, Rgb};
use wordcraft_fonts::FaceRef;
use wordcraft_geom::Rect;
use wordcraft_layout::display::{DisplayOptions, Draw, Paint, Stroke as LineStyle};
use wordcraft_layout::{LayoutCache, LayoutOptions, Page};
use wordcraft_metafile::PlacedItem;

/// Millimetres per point.
const K: f64 = 25.4 / 72.0;

/// Lay `doc` out and write it as OFD.
pub fn export(doc: &Document) -> Result<Vec<u8>, String> {
    let lay = wordcraft_layout::layout(
        doc,
        &mut LayoutCache::new(),
        &LayoutOptions {
            hide_deleted: true,
            ..Default::default()
        },
    );
    if lay.pages.is_empty() {
        return Err("the document has no pages".to_string());
    }
    let mut w = Writer {
        doc,
        out: ofd::Ofd::new(meta(doc)),
        fonts: HashMap::new(),
        cmaps: HashMap::new(),
        images: HashMap::new(),
        vectors: HashMap::new(),
        objects: Vec::new(),
        ctm: Affine::IDENTITY,
        clips: Vec::new(),
    };
    for page in &lay.pages {
        w.page(page);
    }
    w.out.write()
}

fn meta(doc: &Document) -> ofd::Meta {
    let c = &doc.core;
    let date = c
        .created
        .get(..10)
        .filter(|d| d.len() == 10 && d.as_bytes().get(4) == Some(&b'-'))
        .map(str::to_string)
        .unwrap_or_else(today);
    ofd::Meta {
        title: c.title.trim().to_string(),
        author: c.creator.trim().to_string(),
        subject: if c.subject.trim().is_empty() {
            c.description.trim().to_string()
        } else {
            c.subject.trim().to_string()
        },
        keywords: c
            .keywords
            .split([',', ';', '，', '；'])
            .map(|k| k.trim().to_string())
            .filter(|k| !k.is_empty())
            .collect(),
        creator: "Wenshu".to_string(),
        creator_version: env!("CARGO_PKG_VERSION").to_string(),
        creation_date: date,
        doc_id: String::new(),
    }
}

/// Today's date (UTC), `YYYY-MM-DD`.
fn today() -> String {
    let days = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() / 86_400);
    let z = i64::try_from(days).unwrap_or(0) + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}")
}

fn ok(v: f32) -> bool {
    v.is_finite() && v.abs() < 1e6
}

fn color(c: Rgb) -> ofd::Color {
    ofd::Color(c.0, c.1, c.2)
}

fn alpha(a: f32) -> u8 {
    if a.is_finite() {
        (a.clamp(0.0, 1.0) * 255.0).round() as u8
    } else {
        255
    }
}

fn affine(m: [f32; 6]) -> Affine {
    Affine::new(m.map(f64::from))
}

/// A metafile picture, parsed once, with its bitmaps embedded.
struct Vector {
    pic: wordcraft_metafile::Picture,
    images: Vec<Option<ofd::ImageId>>,
}

struct Writer<'d> {
    doc: &'d Document,
    out: ofd::Ofd<'static>,
    fonts: HashMap<u32, ofd::FontId>,
    cmaps: HashMap<u32, Arc<HashMap<u32, char>>>,
    /// Embedded pictures by media key (None: undecodable) with their pixel size.
    images: HashMap<String, Option<ofd::ImageId>>,
    vectors: HashMap<String, Option<Arc<Vector>>>,
    /// The current page's objects.
    objects: Vec<ofd::Object>,
    /// Layout points (the current turn, rotation…) → page points.
    ctm: Affine,
    /// Clip areas in page millimetres.
    clips: Vec<Vec<ofd::Seg>>,
}

impl Writer<'_> {
    fn page(&mut self, page: &Page) {
        let (w, h) = (page.w - page.markup.max(0.0), page.h);
        self.objects.clear();
        self.ctm = Affine::IDENTITY;
        self.clips.clear();
        if let Some(c) = self.doc.settings.page_color {
            self.rect_fill(&Rect::new(0.0, 0.0, w, h), &Paint::Solid(c, 1.0));
        }
        if let Some(pic) = self.doc.settings.page_picture.clone() {
            self.picture(&Rect::new(0.0, 0.0, w, h), &pic.media, &[0.0; 4], 1.0);
        }
        if let Some(wm) = self.doc.settings.watermark.clone()
            && !wm.text.trim().is_empty()
        {
            self.watermark(&wm, w, h, page.body);
        }
        let opts = DisplayOptions {
            marks: false,
            dim_header: false,
            dim_body: false,
            markup: false,
            placeholders: false,
            ..Default::default()
        };
        for d in wordcraft_layout::display::page_display(self.doc, page, &opts) {
            self.draw(&d);
        }
        let w = if ok(w) && w > 1.0 {
            f64::from(w)
        } else {
            595.3
        };
        let h = if ok(h) && h > 1.0 {
            f64::from(h)
        } else {
            841.9
        };
        self.out.pages.push(ofd::Page {
            width: w * K,
            height: h * K,
            objects: std::mem::take(&mut self.objects),
        });
    }

    fn common(&self, a: f32) -> ofd::Common {
        ofd::Common {
            ctm: None,
            clips: self.clips.clone(),
            alpha: alpha(a),
        }
    }

    /// A layout point in page millimetres.
    fn at(&self, x: f64, y: f64) -> (f64, f64) {
        let p = self.ctm * Point::new(x, y);
        (p.x * K, p.y * K)
    }

    /// How much the current transform scales lengths (strokes keep their weight when turned).
    fn scale(&self) -> f64 {
        self.ctm.determinant().abs().sqrt()
    }

    fn segs(&self, path: &BezPath) -> Vec<ofd::Seg> {
        let mut out = Vec::new();
        for el in path.elements() {
            let s = match *el {
                PathEl::MoveTo(p) => {
                    let (x, y) = self.at(p.x, p.y);
                    ofd::Seg::Move(x, y)
                }
                PathEl::LineTo(p) => {
                    let (x, y) = self.at(p.x, p.y);
                    ofd::Seg::Line(x, y)
                }
                PathEl::QuadTo(a, p) => {
                    let ((ax, ay), (x, y)) = (self.at(a.x, a.y), self.at(p.x, p.y));
                    ofd::Seg::Quad(ax, ay, x, y)
                }
                PathEl::CurveTo(a, b, p) => {
                    let ((ax, ay), (bx, by), (x, y)) =
                        (self.at(a.x, a.y), self.at(b.x, b.y), self.at(p.x, p.y));
                    ofd::Seg::Cubic(ax, ay, bx, by, x, y)
                }
                PathEl::ClosePath => ofd::Seg::Close,
            };
            if let ofd::Seg::Move(x, y)
            | ofd::Seg::Line(x, y)
            | ofd::Seg::Quad(_, _, x, y)
            | ofd::Seg::Cubic(_, _, _, _, x, y) = s
                && !(x.is_finite() && y.is_finite() && x.abs() < 1e6 && y.abs() < 1e6)
            {
                return Vec::new();
            }
            out.push(s);
        }
        out
    }

    fn fill_of(&self, p: &Paint) -> (ofd::Fill, f32) {
        let stops = |st: &[(f32, Rgb, f32)]| -> Vec<(f64, ofd::Color)> {
            st.iter()
                .map(|(o, c, _)| (f64::from(*o), color(*c)))
                .collect()
        };
        let most = |st: &[(f32, Rgb, f32)]| st.iter().map(|s| s.2).fold(0.0f32, f32::max);
        match p {
            Paint::Solid(c, a) => (ofd::Fill::Solid(color(*c)), *a),
            Paint::Linear { p0, p1, stops: st } => {
                let (s, e) = (
                    self.at(f64::from(p0.0), f64::from(p0.1)),
                    self.at(f64::from(p1.0), f64::from(p1.1)),
                );
                (
                    ofd::Fill::Axial {
                        start: s,
                        end: e,
                        stops: stops(st),
                    },
                    most(st),
                )
            }
            Paint::Radial { c, r, stops: st } => {
                let center = self.at(f64::from(c.0), f64::from(c.1));
                (
                    ofd::Fill::Radial {
                        center,
                        radius: f64::from(*r) * self.scale() * K,
                        stops: stops(st),
                    },
                    most(st),
                )
            }
        }
    }

    /// A path in layout points: filled and/or stroked (width in points).
    fn path(
        &mut self,
        path: &BezPath,
        fill: Option<&Paint>,
        stroke: Option<(Rgb, f32, f32, Vec<f32>, ofd::Cap, ofd::Join)>,
        even_odd: bool,
    ) {
        if fill.is_none() && stroke.is_none() {
            return;
        }
        let segs = self.segs(path);
        if segs.is_empty() {
            return;
        }
        let fill = fill.map(|p| self.fill_of(p));
        let k = self.scale() * K;
        let a = match (&fill, &stroke) {
            (Some((_, a)), _) => *a,
            (None, Some(s)) => s.2,
            (None, None) => 1.0,
        };
        let stroke = stroke.map(|(c, w, _, dash, cap, join)| ofd::Stroke {
            color: color(c),
            width: f64::from(w) * k,
            dash: dash.iter().map(|d| f64::from(*d) * k).collect(),
            cap,
            join,
        });
        self.objects.push(ofd::Object::Path(ofd::Path {
            common: self.common(a),
            segs,
            fill: fill.map(|f| f.0),
            stroke,
            even_odd,
        }));
    }

    fn rect_fill(&mut self, r: &Rect, p: &Paint) {
        if let Some(b) = rect_bez(r) {
            self.path(&b, Some(p), None, false);
        }
    }

    fn with(&mut self, m: Affine, f: impl FnOnce(&mut Self)) {
        let saved = self.ctm;
        self.ctm = saved * m;
        f(self);
        self.ctm = saved;
    }

    fn clipped(&mut self, r: &Rect, f: impl FnOnce(&mut Self)) {
        let Some(b) = rect_bez(r) else { return };
        let area = self.segs(&b);
        if area.is_empty() {
            return;
        }
        self.clips.push(area);
        f(self);
        self.clips.pop();
    }

    fn draw(&mut self, d: &Draw) {
        match d {
            Draw::Figure { draws, .. } => {
                for d in draws {
                    self.draw(d);
                }
            }
            Draw::Glyphs {
                face,
                size,
                glyphs,
                color: c,
                alpha: a,
                synth_bold,
                synth_italic,
                text,
                ranges,
                x_scale,
                ..
            } => {
                let style = Style {
                    color: *c,
                    alpha: *a,
                    bold: *synth_bold,
                    italic: *synth_italic,
                };
                match glyphs.first().map(|g| g.1).filter(|_| {
                    x_scale.is_finite() && *x_scale > 0.0 && (x_scale - 1.0).abs() > 1e-4
                }) {
                    // Scaled letters (`w:w`): stretched from the run's start, positions brought back.
                    Some(x0) => {
                        let k = *x_scale;
                        let moved: Vec<(u32, f32, f32)> = glyphs
                            .iter()
                            .map(|&(g, x, y)| (g, x0 + (x - x0) / k, y))
                            .collect();
                        let m = Affine::new([
                            f64::from(k),
                            0.0,
                            0.0,
                            1.0,
                            f64::from(x0 * (1.0 - k)),
                            0.0,
                        ]);
                        self.with(m, |me| me.glyphs(face, *size, &moved, style, text, ranges));
                    }
                    None => self.glyphs(face, *size, glyphs, style, text, ranges),
                }
            }
            Draw::Fill {
                rect,
                color: c,
                alpha: a,
                ..
            } => self.rect_fill(rect, &Paint::Solid(*c, *a)),
            Draw::Line {
                x0,
                y0,
                x1,
                y1,
                width,
                color: c,
                stroke,
                alpha: a,
                ..
            } => self.line((*x0, *y0, *x1, *y1), *width, *c, *stroke, *a),
            Draw::Image {
                rect,
                media,
                crop,
                alpha: a,
                ..
            } => self.picture(rect, media, crop, *a),
            Draw::Path {
                segs,
                fill,
                stroke,
                stroke_width,
                dash,
                cap,
            } => {
                let b = wordcraft_layout::display::seg_path(segs);
                let w = if stroke_width.is_finite() && *stroke_width > 0.0 {
                    stroke_width.clamp(0.25, 200.0)
                } else {
                    0.75
                };
                let st = stroke.map(|c| (c, w, 1.0, dash.clone(), cap_of(*cap), ofd::Join::Miter));
                self.path(&b, fill.as_ref(), st, false);
            }
            Draw::Ink {
                pts,
                color: c,
                width,
                alpha: a,
            } => {
                let mut b = BezPath::new();
                let mut used = 0usize;
                let mut last = Point::ZERO;
                for &(x, y) in pts.iter().filter(|(x, y)| ok(*x) && ok(*y)) {
                    let p = Point::new(f64::from(x), f64::from(y));
                    if used > 0 {
                        b.line_to(p)
                    } else {
                        b.move_to(p)
                    }
                    used += 1;
                    last = p;
                }
                if used == 1 {
                    b.line_to(last);
                }
                if used > 0 {
                    let w = if width.is_finite() {
                        width.clamp(0.25, 200.0)
                    } else {
                        1.0
                    };
                    self.path(
                        &b,
                        None,
                        Some((*c, w, *a, Vec::new(), ofd::Cap::Round, ofd::Join::Round)),
                        false,
                    );
                }
            }
            Draw::Shape {
                rect,
                kind,
                fill,
                stroke,
                stroke_width,
                effects,
                corner,
                dash,
                cap,
            } => self.shape(
                rect,
                *kind,
                *corner,
                fill.as_ref(),
                *stroke,
                *stroke_width,
                effects,
                dash,
                *cap,
            ),
            Draw::Art {
                path,
                fx,
                color: c,
                alpha: a,
                ..
            } => {
                for l in wordcraft_layout::wordart::art_layers(path, fx, *c, *a) {
                    match l.stroke {
                        Some(w) => {
                            let (fill, a) = match &l.paint {
                                Paint::Solid(c, a) => (*c, *a),
                                Paint::Linear { stops, .. } | Paint::Radial { stops, .. } => {
                                    stops.first().map_or((Rgb(0, 0, 0), 1.0), |s| (s.1, s.2))
                                }
                            };
                            self.path(
                                &l.path,
                                None,
                                Some((
                                    fill,
                                    w.clamp(0.1, 400.0),
                                    a,
                                    Vec::new(),
                                    ofd::Cap::Butt,
                                    ofd::Join::Round,
                                )),
                                false,
                            );
                        }
                        None => self.path(&l.path, Some(&l.paint), None, false),
                    }
                }
            }
            Draw::Mark { .. } | Draw::MarkText { .. } => {}
            Draw::Clip { rect, items } => self.clipped(rect, |me| {
                for it in items {
                    me.draw(it);
                }
            }),
            Draw::Turned { x, y, turn, items } => {
                if ok(*x) && ok(*y) {
                    self.with(affine(Draw::turn_matrix(*turn, *x, *y)), |me| {
                        for it in items {
                            me.draw(it);
                        }
                    });
                }
            }
            Draw::Rotated {
                cx,
                cy,
                spin,
                items,
            } => {
                if ok(*cx) && ok(*cy) {
                    self.with(affine(spin.matrix(*cx, *cy)), |me| {
                        for it in items {
                            me.draw(it);
                        }
                    });
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn shape(
        &mut self,
        rect: &Rect,
        kind: ShapeKind,
        corner: Option<u32>,
        fill: Option<&Paint>,
        stroke: Option<Rgb>,
        stroke_width: f32,
        effects: &wordcraft_doc::effects::ShapeEffects,
        dash: &[f32],
        cap: wordcraft_doc::connector::LineCap,
    ) {
        let can_fill = kind != ShapeKind::Line;
        let fill = fill.filter(|_| can_fill);
        let sw = if stroke_width.is_finite() {
            stroke_width.clamp(0.25, 200.0)
        } else {
            0.75
        };
        let fx = effects.sanitized();
        // Shadow and glow: bands of the silhouette behind the shape, as the PDF export draws them.
        if !fx.is_empty() && (fill.is_some() || stroke.is_some()) {
            let silhouette = |me: &mut Self, c: Rgb, a: f32, g: f32, dx: f32, dy: f32| {
                if fill.is_some() {
                    let grow = g + if stroke.is_some() { sw / 2.0 } else { 0.0 };
                    if let Some(p) = shape_bez(kind, &grown(rect, grow, dx, dy), corner) {
                        me.path(&p, Some(&Paint::Solid(c, a)), None, false);
                    }
                } else if sw + 2.0 * g > 0.05
                    && let Some(p) = shape_bez(kind, &grown(rect, 0.0, dx, dy), corner)
                {
                    me.path(
                        &p,
                        None,
                        Some((
                            c,
                            sw + 2.0 * g,
                            a,
                            Vec::new(),
                            ofd::Cap::Butt,
                            ofd::Join::Miter,
                        )),
                        false,
                    );
                }
            };
            if let Some(sh) = fx.shadow {
                let (dx, dy) = sh.offset();
                for (g, a) in
                    wordcraft_doc::effects::bands(-sh.blur / 2.0, sh.blur / 2.0, sh.opacity())
                {
                    silhouette(self, sh.color, a, g, dx, dy);
                }
            }
            if let Some(gl) = fx.glow {
                for (g, a) in wordcraft_doc::effects::bands(0.0, gl.size, gl.opacity()) {
                    silhouette(self, gl.color, a, g, 0.0, 0.0);
                }
            }
        }
        if let (Some(rad), Some(c)) = (fx.soft_edge, fill) {
            let rad = rad.min(rect.w.min(rect.h) / 2.0).max(0.0);
            for (g, a) in wordcraft_doc::effects::bands(-rad, 0.0, 1.0) {
                if let Some(p) = shape_bez(kind, &grown(rect, g, 0.0, 0.0), corner) {
                    self.path(&p, Some(&c.faded(a)), None, false);
                }
            }
            return;
        }
        let Some(p) = shape_bez(kind, rect, corner) else {
            return;
        };
        let st = stroke.map(|c| (c, sw, 1.0, dash.to_vec(), cap_of(cap), ofd::Join::Miter));
        self.path(&p, fill, st, false);
    }

    fn line(
        &mut self,
        (x0, y0, x1, y1): (f32, f32, f32, f32),
        width: f32,
        c: Rgb,
        style: LineStyle,
        a: f32,
    ) {
        if ![x0, y0, x1, y1].iter().all(|v| ok(*v)) {
            return;
        }
        let w = if width.is_finite() {
            width.clamp(0.1, 100.0)
        } else {
            0.5
        };
        let seg = |me: &mut Self, dx: f32, dy: f32, width: f32, dash: Vec<f32>| {
            let mut b = BezPath::new();
            b.move_to((f64::from(x0 + dx), f64::from(y0 + dy)));
            b.line_to((f64::from(x1 + dx), f64::from(y1 + dy)));
            me.path(
                &b,
                None,
                Some((c, width, a, dash, ofd::Cap::Butt, ofd::Join::Miter)),
                false,
            );
        };
        let (dx, dy) = (x1 - x0, y1 - y0);
        let len = (dx * dx + dy * dy).sqrt();
        match style {
            LineStyle::Solid => seg(self, 0.0, 0.0, w, Vec::new()),
            LineStyle::Dotted
            | LineStyle::Dashed
            | LineStyle::BorderDotted
            | LineStyle::BorderDashed
            | LineStyle::BorderDotDash => {
                let dash = style
                    .dashes()
                    .unwrap_or(&[])
                    .iter()
                    .map(|k| k * w)
                    .collect();
                seg(self, 0.0, 0.0, w, dash);
            }
            LineStyle::Double => {
                let l = len.max(1e-3);
                let (nx, ny) = (-dy / l * w * 0.9, dx / l * w * 0.9);
                seg(self, nx, ny, w * 0.6, Vec::new());
                seg(self, -nx, -ny, w * 0.6, Vec::new());
            }
            LineStyle::Triple => {
                let l = len.max(1e-3);
                let (nx, ny) = (-dy / l * w * 1.8, dx / l * w * 1.8);
                seg(self, nx, ny, w * 0.6, Vec::new());
                seg(self, 0.0, 0.0, w * 0.6, Vec::new());
                seg(self, -nx, -ny, w * 0.6, Vec::new());
            }
            LineStyle::Wave => {
                if len < 1e-3 {
                    return;
                }
                let amp = (w * 1.2).max(0.8);
                let step = amp * 2.0;
                let (ux, uy) = (dx / len, dy / len);
                let (nx, ny) = (-uy, ux);
                let mut b = BezPath::new();
                b.move_to((f64::from(x0), f64::from(y0)));
                let (mut t, mut up) = (0.0f32, true);
                while t < len && t < 20_000.0 {
                    let nt = (t + step).min(len);
                    let mid = (t + nt) / 2.0;
                    let sign = if up { -1.0 } else { 1.0 };
                    b.quad_to(
                        (
                            f64::from(x0 + ux * mid + nx * amp * sign),
                            f64::from(y0 + uy * mid + ny * amp * sign),
                        ),
                        (f64::from(x0 + ux * nt), f64::from(y0 + uy * nt)),
                    );
                    t = nt;
                    up = !up;
                }
                self.path(
                    &b,
                    None,
                    Some((c, w * 0.8, a, Vec::new(), ofd::Cap::Round, ofd::Join::Miter)),
                    false,
                );
            }
        }
    }

    fn font(&mut self, face: &FaceRef) -> ofd::FontId {
        let out = &mut self.out;
        *self.fonts.entry(face.id()).or_insert_with(|| {
            let f = face.get();
            out.add_font(ofd::FontFace {
                name: f.family.clone(),
                family: f.family.clone(),
                bold: f.weight >= 600.0,
                italic: f.italic,
                serif: false,
                fixed_width: false,
                data: f.data(),
                index: f.index(),
            })
        })
    }

    fn cmap(&mut self, face: &FaceRef) -> Arc<HashMap<u32, char>> {
        self.cmaps
            .entry(face.id())
            .or_insert_with(|| {
                let mut m = HashMap::new();
                for (c, g) in face.chars() {
                    m.entry(g).or_insert(c);
                }
                Arc::new(m)
            })
            .clone()
    }

    /// A run of glyphs (layout points, in the current space) as a text object.
    fn glyphs(
        &mut self,
        face: &FaceRef,
        size: f32,
        glyphs: &[(u32, f32, f32)],
        style: Style,
        text: &str,
        known: &[std::ops::Range<usize>],
    ) {
        if !size.is_finite()
            || size <= 0.0
            || size > 5000.0
            || glyphs.is_empty()
            || !glyphs.iter().all(|(_, x, y)| ok(*x) && ok(*y))
        {
            return;
        }
        // Each glyph's characters: the layout's mapping when it has one, else the font's cmap.
        let valid = known.len() == glyphs.len()
            && known
                .iter()
                .all(|r| r.start < r.end && text.get(r.clone()).is_some());
        let per_glyph: Vec<String> = if valid {
            known
                .iter()
                .map(|r| text.get(r.clone()).unwrap_or_default().to_string())
                .collect()
        } else {
            // One glyph a character: pair them (glyphs the font has no character for, such as a
            // missing glyph's box, keep the text they stand for); else the font's cmap.
            let chars: Vec<char> = text.chars().collect();
            let cmap = self.cmap(face);
            glyphs
                .iter()
                .enumerate()
                .map(|(i, (g, _, _))| {
                    let c = if chars.len() == glyphs.len() {
                        chars.get(i).copied()
                    } else {
                        cmap.get(g).copied().or_else(|| chars.get(i).copied())
                    };
                    c.unwrap_or('\u{FFFD}').to_string()
                })
                .collect()
        };
        let mut codes: Vec<ofd::TextCode> = Vec::new();
        let mut cur: Option<ofd::TextCode> = None;
        let mut prev: Option<(f64, f64)> = None;
        let mut i = 0;
        while i < glyphs.len() {
            // A cluster: the glyphs sharing one range of the text.
            let mut j = i + 1;
            if valid {
                while j < glyphs.len() && known.get(j) == known.get(i) {
                    j += 1;
                }
            }
            let chars = per_glyph.get(i).cloned().unwrap_or_default();
            let Some(&(_, gx, gy)) = glyphs.get(i) else {
                break;
            };
            let (x, y) = (f64::from(gx) * K, f64::from(gy) * K);
            let ids: Vec<u32> = glyphs
                .get(i..j)
                .unwrap_or_default()
                .iter()
                .map(|g| g.0)
                .collect();
            if chars.chars().count() == 1 && ids.len() == 1 {
                match (&mut cur, prev) {
                    (Some(c), Some((px, py))) => {
                        c.dx.push(x - px);
                        c.dy.push(y - py);
                        c.text.push_str(&chars);
                        c.glyphs.extend(ids);
                    }
                    _ => {
                        cur = Some(ofd::TextCode {
                            x,
                            y,
                            text: chars,
                            glyphs: ids,
                            dx: Vec::new(),
                            dy: Vec::new(),
                        })
                    }
                }
                prev = Some((x, y));
            } else if !chars.is_empty() {
                codes.extend(cur.take());
                prev = None;
                let n = chars.chars().count();
                codes.push(ofd::TextCode {
                    x,
                    y,
                    text: chars,
                    glyphs: ids,
                    dx: vec![0.0; n.saturating_sub(1)],
                    dy: Vec::new(),
                });
            }
            i = j;
        }
        codes.extend(cur);
        for c in &mut codes {
            if c.dy.iter().all(|d| d.abs() < 5e-4) {
                c.dy.clear();
            }
        }
        if codes.is_empty() {
            return;
        }
        let font = self.font(face);
        let mut m = self.ctm;
        if style.italic {
            let base = baseline(glyphs).unwrap_or(glyphs.first().map_or(0.0, |g| g.2));
            m *= Affine::new([1.0, 0.0, -0.21, 1.0, 0.21 * f64::from(base), 0.0]);
        }
        let c = m.as_coeffs();
        let ctm = (m != Affine::IDENTITY).then(|| [c[0], c[1], c[2], c[3], c[4] * K, c[5] * K]);
        let size_mm = f64::from(size) * K;
        self.objects.push(ofd::Object::Text(ofd::Text {
            common: ofd::Common {
                ctm,
                clips: self.clips.clone(),
                alpha: alpha(style.alpha),
            },
            font,
            size: size_mm,
            fill: Some(color(style.color)),
            stroke: style.bold.then(|| (color(style.color), size_mm * 0.03)),
            hscale: 1.0,
            codes,
        }));
    }

    fn image(&mut self, key: &str) -> Option<ofd::ImageId> {
        if let Some(i) = self.images.get(key) {
            return *i;
        }
        let id = self
            .doc
            .media
            .get(key)
            .and_then(encode)
            .map(|(bytes, fmt)| self.out.add_image(bytes, fmt));
        if id.is_none() {
            log::warn!("OFD: picture `{key}` could not be decoded");
        }
        self.images.insert(key.to_string(), id);
        id
    }

    fn vector(&mut self, key: &str) -> Option<Arc<Vector>> {
        let bytes = self.doc.media.get(key)?;
        if !wordcraft_metafile::is_metafile(bytes) {
            return None;
        }
        if let Some(v) = self.vectors.get(key) {
            return v.clone();
        }
        let v = wordcraft_metafile::parse(bytes)
            .ok()
            .filter(wordcraft_metafile::Picture::has_size)
            .map(|mut pic| {
                let images = pic
                    .items
                    .iter_mut()
                    .map(|it| {
                        it.take_pixels()
                            .and_then(|(w, h, px)| png(w, h, px))
                            .map(|b| self.out.add_image(b, ofd::ImageFormat::Png))
                    })
                    .collect();
                Arc::new(Vector { pic, images })
            });
        self.vectors.insert(key.to_string(), v.clone());
        v
    }

    /// An image placed over the layout rectangle `r` (points) in the current space.
    fn place_image(&mut self, id: ofd::ImageId, r: kurbo::Rect, a: f32) {
        let m = self.ctm * Affine::new([r.width(), 0.0, 0.0, r.height(), r.x0, r.y0]);
        let c = m.as_coeffs().map(|v| v * K);
        if c.iter().all(|v| v.is_finite()) {
            let common = ofd::Common {
                ctm: Some(c),
                clips: self.clips.clone(),
                alpha: alpha(a),
            };
            self.objects
                .push(ofd::Object::Image(ofd::Image { common, image: id }));
        }
    }

    fn picture(&mut self, rect: &Rect, media: &str, crop: &[f32; 4], a: f32) {
        if !(ok(rect.x) && ok(rect.y) && ok(rect.w) && ok(rect.h)) || rect.w <= 0.0 || rect.h <= 0.0
        {
            return;
        }
        if let Some(v) = self.vector(media) {
            let target = kurbo::Rect::new(
                f64::from(rect.x),
                f64::from(rect.y),
                f64::from(rect.x + rect.w),
                f64::from(rect.y + rect.h),
            );
            self.clipped(rect, |me| {
                for it in v.pic.place(target, *crop) {
                    match it {
                        PlacedItem::Path {
                            path,
                            fill,
                            stroke,
                            even_odd,
                        } => {
                            let fill = fill.map(|c| {
                                Paint::Solid(Rgb(c[0], c[1], c[2]), f32::from(c[3]) / 255.0 * a)
                            });
                            let stroke = stroke.map(|(c, w)| {
                                (
                                    Rgb(c[0], c[1], c[2]),
                                    (w.max(0.25)) as f32,
                                    f32::from(c[3]) / 255.0 * a,
                                    Vec::new(),
                                    ofd::Cap::Round,
                                    ofd::Join::Round,
                                )
                            });
                            me.path(&path, fill.as_ref(), stroke, even_odd);
                        }
                        PlacedItem::Bitmap { index, rect } => {
                            if let Some(Some(id)) = v.images.get(index) {
                                me.place_image(*id, rect, a);
                            }
                        }
                        PlacedItem::Text { index, tf } => {
                            if let Some(wordcraft_metafile::Item::Text(t)) = v.pic.items.get(index)
                            {
                                me.with(tf, |me| me.vector_text(t, a));
                            }
                        }
                    }
                }
            });
            return;
        }
        let Some(id) = self.image(media) else {
            self.rect_fill(rect, &Paint::Solid(Rgb(0xD0, 0xD0, 0xD0), 1.0));
            return;
        };
        let [cl, ct, vis_w, vis_h] = wordcraft_metafile::crop_window(*crop);
        let (fw, fh) = (f64::from(rect.w) / vis_w, f64::from(rect.h) / vis_h);
        let (x, y) = (f64::from(rect.x) - cl * fw, f64::from(rect.y) - ct * fh);
        let frame = kurbo::Rect::new(x, y, x + fw, y + fh);
        if crop.iter().any(|c| c.is_finite() && *c != 0.0) {
            self.clipped(rect, |me| me.place_image(id, frame, a));
        } else {
            self.place_image(id, frame, a);
        }
    }

    /// A metafile text run, in its own space (see `PlacedItem::Text`).
    fn vector_text(&mut self, t: &wordcraft_metafile::Text, a: f32) {
        let face = wordcraft_fonts::word::resolve(&t.font, t.bold, t.italic).face;
        let shaped = wordcraft_fonts::shape(&face, &t.text, &[], |c| c);
        let g: Vec<_> = shaped
            .iter()
            .map(|g| {
                (
                    g.cluster,
                    f64::from(g.x_advance),
                    f64::from(g.x_offset),
                    f64::from(g.y_offset),
                )
            })
            .collect();
        let Some(lay) = t.layout(face.upem, face.ascent, face.descent, &g) else {
            return;
        };
        let glyphs: Vec<(u32, f32, f32)> = shaped
            .iter()
            .zip(&lay.glyphs)
            .map(|(g, (x, y))| (g.gid, *x as f32, *y as f32))
            .collect();
        let ranges = cluster_ranges(
            &shaped.iter().map(|g| g.cluster).collect::<Vec<_>>(),
            &t.text,
        );
        let style = Style {
            color: Rgb(t.color[0], t.color[1], t.color[2]),
            alpha: f32::from(t.color[3]) / 255.0 * a,
            bold: false,
            italic: false,
        };
        self.glyphs(&face, lay.em as f32, &glyphs, style, &t.text, &ranges);
    }

    /// The watermark text stretched over its box, as the PDF export draws it.
    fn watermark(&mut self, wm: &wordcraft_doc::Watermark, w: f32, h: f32, area: Rect) {
        let face = wordcraft_fonts::word::resolve(&wm.font, false, false).face;
        let text: String = wm.text.chars().take(200).collect();
        let shaped = wordcraft_fonts::shape(&face, &text, &[], |c| c);
        let upem = face.upem.max(1.0) as f32;
        let raw_w: f32 = shaped.iter().map(|g| g.x_advance as f32).sum::<f32>() / upem;
        if raw_w <= 0.0 || !raw_w.is_finite() {
            return;
        }
        let [bw, bh] = wm.frame(w, h);
        let size = bh.min(2000.0);
        let stretch = bw / (raw_w * size);
        let descent = (wordcraft_fonts::word::line_metrics(&face).1 / face.upem.max(1.0)) as f32;
        let angle = if wm.diagonal {
            -std::f32::consts::FRAC_PI_4
        } else {
            0.0
        };
        let (c, sn) = (angle.cos(), angle.sin());
        let (dx, dy) = (-bw / 2.0, bh / 2.0 - descent.clamp(0.0, 1.0) * bh);
        let (cx, cy) = if area.w > 0.0 && area.h > 0.0 {
            (area.x + area.w / 2.0, area.y + area.h / 2.0)
        } else {
            (w / 2.0, h.min(2000.0) / 2.0)
        };
        let k = size / upem;
        let mut x = 0.0;
        let mut glyphs = Vec::with_capacity(shaped.len());
        for g in &shaped {
            glyphs.push((g.gid, x + g.x_offset as f32 * k, -(g.y_offset as f32) * k));
            x += g.x_advance as f32 * k;
        }
        let ranges = cluster_ranges(&shaped.iter().map(|g| g.cluster).collect::<Vec<_>>(), &text);
        let m = affine([
            c * stretch,
            sn * stretch,
            -sn,
            c,
            c * dx - sn * dy + cx,
            sn * dx + c * dy + cy,
        ]);
        let style = Style {
            color: wm.color,
            alpha: if wm.semitransparent { 0.5 } else { 1.0 },
            bold: false,
            italic: false,
        };
        self.with(m, |me| {
            me.glyphs(&face, size, &glyphs, style, &text, &ranges)
        });
    }
}

#[derive(Clone, Copy)]
struct Style {
    color: Rgb,
    alpha: f32,
    bold: bool,
    italic: bool,
}

/// Each shaped glyph's byte range of `text`, from the glyphs' cluster starts.
fn cluster_ranges(clusters: &[usize], text: &str) -> Vec<std::ops::Range<usize>> {
    let mut starts: Vec<usize> = clusters.to_vec();
    starts.sort_unstable();
    starts.dedup();
    clusters
        .iter()
        .map(|&s| {
            let end = starts
                .iter()
                .copied()
                .find(|&e| e > s)
                .unwrap_or(text.len());
            s.min(end)..end
        })
        .collect()
}

/// The y most of a run's glyphs share (marks sit higher or lower).
fn baseline(glyphs: &[(u32, f32, f32)]) -> Option<f32> {
    let mut counts: Vec<(f32, usize)> = Vec::new();
    for &(_, _, y) in glyphs {
        match counts.iter_mut().find(|(v, _)| (v - y).abs() < 0.01) {
            Some(c) => c.1 += 1,
            None => counts.push((y, 1)),
        }
    }
    counts.into_iter().max_by_key(|c| c.1).map(|c| c.0)
}

fn cap_of(c: wordcraft_doc::connector::LineCap) -> ofd::Cap {
    match c {
        wordcraft_doc::connector::LineCap::Flat => ofd::Cap::Butt,
        wordcraft_doc::connector::LineCap::Square => ofd::Cap::Square,
        wordcraft_doc::connector::LineCap::Round => ofd::Cap::Round,
    }
}

fn grown(r: &Rect, g: f32, dx: f32, dy: f32) -> Rect {
    Rect {
        x: r.x - g + dx,
        y: r.y - g + dy,
        w: r.w + 2.0 * g,
        h: r.h + 2.0 * g,
    }
}

fn rect_bez(r: &Rect) -> Option<BezPath> {
    if !(ok(r.x) && ok(r.y) && ok(r.w) && ok(r.h)) || r.w <= 0.0 || r.h <= 0.0 {
        return None;
    }
    let (x0, y0, x1, y1) = (
        f64::from(r.x),
        f64::from(r.y),
        f64::from(r.x + r.w),
        f64::from(r.y + r.h),
    );
    let mut b = BezPath::new();
    b.move_to((x0, y0));
    b.line_to((x1, y0));
    b.line_to((x1, y1));
    b.line_to((x0, y1));
    b.close_path();
    Some(b)
}

/// A basic shape's outline in its rectangle: the same geometry as WordCraft's renderer and PDF
/// export (`wordcraft-pdf`'s `shape_path`).
fn shape_bez(kind: ShapeKind, r: &Rect, corner: Option<u32>) -> Option<BezPath> {
    let flat_ok = kind.is_open() && r.w.max(r.h) > 0.0 && r.w.min(r.h) >= 0.0;
    if !(ok(r.x) && ok(r.y) && ok(r.w) && ok(r.h)) || !(flat_ok || r.w > 0.0 && r.h > 0.0) {
        return None;
    }
    let f = f64::from;
    let (x0, y0, x1, y1) = (f(r.x), f(r.y), f(r.x + r.w), f(r.y + r.h));
    let (w, h) = (f(r.w), f(r.h));
    let (cx, cy) = (x0 + w / 2.0, y0 + h / 2.0);
    let mut b = BezPath::new();
    let poly = |b: &mut BezPath, pts: &[(f64, f64)]| {
        for (i, p) in pts.iter().enumerate() {
            if i == 0 { b.move_to(*p) } else { b.line_to(*p) }
        }
        b.close_path();
    };
    const KAPPA: f64 = 0.552_284_8;
    let ellipse = |b: &mut BezPath, cx: f64, cy: f64, rx: f64, ry: f64| {
        b.move_to((cx + rx, cy));
        b.curve_to(
            (cx + rx, cy + ry * KAPPA),
            (cx + rx * KAPPA, cy + ry),
            (cx, cy + ry),
        );
        b.curve_to(
            (cx - rx * KAPPA, cy + ry),
            (cx - rx, cy + ry * KAPPA),
            (cx - rx, cy),
        );
        b.curve_to(
            (cx - rx, cy - ry * KAPPA),
            (cx - rx * KAPPA, cy - ry),
            (cx, cy - ry),
        );
        b.curve_to(
            (cx + rx * KAPPA, cy - ry),
            (cx + rx, cy - ry * KAPPA),
            (cx + rx, cy),
        );
        b.close_path();
    };
    match kind {
        ShapeKind::Rectangle | ShapeKind::TextBox | ShapeKind::Freeform => {
            poly(&mut b, &[(x0, y0), (x1, y0), (x1, y1), (x0, y1)])
        }
        ShapeKind::RoundedRectangle => {
            let rad = f(wordcraft_doc::connector::corner_radius(corner, r.w, r.h));
            let k = rad * (1.0 - KAPPA);
            b.move_to((x0 + rad, y0));
            b.line_to((x1 - rad, y0));
            b.curve_to((x1 - k, y0), (x1, y0 + k), (x1, y0 + rad));
            b.line_to((x1, y1 - rad));
            b.curve_to((x1, y1 - k), (x1 - k, y1), (x1 - rad, y1));
            b.line_to((x0 + rad, y1));
            b.curve_to((x0 + k, y1), (x0, y1 - k), (x0, y1 - rad));
            b.line_to((x0, y0 + rad));
            b.curve_to((x0, y0 + k), (x0 + k, y0), (x0 + rad, y0));
            b.close_path();
        }
        ShapeKind::Ellipse => ellipse(&mut b, cx, cy, w / 2.0, h / 2.0),
        ShapeKind::FlowchartOr => {
            ellipse(&mut b, cx, cy, w / 2.0, h / 2.0);
            b.move_to((x0, cy));
            b.line_to((x1, cy));
            b.move_to((cx, y0));
            b.line_to((cx, y1));
        }
        ShapeKind::Triangle => poly(&mut b, &[(cx, y0), (x1, y1), (x0, y1)]),
        ShapeKind::RightTriangle => poly(&mut b, &[(x0, y0), (x1, y1), (x0, y1)]),
        ShapeKind::Diamond => poly(&mut b, &[(cx, y0), (x1, cy), (cx, y1), (x0, cy)]),
        ShapeKind::Line
        | ShapeKind::StraightConnector
        | ShapeKind::ElbowConnector
        | ShapeKind::CurvedConnector => {
            b.move_to((x0, y0));
            b.line_to((x1, y1));
        }
        ShapeKind::Arrow | ShapeKind::ArrowLeft | ShapeKind::ArrowUp | ShapeKind::ArrowDown => {
            let pts: Vec<(f64, f64)> = wordcraft_doc::wrap::arrow_outline(kind, r.w, r.h)
                .iter()
                .map(|&(u, v)| (x0 + f(u) * w, y0 + f(v) * h))
                .collect();
            poly(&mut b, &pts);
        }
        ShapeKind::Star => {
            let pts: Vec<(f64, f64)> = (0..10)
                .map(|i| {
                    let a = std::f64::consts::PI * f64::from(i) / 5.0 - std::f64::consts::FRAC_PI_2;
                    let rad = if i % 2 == 0 { 1.0 } else { 0.4 };
                    (cx + a.cos() * w / 2.0 * rad, cy + a.sin() * h / 2.0 * rad)
                })
                .collect();
            poly(&mut b, &pts);
        }
        ShapeKind::Heart => {
            b.move_to((cx, y0 + h * 0.3));
            b.curve_to((cx, y0), (x0, y0), (x0, y0 + h * 0.3));
            b.curve_to((x0, y0 + h * 0.6), (cx - w * 0.1, y0 + h * 0.75), (cx, y1));
            b.curve_to(
                (cx + w * 0.1, y0 + h * 0.75),
                (x1, y0 + h * 0.6),
                (x1, y0 + h * 0.3),
            );
            b.curve_to((x1, y0), (cx, y0), (cx, y0 + h * 0.3));
            b.close_path();
        }
    }
    Some(b)
}

/// Largest picture side we embed (pixels).
const MAX_SIDE: u32 = 30_000;

/// A picture as OFD readers take it: a JPEG in RGB or grey as it is, anything else as PNG.
fn encode(bytes: &Arc<Vec<u8>>) -> Option<(Vec<u8>, ofd::ImageFormat)> {
    if !wordcraft_geom::picture::probe(bytes)
        .is_ok_and(|p| p.fits(wordcraft_geom::MAX_DECODED_BYTES))
    {
        return None;
    }
    let fmt = image::guess_format(bytes).ok()?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(MAX_SIDE);
    limits.max_image_height = Some(MAX_SIDE);
    let mut r = image::ImageReader::new(std::io::Cursor::new(bytes.as_slice()))
        .with_guessed_format()
        .ok()?;
    r.limits(limits);
    let img = r.decode().ok()?;
    if fmt == image::ImageFormat::Jpeg
        && matches!(img.color(), image::ColorType::L8 | image::ColorType::Rgb8)
    {
        return Some((bytes.as_ref().clone(), ofd::ImageFormat::Jpeg));
    }
    if fmt == image::ImageFormat::Png {
        return Some((bytes.as_ref().clone(), ofd::ImageFormat::Png));
    }
    let (w, h) = (img.width(), img.height());
    png(w, h, img.into_rgba8().into_raw()).map(|b| (b, ofd::ImageFormat::Png))
}

fn png(w: u32, h: u32, rgba: Vec<u8>) -> Option<Vec<u8>> {
    let img = image::RgbaImage::from_raw(w, h, rgba)?;
    let mut out = std::io::Cursor::new(Vec::new());
    img.write_to(&mut out, image::ImageFormat::Png).ok()?;
    Some(out.into_inner())
}
