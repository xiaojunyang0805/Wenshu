"""Make reference renders with a licensed WPS Office on Windows: .docx → PDF → ref/<stem>.p<N>.png.

    python corpus/tools/wps_render.py [--only PATTERN] [--dpi 144] [--force] [--files DIR --out DIR] [--no-png]

Drives WPS Writer through COM (`KWps.Application`, Word-compatible object model): opens each
document read-only, exports a PDF, closes it, then rasterises the PDF with PyMuPDF. The PDFs are
kept in `ref/pdf/` (WPS's page count is the reference page count). Everything lands under
`corpus/ref/`, which is git-ignored: WPS output is not shared until its licence says it may be.

WPS may show its window while it works; run it when nobody is using the machine.
"""
import argparse
import fnmatch
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
WD_EXPORT_PDF = 17  # wdExportFormatPDF


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--only", default="*.docx")
    ap.add_argument("--dpi", type=int, default=144)
    ap.add_argument("--force", action="store_true")
    ap.add_argument("--files", type=Path, default=ROOT / "files", help="documents to render")
    ap.add_argument("--out", type=Path, default=ROOT / "ref", help="where the PNGs (and pdf/) go")
    ap.add_argument("--no-png", action="store_true", help="export the PDFs only")
    a = ap.parse_args()
    import fitz  # PyMuPDF
    import pythoncom
    import win32com.client

    files, ref = a.files.resolve(), a.out.resolve()
    pdfs = ref / "pdf"
    pdfs.mkdir(parents=True, exist_ok=True)
    docs = sorted(p for p in files.glob("*.docx") if fnmatch.fnmatch(p.name, a.only))
    pythoncom.CoInitialize()
    app = win32com.client.DispatchEx("KWps.Application")
    app.Visible = False
    app.DisplayAlerts = 0
    done = failed = 0
    try:
        for p in docs:
            pdf = pdfs / f"{p.stem}.pdf"
            if a.force or not pdf.exists():
                t = time.time()
                try:
                    doc = app.Documents.Open(str(p), False, True)  # ConfirmConversions, ReadOnly
                    doc.ExportAsFixedFormat(str(pdf), WD_EXPORT_PDF)
                    doc.Close(0)
                except Exception as e:  # one bad document never stops the run
                    print(f"FAIL {p.name}: {e}", file=sys.stderr)
                    failed += 1
                    continue
                print(f"{p.name}: {time.time() - t:.1f}s")
            if a.no_png:
                done += 1
                continue
            for old in ref.glob(f"{p.stem}.p*.png"):
                old.unlink()
            with fitz.open(pdf) as d:
                for i, page in enumerate(d, 1):
                    page.get_pixmap(dpi=a.dpi).save(ref / f"{p.stem}.p{i}.png")
            done += 1
    finally:
        app.Quit()
    print(f"{done} rendered, {failed} failed")


if __name__ == "__main__":
    main()
