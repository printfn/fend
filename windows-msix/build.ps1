$ErrorActionPreference = "Stop"

# $PSScriptRoot is the directory of this script

if (Test-Path $PSScriptRoot\build) {
    Remove-Item -Recurse -Force $PSScriptRoot\build
}

mkdir $PSScriptRoot\build
Copy-Item $PSScriptRoot\..\target\release\fend.exe $PSScriptRoot\build
Copy-Item $PSScriptRoot\AppxManifest.xml $PSScriptRoot\build
Copy-Item $PSScriptRoot\..\icon\f-icon-128.png $PSScriptRoot\build

& "C:\Program Files (x86)\Windows Kits\10\App Certification Kit\makeappx.exe" pack /d $PSScriptRoot\build /p $PSScriptRoot\fend.msix /verbose
& "C:\Program Files (x86)\Windows Kits\10\App Certification Kit\signtool.exe" sign /fd SHA256 /a /f $PSScriptRoot\fend-signing-cert.pfx /p $Env:WINDOWS_CERT_PASSWORD $PSScriptRoot\fend.msix
