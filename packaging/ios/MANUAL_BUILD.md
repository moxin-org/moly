# iOS TestFlight Upload - Manual Build Script

This document provides a complete copy-paste script for building, signing, and uploading the Moly iOS app to TestFlight manually from your local machine.

## Prerequisites Setup

Before running the script, ensure you have:

1. **Apple Distribution Certificate** installed in Keychain
   - Get SHA-1 fingerprint: `security find-certificate -c "Apple Distribution" -a -Z | grep SHA-1 | head -n 1 | awk '{print $3}'`

2. **App Store Provisioning Profile** installed at `~/Library/MobileDevice/Provisioning Profiles/`
   - Get UUID: `security cms -D -i ~/Library/MobileDevice/Provisioning\ Profiles/<YOUR_PROFILE>.mobileprovision | grep UUID -A 1 | grep string | sed 's/.*<string>\(.*\)<\/string>.*/\1/'`

3. **App Store Connect API Key** saved at `~/private_keys/AuthKey_<YOUR_KEY_ID>.p8`
   - Get Key ID and Issuer ID from App Store Connect → Users and Access → Keys

4. **Development Tools**
   - Xcode Command Line Tools: `xcode-select --install`
   - cargo-makepad: `cargo install --git https://github.com/makepad/makepad.git --branch dev cargo-makepad`
   - Rust 1.89+: `rustup update`

## Complete Build and Upload Script

Replace the placeholders at the top with your values, then run the entire script. The process takes about 5-10 minutes.

