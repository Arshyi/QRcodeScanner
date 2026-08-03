[CmdletBinding()]
param(
    [Parameter(Mandatory)] [string]$InstallerPath,
    [Parameter(Mandatory)] [string]$OutputDirectory,
    [Parameter(Mandatory)]
    [ValidatePattern('^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$')]
    [string]$Version,
    [Parameter(Mandatory)]
    [ValidatePattern('^[0-9a-fA-F]{40}$')]
    [string]$Commit
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$installer = Get-Item -LiteralPath $InstallerPath
$output = New-Item -ItemType Directory -Force -Path $OutputDirectory
$expectedName = "QRForge_${Version}_x64-setup.exe"
if ($installer.Name -ne $expectedName) {
    throw "Installer is $($installer.Name); expected $expectedName."
}

$hash = (Get-FileHash -LiteralPath $installer.FullName -Algorithm SHA256).Hash.ToUpperInvariant()
$checksumPath = Join-Path $output.FullName 'SHA256SUMS.txt'
$manifestPath = Join-Path $output.FullName 'release-manifest.json'
Set-Content -LiteralPath $checksumPath -Encoding utf8NoBOM -Value "$hash  $($installer.Name)"

$manifest = [ordered]@{
    schemaVersion = 1
    product = 'QRForge'
    version = $Version
    commit = $Commit.ToLowerInvariant()
    platform = 'windows'
    architecture = 'x86_64'
    installer = [ordered]@{
        file = $installer.Name
        bytes = $installer.Length
        sha256 = $hash
        signing = 'unsigned-release-candidate'
    }
    generatedAtUtc = [DateTime]::UtcNow.ToString('o')
}
$manifest | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath $manifestPath -Encoding utf8NoBOM

[pscustomobject]@{
    Installer = $installer.Name
    Sha256 = $hash
    Manifest = $manifestPath
    Checksums = $checksumPath
    Signing = 'unsigned-release-candidate'
}
