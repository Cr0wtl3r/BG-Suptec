#Requires -Version 5.1
<#
.SYNOPSIS
    Builda o BG-SupTec standalone: gera (ou reaproveita) auth.hash e roda
    `cargo tauri build`, deixando `bg-suptec.exe`, `kms.json` e `auth.hash`
    juntos em `dist-standalone\` para distribuição (USB, rede interna etc.).

.PARAMETER Senha
    Senha em texto puro a ser hasheada com argon2id (via `generate_hash`) e
    gravada em `auth.hash`. Se omitida e `auth.hash` já existir na raiz do
    projeto, o arquivo existente é reaproveitado sem prompt.

.PARAMETER SkipBundle
    Pula a geração de instaladores NSIS/MSI (`cargo tauri build --no-bundle`),
    produzindo apenas o `.exe` standalone — mais rápido para iteração local.

.EXAMPLE
    .\build.ps1 -Senha "minhaSenhaForte123"

.EXAMPLE
    .\build.ps1 -SkipBundle
#>
param(
    [string]$Senha,
    [switch]$SkipBundle
)

$ErrorActionPreference = "Stop"
$root = $PSScriptRoot
$authHashPath = Join-Path $root "auth.hash"
$kmsJsonPath = Join-Path $root "kms.json"
$distDir = Join-Path $root "dist-standalone"

Write-Host "==> BG-SupTec build" -ForegroundColor Cyan

if (-not (Test-Path $kmsJsonPath)) {
    throw "kms.json não encontrado em '$kmsJsonPath'. Crie o arquivo de config antes de buildar (veja README.md)."
}

if ($Senha) {
    Write-Host "==> Gerando auth.hash (argon2id) a partir da senha informada..." -ForegroundColor Cyan
    cargo build --release --bin generate_hash --manifest-path "$root\src-tauri\Cargo.toml"
    if ($LASTEXITCODE -ne 0) { throw "Falha ao compilar generate_hash." }

    $hashExe = Join-Path $root "src-tauri\target\release\generate_hash.exe"
    $hash = & $hashExe $Senha
    if ($LASTEXITCODE -ne 0 -or -not $hash) { throw "Falha ao gerar o hash da senha." }

    Set-Content -Path $authHashPath -Value $hash -NoNewline
    Write-Host "    auth.hash gravado em '$authHashPath'." -ForegroundColor Green
}
elseif (-not (Test-Path $authHashPath)) {
    throw "auth.hash não encontrado e nenhuma -Senha foi informada. Rode: .\build.ps1 -Senha `"suaSenha`""
}
else {
    Write-Host "==> Reaproveitando auth.hash existente em '$authHashPath'." -ForegroundColor Yellow
}

Write-Host "==> Compilando frontend + backend (cargo tauri build)..." -ForegroundColor Cyan
Push-Location $root
try {
    if ($SkipBundle) {
        cargo tauri build --no-bundle
    }
    else {
        cargo tauri build
    }
    if ($LASTEXITCODE -ne 0) { throw "cargo tauri build falhou." }
}
finally {
    Pop-Location
}

$builtExe = Join-Path $root "src-tauri\target\release\bg-suptec.exe"
if (-not (Test-Path $builtExe)) {
    throw "Build concluído mas '$builtExe' não foi encontrado."
}

Write-Host "==> Montando pacote standalone em '$distDir'..." -ForegroundColor Cyan
New-Item -ItemType Directory -Force -Path $distDir | Out-Null
Copy-Item -Path $builtExe -Destination $distDir -Force
Copy-Item -Path $kmsJsonPath -Destination $distDir -Force
Copy-Item -Path $authHashPath -Destination $distDir -Force

Write-Host "==> Pronto. Conteúdo de '$distDir':" -ForegroundColor Green
Get-ChildItem $distDir | Format-Table Name, Length -AutoSize
