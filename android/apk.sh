#!/bin/bash

if [ ! -d "../target/packaged" ]; then
    mkdir -p "../target/packaged"
fi

ARCH="arm-linux-androideabi"
ARCH_LIB="armeabi"

SDK_TOOLCHAIN_PATH="/home/porya/Android/Sdk/build-tools/34.0.0"  # Replace this with your actual path

"$SDK_TOOLCHAIN_PATH/aapt" package -f -F "../target/packaged/TRS_24-unsigned.apk" -M "/AndroidManifest.xml" -I "./android.jar"

if [ -f "../target/$ARCH/debug/libtrs_24.so" ]; then
    cp ../target/$ARCH/debug/libtrs_24.so lib/$ARCH_LIB/libtrs_24.so

    "$SDK_TOOLCHAIN_PATH/aapt" add "../target/packaged/TRS_24-unsigned.apk" "lib/$ARCH_LIB/libtrs_24.so"

    "$SDK_TOOLCHAIN_PATH/zipalign" -f -v 4 "../target/packaged/TRS_24-unsigned.apk" "../target/packaged/TRS_24.apk"

    "$SDK_TOOLCHAIN_PATH/apksigner" sign --ks "release.keystore" --ks-pass "pass:123456" "../target/packaged/TRS_24.apk"

    echo "Done. Check target/packaged in root dir for the apk bundle."

    adb install -r "../target/packaged/TRS_24.apk"
    adb shell am start -n rust.trs_24/android.app.NativeActivity

    adb shell input keyevent KEYCODE_WAKE
else
    echo "File 'libtrs_24.so' does not exist."
fi
