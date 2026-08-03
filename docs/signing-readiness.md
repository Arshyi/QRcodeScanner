# Windows signing readiness

Current QRForge executables and NSIS installers are **unsigned release
candidates**. Windows SmartScreen may warn even when SHA-256 matches because an
unsigned project has no publisher identity or reputation.

## Future certificate options

A production process may use a publicly trusted organization-validation or
extended-validation Windows code-signing certificate, or a managed cloud/HSM
signing service whose chain is trusted by target Windows versions. Confirm the
legal publisher name, EKU for code signing, key custody, export rules, rate
limits, audit logging, and renewal lead time before purchase.

The private key must live in a hardware token, HSM, or protected CI signing
service. Never store a PFX, private key, password, certificate token, or
long-lived cloud credential in Git, workflow YAML, build artifacts, logs, or a
developer environment file. Use a protected GitHub Environment with required
reviewers and short-lived identity where the signing provider supports it.
Untrusted pull-request code must never reach signing credentials.

## Intended signing sequence

1. Validate and build from the reviewed immutable commit.
2. Sign the final `qrforge.exe` before NSIS packaging.
3. Package the signed executable.
4. Sign the final NSIS installer.
5. Timestamp both signatures with the certificate authority's RFC 3161 HTTPS
   timestamp service so valid historical signatures survive certificate
   expiry.
6. Verify signatures and file hashes on a separate clean runner.
7. Produce the manifest only after final signing, marking its real signing
   state and certificate identity.

Example verification on Windows:

```powershell
Get-AuthenticodeSignature .\qrforge.exe | Format-List Status,StatusMessage,SignerCertificate,TimeStamperCertificate
Get-AuthenticodeSignature .\QRForge_0.1.2_x64-setup.exe | Format-List Status,StatusMessage,SignerCertificate,TimeStamperCertificate
Get-FileHash .\QRForge_0.1.2_x64-setup.exe -Algorithm SHA256
```

The current RC workflow deliberately contains no optional signing step: this
prevents a credential-bearing path from being enabled accidentally. A future
signing job should be isolated after validation, use least privilege, accept
only protected tags, have environment approval, and upload no key material.

Track certificate expiry, timestamp availability, access/audit events, and a
documented revocation contact. If compromise is suspected, disable the signing
job, revoke the certificate/provider key, preserve audit evidence, notify
users, and reissue from a clean reviewed commit.
