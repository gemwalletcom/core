LIB_NAME := gemstone
STATIC_LIB_NAME := lib$(LIB_NAME).a
DY_LIB_NAME := lib$(LIB_NAME).dylib
UDL_NAME := $(LIB_NAME).udl
BUILD_MODE := release

FW_NAME := Gemstone
FW_FFI_NAME := $(FW_NAME)FFI
FW_FFI_FILE := $(FW_FFI_NAME).framework
XC_FW_FFI_NAME := $(FW_FFI_NAME).xcframework
MODULE_MAP := $(FW_FFI_NAME).modulemap

TARGET_DIR := ../target
GEN_SWIFT_FOLDER := generated/swift
GEN_KOTLIN_FOLDER := generated/kotlin
TARGET_XC_FW_FOLDER := target/spm

prepare-apple:
	rustup toolchain install nightly
	rustup target add aarch64-apple-ios-sim aarch64-apple-ios x86_64-apple-ios
	rustup component add rust-src --toolchain nightly

prepare-android:
	rustup toolchain install stable
	rustup target add x86_64-linux-android
	rustup target add x86_64-unknown-linux-gnu
	rustup target add aarch64-linux-android
	rustup target add armv7-linux-androideabi
	rustup target add i686-linux-android

apple:
	@make build-targets
	@make bindgen-swift
	@make assemble-frameworks
	@make xcframework
	@make cp-xcframework-source

build-targets:
	cargo build --lib --target x86_64-apple-ios --$(BUILD_MODE)
	cargo build --lib --target aarch64-apple-ios-sim --$(BUILD_MODE)
	cargo +nightly build -Z build-std --lib --target aarch64-apple-ios-macabi --$(BUILD_MODE)
	cargo +nightly build -Z build-std --lib --target x86_64-apple-ios-macabi --$(BUILD_MODE)
	cargo build --lib --target aarch64-apple-ios --$(BUILD_MODE)

bindgen-swift:
	mkdir -p $(GEN_SWIFT_FOLDER)
	cargo build --$(BUILD_MODE)
	cargo run -p uniffi-bindgen generate --language swift --lib-file $(TARGET_DIR)/$(BUILD_MODE)/$(DY_LIB_NAME) src/$(UDL_NAME) -o $(GEN_SWIFT_FOLDER)
	# append framework prefix
	sed -i '' 's/module\ $(FW_FFI_NAME)/framework\ module\ $(FW_FFI_NAME)/' $(GEN_SWIFT_FOLDER)/$(MODULE_MAP)

bindgen-kotlin:
	mkdir -p $(GEN_KOTLIN_FOLDER)
	cargo run -p uniffi-bindgen generate src/$(UDL_NAME) --language kotlin -o $(GEN_KOTLIN_FOLDER)
	# strip uniffi_ prefix
	sed -i '' 's/return "uniffi_$(FW_NAME)"/return "$(LIB_NAME)"/' $(GEN_KOTLIN_FOLDER)/uniffi/$(FW_NAME)/$(FW_NAME).kt

