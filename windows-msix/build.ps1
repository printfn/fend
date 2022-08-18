$ErrorActionPreference = "Stop"

<#

Before running this script, make sure the FEND_VERSION and WINDOWS_CERT_PASSWORD environment variables are set.

For example:
$Env:FEND_VERSION = "1.0.0"
$Env:WINDOWS_CERT_PASSWORD = "MyPassword"

#>

# $PSScriptRoot is the directory of this script

if (Test-Path $PSScriptRoot\build) {
    Remove-Item -Recurse -Force $PSScriptRoot\build
}

if (Test-Path $PSScriptRoot\fend.msix) {
    Remove-Item -Force $PSScriptRoot\fend.msix
}

if (Test-Path $PSScriptRoot\fend-windows-x64.msix) {
    Remove-Item -Force $PSScriptRoot\fend-windows-x64.msix
}

mkdir $PSScriptRoot\build
Copy-Item $PSScriptRoot\..\target\release\fend.exe $PSScriptRoot\build
(Get-Content $PSScriptRoot\AppxManifest.xml).replace('$FEND_VERSION', $Env:FEND_VERSION) | Set-Content $PSScriptRoot\build\AppxManifest.xml
Copy-Item $PSScriptRoot\..\icon\fend-icon-44.png $PSScriptRoot\build
Copy-Item $PSScriptRoot\..\icon\fend-icon-150.png $PSScriptRoot\build

& "C:\Program Files (x86)\Windows Kits\10\App Certification Kit\makeappx.exe" pack /d $PSScriptRoot\build /p $PSScriptRoot\fend.msix /verbose
& "C:\Program Files (x86)\Windows Kits\10\App Certification Kit\signtool.exe" sign /fd SHA256 /a /f $PSScriptRoot\fend-signing-cert.pfx /p $Env:WINDOWS_CERT_PASSWORD $PSScriptRoot\fend.msix

mv $PSScriptRoot\fend.msix $PSScriptRoot\fend-windows-x64.msix
