#!/usr/bin/env bash
set -euo pipefail

COMMIT="${1:?Usage: just upgrade-test <commit-or-tag>}"
REPO_ROOT="$(git rev-parse --show-toplevel)"
WORKTREE_DIR="/tmp/gem-upgrade-test-$$"
SIMULATOR_NAME="${SIMULATOR_NAME:-iPhone 17}"
SIMULATOR_DEST="platform=iOS Simulator,name=$SIMULATOR_NAME"
OLD_DERIVED_DATA="$WORKTREE_DIR/build/DerivedData"

cleanup() {
    echo "==> Cleaning up worktree"
    cd "$REPO_ROOT"
    git worktree remove --force "$WORKTREE_DIR" 2>/dev/null || true
}
trap cleanup EXIT

echo "==> Phase 1: Build old version ($COMMIT)"

git worktree add "$WORKTREE_DIR" "$COMMIT"
cd "$WORKTREE_DIR"
git submodule update --init

echo "==> Generating stone (old)"
just generate-stone

echo "==> Building old version for UI testing"
set -o pipefail && xcodebuild -project Gem.xcodeproj \
    -scheme GemUITests \
    -testPlan ui_tests \
    ONLY_ACTIVE_ARCH=YES \
    -destination "$SIMULATOR_DEST" \
    -derivedDataPath "$OLD_DERIVED_DATA" \
    -allowProvisioningUpdates \
    -allowProvisioningDeviceRegistration \
    build-for-testing | xcbeautify --quieter --is-ci

echo "==> Resetting simulator"
cd "$REPO_ROOT"
just reset-simulator

echo "==> Running ImportWalletReceiveBitcoinUITests (old version)"
cd "$WORKTREE_DIR"
set -o pipefail && xcodebuild -project Gem.xcodeproj \
    -scheme GemUITests \
    -testPlan ui_tests \
    ONLY_ACTIVE_ARCH=YES \
    -destination "$SIMULATOR_DEST" \
    -derivedDataPath "$OLD_DERIVED_DATA" \
    -allowProvisioningUpdates \
    -allowProvisioningDeviceRegistration \
    -only-testing GemUITests/ImportWalletReceiveBitcoinUITests \
    test-without-building | xcbeautify --quieter --is-ci

echo "==> Phase 2: Build current version"
cd "$REPO_ROOT"

just build-for-testing-ui

DERIVED_DATA="build/DerivedData"

echo "==> Running UpgradeVerificationTests (current version)"
set -o pipefail && xcodebuild -project Gem.xcodeproj \
    -scheme GemUITests \
    -testPlan ui_tests \
    ONLY_ACTIVE_ARCH=YES \
    -destination "$SIMULATOR_DEST" \
    -derivedDataPath "$DERIVED_DATA" \
    -allowProvisioningUpdates \
    -allowProvisioningDeviceRegistration \
    -only-testing GemUITests/UpgradeVerificationTests \
    test-without-building | xcbeautify --quieter --is-ci

echo "==> Upgrade test passed!"
