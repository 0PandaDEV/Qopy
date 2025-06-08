#!/bin/bash

if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

set -e

required_vars=("APPLE_CERTIFICATE" "APPLE_CERTIFICATE_PASSWORD" "APPLE_ID" "APPLE_ID_PASSWORD" "KEYCHAIN_PASSWORD" "APP_BUNDLE_ID")
for var in "${required_vars[@]}"; do
    if [ -z "${!var}" ]; then
        exit 1
    fi
done

bun run tauri build

rm -f certificate.p12
echo "$APPLE_CERTIFICATE" | base64 --decode > certificate.p12 2>/dev/null
security import certificate.p12 -P "$APPLE_CERTIFICATE_PASSWORD" -A 2>/dev/null

SIGNING_IDENTITY=$(security find-identity -v -p codesigning | grep "Apple Development" | head -1 | awk -F '"' '{print $2}')

if [ -z "$SIGNING_IDENTITY" ]; then
    exit 1
fi

codesign --force --options runtime --sign "$SIGNING_IDENTITY" src-tauri/target/release/bundle/macos/*.app 2>/dev/null

rm -f certificate.p12

hdiutil create -volname "Qopy" -srcfolder src-tauri/target/release/bundle/dmg -ov -format UDZO Qopy.dmg

codesign --force --sign "$APPLE_CERTIFICATE" Qopy.dmg 2>/dev/null

xcrun notarytool submit Qopy.dmg --apple-id "$APPLE_ID" --password "$APPLE_ID_PASSWORD" --team-id "$APPLE_CERTIFICATE" --wait

xcrun stapler staple Qopy.dmg

exit 0
