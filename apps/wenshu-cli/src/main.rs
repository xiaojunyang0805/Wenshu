//! `wenshu`: Wenshu from the command line.

use std::path::Path;
use std::process::ExitCode;

use wenshu_core::Doc;

const USAGE: &str = "wenshu — Wenshu from the command line

USAGE:
  wenshu info <file>                            pages, paragraphs, properties (JSON)
  wenshu render <file> <out.png> [--page N] [--scale S]
                                                 render one page (default page 1, scale 2)
  wenshu convert <in> <out>                     convert by extension: docx, pdf, odt, rtf, html, md, txt
  wenshu gongwen new <spec.json> <out.docx>     write a GB/T 9704 official document
  wenshu gongwen check <file.docx> [--json]     check a document against GB/T 9704 (exit 1 on failures)
  wenshu fidelity <corpus> <out> [--scale S]    render <corpus>/files, compare with <corpus>/ref, write <out>/report.json
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("wenshu: {e}");
            ExitCode::from(2)
        }
    }
}

fn value<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
}

/// Arguments that aren't flags or flag values.
fn positionals(args: &[String]) -> Vec<&str> {
    let mut out = Vec::new();
    let mut skip = false;
    for a in args {
        if skip {
            skip = false;
        } else if a == "--page" || a == "--scale" {
            skip = true;
        } else if !a.starts_with("--") {
            out.push(a.as_str());
        }
    }
    out
}

fn arg<'a>(p: &[&'a str], i: usize) -> Result<&'a str, String> {
    p.get(i)
        .copied()
        .ok_or_else(|| format!("missing argument\n\n{USAGE}"))
}

fn parse<T: std::str::FromStr>(args: &[String], flag: &str, default: T) -> Result<T, String> {
    value(args, flag).map_or(Ok(default), |v| {
        v.parse().map_err(|_| format!("invalid {flag} `{v}`"))
    })
}

fn json_out(v: &impl serde::Serialize) -> Result<(), String> {
    println!(
        "{}",
        serde_json::to_string_pretty(v).map_err(|e| e.to_string())?
    );
    Ok(())
}

fn run(args: &[String]) -> Result<ExitCode, String> {
    let p = positionals(args);
    match p.first().copied() {
        None | Some("help") => {
            print!("{USAGE}");
            Ok(ExitCode::SUCCESS)
        }
        Some("info") => {
            json_out(&Doc::open(Path::new(arg(&p, 1)?), None)?.info()?)?;
            Ok(ExitCode::SUCCESS)
        }
        Some("render") => {
            let mut doc = Doc::open(Path::new(arg(&p, 1)?), None)?;
            let r = doc.render_png(
                parse(args, "--page", 1)?,
                parse(args, "--scale", 2.0)?,
                Path::new(arg(&p, 2)?),
            )?;
            println!("{r}");
            Ok(ExitCode::SUCCESS)
        }
        Some("convert") => {
            let out = arg(&p, 2)?;
            Doc::open(Path::new(arg(&p, 1)?), None)?.save_as(Path::new(out))?;
            eprintln!("wrote {out}");
            Ok(ExitCode::SUCCESS)
        }
        Some("gongwen") => match p.get(1).copied() {
            Some("new") => {
                let spec = std::fs::read_to_string(arg(&p, 2)?).map_err(|e| e.to_string())?;
                let doc: wenshu_gongwen::Gongwen =
                    serde_json::from_str(&spec).map_err(|e| format!("spec: {e}"))?;
                let out = arg(&p, 3)?;
                let file = std::fs::File::create(out).map_err(|e| format!("{out}: {e}"))?;
                wenshu_gongwen::write_docx(&doc, file)?;
                eprintln!("wrote {out}");
                Ok(ExitCode::SUCCESS)
            }
            Some("check") => {
                let findings = wenshu_gongwen::check_docx(Path::new(arg(&p, 2)?))?;
                if args.iter().any(|a| a == "--json") {
                    json_out(&findings)?;
                } else {
                    for f in &findings {
                        println!(
                            "{} {:<12} expected {:<34} got {}",
                            if f.ok { "✔" } else { "✘" },
                            f.rule,
                            f.expected,
                            f.actual
                        );
                    }
                }
                Ok(if findings.iter().all(|f| f.ok) {
                    ExitCode::SUCCESS
                } else {
                    ExitCode::from(1)
                })
            }
            _ => Err(format!("gongwen: expected `new` or `check`\n\n{USAGE}")),
        },
        Some("fidelity") => {
            let out = Path::new(arg(&p, 2)?);
            let report =
                wenshu_fidelity::run(Path::new(arg(&p, 1)?), out, parse(args, "--scale", 1.0)?)?;
            let json = serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?;
            std::fs::write(out.join("report.json"), json).map_err(|e| e.to_string())?;
            for d in &report.documents {
                let scores: Vec<String> = d
                    .page_scores
                    .iter()
                    .map(|s| s.score.map_or("–".into(), |v| format!("{v:.3}")))
                    .collect();
                let status = d.error.as_deref().unwrap_or("ok");
                println!(
                    "{:<40} pages {:>3}  ref {:>3}  scores [{}]  {status}",
                    d.file.display(),
                    d.pages,
                    d.reference_pages.map_or("–".into(), |n| n.to_string()),
                    scores.join(" ")
                );
            }
            println!(
                "mean score {}  page counts {}/{}  failures {}",
                report.mean_score.map_or("–".into(), |v| format!("{v:.3}")),
                report.page_count_matches,
                report.page_count_compared,
                report.failures
            );
            Ok(if report.failures == 0 {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            })
        }
        Some(other) => Err(format!("unknown command `{other}`\n\n{USAGE}")),
    }
}
