# Windows MSIX Installer

This folder contains the necessary files to build an MSIX installer for Windows.

## Certificate

Building fend requires a valid certificate. See
[here](https://docs.microsoft.com/en-us/windows/msix/package/create-certificate-package-signing)
for more info.

This command will create a new self-signed certificate:

```ps1
New-SelfSignedCertificate -Type Custom -Subject "CN=printfn, O=printfn" -KeyUsage DigitalSignature -FriendlyName "fend package signing certificate" -CertStoreLocation "Cert:\CurrentUser\My" -TextExtension @("2.5.29.37={text}1.3.6.1.5.5.7.3.3", "2.5.29.19={text}")
```

Note the returned thumbprint.

This will export the certificate to a local password-protected file:

```ps1
$PFXPass = ConvertTo-SecureString -String "MyPassword" -Force -AsPlainText
Export-PfxCertificate -Cert cert:\CurrentUser\My\969D30F2816F4552F429511EFF9C6F2979E4B2F5 -Password $PFXPass -FilePath fend-signing-cert.pfx
```

## Installation

Because fend is signed with a self-signed certificate, the
certificate needs to be trusted before installation.
These steps are needed to trust fend's certificate:

1. Right-click the MSIX file, and open the "Properties" window
2. Open to the "Digital Signatures" tab
3. Select the signature and click on "Details"
4. In the "General" tab, click on "View Certificate"
5. In the "General" tab, click on "Install Certificate..."
6. **IMPORTANT**: Change the store location to "Local Machine", then click "Next"
7. Choose the option "Place all certificates in the following store", then click on "Browse..." and select the "Trusted People" store. The checkbox "Show physical stores" should be disabled. Then click on "OK" to confirm the store, and click on "Next".
8. Click on "Finish" to import the certificate.
9. Close the other windows, and proceed to install fend by double-clicking the MSIX file.
