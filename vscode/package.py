"""Package the VS Code extension as a .vsix without npm: a .vsix is a zip of the extension files
under `extension/`, a VSIX manifest and a content-types file (the layout `vsce package` writes).

    python vscode/package.py            -> dist/wenshu-preview-<version>.vsix

The extension is plain JavaScript with no dependencies, so there is nothing to build first.
"""
import json
import os
import sys
import zipfile
from xml.sax.saxutils import escape

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
FILES = [
    'package.json',
    'package.nls.json',
    'package.nls.zh-cn.json',
    'extension.js',
    'README.md',
    'CHANGELOG.md',
    'l10n/bundle.l10n.zh-cn.json',
    'media/icon.png',
]


def main():
    pkg = json.load(open(os.path.join(HERE, 'package.json'), encoding='utf-8'))
    nls = json.load(open(os.path.join(HERE, 'package.nls.json'), encoding='utf-8'))
    text = lambda v: nls.get(v[1:-1], v) if v.startswith('%') and v.endswith('%') else v
    name, version, publisher = pkg['name'], pkg['version'], pkg['publisher']
    missing = [f for f in FILES if not os.path.exists(os.path.join(HERE, f))]
    if missing:
        sys.exit(f'missing: {missing}')
    licence = open(os.path.join(ROOT, 'LICENSE-APACHE'), encoding='utf-8').read() + '\n\n' + \
        open(os.path.join(ROOT, 'LICENSE-MIT'), encoding='utf-8').read()
    manifest = f'''<?xml version="1.0" encoding="utf-8"?>
<PackageManifest Version="2.0.0" xmlns="http://schemas.microsoft.com/developer/vsx-schema/2011" xmlns:d="http://schemas.microsoft.com/developer/vsx-schema-design/2011">
  <Metadata>
    <Identity Language="en-US" Id="{name}" Version="{version}" Publisher="{publisher}" />
    <DisplayName>{escape(text(pkg['displayName']))}</DisplayName>
    <Description xml:space="preserve">{escape(text(pkg['description']))}</Description>
    <Tags>{escape(','.join(pkg.get('keywords', [])))}</Tags>
    <Categories>{escape(','.join(pkg.get('categories', [])))}</Categories>
    <GalleryFlags>Public</GalleryFlags>
    <Properties>
      <Property Id="Microsoft.VisualStudio.Code.Engine" Value="{escape(pkg['engines']['vscode'])}" />
      <Property Id="Microsoft.VisualStudio.Code.ExtensionDependencies" Value="" />
      <Property Id="Microsoft.VisualStudio.Code.ExtensionPack" Value="" />
      <Property Id="Microsoft.VisualStudio.Code.ExtensionKind" Value="{','.join(pkg.get('extensionKind', ['workspace']))}" />
      <Property Id="Microsoft.VisualStudio.Code.LocalizedLanguages" Value="" />
      <Property Id="Microsoft.VisualStudio.Services.Links.Source" Value="{escape(pkg['repository']['url'])}" />
      <Property Id="Microsoft.VisualStudio.Services.GitHubFlavoredMarkdown" Value="true" />
      <Property Id="Microsoft.VisualStudio.Services.Content.Pricing" Value="Free" />
    </Properties>
    <License>extension/LICENSE.txt</License>
    <Icon>extension/media/icon.png</Icon>
  </Metadata>
  <Installation>
    <InstallationTarget Id="Microsoft.VisualStudio.Code" />
  </Installation>
  <Dependencies />
  <Assets>
    <Asset Type="Microsoft.VisualStudio.Code.Manifest" Path="extension/package.json" Addressable="true" />
    <Asset Type="Microsoft.VisualStudio.Services.Content.Details" Path="extension/README.md" Addressable="true" />
    <Asset Type="Microsoft.VisualStudio.Services.Content.Changelog" Path="extension/CHANGELOG.md" Addressable="true" />
    <Asset Type="Microsoft.VisualStudio.Services.Content.License" Path="extension/LICENSE.txt" Addressable="true" />
    <Asset Type="Microsoft.VisualStudio.Services.Icons.Default" Path="extension/media/icon.png" Addressable="true" />
  </Assets>
</PackageManifest>
'''
    types = '''<?xml version="1.0" encoding="utf-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension=".json" ContentType="application/json" /><Default Extension=".js" ContentType="application/javascript" /><Default Extension=".md" ContentType="text/markdown" /><Default Extension=".txt" ContentType="text/plain" /><Default Extension=".png" ContentType="image/png" /><Default Extension=".vsixmanifest" ContentType="text/xml" /></Types>
'''
    out_dir = os.path.join(ROOT, 'dist')
    os.makedirs(out_dir, exist_ok=True)
    out = os.path.join(out_dir, f'{name}-{version}.vsix')
    with zipfile.ZipFile(out, 'w', zipfile.ZIP_DEFLATED) as z:
        z.writestr('[Content_Types].xml', types)
        z.writestr('extension.vsixmanifest', manifest)
        z.writestr('extension/LICENSE.txt', licence)
        for f in FILES:
            z.write(os.path.join(HERE, f), 'extension/' + f)
    print(f'wrote {out} ({os.path.getsize(out) // 1024} KB)')


if __name__ == '__main__':
    main()
