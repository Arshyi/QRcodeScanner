param(
  [string]$Binary = ""
)

$ErrorActionPreference = "Stop"
if (-not $Binary) {
  $Binary = Join-Path $PSScriptRoot "..\target\release\qrforge-wgc-spike.exe"
}
$Binary = [IO.Path]::GetFullPath($Binary)
$animator = Start-Process powershell.exe -ArgumentList @(
  "-NoProfile",
  "-ExecutionPolicy", "Bypass",
  "-File", (Join-Path $PSScriptRoot "animate.ps1")
) -PassThru
try {
  Start-Sleep -Seconds 2
  & $Binary
  if ($LASTEXITCODE -ne 0) {
    throw "WGC benchmark exited with code $LASTEXITCODE"
  }
}
finally {
  Stop-Process -Id $animator.Id -Force -ErrorAction SilentlyContinue
}