```bash
# =============================================================================
# CONFIGURATION - Replace these with your actual values
# =============================================================================
CERT_SHA1="<YOUR_CERTIFICATE_SHA1>"  # e.g., 8670295495F61C7AB19FD70E2ADDCBCDC76E61C1
CERT_NAME="<YOUR_CERTIFICATE_NAME>"  # e.g., "Apple Distribution: Your Name (TEAM_ID)"
PROFILE_UUID="<YOUR_PROFILE_UUID>"   # e.g., 001d53e8-f724-4746-abdd-4babf17a07d9
API_KEY_ID="<YOUR_API_KEY_ID>"       # e.g., M92599MZQ5
API_ISSUER_ID="<YOUR_API_ISSUER_ID>" # e.g., 3c642d0d-74ad-4899-8709-292dec70b58c
BUILD_NUMBER="<INCREMENT_THIS>"      # Increment for each TestFlight upload (e.g., 1, 2, 3, ...)

# =============================================================================
# STEP 1: Build the iOS app with cargo-makepad
# =============================================================================
cd moly

# Temporarily rename binary for iOS build (cargo-makepad expects "moly")
sed -i '' 's/name = "_moly_app"/name = "moly"/' Cargo.toml

# Build for iOS device (will fail with "device not found" but app builds successfully)
cargo makepad apple ios --org=org.moxin --app=moly \
  --profile=$PROFILE_UUID \
  --cert=$CERT_SHA1 \
  --device=IPhone \
  run-device -p moly --release

# Restore original binary name
git checkout Cargo.toml

# =============================================================================
# STEP 2: Compile Asset Catalog
# =============================================================================

# Compile pre-built asset catalog into Assets.car file
xcrun actool ./packaging/ios-icons/Assets.xcassets \
  --compile ./target/makepad-apple-app/aarch64-apple-ios/release/moly.app \
  --platform iphoneos \
  --minimum-deployment-target 14.0 \
  --app-icon AppIcon \
  --output-partial-info-plist /tmp/AssetInfo.plist

# =============================================================================
# STEP 3: Patch Info.plist with iOS-specific keys and version numbers
# =============================================================================

# Navigate to the app bundle directory for convenience
cd ./target/makepad-apple-app/aarch64-apple-ios/release/moly.app

# First, merge actool's generated plist (contains CFBundleIcons dictionary)
/usr/libexec/PlistBuddy -c "Merge /tmp/AssetInfo.plist" Info.plist

# Manually add iOS-specific keys (PlistBuddy doesn't expand ~ in paths, so we add them directly)
# These keys are required by Apple for App Store submission

# Add CFBundlePackageType (required)
/usr/libexec/PlistBuddy -c "Add :CFBundlePackageType string APPL" Info.plist 2>/dev/null || \
/usr/libexec/PlistBuddy -c "Set :CFBundlePackageType APPL" Info.plist

# Add CFBundleIconName for asset catalog (required for iOS 11+)
/usr/libexec/PlistBuddy -c "Add :CFBundleIconName string AppIcon" Info.plist 2>/dev/null || \
/usr/libexec/PlistBuddy -c "Set :CFBundleIconName AppIcon" Info.plist

# Add UILaunchScreen dictionary (required for iPad multitasking)
/usr/libexec/PlistBuddy -c "Add :UILaunchScreen dict" Info.plist 2>/dev/null || true
/usr/libexec/PlistBuddy -c "Add :UILaunchScreen:UIImageName string AppIcon60x60" Info.plist 2>/dev/null || \
/usr/libexec/PlistBuddy -c "Set :UILaunchScreen:UIImageName AppIcon60x60" Info.plist
/usr/libexec/PlistBuddy -c "Add :UILaunchScreen:UIColorName string LaunchScreenBackground" Info.plist 2>/dev/null || \
/usr/libexec/PlistBuddy -c "Set :UILaunchScreen:UIColorName LaunchScreenBackground" Info.plist

# Add encryption declaration (required for TestFlight external testing)
/usr/libexec/PlistBuddy -c "Add :ITSAppUsesNonExemptEncryption bool false" Info.plist 2>/dev/null || \
/usr/libexec/PlistBuddy -c "Set :ITSAppUsesNonExemptEncryption false" Info.plist

# IMPORTANT: Set version keys (cargo-makepad generates hardcoded "1.0.0", we must override)
# Extract version from Cargo.toml and strip any non-numeric suffixes (Apple requires numeric-only)
VERSION=$(cd ~/dev/work/moly && cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version' | sed 's/-.*$//')

echo "Setting version: $VERSION (build: $BUILD_NUMBER)"
/usr/libexec/PlistBuddy -c "Set :CFBundleShortVersionString $VERSION" Info.plist
/usr/libexec/PlistBuddy -c "Set :CFBundleVersion $BUILD_NUMBER" Info.plist

# Verify all required keys are present
echo "✅ Current bundle configuration:"
/usr/libexec/PlistBuddy -c "Print :CFBundleShortVersionString" Info.plist
/usr/libexec/PlistBuddy -c "Print :CFBundleVersion" Info.plist
/usr/libexec/PlistBuddy -c "Print :CFBundlePackageType" Info.plist
/usr/libexec/PlistBuddy -c "Print :CFBundleIconName" Info.plist
/usr/libexec/PlistBuddy -c "Print :ITSAppUsesNonExemptEncryption" Info.plist

# Return to project root
cd ~/dev/work/moly

# =============================================================================
# STEP 4: Extract entitlements from provisioning profile
# =============================================================================
security cms -D -i ~/Library/MobileDevice/Provisioning\ Profiles/${PROFILE_UUID}.mobileprovision > /tmp/profile.plist
/usr/libexec/PlistBuddy -x -c "Print :Entitlements" /tmp/profile.plist > /tmp/entitlements.plist

# =============================================================================
# STEP 5: Re-sign the app (required after modifying bundle contents)
# =============================================================================
codesign --force --sign "$CERT_NAME" \
  --entitlements /tmp/entitlements.plist \
  --timestamp=none \
  ./target/makepad-apple-app/aarch64-apple-ios/release/moly.app

# =============================================================================
# STEP 6: Verify the signature is valid
# =============================================================================
codesign -vvv ./target/makepad-apple-app/aarch64-apple-ios/release/moly.app

echo "✅ Code signature verification:"
codesign -d --entitlements - ./target/makepad-apple-app/aarch64-apple-ios/release/moly.app

# =============================================================================
# STEP 7: Create the .ipa package
# =============================================================================
cd ./target/makepad-apple-app/aarch64-apple-ios/release
rm -rf Payload Moly-${VERSION}-ios.ipa

# Use ditto to preserve all macOS extended attributes and metadata
ditto moly.app Payload/moly.app

# Create IPA using ditto (required to preserve Info.plist extended attributes)
ditto -c -k --sequesterRsrc --keepParent Payload Moly-${VERSION}-ios.ipa

echo "✅ Created IPA: $(pwd)/Moly-${VERSION}-ios.ipa"
ls -lh Moly-${VERSION}-ios.ipa

# =============================================================================
# STEP 8: Upload to TestFlight via App Store Connect API
# =============================================================================
echo "📤 Uploading to TestFlight..."
xcrun altool --upload-app --type ios \
  --file Moly-${VERSION}-ios.ipa \
  --apiKey $API_KEY_ID \
  --apiIssuer $API_ISSUER_ID

echo ""
echo "✅ Upload complete!"
echo "📱 The build will appear in App Store Connect in ~30 minutes after processing."
echo "🔗 https://appstoreconnect.apple.com/apps/<YOUR_APP_ID>/testflight/ios"
```

