#!/bin/bash

# ------------------------------- #

# Can be one of: arm-linux-androideabi, armv7-linux-androideabi, i686-linux-android, x86_64-linux-android, aarch64-linux-android, thumbv7neon-linux-androideabi
# For more information on which to pick, visit: https://doc.rust-lang.org/rustc/platform-support.html
ARCH="arm-linux-androideabi"

# Can be one of: x86, x86_64, armeabi, armeabi-v7a, arm64-v8a
ARCH_LIB="armeabi" 

# Replace this with your actual SDK build-tools path
SDK_TOOLCHAIN_PATH="/home/porya/Android/Sdk/build-tools/34.0.0" 

# Your app name
# Ensure this matches with your Cargo.toml's [package] --> name
APP_NAME="trs_24"

# Keystore password used for signing the APK
# ! - Be cautious about sharing this password - !
KEYSTORE_PASSWORD="123456"

# ------------------------------- #

if [ ! -d "../target/packaged" ]; then
    mkdir -p "../target/packaged"
fi

if [ ! -d "lib" ]; then
    mkdir -p "lib"
fi

if [ ! -d "lib/$ARCH_LIB" ]; then
    mkdir -p "lib/$ARCH_LIB"
fi

if [ ! -f "android.jar" ]; then
    echo "Missing android.jar file. Please find the file inside of your NDK directory and copy it in the same directory of this shell script."
    echo "You may try the android.jar file provided with TRS_24: https://github.com/p0ryae/TRS_24/blob/main/android/android.jar"
else

if [ ! -f "AndroidManifest.xml" ]; then
    echo "Missing AndroidManifest.xml file. PLease have the file in the same directory of this shell script."
    echo "You may try the AndroidManifest.xml file provided with TRS_24: https://github.com/p0ryae/TRS_24/blob/main/android/AndroidManifest.xml"
    echo "Ensure you have the right tweaks for your app within the Manifest! Don't just copy it and use it."
else

APP_NAME_LOWERCASED=$(echo $APP_NAME | tr '[A-Z]' '[a-z]'])

"$SDK_TOOLCHAIN_PATH/aapt" package -f -F "../target/packaged/$APP_NAME_LOWERCASED-unsigned.apk" -M "AndroidManifest.xml" -I "./android.jar"

if [ -f "../target/$ARCH/debug/lib$APP_NAME_LOWERCASED.so" ]; then
    cp ../target/$ARCH/debug/lib$APP_NAME_LOWERCASED.so lib/$ARCH_LIB/lib$APP_NAME_LOWERCASED.so

    "$SDK_TOOLCHAIN_PATH/aapt" add "../target/packaged/$APP_NAME_LOWERCASED-unsigned.apk" "lib/$ARCH_LIB/lib$APP_NAME_LOWERCASED.so"

    "$SDK_TOOLCHAIN_PATH/zipalign" -f -v 4 "../target/packaged/$APP_NAME_LOWERCASED-unsigned.apk" "../target/packaged/$APP_NAME_LOWERCASED.apk"

    "$SDK_TOOLCHAIN_PATH/apksigner" sign --ks "release.keystore" --ks-pass "pass:$KEYSTORE_PASSWORD" "../target/packaged/$APP_NAME_LOWERCASED.apk"

    echo "Done. Check target/packaged in root dir for the apk bundle."

    adb install -r "../target/packaged/$APP_NAME_LOWERCASED.apk"

    adb shell am start -n rust.$APP_NAME_LOWERCASED/android.app.NativeActivity
    
    # The line below is optional and used for waking the android screen
    adb shell input keyevent KEYCODE_WAKE
else
    echo "File 'lib$APP_NAME_LOWERCASED.so' does not exist."
fi

fi
fi
# --------------------------------- #