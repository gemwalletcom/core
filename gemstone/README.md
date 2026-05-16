# Gemstone

Gemstone is the essential cross platform library used by Gem Wallet clients (mainly iOS and Android).

## Build

iOS example

```bash
just build-ios
just test-ios
```

`just build-ios` and `just test-ios` create the local Swift package sources, build the native Gemstone static library, and build the SwiftPM-based iOS test harness.

Android

```bash
just bindgen-kotlin && just build-android
```

you can check out `tests` folder to see how to use it.
