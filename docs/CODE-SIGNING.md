# Code signing: notes for a later decision

Status: **researched 2026-11-09, no decision.** Business item; per the work order it waits until
the engineering work is closed and a release is out (see docs/MVP.md "Next").

## Why it matters

Unsigned Windows downloads (our .zip and .msi) trigger SmartScreen's "Windows protected your PC"
warning and some antivirus heuristics. Linux packages (.deb, .tar.gz, AppImage) need no signing;
a signed apt repository is a separate, later question.

No certificate makes the warning disappear on day one: since 2024 even EV certificates no longer
get instant SmartScreen trust. Reputation builds with downloads of signed files, so signing early
(and keeping the same identity) is what counts.

## Options (prices as of 2026-11)

| Option | Cost | Publisher name shown | Conditions / catches |
|---|---|---|---|
| **SignPath Foundation** | free | "SignPath Foundation" (not us) | Open source only (OSI licence, no commercial dual-licensing: our Apache-2.0 OR MIT very likely fine, unconfirmed). Wants an already released project with some reputation, builds from the public repo by CI (GitHub Actions integration), manual approval of each release, MFA for team members, a code-signing policy page on our site/README. Acceptance is at their discretion. |
| **Certum Open Source Code Signing** (cloud, SimplySign) | from €49/yr; €69 with card reader | our own name (individual) | For open-source developers; identity check with ID documents. Certificates max 459 days since 2026-02-27. Signing via their cloud app/key, scriptable in CI with some effort. |
| Commercial OV | ≈ $220/yr (Sectigo via resellers) – $438/yr (DigiCert) | our name or company | Since 2023 the key must sit on a hardware token or cloud HSM; CI signing needs a cloud HSM option (extra cost at some vendors). |
| Commercial EV | ≈ $290–400+/yr | company only | Needs a registered company. No SmartScreen advantage since 2024; only required for kernel drivers. Not useful for us. |
| Azure Artifact Signing (was Trusted Signing) | ≈ $10/month | our name/company | New sign-ups limited to organisations in the US/Canada with 3+ years' history; individual sign-ups paused. **Not open to us now.** |

## Questions to settle when deciding

1. Is "SignPath Foundation" as the shown publisher acceptable, or do we want our own name
   (Certum, ~€49/yr)? For a Chinese-market product a company name may matter later (OV then).
2. Will there be a company (Phase 1 CN, see BP/) soon? If yes, an OV certificate in the company
   name avoids switching identity (and losing SmartScreen reputation) later.
3. Who holds the key and approves releases (MFA, CI secrets, release.yml signing step)?
4. Also sign the MSI's inner exe and the CLI (`wenshu.exe`), not only the installer.

Current lean: apply to SignPath (free) once a release with download history is out; Certum as
fallback; OV only when there is a company.

## Sources

- Microsoft Learn, code signing options: https://learn.microsoft.com/windows/apps/develop/smart-app-control/code-signing-for-smart-app-control
- SignPath Foundation terms: https://signpath.org/terms
- Certum open source code signing: https://shop.certum.eu/open-source-code-signing-on-simplysign.html
- OV/EV price comparison: https://www.sslinsights.com (code signing certificate price lists)
- Trusted/Artifact Signing eligibility: Microsoft Learn Q&A threads on Trusted Signing individual
  and region availability (2025–2026)
