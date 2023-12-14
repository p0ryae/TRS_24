@echo off
setlocal enabledelayedexpansion

REM ------------------------------- #

REM Can be one of: arm-linux-androideabi, armv7-linux-androideabi, i686-linux-android, x86_64-linux-android, aarch64-linux-android, thumbv7neon-linux-androideabi
REM For more information on which to pick, visit: https://doc.rust-lang.org/rustc/platform-support.html
SET ARCH=

REM Can be one of: x86, x86_64, armeabi, armeabi-v7a, arm64-v8a
SET ARCH_LIB=

REM Replace this with your actual SDK build-tools path
SET SDK_TOOLCHAIN_PATH=

REM Your app name
REM Ensure this MATCHES with your Cargo.toml's [package] --> name
REM AVOID WHITESPACES!
SET APP_NAME=

REM Your app label
REM Having whitespaces is allowed
SET APP_LABEL=

REM Keystore password used for signing the APK
REM ! - Be cautious about sharing this password - !
SET KEYSTORE_PASSWORD=

REM ------------------------------- #

IF NOT EXIST "..\target\packaged" mkdir "..\target\packaged"
IF NOT EXIST "lib" mkdir "lib"
IF NOT EXIST "lib\%ARCH_LIB%" mkdir "lib\%ARCH_LIB%"

IF NOT EXIST "android.jar" (
    echo Missing android.jar file. Please find the file inside of your NDK directory and copy it in the same directory of this batch script.
    echo You may try the android.jar file provided with TRS_24: https://github.com/p0ryae/TRS_24/blob/main/android/android.jar
) ELSE (
    IF NOT EXIST "AndroidManifest.xml" (
        echo Missing AndroidManifest.xml file. Please have the file in the same directory of this batch script.
        echo You may try the AndroidManifest.xml file provided with TRS_24: https://github.com/p0ryae/TRS_24/blob/main/android/AndroidManifest.xml
        echo Ensure you have the right tweaks for your app within the Manifest! Don't just copy it and use it.
    ) ELSE (
        SET "APP_NAME_LOWERCASED=!APP_NAME:~0,1!"
        SET "APP_NAME_LOWERCASED=!APP_NAME_LOWERCASED![!APP_NAME:~1!]"

        SET "SED_CMD=sed -i 's/LIBNAME/%APP_NAME_LOWERCASED%/g' AndroidManifest.xml"
        FOR /F "delims=" %%i IN ('!SED_CMD!') DO !%%i!

        SET "SED_CMD=sed -i 's/APPLABEL/%APP_LABEL%/g' AndroidManifest.xml"
        FOR /F "delims=" %%i IN ('!SED_CMD!') DO !%%i!

        "%SDK_TOOLCHAIN_PATH%\aapt" package -f -F "..\target\packaged\%APP_NAME_LOWERCASED%-unsigned.apk" -M "AndroidManifest.xml" -I ".\android.jar"

        IF EXIST "..\target\%ARCH%\release\lib%APP_NAME_LOWERCASED%.so" (
            copy "..\target\%ARCH%\release\lib%APP_NAME_LOWERCASED%.so" "lib\%ARCH_LIB%\lib%APP_NAME_LOWERCASED%.so"

            "%SDK_TOOLCHAIN_PATH%\aapt" add "..\target\packaged\%APP_NAME_LOWERCASED%-unsigned.apk" "lib\%ARCH_LIB%\lib%APP_NAME_LOWERCASED%.so"

            "%SDK_TOOLCHAIN_PATH%\zipalign" -f -v 4 "..\target\packaged\%APP_NAME_LOWERCASED%-unsigned.apk" "..\target\packaged\%APP_NAME_LOWERCASED%.apk"

            "%SDK_TOOLCHAIN_PATH%\apksigner" sign --ks "release.keystore" --ks-pass "pass:%KEYSTORE_PASSWORD%" "..\target\packaged\%APP_NAME_LOWERCASED%.apk"

            echo Done. Check target\packaged in root dir for the apk bundle.

            adb install -r "..\target\packaged\%APP_NAME_LOWERCASED%.apk"

            adb shell am start -n rust.%APP_NAME_LOWERCASED%/android.app.NativeActivity

            REM The line below is optional and used for waking the Android screen
            adb shell input keyevent KEYCODE_WAKE

            SET "SED_CMD=sed -i 's/%APP_NAME_LOWERCASED%/LIBNAME/g' AndroidManifest.xml"
            FOR /F "delims=" %%i IN ('!SED_CMD!') DO !%%i!

            SET "SED_CMD=sed -i 's/%APP_LABEL%/APPLABEL/g' AndroidManifest.xml"
            FOR /F "delims=" %%i IN ('!SED_CMD!') DO !%%i!
        ) ELSE (
            echo File 'lib%APP_NAME_LOWERCASED%.so' does not exist.
        )
    )
)

REM --------------------------------- #
