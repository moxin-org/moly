# iOS TestFlight Upload - Manual Process

This document describes the manual steps to build, sign, and upload the Moly iOS app to TestFlight.

## Prerequisites

1. **Apple Distribution Certificate** installed in Keychain
   - Certificate: "Apple Distribution: Julian Montes de Oca (Y6MN6N88UF)"
   - SHA1: `8670295495F61C7AB19FD70E2ADDCBCDC76E61C1`

2. **App Store Provisioning Profile** installed
   - Profile UUID: `001d53e8-f724-4746-abdd-4babf17a07d9`
   - Location: `~/Library/MobileDevice/Provisioning Profiles/001d53e8-f724-4746-abdd-4babf17a07d9.mobileprovision`
   - Bundle ID: `org.moxin.moly`

3. **App Store Connect API Key**
   - Key ID: `M92599MZQ5`
   - Issuer ID: `3c642d0d-74ad-4899-8709-292dec70b58c`
   - Key file: `~/private_keys/AuthKey_M92599MZQ5.p8`

4. **Tools Required**
   - Xcode Command Line Tools
   - cargo-makepad
   - Rust toolchain 1.89+

## Complete Build and Upload Script

Run these commands in order. The entire process takes about 5-10 minutes depending on your machine.

```bash
# =============================================================================
# STEP 1: Build the iOS app with cargo-makepad
# =============================================================================
cd ~/dev/work/moly

# Temporarily rename binary for iOS build (cargo-makepad expects "moly")
sed -i '' 's/name = "_moly_app"/name = "moly"/' Cargo.toml

# Build for iOS device (will fail with "device not found" but app builds successfully)
cargo makepad apple ios --org=org.moxin --app=moly \
  --profile=001d53e8-f724-4746-abdd-4babf17a07d9 \
  --cert=8670295495F61C7AB19FD70E2ADDCBCDC76E61C1 \
  --device=IPhone \
  run-device -p moly --release

# Restore original binary name
git checkout Cargo.toml

# =============================================================================
# STEP 2: Create and compile Asset Catalog (required by Apple)
# =============================================================================

# Create asset catalog directory structure
mkdir -p /tmp/Assets.xcassets/AppIcon.appiconset

# Create Contents.json for the asset catalog
cat > /tmp/Assets.xcassets/AppIcon.appiconset/Contents.json << 'EOF'
{
  "images" : [
    {
      "filename" : "AppIcon-60@2x.png",
      "idiom" : "iphone",
      "scale" : "2x",
      "size" : "60x60"
    },
    {
      "filename" : "AppIcon-60@3x.png",
      "idiom" : "iphone",
      "scale" : "3x",
      "size" : "60x60"
    },
    {
      "filename" : "AppIcon-76@2x.png",
      "idiom" : "ipad",
      "scale" : "2x",
      "size" : "76x76"
    },
    {
      "filename" : "AppIcon-83.5@2x.png",
      "idiom" : "ipad",
      "scale" : "2x",
      "size" : "83.5x83.5"
    },
    {
      "filename" : "AppIcon-1024.png",
      "idiom" : "ios-marketing",
      "scale" : "1x",
      "size" : "1024x1024"
    }
  ],
  "info" : {
    "author" : "xcode",
    "version" : 1
  }
}
EOF

# Copy icon files with correct naming for asset catalog
cp ./packaging/ios-icons/AppIcon120x120.png /tmp/Assets.xcassets/AppIcon.appiconset/AppIcon-60@2x.png
cp ./packaging/ios-icons/AppIcon180x180.png /tmp/Assets.xcassets/AppIcon.appiconset/AppIcon-60@3x.png
cp ./packaging/ios-icons/AppIcon152x152.png /tmp/Assets.xcassets/AppIcon.appiconset/AppIcon-76@2x.png
cp ./packaging/ios-icons/AppIcon167x167.png /tmp/Assets.xcassets/AppIcon.appiconset/AppIcon-83.5@2x.png
cp ./packaging/ios-icons/AppIcon1024x1024.png /tmp/Assets.xcassets/AppIcon.appiconset/AppIcon-1024.png

# Compile asset catalog into Assets.car file
xcrun actool /tmp/Assets.xcassets \
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
BUILD_NUMBER="2"  # INCREMENT THIS for each TestFlight upload (Apple requires unique build numbers)

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
security cms -D -i ~/Library/MobileDevice/Provisioning\ Profiles/001d53e8-f724-4746-abdd-4babf17a07d9.mobileprovision > /tmp/profile.plist
/usr/libexec/PlistBuddy -x -c "Print :Entitlements" /tmp/profile.plist > /tmp/entitlements.plist

# =============================================================================
# STEP 5: Re-sign the app (required after modifying bundle contents)
# =============================================================================
codesign --force --sign "Apple Distribution: Julian Montes de Oca (Y6MN6N88UF)" \
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
rm -rf Payload Moly-0.2.2-ios.ipa

# Use ditto to preserve all macOS extended attributes and metadata
ditto moly.app Payload/moly.app

# Create IPA using ditto (required to preserve Info.plist extended attributes)
ditto -c -k --sequesterRsrc --keepParent Payload Moly-0.2.2-ios.ipa

echo "✅ Created IPA: $(pwd)/Moly-0.2.2-ios.ipa"
ls -lh Moly-0.2.2-ios.ipa

# =============================================================================
# STEP 8: Upload to TestFlight via App Store Connect API
# =============================================================================
echo "📤 Uploading to TestFlight..."
xcrun altool --upload-app --type ios \
  --file Moly-0.2.2-ios.ipa \
  --apiKey M92599MZQ5 \
  --apiIssuer 3c642d0d-74ad-4899-8709-292dec70b58c

echo ""
echo "✅ Upload complete!"
echo "📱 The build will appear in App Store Connect in ~30 minutes after processing."
echo "🔗 https://appstoreconnect.apple.com/apps/6738328099/testflight/ios"
```

