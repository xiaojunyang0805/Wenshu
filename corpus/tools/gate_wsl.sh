#!/usr/bin/env bash
# The Linux half of the regression gate, from Windows through WSL:
#   MSYS_NO_PATHCONV=1 wsl.exe -d Ubuntu-22.04 -- bash /mnt/d/27_Project/Wenshu/corpus/tools/gate_wsl.sh
# Brings a Linux clone (~/wenshu) to the Windows checkout's main, applies the patch queue, builds
# the CLI and runs the gate there with the packaged fonts (the installed wenshu .deb), reading the
# corpus and its WPS references from the Windows checkout. Nothing is pushed or fetched remotely.
set -euo pipefail
WIN=${WIN:-/mnt/d/27_Project/Wenshu}
cd ~/wenshu
git fetch -q "$WIN" main
git checkout -q --detach FETCH_HEAD
git log --oneline -1
sh scripts/sync-upstream.sh > /dev/null
. ~/.cargo/env
cargo build --release -p wenshu-cli 2>&1 | tail -1
rm -rf ~/gate-linux
WENSHU_FONTS_DIR=/usr/share/wenshu/fonts ./eval/target/wordcraft/release/wenshu fidelity "$WIN/corpus" ~/gate-linux > ~/gate-linux.log 2>&1 || true
tail -1 ~/gate-linux.log
cp ~/gate-linux/report.json "$WIN/out/gate-linux-report.json"
python3 "$WIN/corpus/tools/gate.py" check ~/gate-linux/report.json --platform linux