assemble-frameworks:
	cd $(TARGET_DIR) && find . -type d -name $(FW_FFI_FILE) | xargs rm -rf
	cd $(TARGET_DIR)/x86_64-apple-ios/$(BUILD_MODE) && mkdir -p $(FW_FFI_FILE) && cd $(FW_FFI_FILE) && mkdir Headers Modules Resources && pwd && cp ../../../../$(LIB_NAME)/$(GEN_SWIFT_FOLDER)/$(MODULE_MAP) ./Modules/module.modulemap && cp ../../../../$(LIB_NAME)/$(GEN_SWIFT_FOLDER)/$(FW_FFI_NAME).h ./Headers && cp ../$(STATIC_LIB_NAME) ./$(FW_FFI_NAME) && cp ../../../../$(LIB_NAME)/src/Info.plist ./Resources
	cd $(TARGET_DIR)/aarch64-apple-ios-sim/$(BUILD_MODE) && mkdir -p $(FW_FFI_FILE) && cd $(FW_FFI_FILE) && mkdir Headers Modules Resources && cp ../../../../$(LIB_NAME)/$(GEN_SWIFT_FOLDER)/$(MODULE_MAP) ./Modules/module.modulemap && cp ../../../../$(LIB_NAME)/$(GEN_SWIFT_FOLDER)/$(FW_FFI_NAME).h ./Headers && cp ../$(STATIC_LIB_NAME) ./$(FW_FFI_NAME) && cp ../../../../$(LIB_NAME)/src/Info.plist ./Resources
	cd $(TARGET_DIR)/aarch64-apple-ios/$(BUILD_MODE) && mkdir -p $(FW_FFI_FILE) && cd $(FW_FFI_FILE) && mkdir Headers Modules Resources && cp ../../../../$(LIB_NAME)/$(GEN_SWIFT_FOLDER)/$(MODULE_MAP) ./Modules/module.modulemap && cp ../../../../$(LIB_NAME)/$(GEN_SWIFT_FOLDER)/$(FW_FFI_NAME).h ./Headers && cp ../$(STATIC_LIB_NAME) ./$(FW_FFI_NAME) && cp ../../../../$(LIB_NAME)/src/Info.plist ./Resources
	cd $(TARGET_DIR)/aarch64-apple-ios-macabi/$(BUILD_MODE) && mkdir -p $(FW_FFI_FILE) && cd $(FW_FFI_FILE) && mkdir Headers Modules Resources && cp ../../../../$(LIB_NAME)/$(GEN_SWIFT_FOLDER)/$(MODULE_MAP) ./Modules/module.modulemap && cp ../../../../$(LIB_NAME)/$(GEN_SWIFT_FOLDER)/$(FW_FFI_NAME).h ./Headers && cp ../$(STATIC_LIB_NAME) ./$(FW_FFI_NAME) && cp ../../../../$(LIB_NAME)/src/Info.plist ./Resources
	cd $(TARGET_DIR)/x86_64-apple-ios-macabi/$(BUILD_MODE) && mkdir -p $(FW_FFI_FILE) && cd $(FW_FFI_FILE) && mkdir Headers Modules Resources && cp ../../../../$(LIB_NAME)/$(GEN_SWIFT_FOLDER)/$(MODULE_MAP) ./Modules/module.modulemap && cp ../../../../$(LIB_NAME)/$(GEN_SWIFT_FOLDER)/$(FW_FFI_NAME).h ./Headers && cp ../$(STATIC_LIB_NAME) ./$(FW_FFI_NAME) && cp ../../../../$(LIB_NAME)/src/Info.plist ./Resources

xcframework:
	rm -rf $(TARGET_XC_FW_FOLDER)/Sources/$(XC_FW_FFI_NAME) && mkdir -p $(TARGET_XC_FW_FOLDER)/Sources
	lipo -create $(TARGET_DIR)/x86_64-apple-ios/$(BUILD_MODE)/$(FW_FFI_FILE)/$(FW_FFI_NAME) $(TARGET_DIR)/aarch64-apple-ios-sim/$(BUILD_MODE)/$(FW_FFI_FILE)/$(FW_FFI_NAME) -output $(TARGET_DIR)/aarch64-apple-ios-sim/$(BUILD_MODE)/$(FW_FFI_FILE)/$(FW_FFI_NAME)
	lipo -create $(TARGET_DIR)/x86_64-apple-ios-macabi/$(BUILD_MODE)/$(FW_FFI_FILE)/$(FW_FFI_NAME) $(TARGET_DIR)/aarch64-apple-ios-macabi/$(BUILD_MODE)/$(FW_FFI_FILE)/$(FW_FFI_NAME) -output $(TARGET_DIR)/aarch64-apple-ios-macabi/$(BUILD_MODE)/$(FW_FFI_FILE)/$(FW_FFI_NAME)
	xcodebuild -create-xcframework -framework $(TARGET_DIR)/aarch64-apple-ios/$(BUILD_MODE)/$(FW_FFI_FILE) -framework $(TARGET_DIR)/aarch64-apple-ios-sim/$(BUILD_MODE)/$(FW_FFI_FILE) -framework $(TARGET_DIR)/aarch64-apple-ios-macabi/$(BUILD_MODE)/$(FW_FFI_FILE) -output $(TARGET_XC_FW_FOLDER)/Sources/$(XC_FW_FFI_NAME)

cp-xcframework-source:
	mkdir -p $(TARGET_XC_FW_FOLDER)/Sources/$(FW_NAME)
	cp src/Package.swift $(TARGET_XC_FW_FOLDER)
	cp $(GEN_SWIFT_FOLDER)/${FW_NAME}.swift $(TARGET_XC_FW_FOLDER)/Sources/$(FW_NAME)

android:
	#make sure you install sdkman and jdk17
	cp -Rf $(GEN_KOTLIN_FOLDER)/uniffi tests/android/GemTest/app/src/main/java
	cd tests/android/GemTest && ./gradlew cargoBuild --info
