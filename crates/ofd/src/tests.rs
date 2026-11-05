use super::*;
use skrifa::MetadataProvider;
use std::io::Read;

fn serif() -> Vec<u8> {
    let p = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../upstream/wordcraft/assets/fonts/SourceSerif4-Regular.ttf"
    );
    std::fs::read(p).unwrap()
}

fn unzip(bytes: &[u8]) -> BTreeMap<String, Vec<u8>> {
    let mut z = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
    let mut out = BTreeMap::new();
    for i in 0..z.len() {
        let mut f = z.by_index(i).unwrap();
        let mut v = Vec::new();
        f.read_to_end(&mut v).unwrap();
        out.insert(f.name().to_string(), v);
    }
    out
}

fn well_formed(xml: &[u8]) {
    let mut r = quick_xml::Reader::from_reader(xml);
    let mut buf = Vec::new();
    let mut depth = 0i32;
    loop {
        match r.read_event_into(&mut buf).unwrap() {
            quick_xml::events::Event::Start(_) => depth += 1,
            quick_xml::events::Event::End(_) => depth -= 1,
            quick_xml::events::Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    assert_eq!(depth, 0);
}

#[test]
fn a_page_of_text_and_shapes_makes_a_complete_package() {
    let data = serif();
    let font = skrifa::FontRef::new(&data).unwrap();
    let cmap = font.charmap();
    let text = "Hello & <OFD>";
    let glyphs: Vec<u32> = text
        .chars()
        .map(|c| cmap.map(c).unwrap().to_u32())
        .collect();
    let mut ofd = Ofd::new(Meta {
        title: "测试".into(),
        creator: "Wenshu".into(),
        creation_date: "2026-11-05".into(),
        ..Default::default()
    });
    let f = ofd.add_font(FontFace {
        name: "Source Serif 4".into(),
        family: "Source Serif 4".into(),
        bold: false,
        italic: false,
        serif: true,
        fixed_width: false,
        data: &data,
        index: 0,
    });
    let n = text.chars().count();
    ofd.pages.push(Page {
        width: 210.0,
        height: 297.0,
        links: vec![],
        objects: vec![
            Object::Text(Text {
                common: Common::default(),
                font: f,
                size: 4.2,
                fill: Some(Color(0, 0, 0)),
                stroke: None,
                hscale: 1.0,
                codes: vec![TextCode {
                    x: 20.0,
                    y: 30.0,
                    text: text.into(),
                    glyphs,
                    dx: vec![2.1; n - 1],
                    dy: vec![],
                }],
            }),
            Object::Path(Path {
                common: Common::default(),
                segs: vec![Seg::Move(20.0, 35.0), Seg::Line(190.0, 35.0)],
                fill: None,
                stroke: Some(Stroke {
                    color: Color(255, 0, 0),
                    width: 0.5,
                    dash: vec![],
                    cap: Cap::Butt,
                    join: Join::Miter,
                }),
                even_odd: false,
            }),
        ],
    });
    let files = unzip(&ofd.write().unwrap());
    for name in [
        "OFD.xml",
        "Doc_0/Document.xml",
        "Doc_0/PublicRes.xml",
        "Doc_0/DocumentRes.xml",
        "Doc_0/Pages/Page_0/Content.xml",
    ] {
        well_formed(files.get(name).unwrap_or_else(|| panic!("{name} missing")));
    }
    let page = String::from_utf8(files["Doc_0/Pages/Page_0/Content.xml"].clone()).unwrap();
    assert!(page.contains("Hello &amp; &lt;OFD&gt;"), "{page}");
    assert!(page.contains(r#"DeltaX="g 12 2.1""#), "{page}");
    assert!(
        !page.contains("CGTransform"),
        "every glyph is the cmap's: {page}"
    );
    assert!(page.contains("M 20 35 L 190 35"));
    // The embedded subset is a font on its own: its cmap maps the characters used to glyphs with
    // outlines, and it is far smaller than the original.
    let (name, sub) = files.iter().find(|(n, _)| n.ends_with(".ttf")).unwrap();
    assert!(
        sub.len() < data.len() / 4,
        "{name}: {} vs {}",
        sub.len(),
        data.len()
    );
    let sf = skrifa::FontRef::new(sub).unwrap();
    let outlines = sf.outline_glyphs();
    for c in "Helo&<OFD>".chars() {
        let g = sf
            .charmap()
            .map(c)
            .unwrap_or_else(|| panic!("{c} not mapped"));
        assert!(outlines.get(g).is_some(), "{c}");
    }
    assert!(sf.charmap().map('Z').is_none());
    let ofd_xml = String::from_utf8(files["OFD.xml"].clone()).unwrap();
    assert!(
        ofd_xml.contains("<ofd:Title>测试</ofd:Title>")
            && ofd_xml.contains("<ofd:DocRoot>Doc_0/Document.xml</ofd:DocRoot>")
    );
}

#[test]
fn a_glyph_the_cmap_does_not_give_gets_a_cg_transform() {
    let data = serif();
    let font = skrifa::FontRef::new(&data).unwrap();
    let a = font.charmap().map('a').unwrap().to_u32();
    let b = font.charmap().map('b').unwrap().to_u32();
    let mut ofd = Ofd::new(Meta::default());
    let f = ofd.add_font(FontFace {
        name: "S".into(),
        family: "S".into(),
        bold: false,
        italic: false,
        serif: true,
        fixed_width: false,
        data: &data,
        index: 0,
    });
    let code = |text: &str, glyphs: Vec<u32>| TextCode {
        x: 0.0,
        y: 10.0,
        text: text.into(),
        glyphs,
        dx: vec![2.0; text.chars().count().saturating_sub(1)],
        dy: vec![],
    };
    ofd.pages.push(Page {
        width: 100.0,
        height: 100.0,
        links: vec![],
        objects: vec![Object::Text(Text {
            common: Common::default(),
            font: f,
            size: 4.0,
            fill: Some(Color(0, 0, 0)),
            stroke: None,
            hscale: 1.0,
            // 'a' first drawn as its own glyph, then (an alternate) as b's; a two-character cluster.
            codes: vec![code("aa", vec![a, b]), code("ab", vec![a])],
        })],
    });
    let files = unzip(&ofd.write().unwrap());
    let page = String::from_utf8(files["Doc_0/Pages/Page_0/Content.xml"].clone()).unwrap();
    assert!(
        page.contains(r#"<ofd:CGTransform CodePosition="1" CodeCount="1" GlyphCount="1">"#),
        "{page}"
    );
    assert!(
        page.contains(r#"<ofd:CGTransform CodePosition="0" CodeCount="2" GlyphCount="1">"#),
        "{page}"
    );
}

#[test]
fn numbers_and_steps_are_short() {
    assert_eq!(num(1.0), "1");
    assert_eq!(num(-0.0001), "0");
    assert_eq!(num(12.34567), "12.346");
    assert_eq!(deltas(&[1.0, 1.0, 1.0, 2.5, 2.5]), "g 3 1 2.5 2.5");
    assert_eq!(esc("a\u{1}b"), "a b");
}

/// Clip areas are filled paths in the object's own space: a picture's clip goes back through its
/// CTM to the unit square.
#[test]
fn a_clip_is_filled_and_in_the_objects_space() {
    let mut ofd = Ofd::new(Meta::default());
    let im = ofd.add_image(vec![0x89, b'P', b'N', b'G'], ImageFormat::Png);
    let clip = vec![
        Seg::Move(30.0, 10.0),
        Seg::Line(70.0, 10.0),
        Seg::Line(70.0, 30.0),
        Seg::Line(30.0, 30.0),
        Seg::Close,
    ];
    ofd.pages.push(Page {
        width: 100.0,
        height: 100.0,
        links: vec![],
        objects: vec![Object::Image(Image {
            common: Common {
                ctm: Some([80.0, 0.0, 0.0, 20.0, 10.0, 10.0]),
                clips: vec![clip],
                alpha: 255,
            },
            image: im,
        })],
    });
    let files = unzip(&ofd.write().unwrap());
    let page = String::from_utf8(files["Doc_0/Pages/Page_0/Content.xml"].clone()).unwrap();
    assert!(page.contains(r#"Stroke="false" Fill="true"><ofd:AbbreviatedData>M 0.25 0 L 0.75 0 L 0.75 1 L 0.25 1 C<"#), "{page}");
    assert!(!page.contains("-0 "), "{page}");
}

/// Links are link annotations: an invisible rectangle with the click action, a Goto to a page
/// by its ID or a URI; the outline is a tree of Gotos.
#[test]
fn links_and_the_outline() {
    let mut ofd = Ofd::new(Meta::default());
    let dest = Dest {
        page: 1,
        x: 20.0,
        y: 30.0,
    };
    for _ in 0..2 {
        ofd.pages.push(Page {
            width: 100.0,
            height: 100.0,
            objects: vec![],
            links: vec![],
        });
    }
    ofd.pages[0].links = vec![
        PageLink {
            rect: [10.0, 10.0, 30.0, 5.0],
            link: Link::Goto(dest),
        },
        PageLink {
            rect: [10.0, 20.0, 30.0, 5.0],
            link: Link::Uri("https://example.org/?a=1&b=2".into()),
        },
    ];
    ofd.outline = vec![OutlineItem {
        title: "一、总则".into(),
        dest,
        children: vec![OutlineItem {
            title: "（一）".into(),
            dest,
            children: vec![],
        }],
    }];
    let files = unzip(&ofd.write().unwrap());
    let doc = String::from_utf8(files["Doc_0/Document.xml"].clone()).unwrap();
    well_formed(doc.as_bytes());
    let second_page = doc
        .split(r#"<ofd:Page ID=""#)
        .nth(2)
        .unwrap()
        .split('"')
        .next()
        .unwrap()
        .to_string();
    assert!(doc.contains(&format!(r#"<ofd:OutlineElem Title="一、总则"><ofd:Actions><ofd:Action Event="CLICK"><ofd:Goto><ofd:Dest Type="XYZ" PageID="{second_page}" Left="20" Top="30"/>"#)), "{doc}");
    assert!(doc.contains("<ofd:OutlineElem Title=\"（一）\">"));
    assert!(doc.ends_with("</ofd:Outlines><ofd:Annotations>Annots/Annotations.xml</ofd:Annotations></ofd:Document>"), "{doc}");
    let list = String::from_utf8(files["Doc_0/Annots/Annotations.xml"].clone()).unwrap();
    assert!(
        list.contains("<ofd:FileLoc>Page_0/Annotation.xml</ofd:FileLoc>")
            && !list.contains("Page_1"),
        "{list}"
    );
    let annots = String::from_utf8(files["Doc_0/Annots/Page_0/Annotation.xml"].clone()).unwrap();
    well_formed(annots.as_bytes());
    assert_eq!(annots.matches(r#"Type="Link""#).count(), 2);
    assert!(annots.contains(r#"<ofd:Appearance Boundary="10 20 30 5">"#));
    assert!(
        annots.contains(r#"<ofd:URI URI="https://example.org/?a=1&amp;b=2"/>"#),
        "{annots}"
    );
}