## How to Use This Script

1. **Copy the script** to a text editor
2. **Replace the placeholders** in the configuration section at the top:
   - `<YOUR_CERTIFICATE_SHA1>`: Run `security find-certificate -c "Apple Distribution" -a -Z | grep SHA-1 | head -n 1 | awk '{print $3}'`
   - `<YOUR_CERTIFICATE_NAME>`: Your full certificate name from Keychain (e.g., "Apple Distribution: Your Name (TEAM_ID)")
   - `<YOUR_PROFILE_UUID>`: Your provisioning profile UUID
   - `<YOUR_API_KEY_ID>`: From App Store Connect
   - `<YOUR_API_ISSUER_ID>`: From App Store Connect
   - `<INCREMENT_THIS>`: Start with 1, increment for each upload (2, 3, 4, ...)
   - `<YOUR_APP_ID>`: Your app's ID from App Store Connect (in the final URL)

3. **Save the script** with your values somewhere convenient (e.g., `~/ios-build.sh`)
4. **Make it executable**: `chmod +x ~/ios-build.sh`
5. **Run it**: `~/ios-build.sh`

## Important Notes

### Build Number Requirement

Apple requires that each TestFlight upload has a **unique build number**. You must increment `BUILD_NUMBER` for every upload.

If you forget to increment it, Apple will reject the upload with a "duplicate build number" error.

### Version Format

Apple requires version strings to be numeric only:
- ✅ Valid: `0.2.2`, `1.0.0`, `2.1.3`
- ❌ Invalid: `0.2.2-rc1`, `1.0.0-beta`

The script automatically strips any non-numeric suffixes from your `Cargo.toml` version using `sed 's/-.*$//'`.

### Expected "device not found" Error

In Step 1, you'll see an error like:
```
Error: could not find device "IPhone"
```

**This is expected and normal!** The app builds successfully even though cargo-makepad can't deploy it to a physical device. The built app is at `target/makepad-apple-app/aarch64-apple-ios/release/moly.app`.

### Troubleshooting

**"invalid signature" errors:**
- Make sure you ran Step 5 (re-signing) after Step 3 (Info.plist patching)
- Any modification to the app bundle invalidates the signature

**Asset catalog errors:**
- Verify `Assets.car` exists in the app bundle after Step 2
- Check that all icon files exist in `packaging/ios-icons/`

**Upload authentication errors:**
- Verify API key file exists at `~/private_keys/AuthKey_<YOUR_KEY_ID>.p8`
- Ensure the API key has "Admin" or "App Manager" role in App Store Connect
- Double-check Key ID and Issuer ID are correct

**TestFlight processing:**
- Builds typically appear in App Store Connect within 5-30 minutes
- Full processing can take 10-60 minutes
- Check for email notifications from Apple about the build status

## See Also

- [README.md](./README.md) - Overview and CI/CD setup
- [GitHub Actions workflow](../../.github/workflows/release.yml) - Automated build configuration
