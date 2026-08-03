[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [ValidatePattern('^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$')]
    [string]$ExpectedVersion
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$testRoot = Join-Path $repositoryRoot "target\release-tooling-tests\$([guid]::NewGuid().ToString('N'))"
New-Item -ItemType Directory -Force -Path $testRoot | Out-Null

try {
    & (Join-Path $PSScriptRoot 'Test-VersionConsistency.ps1') -ExpectedVersion $ExpectedVersion |
        Out-Null
    $invalidVersionFailed = $false
    try {
        & (Join-Path $PSScriptRoot 'Test-VersionConsistency.ps1') -ExpectedVersion '999.999.999' |
            Out-Null
    }
    catch {
        $invalidVersionFailed = $true
    }
    if (-not $invalidVersionFailed) {
        throw 'Version validation accepted an intentionally wrong expected version.'
    }

    $commit = (git -C $repositoryRoot rev-parse HEAD).Trim()
    $installerName = "QRForge_${ExpectedVersion}_x64-setup.exe"
    $installerPath = Join-Path $testRoot $installerName
    Set-Content -LiteralPath $installerPath -Encoding utf8NoBOM -Value 'QRForge manifest fixture'
    $result = & (Join-Path $PSScriptRoot 'New-BuildManifest.ps1') `
        -InstallerPath $installerPath `
        -OutputDirectory $testRoot `
        -Version $ExpectedVersion `
        -Commit $commit
    $manifestPath = Join-Path $testRoot 'release-manifest.json'
    $manifestText = Get-Content -Raw -LiteralPath $manifestPath
    $manifest = $manifestText | ConvertFrom-Json -AsHashtable
    $actualHash = (Get-FileHash -LiteralPath $installerPath -Algorithm SHA256).Hash
    if ($manifest.installer.sha256 -ne $actualHash -or $result.Sha256 -ne $actualHash) {
        throw 'Generated manifest SHA-256 does not match the installer.'
    }
    if ($manifest.installer.signing -ne 'unsigned-release-candidate') {
        throw 'Generated manifest does not label the candidate unsigned.'
    }
    if ($manifestText.Contains($repositoryRoot, [StringComparison]::OrdinalIgnoreCase)) {
        throw 'Generated manifest contains an absolute repository path.'
    }
    $checksum = Get-Content -Raw -LiteralPath (Join-Path $testRoot 'SHA256SUMS.txt')
    if ($checksum -notmatch "^$actualHash  $([regex]::Escape($installerName))\s*$") {
        throw 'Generated checksum file has unexpected content.'
    }

    $wrongPath = Join-Path $testRoot 'wrong-name.exe'
    Set-Content -LiteralPath $wrongPath -Encoding utf8NoBOM -Value 'wrong name'
    $invalidNameFailed = $false
    try {
        & (Join-Path $PSScriptRoot 'New-BuildManifest.ps1') `
            -InstallerPath $wrongPath `
            -OutputDirectory $testRoot `
            -Version $ExpectedVersion `
            -Commit $commit | Out-Null
    }
    catch {
        $invalidNameFailed = $true
    }
    if (-not $invalidNameFailed) {
        throw 'Manifest generation accepted an invalid installer name.'
    }

    [pscustomobject]@{
        Version = $ExpectedVersion
        ManifestHash = $actualHash
        InvalidVersionRejected = $invalidVersionFailed
        InvalidInstallerNameRejected = $invalidNameFailed
        Status = 'passed'
    }
}
finally {
    if (Test-Path -LiteralPath $testRoot) {
        Remove-Item -LiteralPath $testRoot -Recurse -Force
    }
}
