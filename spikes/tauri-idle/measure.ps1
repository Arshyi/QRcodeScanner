param(
  [int]$WarmupSeconds = 10,
  [int]$SampleSeconds = 30,
  [string]$Binary = "",
  [string]$Arguments = ""
)

$ErrorActionPreference = "Stop"
if (-not $Binary) {
  $Binary = Join-Path $PSScriptRoot "..\target\release\qrforge-tauri-idle-spike.exe"
}
$Binary = [IO.Path]::GetFullPath($Binary)
if (-not (Test-Path -LiteralPath $Binary)) {
  throw "Release binary not found: $Binary"
}

if ($Arguments) {
  $root = Start-Process -FilePath $Binary -ArgumentList $Arguments -PassThru -WindowStyle Hidden
}
else {
  $root = Start-Process -FilePath $Binary -PassThru -WindowStyle Hidden
}
try {
  Start-Sleep -Seconds $WarmupSeconds
  $logicalProcessors = [Environment]::ProcessorCount
  $samples = @()
  $previousCpu = 0.0
  $previousTime = Get-Date

  for ($sample = 0; $sample -lt $SampleSeconds; $sample++) {
    $processTable = Get-CimInstance Win32_Process | Select-Object ProcessId,ParentProcessId
    $ids = [Collections.Generic.HashSet[int]]::new()
    [void]$ids.Add($root.Id)
    do {
      $previousCount = $ids.Count
      foreach ($process in $processTable) {
        if ($ids.Contains([int]$process.ParentProcessId)) {
          [void]$ids.Add([int]$process.ProcessId)
        }
      }
    } while ($ids.Count -gt $previousCount)

    $processes = @($ids | ForEach-Object { Get-Process -Id $_ -ErrorAction SilentlyContinue })
    $now = Get-Date
    $cpuTotal = ($processes | Measure-Object CPU -Sum).Sum
    $privateBytes = ($processes | Measure-Object PrivateMemorySize64 -Sum).Sum
    if ($sample -gt 0) {
      $elapsed = ($now - $previousTime).TotalSeconds
      $cpuPercent = (($cpuTotal - $previousCpu) / $elapsed / $logicalProcessors) * 100.0
      $samples += [pscustomobject]@{
        cpu_percent = $cpuPercent
        private_mib = $privateBytes / 1MB
        process_count = $processes.Count
      }
    }
    $previousCpu = $cpuTotal
    $previousTime = $now
    Start-Sleep -Seconds 1
  }

  function Measure-Stats([double[]]$Values) {
    $sorted = @($Values | Sort-Object)
    $medianIndex = [Math]::Ceiling(($sorted.Count - 1) * 0.50)
    $p95Index = [Math]::Ceiling(($sorted.Count - 1) * 0.95)
    [ordered]@{
      min = $sorted[0]
      median = $sorted[$medianIndex]
      p95 = $sorted[$p95Index]
      max = $sorted[-1]
      mean = ($sorted | Measure-Object -Average).Average
    }
  }

  [ordered]@{
    spike = "tauri-idle"
    binary = $Binary
    arguments = $Arguments
    warmup_seconds = $WarmupSeconds
    sample_seconds = $SampleSeconds
    logical_processors = $logicalProcessors
    cpu_percent = Measure-Stats @($samples.cpu_percent)
    private_mib = Measure-Stats @($samples.private_mib)
    process_count_max = ($samples.process_count | Measure-Object -Maximum).Maximum
  } | ConvertTo-Json -Depth 5
}
finally {
  Stop-Process -Id $root.Id -Force -ErrorAction SilentlyContinue
}
