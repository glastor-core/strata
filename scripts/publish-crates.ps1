<#
.SYNOPSIS
  Script de publicación ordenada para el workspace GLASTOR® Strata en crates.io.

.DESCRIPTION
  Publica los crates en orden topológico estricto:
  1. glastor-aria-core
  2. glastor-css-binder
  3. Espera 30s a que el índice de crates.io registre los metadatos
  4. glastor-ui-cli

.PARAMETER DryRun
  Si se especifica, ejecuta `cargo publish --dry-run` sin subir a crates.io.

.PARAMETER Token
  Token de API de crates.io (opcional si ya está configurado en `cargo login`).
#>

param(
    [switch]$DryRun,
    [string]$Token
)

$ErrorActionPreference = "Stop"

Write-Host "========================================================" -ForegroundColor Cyan
Write-Host "  GLASTOR® Strata — Pipeline de Publicación a crates.io" -ForegroundColor Cyan
Write-Host "========================================================" -ForegroundColor Cyan

$extraArgs = @()
if ($DryRun) {
    Write-Host "[MODO] Dry-Run activo (simulación sin publicación real)" -ForegroundColor Yellow
    $extraArgs += "--dry-run"
} elseif ($Token) {
    $extraArgs += "--token"
    $extraArgs += $Token.Trim()
} else {
    $hasCredentials = (Test-Path "$env:USERPROFILE\.cargo\credentials.toml") -or (Test-Path "$env:USERPROFILE\.cargo\credentials")
    if (-not $hasCredentials) {
        Write-Host "`n[Aviso] No se detectaron credenciales guardadas en ~/.cargo/credentials.toml." -ForegroundColor Yellow
        Write-Host "Puedes generar tu token en: https://crates.io/settings/tokens" -ForegroundColor Cyan
        $inputToken = Read-Host "`nPor favor ingresa tu Token de API de crates.io (comienza con cio...)"
        if ([string]::IsNullOrWhiteSpace($inputToken)) {
            Write-Host "Operación cancelada. Ejecuta: .\scripts\publish-crates.ps1 -Token 'TU_TOKEN'" -ForegroundColor Red
            exit 1
        }
        $Token = $inputToken.Trim()
        $extraArgs += "--token"
        $extraArgs += $Token
    }
}

# 1. Pre-flight checks
Write-Host "`n[Paso 1/4] Verificando tests y linter del workspace..." -ForegroundColor Green
cargo fmt --all --check
if ($LASTEXITCODE -ne 0) { throw "Fallo en cargo fmt" }

cargo clippy --workspace --all-targets -- -D warnings
if ($LASTEXITCODE -ne 0) { throw "Fallo en cargo clippy" }

cargo test --workspace --all-features
if ($LASTEXITCODE -ne 0) { throw "Fallo en cargo test" }

# 2. Publicar glastor-aria-core
Write-Host "`n[Paso 2/4] Publicando glastor-aria-core v0.1.0..." -ForegroundColor Green
cargo publish -p glastor-aria-core @extraArgs
if ($LASTEXITCODE -ne 0) { throw "Error al publicar glastor-aria-core" }
Write-Host "✔ glastor-aria-core completado." -ForegroundColor Green

# 3. Publicar glastor-css-binder
Write-Host "`n[Paso 3/4] Publicando glastor-css-binder v0.1.0..." -ForegroundColor Green
cargo publish -p glastor-css-binder @extraArgs
if ($LASTEXITCODE -ne 0) { throw "Error al publicar glastor-css-binder" }
Write-Host "✔ glastor-css-binder completado." -ForegroundColor Green

# 4. Esperar propagación del índice crates.io antes de publicar el CLI dependiente
if (-not $DryRun) {
    Write-Host "`n[Pausa de Propagación] Esperando 30 segundos para actualización del índice en crates.io..." -ForegroundColor Yellow
    Start-Sleep -Seconds 30
}

# 5. Publicar glastor-ui-cli
Write-Host "`n[Paso 4/4] Publicando glastor-ui-cli v0.1.0..." -ForegroundColor Green
cargo publish -p glastor-ui-cli @extraArgs
if ($LASTEXITCODE -ne 0) { throw "Error al publicar glastor-ui-cli" }
Write-Host "✔ glastor-ui-cli completado." -ForegroundColor Green

Write-Host "`n========================================================" -ForegroundColor Green
Write-Host "  ¡Publicación de GLASTOR® Strata v0.1.0 exitosa!" -ForegroundColor Green
Write-Host "  https://crates.io/crates/glastor-aria-core" -ForegroundColor Cyan
Write-Host "  https://crates.io/crates/glastor-css-binder" -ForegroundColor Cyan
Write-Host "  https://crates.io/crates/glastor-ui-cli" -ForegroundColor Cyan
Write-Host "========================================================" -ForegroundColor Green

