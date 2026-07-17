[CmdletBinding()]
param(
    [string]$Owner = "TeamXcelerator",
    [string]$UserName = "TeamXceleratorDev",
    [switch]$ReplaceCredential
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

function Invoke-GitCredentialFill {
    $request = "protocol=https`nhost=github.com`n`n"
    $response = $request | git credential fill 2>$null
    if ($LASTEXITCODE -ne 0) { return $null }
    return ($response -join "`n")
}

function Save-GitHubCredential {
    param([securestring]$Token)

    $pointer = [Runtime.InteropServices.Marshal]::SecureStringToBSTR($Token)
    try {
        $plain = [Runtime.InteropServices.Marshal]::PtrToStringBSTR($pointer)
        $credential = "protocol=https`nhost=github.com`nusername=$UserName`npassword=$plain`n`n"
        $credential | git credential approve
        if ($LASTEXITCODE -ne 0) {
            throw "Git Credential Manager rejected the credential."
        }
    }
    finally {
        if ($null -ne $pointer) {
            [Runtime.InteropServices.Marshal]::ZeroFreeBSTR($pointer)
        }
        Remove-Variable plain, credential -ErrorAction SilentlyContinue
    }
}

if ($ReplaceCredential -or -not (Invoke-GitCredentialFill)) {
    $token = Read-Host "GitHub PAT (stored by Git Credential Manager; input is hidden)" -AsSecureString
    Save-GitHubCredential -Token $token
    Remove-Variable token
}

$toolkitRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\xcelerator-toolkit")).Path
$repositories = @(
    "xcelerator-cache-private-registry",
    "xcelerator-cache-private-quadrature-0001",
    "xcelerator-cache-private-ccm-components-0001",
    "xcelerator-cache-private-ccm-matrices-0001",
    "xcelerator-cache-private-weil-states-0001",
    "xcelerator-cache-private-prolate-0001",
    "xcelerator-cache-private-ccm-roots-0001",
    "xcelerator-cache-private-ccm-evidence-0001"
)

Write-Host "Checking authenticated write permission without modifying GitHub..."
foreach ($repository in $repositories) {
    $target = "$Owner/$repository"
    cargo run --quiet --manifest-path (Join-Path $toolkitRoot "Cargo.toml") `
        -p xc-cache --example github_permission_preflight -- $target | Out-Null
    if ($LASTEXITCODE -ne 0) {
        throw "Private-cache permission preflight failed for $target."
    }
    Write-Host "  write authorized: $target"
}

Write-Host "Private-cache authentication is ready. No repository was modified."
