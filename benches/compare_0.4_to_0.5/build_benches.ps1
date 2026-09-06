$ErrorActionPreference = 'Stop'

$projectRoot = $PSScriptRoot
$manifestPath = Join-Path $projectRoot 'Cargo.toml'
$binaryPath = Join-Path $projectRoot 'target\release\sscanf_test.exe'
$outputDirectory = Join-Path $projectRoot 'binaries'
$originalManifest = Get-Content -Path $manifestPath -Raw

New-Item -ItemType Directory -Path $outputDirectory -Force | Out-Null

try {
    foreach ($version in @('0.4.4', '0.5.0')) {
        $manifest = $originalManifest -replace 'sscanf\s*=\s*"[^"]+"', ('sscanf = "{0}"' -f $version)
        [System.IO.File]::WriteAllText($manifestPath, $manifest, (New-Object System.Text.UTF8Encoding -ArgumentList $false))

        foreach ($complex in @($false, $true)) {
            foreach ($multi in @($false, $true)) {
                $features = @()
                if ($complex) {
                    $features += 'complex'
                }
                if ($multi) {
                    $features += 'multi'
                }
                if ($version -eq '0.5.0') {
                    $features += 'new_sscanf'
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
                $outputName = "sscanf_{0}_{1}_{2}.exe" -f $complexName, $multiName, $version
                Copy-Item -Path $binaryPath -Destination (Join-Path $outputDirectory $outputName) -Force
            }
        }
    }
}
finally {
    [System.IO.File]::WriteAllText($manifestPath, $originalManifest, (New-Object System.Text.UTF8Encoding -ArgumentList $false))
}

foreach ($complex in @($false, $true)) {
    # foreach ($multi in @($false, $true)) {
        $complexName = if ($complex) { 'complex' } else { 'simple' }
        $multiName = 'multi' # if ($multi) { 'multi' } else { 'single' }
        $binary044 = Join-Path $outputDirectory ("sscanf_{0}_{1}_0.4.4.exe" -f $complexName, $multiName)
        $binary050 = Join-Path $outputDirectory ("sscanf_{0}_{1}_0.5.0.exe" -f $complexName, $multiName)

        & hyperfine -w 10 -r 100 $binary044 $binary050
        if ($LASTEXITCODE -ne 0) {
            throw "Hyperfine benchmark failed for complex=$complex, multi=$multi."
        }
    # }
}

