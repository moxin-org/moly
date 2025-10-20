# iOS TestFlight Build and Upload

This document describes the process for building, signing, and uploading the Moly iOS app to TestFlight.

## Prerequisites

### 1. Apple Distribution Certificate

You need an Apple Distribution certificate installed in your Keychain:
- Certificate name format: `Apple Distribution: Your Name (TEAM_ID)`
- Get the SHA-1 fingerprint: `security find-certificate -c "Apple Distribution" -a -Z | grep SHA-1`

### 2. App Store Provisioning Profile

Required provisioning profile:
- Type: App Store Distribution
- Bundle ID: `org.moxin.moly`
- Location: `~/Library/MobileDevice/Provisioning Profiles/`
- Get the UUID: `security cms -D -i /path/to/profile.mobileprovision | grep UUID -A 1`

### 3. App Store Connect API Key

Create an API key in App Store Connect (Users and Access → Keys):
- Role: Admin or App Manager
- Download the `.p8` key file
- Note the Key ID and Issuer ID

### 4. Development Tools

- Xcode Command Line Tools (15.0+)
- cargo-makepad: `cargo install --git https://github.com/makepad/makepad.git --branch dev cargo-makepad`
- Rust toolchain 1.89+

## Build Process Overview

The build process consists of these steps:

1. **Build the iOS app** using cargo-makepad with distribution signing
2. **Compile asset catalog** to create `Assets.car` (required by Apple for iOS 11+)
3. **Patch Info.plist** with iOS-specific keys and version numbers
4. **Extract entitlements** from the provisioning profile
5. **Re-sign the app** (required after modifying bundle contents)
6. **Create the IPA** using `ditto` to preserve extended attributes
7. **Upload to TestFlight** via App Store Connect API

## Automated CI/CD Build

The GitHub Actions workflow in `.github/workflows/release.yml` automates the entire process.

### Required GitHub Secrets

Configure these secrets in your repository settings:

| Secret Name | Description | How to Generate |
|------------|-------------|-----------------|
| `BUILD_CERTIFICATE_BASE64` | Distribution certificate (.p12) | `base64 -i certificate.p12 \| pbcopy` |
| `P12_PASSWORD` | Certificate password | Password used when exporting .p12 |
| `BUILD_PROVISION_PROFILE_BASE64` | App Store provisioning profile | `base64 -i profile.mobileprovision \| pbcopy` |
| `KEYCHAIN_PASSWORD` | Temporary keychain password | Any secure random string |
| `APP_STORE_CONNECT_API_KEY_CONTENT` | API key file contents (.p8) | `cat AuthKey_XXXXX.p8` |
| `APP_STORE_CONNECT_KEY_ID` | API Key ID | From App Store Connect |
| `APP_STORE_CONNECT_ISSUER_ID` | Issuer ID | From App Store Connect |
| `MOLY_RELEASE` | GitHub token | Personal access token with repo access |

### CI Configuration Notes

- **Runner**: Must use `macos-15` for Xcode 16 / iOS 18 SDK
- **Build number**: Auto-increments using `github.run_number`
- **Version number**: Extracted from `Cargo.toml` (strips non-numeric suffixes like `-rc1`)
- **Signing**: Extracts certificate fingerprint and profile UUID automatically

## Manual Build (Local Development)

For manual builds, see [MANUAL_BUILD.md](./MANUAL_BUILD.md) for a complete copy-paste script with placeholders for your certificate and provisioning profile details.

## Important Technical Details

### Why Asset Catalog is Required

Apple requires iOS 11+ apps to use Asset Catalogs for app icons. The `actool` command compiles PNG icons into an `Assets.car` file that satisfies Apple's requirements. Loose PNG files are no longer accepted for App Store submission.

### Why Re-signing is Required

cargo-makepad signs the app during the build, but we need to:
1. Add the compiled asset catalog (`Assets.car`)
2. Patch the Info.plist with iOS-specific keys

Both operations modify the app bundle, which invalidates the original signature. We must re-sign after these modifications.

### Version Numbers

Apple requires specific version formats:
- `CFBundleShortVersionString`: Numeric only (e.g., `0.2.2`)
- `CFBundleVersion`: Build number (e.g., `42`, `100`)

cargo-makepad generates hardcoded version `1.0.0`, so we override it using PlistBuddy after the build.

### Binary Naming

The main Moly app binary is named `_moly_app` in `Cargo.toml` to avoid conflicts with the `moly-runner` binary. For iOS builds, we temporarily rename it to `moly` because cargo-makepad expects this name.

### IPA Packaging

We use `ditto` instead of `zip` to create the IPA because:
- `ditto` preserves macOS extended attributes
- Info.plist has extended attributes that Apple's validation requires
- Regular `zip` strips these attributes, causing validation failures

## Troubleshooting

### "device not found" during build

This is expected when building for device without one connected. The app is still built successfully in `target/makepad-apple-app/aarch64-apple-ios/release/moly.app`.

### "invalid signature" errors

Run the re-signing step again. Any modification to the app bundle (adding files, editing plists) invalidates the signature.

### Asset catalog errors

Ensure:
1. The asset catalog was compiled successfully (check for `Assets.car` in the app bundle)
2. Info.plist has `CFBundleIconName = "AppIcon"`
3. All required icon sizes exist in `packaging/ios-icons/`

### Upload fails with authentication errors

Verify:
1. API key file exists and is valid
2. Key ID and Issuer ID are correct
3. The API key has "Admin" or "App Manager" role in App Store Connect

### TestFlight processing

After upload:
- Build appears in App Store Connect within 5-30 minutes
- Processing can take 10-60 minutes depending on Apple's load
- Check https://appstoreconnect.apple.com/apps/6738328099/testflight/ios

## References

- [App Store Connect API documentation](https://developer.apple.com/documentation/appstoreconnectapi)
- [cargo-makepad documentation](https://github.com/makepad/makepad)
- [Apple code signing guide](https://developer.apple.com/support/code-signing/)
