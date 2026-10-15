$ErrorActionPreference = 'Stop'

cargo fmt --all -- --check *> $null
$needsFormatting = $LASTEXITCODE -ne 0
cargo fmt --all
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

if (-not $needsFormatting) {
    Write-Output '[pre-commit] Formatting complete: no changes needed.'
} else {
    Write-Output '[pre-commit] Formatting complete: files were changed.'
}
