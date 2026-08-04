[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [ValidatePattern('^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$')]
    [string]$ExpectedVersion,
    [Parameter(Mandatory)]
    [ValidatePattern('^[0-9A-Za-z._/-]+$')]
    [string]$ExpectedRef,
    [Parameter()] [ValidatePattern('^[0-9a-fA-F]{40}$')] [string]$ExpectedCommit
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path

function Invoke-Checked {
    param(
        [Parameter(Mandatory)] [string]$FilePath,
        [Parameter(Mandatory)] [string[]]$ArgumentList,
        [Parameter()] [string]$WorkingDirectory = $repositoryRoot
    )
    Push-Location $WorkingDirectory
    try {
        & $FilePath @ArgumentList
        if ($LASTEXITCODE -ne 0) {
            throw "$FilePath $($ArgumentList -join ' ') failed with exit code $LASTEXITCODE."
        }
    }
    finally {
        Pop-Location
    }
}

function Assert-CleanRepository {
    $changes = @(git status --porcelain=v1 --untracked-files=all)
    if ($LASTEXITCODE -ne 0) {
        throw 'git status failed.'
    }
    if ($changes.Count -ne 0) {
        throw "Release validation requires a clean repository:`n$($changes -join [Environment]::NewLine)"
    }
}

Push-Location $repositoryRoot
try {
    Assert-CleanRepository
    if ($ExpectedRef.Contains('..')) {
        throw 'ExpectedRef may not contain a double-dot revision expression.'
    }
    $currentCommitOutput = @(git rev-parse HEAD)
    if ($LASTEXITCODE -ne 0 -or $currentCommitOutput.Count -ne 1) {
        throw 'Unable to resolve the current Git commit.'
    }
    $currentCommit = $currentCommitOutput[0].Trim().ToLowerInvariant()
    $expectedRefOutput = @(git rev-parse --verify "$ExpectedRef^{commit}")
    if ($LASTEXITCODE -ne 0 -or $expectedRefOutput.Count -ne 1) {
        throw "Unable to resolve intended ref $ExpectedRef to a commit."
    }
    $expectedRefCommit = $expectedRefOutput[0].Trim().ToLowerInvariant()
    if ($expectedRefCommit -ne $currentCommit) {
        throw "HEAD $currentCommit does not resolve from intended ref $ExpectedRef."
    }
    $symbolicRefOutput = @(git branch --show-current)
    if ($LASTEXITCODE -ne 0) {
        throw 'Unable to inspect the current Git branch.'
    }
    $symbolicRef = ($symbolicRefOutput -join '').Trim()
    if ($symbolicRef -and $symbolicRef -ne $ExpectedRef) {
        throw "Current branch is $symbolicRef; expected $ExpectedRef."
    }
    if (-not $ExpectedCommit) {
        $ExpectedCommit = $currentCommit
    }
    if ($currentCommit -ne $ExpectedCommit.ToLowerInvariant()) {
        throw "Current commit is $currentCommit; expected $ExpectedCommit."
    }
    $env:QRFORGE_BUILD_COMMIT = $currentCommit

    & (Join-Path $PSScriptRoot 'Test-VersionConsistency.ps1') -ExpectedVersion $ExpectedVersion
    if ($LASTEXITCODE -ne 0) { throw 'Version consistency validation failed.' }
    & (Join-Path $PSScriptRoot 'Test-ReleaseTooling.ps1') -ExpectedVersion $ExpectedVersion
    if ($LASTEXITCODE -ne 0) { throw 'Release tooling tests failed.' }

    Invoke-Checked 'cargo' @('fmt', '--all', '--', '--check')
    Invoke-Checked 'cargo' @('fmt', '--manifest-path', 'spikes/Cargo.toml', '--all', '--', '--check')
    Invoke-Checked 'cargo' @('check', '--workspace', '--all-targets', '--locked')
    Invoke-Checked 'cargo' @('test', '--workspace', '--locked')
    Invoke-Checked 'cargo' @('clippy', '--workspace', '--all-targets', '--locked', '--', '-D', 'warnings')
    Invoke-Checked 'cargo' @('check', '--manifest-path', 'spikes/Cargo.toml', '--workspace', '--all-targets', '--locked')
    Invoke-Checked 'cargo' @('test', '--manifest-path', 'spikes/Cargo.toml', '--workspace', '--locked')
    Invoke-Checked 'cargo' @('clippy', '--manifest-path', 'spikes/Cargo.toml', '--workspace', '--all-targets', '--locked', '--', '-D', 'warnings')

    $desktop = Join-Path $repositoryRoot 'apps/desktop'
    Invoke-Checked 'npm' @('ci', '--no-audit', '--fund=false') $desktop
    foreach ($script in 'format:check', 'lint', 'typecheck', 'test', 'build') {
        Invoke-Checked 'npm' @('run', $script) $desktop
    }
    & (Join-Path $PSScriptRoot 'Test-DependencyPolicy.ps1')
    if ($LASTEXITCODE -ne 0) { throw 'Dependency policy validation failed.' }

    $reportPath = Join-Path $repositoryRoot 'target/release-validation/decoder-summary.json'
    $env:QRFORGE_REPORT_PATH = $reportPath
    Invoke-Checked 'cargo' @('run', '--release', '--manifest-path', 'spikes/decoder-comparison/Cargo.toml', '--locked', '--bin', 'qrforge-decoder-comparison')
    Invoke-Checked 'cargo' @('build', '--release', '--workspace', '--locked')
    Invoke-Checked 'cargo' @('build', '--release', '--manifest-path', 'spikes/Cargo.toml', '--workspace', '--locked')
    Invoke-Checked 'npm' @('run', 'tauri', '--', 'build', '--bundles', 'nsis') $desktop

    $bundleDirectory = Join-Path $repositoryRoot 'target/release/bundle/nsis'
    $installers = @(Get-ChildItem -LiteralPath $bundleDirectory -Filter '*-setup.exe' -File)
    if ($installers.Count -ne 1) {
        throw "Expected exactly one NSIS installer; found $($installers.Count)."
    }
    $expectedInstaller = "QRForge_${ExpectedVersion}_x64-setup.exe"
    if ($installers[0].Name -ne $expectedInstaller) {
        throw "Installer is $($installers[0].Name); expected $expectedInstaller."
    }
    $manifest = & (Join-Path $PSScriptRoot 'New-BuildManifest.ps1') `
        -InstallerPath $installers[0].FullName `
        -OutputDirectory $bundleDirectory `
        -Version $ExpectedVersion `
        -Commit $currentCommit
    if ($LASTEXITCODE -ne 0) { throw 'Build manifest generation failed.' }

    Assert-CleanRepository
    [pscustomobject]@{
        Version = $ExpectedVersion
        Ref = $ExpectedRef
        Commit = $currentCommit
        Installer = $manifest.Installer
        Sha256 = $manifest.Sha256
        Signing = $manifest.Signing
        Status = 'validated'
    }
}
finally {
    Pop-Location
}