## Troubleshooting

### "device not found" error in Step 1
This is expected! cargo-makepad tries to deploy to a physical device but we don't have one connected. The app is still built successfully in `target/makepad-apple-app/aarch64-apple-ios/release/moly.app`.

### "invalid signature" errors
Make sure you ran Step 5 (re-signing) after any modifications to the app bundle. Modifying Info.plist or adding files invalidates the original signature.

### Asset catalog errors
If you get "Missing required icon file" errors, ensure:
1. The asset catalog was compiled successfully in Step 2
2. An `Assets.car` file exists in the app bundle
3. The Info.plist has `CFBundleIconName = "AppIcon"`

### Upload fails with authentication errors
Verify:
1. API key file exists at `~/private_keys/AuthKey_M92599MZQ5.p8`
2. Key ID and Issuer ID are correct
3. The API key has "Admin" or "App Manager" role in App Store Connect

## CI/CD Automation

The GitHub Actions workflow in `.github/workflows/release.yml` automates this entire process. Key points:

1. **macOS runner version**: Must use `macos-15` for Xcode 16 / iOS 18 SDK
2. **Secrets required**:
   - `BUILD_CERTIFICATE_BASE64`: Distribution certificate (.p12 file, base64 encoded)
   - `P12_PASSWORD`: Certificate password
   - `BUILD_PROVISION_PROFILE_BASE64`: Provisioning profile (.mobileprovision file, base64 encoded)
   - `KEYCHAIN_PASSWORD`: Temporary keychain password
   - `APP_STORE_CONNECT_API_KEY_CONTENT`: API key (.p8 file contents)
   - `APP_STORE_CONNECT_KEY_ID`: API Key ID
   - `APP_STORE_CONNECT_ISSUER_ID`: Issuer ID

## Important Notes

### Why Asset Catalog is Required
Apple requires iOS 11+ apps to use Asset Catalogs for app icons. Loose PNG files are no longer accepted. The `actool` command in Step 2 compiles our PNG icons into an `Assets.car` file that satisfies Apple's requirements.

### Why Re-signing is Required
cargo-makepad signs the app after building, but we need to:
1. Add the compiled asset catalog
2. Patch the Info.plist with iOS-specific keys

Both operations modify the app bundle, which invalidates the original signature. We must re-sign in Step 5.

### Binary Naming
The main Moly app binary is named `_moly_app` in `Cargo.toml` to avoid conflicts with the `moly-runner` binary. For iOS builds, we temporarily rename it to `moly` because cargo-makepad expects this name.

### Version Numbers and cargo-makepad
cargo-makepad generates the iOS Info.plist with hardcoded version numbers (`1.0.0`), ignoring the version in `Cargo.toml`. We must use PlistBuddy's `Set` command to explicitly override the version keys after building.

**For each new TestFlight upload:**
1. Increment `BUILD_NUMBER` in Step 3 (Apple rejects duplicate build numbers)
2. The script automatically extracts the version from `Cargo.toml` and strips non-numeric suffixes

**Apple Version Format Requirements:**
- `CFBundleShortVersionString` must be numeric only: "0.2.0" ✅, "0.2.0-rc1" ❌
- `CFBundleVersion` can be numeric: "2", "3", "100"
- The script uses `sed 's/-.*$//'` to strip suffixes like "-rc1" from Cargo.toml version

**Why We Manually Add Keys Instead of Merging:**
PlistBuddy doesn't expand `~` in file paths, so merging `~/path/to/Info-iOS.plist` fails silently. Instead, we directly add each required key using `Add` and `Set` commands.

## Version History

- **2025-10-14**: Initial documentation
- Moly version: 0.2.2
- iOS minimum deployment target: 14.0
- Xcode version: 16.0 (iOS 18 SDK)
