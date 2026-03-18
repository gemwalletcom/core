XCBEAUTIFY_ARGS := "--quieter --is-ci"
BUILD_THREADS := `sysctl -n hw.ncpu`
SIMULATOR_NAME := env_var_or_default("SIMULATOR_NAME", "iPhone 17")
SIMULATOR_DEST := "platform=iOS Simulator,name=" + SIMULATOR_NAME
DERIVED_DATA := "build/DerivedData"
SPM_CACHE := "build/SourcePackages"
FAST_BUILD_FLAGS := "GCC_OPTIMIZATION_LEVEL=0 SWIFT_OPTIMIZATION_LEVEL=-Onone SWIFT_COMPILATION_MODE=incremental ENABLE_TESTABILITY=NO"

default:
    @just --list

bootstrap: install generate-stone
    @echo "<== Bootstrap done."

install: install-rust install-typeshare install-toolchains install-swifttools

install-rust:
    just core install-rust

install-typeshare:
    @echo "==> Install typeshare-cli"
    just core install-typeshare

install-toolchains:
    @echo "==> Install toolchains for uniffi"
    @cd core && just gemstone install-ios-targets

install-swifttools:
    @echo "==> Install SwiftGen and SwiftFormat"
    @brew install swiftgen swiftformat

download-wallet-core VERSION:
    @echo "==> Install wallet-core {{VERSION}}"
    @curl -sL https://github.com/trustwallet/wallet-core/releases/download/{{VERSION}}/Package.swift -o Packages/WalletCore/Package.swift

setup-git:
    @echo "==> Setup git submodules"
    @git submodule update --init --recursive
    @git config submodule.recurse true

core-upgrade:
    @git submodule update --recursive --remote

spm-resolve:
    @xcodebuild -resolvePackageDependencies -project Gem.xcodeproj -scheme Gem -derivedDataPath {{DERIVED_DATA}} -clonedSourcePackagesDirPath {{SPM_CACHE}}

spm-resolve-all:
    @sh scripts/spm-resolve-all.sh

lint:
    @echo "==> Running SwiftLint"
    @swiftlint --fix --quiet

_build action extra_flags="":
    @set -o pipefail && xcodebuild -project Gem.xcodeproj \
    -scheme Gem \
    ONLY_ACTIVE_ARCH=YES \
    -destination "{{SIMULATOR_DEST}}" \
    -derivedDataPath {{DERIVED_DATA}} \
    -clonedSourcePackagesDirPath {{SPM_CACHE}} \
    -parallelizeTargets \
    -jobs {{BUILD_THREADS}} \
    -showBuildTimingSummary \
    {{extra_flags}} \
    {{action}} | xcbeautify {{XCBEAUTIFY_ARGS}}

# Example: just build
build: (_build "build" FAST_BUILD_FLAGS)

# Example: just build-for-testing
build-for-testing: (_build "build-for-testing")

clean:
    @rm -rf {{DERIVED_DATA}}
    @echo "Build cache cleaned"

run: build
    @echo "==> Installing app on simulator..."
    @xcrun simctl boot "{{SIMULATOR_NAME}}" 2>/dev/null || true
    @open -a Simulator
    @xcrun simctl install "{{SIMULATOR_NAME}}" {{DERIVED_DATA}}/Build/Products/Debug-iphonesimulator/Gem.app
    @echo "==> Launching app..."
    @xcrun simctl launch --console-pty "{{SIMULATOR_NAME}}" com.gemwallet.ios

# Example: just build-package Primitives
build-package PACKAGE:
    @set -o pipefail && xcodebuild -project Gem.xcodeproj \
    -scheme {{PACKAGE}} \
    ONLY_ACTIVE_ARCH=YES \
    -destination "{{SIMULATOR_DEST}}" \
    -derivedDataPath {{DERIVED_DATA}} \
    -clonedSourcePackagesDirPath {{SPM_CACHE}} \
    -parallelizeTargets \
    -jobs {{BUILD_THREADS}} \
    GCC_OPTIMIZATION_LEVEL=0 \
    SWIFT_OPTIMIZATION_LEVEL=-Onone \
    build | xcbeautify {{XCBEAUTIFY_ARGS}}

show-simulator:
    @echo "Destination: {{SIMULATOR_DEST}}"
    @xcrun simctl list devices | grep "iPhone" | head -5 || true

_test action target="":
    @set -o pipefail && xcodebuild -project Gem.xcodeproj \
    -scheme Gem \
    ONLY_ACTIVE_ARCH=YES \
    -destination "{{SIMULATOR_DEST}}" \
    -derivedDataPath {{DERIVED_DATA}} \
    -clonedSourcePackagesDirPath {{SPM_CACHE}} \
    {{ if target != "" { "-only-testing " + target } else { "" } }} \
    -parallel-testing-enabled YES \
    -parallelizeTargets \
    -jobs {{BUILD_THREADS}} \
    {{action}} | xcbeautify {{XCBEAUTIFY_ARGS}}

test-all: (_test "test")

test-without-building: (_test "test-without-building")

# Example: just test PrimitivesTests
test TARGET: (_test "test" TARGET)

_test-ui action:
    @set -o pipefail && xcodebuild -project Gem.xcodeproj \
    -scheme GemUITests \
    -testPlan ui_tests \
    ONLY_ACTIVE_ARCH=YES \
    -destination "{{SIMULATOR_DEST}}" \
    -derivedDataPath {{DERIVED_DATA}} \
    -clonedSourcePackagesDirPath {{SPM_CACHE}} \
    -allowProvisioningUpdates \
    -allowProvisioningDeviceRegistration \
    {{action}} | xcbeautify {{XCBEAUTIFY_ARGS}}

test-ui: reset-simulator (_test-ui "test")

build-for-testing-ui: (_test-ui "build-for-testing")

test-ui-without-building: reset-simulator (_test-ui "test-without-building")

reset-simulator NAME=SIMULATOR_NAME:
    @echo "==> Resetting {{NAME}} simulator to clean state"
    @xcrun simctl shutdown "{{NAME}}" 2>/dev/null || true
    @xcrun simctl erase "{{NAME}}" 2>/dev/null || true
    @xcrun simctl boot "{{NAME}}" 2>/dev/null || true

_localize languages="":
    @sh core/scripts/localize.sh ios Packages/Localization/Sources/Resources {{languages}}
    just generate-model
    just generate-swiftgen

localize: (_localize "en")

localize-all: (_localize)

generate: generate-model generate-swiftgen

generate-model:
    @echo "==> Generate typeshare for iOS"
    @cd core && cargo run --package generate --bin generate ios ../Packages

generate-swiftgen:
    @echo "==> SwiftGen assets and Localizable.strings"
    @swiftgen config run --quiet

export BUILD_MODE := env_var_or_default("BUILD_MODE","")

generate-stone:
    @./scripts/generate-stone.sh $BUILD_MODE

bump TYPE="":
    @sh ./scripts/bump.sh {{TYPE}}

mod core
