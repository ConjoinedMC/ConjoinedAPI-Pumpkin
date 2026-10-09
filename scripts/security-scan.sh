#!/usr/bin/env bash
set -euo pipefail
mkdir -p target/security-tools
tools=target/security-tools
curl --fail --location --silent --show-error https://github.com/google/osv-scanner/releases/download/v2.6.0/osv-scanner_linux_amd64 -o "$tools/osv-scanner"
printf '%s  %s\n' ca69b3d3cd08f889a49dc0a383122f71cc528b83803671df5fd874d97485b108 "$tools/osv-scanner" | sha256sum --check
curl --fail --location --silent --show-error https://github.com/gitleaks/gitleaks/releases/download/v8.30.1/gitleaks_8.30.1_linux_x64.tar.gz -o "$tools/gitleaks.tar.gz"
printf '%s  %s\n' 551f6fc83ea457d62a0d98237cbad105af8d557003051f41f3e7ca7b3f2470eb "$tools/gitleaks.tar.gz" | sha256sum --check
tar -xzf "$tools/gitleaks.tar.gz" -C "$tools" gitleaks
chmod +x "$tools/osv-scanner" "$tools/gitleaks"
"$tools/gitleaks" git . --redact --no-banner
"$tools/osv-scanner" scan source --lockfile "${1:?Supply the resolved SBOM or Cargo.lock}" --format json --output-file target/advisory-report.json
