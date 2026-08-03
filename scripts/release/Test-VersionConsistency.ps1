[CmdletBinding()]
param(
    [Parameter()]
    [ValidatePattern('^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$')]
    [string]$ExpectedVersion
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path

Push-Location $repositoryRoot
try {
    $cargoManifest = Get-Content -Raw -LiteralPath 'Cargo.toml'
    $workspaceSection = [regex]::Match(
        $cargoManifest,
        '(?ms)^\[workspace\.package\]\s*(?<body>.*?)(?=^\[|\z)'
    )
    if (-not $workspaceSection.Success) {
        throw 'Cargo.toml is missing [workspace.package].'
    }
    $versionMatch = [regex]::Match(
        $workspaceSection.Groups['body'].Value,
        '(?m)^version\s*=\s*"(?<version>[^"]+)"\s*$'
    )
    if (-not $versionMatch.Success) {
        throw 'Cargo.toml [workspace.package] is missing version.'
    }
    $canonicalVersion = $versionMatch.Groups['version'].Value
    if ($ExpectedVersion -and $canonicalVersion -ne $ExpectedVersion) {
        throw "Canonical version is $canonicalVersion; expected $ExpectedVersion."
    }

    $desktopPackage = Get-Content -Raw -LiteralPath 'apps/desktop/package.json' |
        ConvertFrom-Json -AsHashtable
    $desktopLock = Get-Content -Raw -LiteralPath 'apps/desktop/package-lock.json' |
        ConvertFrom-Json -AsHashtable
    $tauriConfig = Get-Content -Raw -LiteralPath 'apps/desktop/src-tauri/tauri.conf.json' |
        ConvertFrom-Json -AsHashtable

    $declared = [ordered]@{
        'Cargo.toml [workspace.package]' = $canonicalVersion
        'apps/desktop/package.json' = [string]$desktopPackage.version
        'apps/desktop/package-lock.json root' = [string]$desktopLock.version
        'apps/desktop/package-lock.json package root' = [string]$desktopLock.packages[''].version
        'apps/desktop/src-tauri/tauri.conf.json' = [string]$tauriConfig.version
    }
    foreach ($entry in $declared.GetEnumerator()) {
        if ($entry.Value -ne $canonicalVersion) {
            throw "$($entry.Key) declares $($entry.Value); expected $canonicalVersion."
        }
    }

    $metadataText = (& cargo metadata --format-version 1 --no-deps --locked | Out-String)
    if ($LASTEXITCODE -ne 0) {
        throw "cargo metadata failed with exit code $LASTEXITCODE."
    }
    $metadata = $metadataText | ConvertFrom-Json -AsHashtable
    $workspaceIds = [Collections.Generic.HashSet[string]]::new(
        [string[]]$metadata.workspace_members,
        [StringComparer]::Ordinal
    )
    $workspacePackages = @($metadata.packages | Where-Object { $workspaceIds.Contains([string]$_.id) })
    if ($workspacePackages.Count -eq 0) {
        throw 'cargo metadata returned no workspace packages.'
    }
    foreach ($package in $workspacePackages) {
        if ([string]$package.version -ne $canonicalVersion) {
            throw "Workspace package $($package.name) is $($package.version); expected $canonicalVersion."
        }
    }

    [pscustomobject]@{
        Version = $canonicalVersion
        WorkspacePackages = $workspacePackages.Count
        Status = 'consistent'
    }
}
finally {
    Pop-Location
}
