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

        foreach ($complex in @($false, $true)) {
            foreach ($multi in @($false, $true)) {
                if ($parser -and -not $multi) {
                    continue
                }

                $features = @()
                if ($complex) {
                    $features += 'complex'
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
                    throw "Cargo build failed for version $version, complex=$complex, multi=$multi."
                }

                $complexName = if ($complex) { 'complex' } else { 'simple' }
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

foreach ($complex in @($false, $true)) {
    $complexName = if ($complex) { 'complex' } else { 'simple' }

    $singleBinaries = @(
        Join-Path $outputDirectory "sscanf_${complexName}_single_0.4.4.exe"
        Join-Path $outputDirectory "sscanf_${complexName}_single_0.5.0.exe"
    )
    & hyperfine -w 50 -r 1000 @singleBinaries
    if ($LASTEXITCODE -ne 0) {
        throw "Hyperfine benchmark failed for complex=$complex, multi=false."
    }

    $multiBinaries = @(
        Join-Path $outputDirectory "sscanf_${complexName}_multi_0.4.4.exe"
        Join-Path $outputDirectory "sscanf_${complexName}_multi_0.5.0.exe"
        Join-Path $outputDirectory "sscanf_${complexName}_multi_0.5.0_use_parser.exe"
    )
    & hyperfine -w 5 -r 100 @multiBinaries
    if ($LASTEXITCODE -ne 0) {
        throw "Hyperfine benchmark failed for complex=$complex, multi=true."
    }
}

