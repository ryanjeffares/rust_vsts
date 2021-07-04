#!/bin/bash

# Compile the dylib
cargo build --release

# Make the bundle folder
mkdir -p "vst/Oscicrate.vst/Contents/MacOS"

# Create the PkgInfo
echo "BNDL????" > "vst/Oscicrate.vst/Contents/PkgInfo"

#build the Info.Plist
echo "<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">
<plist version=\"1.0\">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>English</string>

    <key>CFBundleExecutable</key>
    <string>Oscicrate</string>

    <key>CFBundleGetInfoString</key>
    <string>vst</string>

    <key>CFBundleIconFile</key>
    <string></string>

    <key>CFBundleIdentifier</key>
    <string>com.rust-vst.Oscicrate</string>

    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>

    <key>CFBundleName</key>
    <string>Oscicrate</string>

    <key>CFBundlePackageType</key>
    <string>BNDL</string>

    <key>CFBundleVersion</key>
    <string>1.0</string>

    <key>CFBundleSignature</key>
    <string>$((RANDOM % 9999))</string>

    <key>CSResourcesFileMapped</key>
    <string></string>

</dict>
</plist>" > "vst/Oscicrate.vst/Contents/Info.plist"

# move the provided library to the correct location
cp "target/release/librs_synth.dylib" "vst/Oscicrate.vst/Contents/MacOS/Oscicrate"

echo "Created bundle Oscicrate.vst"