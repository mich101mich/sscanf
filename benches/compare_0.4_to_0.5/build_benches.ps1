$ErrorActionPreference = 'Stop'

$projectRoot = $PSScriptRoot
$manifestPath = Join-Path $projectRoot 'Cargo.toml'
$binaryPath = Join-Path $projectRoot 'target\release\sscanf_test.exe'
$outputDirectory = Join-Path $projectRoot 'binaries'
$originalManifest = Get-Content -Path $manifestPath -Raw

New-Item -ItemType Directory -Path $outputDirectory -Force | Out-Null

try {
    foreach ($configuration in @(
            @{ Version = '0.4.4'; Parser = $false },
            @{ Version = '0.5.0'; Parser = $false },
            @{ Version = '0.5.0'; Parser = $true }
        )) {
        $version = $configuration.Version
        $parser = $configuration.Parser
        $manifest = $originalManifest -replace 'sscanf\s*=\s*"[^"]+"', ('sscanf = "{0}"' -f $version)
        [System.IO.File]::WriteAllText($manifestPath, $manifest, (New-Object System.Text.UTF8Encoding -ArgumentList $false))

        foreach ($complexName in @('simple', 'complex', 'complex_no_numbers')) {
            foreach ($multi in @($false, $true)) {
                if ($parser -and -not $multi) {
                    continue
                }

                $features = @()
                if ($complexName -ne 'simple') {
                    $features += $complexName
                }
                if ($multi) {
                    $features += 'multi'
                }
                if ($parser) {
                    $features += 'use_parser'
                }

                $featureArguments = @()
                if ($features.Count -gt 0) {
                    $featureArguments = @('--features', ($features -join ','))
                }

                & cargo build --release @featureArguments
                if ($LASTEXITCODE -ne 0) {
                    throw "Cargo build failed for version $version, target=$complexName, multi=$multi."
                }

                $multiName = if ($multi) { 'multi' } else { 'single' }
                $parserName = if ($parser) { '_use_parser' } else { '' }
                $outputName = "sscanf_{0}_{1}_{2}{3}.exe" -f $complexName, $multiName, $version, $parserName
                Copy-Item -Path $binaryPath -Destination (Join-Path $outputDirectory $outputName) -Force
            }
        }
    }
}
finally {
    [System.IO.File]::WriteAllText($manifestPath, $originalManifest, (New-Object System.Text.UTF8Encoding -ArgumentList $false))
}

function Measure-Benchmark {
    param(
        [string]$Path,
        [string]$Target,
        [string]$Mode,
        [int]$WarmupRuns,
        [int]$Runs
    )

    Write-Host "Running benchmark: Path=$Path, Target=$Target, Mode=$Mode, WarmupRuns=$WarmupRuns, Runs=$Runs"

    for ($run = 0; $run -lt $WarmupRuns; $run++) {
        & $Path *> $null
        if ($LASTEXITCODE -ne 0) {
            throw "Benchmark failed during warmup: $Path"
        }
    }

    $measurement = Measure-Command {
        for ($run = 0; $run -lt $Runs; $run++) {
            & $Path *> $null
            if ($LASTEXITCODE -ne 0) {
                throw "Benchmark failed: $Path"
            }
        }
    }

    [PSCustomObject]@{
        Target              = $Target
        Mode                = $Mode
        Binary              = [System.IO.Path]::GetFileNameWithoutExtension($Path)
        Runs                = $Runs
        TotalMilliseconds   = [math]::Round($measurement.TotalMilliseconds, 2)
        AverageMilliseconds = [math]::Round($measurement.TotalMilliseconds / $Runs, 4)
    }
}

$results = @()
foreach ($complexName in @('simple', 'complex', 'complex_no_numbers')) {
    foreach ($binary in @(
            @{ Name = "sscanf_${complexName}_single_0.4.4.exe"; Mode = 'single' },
            @{ Name = "sscanf_${complexName}_single_0.5.0.exe"; Mode = 'single' },
            @{ Name = "sscanf_${complexName}_multi_0.4.4.exe"; Mode = 'multi' },
            @{ Name = "sscanf_${complexName}_multi_0.5.0.exe"; Mode = 'multi' },
            @{ Name = "sscanf_${complexName}_multi_0.5.0_use_parser.exe"; Mode = 'multi, parser' }
        )) {
        $warmupRuns = if ($binary.Mode -eq 'single') { 50 } else { 5 }
        $runs = if ($binary.Mode -eq 'single') { 1000 } else { 100 }
        $path = Join-Path $outputDirectory $binary.Name
        $results += Measure-Benchmark -Path $path -Target $complexName -Mode $binary.Mode `
            -WarmupRuns $warmupRuns -Runs $runs
    }
}

$results | Format-Table -AutoSize

