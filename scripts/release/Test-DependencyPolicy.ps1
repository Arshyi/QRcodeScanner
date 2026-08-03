[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path

function Invoke-Checked {
    param(
        [Parameter(Mandatory)] [string]$FilePath,
        [Parameter(Mandatory)] [string[]]$ArgumentList
    )
    & $FilePath @ArgumentList
    if ($LASTEXITCODE -ne 0) {
        throw "$FilePath $($ArgumentList -join ' ') failed with exit code $LASTEXITCODE."
    }
}

Push-Location $repositoryRoot
try {
    foreach ($tool in 'cargo-audit', 'cargo-deny') {
        if (-not (Get-Command $tool -ErrorAction SilentlyContinue)) {
            throw "$tool is required. Install it with: cargo install --locked $tool"
        }
    }

    Invoke-Checked 'cargo' @('audit')
    Invoke-Checked 'cargo' @('deny', 'check', 'advisories', 'licenses', 'sources')

    Push-Location 'apps/desktop'
    try {
        Invoke-Checked 'npm' @('audit', '--audit-level=high')
        $lock = Get-Content -Raw -LiteralPath 'package-lock.json' | ConvertFrom-Json -AsHashtable
        $missingLicense = @(
            $lock.packages.GetEnumerator() |
                Where-Object { $_.Key -ne '' -and -not $_.Value.ContainsKey('license') } |
                Select-Object -ExpandProperty Key
        )
        if ($missingLicense.Count -ne 0) {
            throw "npm packages missing lockfile license metadata: $($missingLicense -join ', ')"
        }
        Write-Output "npm license metadata: $($lock.packages.Count - 1) packages checked."
    }
    finally {
        Pop-Location
    }
}
finally {
    Pop-Location
}
