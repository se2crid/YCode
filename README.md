# YCode

iOS Development IDE for linux and windows, built with [Tauri](https://tauri.app/).

Coming soon...

## How it works

- [Theos](https://theos.dev/) is used to build the project into an IPA.
- [apple-private-apis](https://github.com/SideStore/apple-private-apis) is used to login to the Apple Account. Heavy additions have been made to support actually accessing the Developer APIs
- [idevice](https://github.com/jkcoxson/idevice) (will be) used to install the IPA on the device.

- [Sideloader](https://github.com/Dadoum/Sideloader) has been heavily used as a reference for the implementation of the Apple Developer APIs, signing, and sideloading process.

## Progress

**Installing App**

- [x] Login to Apple Account
- [x] Create lockdown connection with device (retrives name)
- [x] Register Device as a development devices
- [x] Create/Save Certificate for YCode
- [x] Create an App ID for the app
- [x] Create & manage an application group for the app
- [x] Acquire a provisioning profile for the app
- [ ] Sign the app
- [ ] Install the app!

**Coding App**

- [x] Rudimentary File Browser
- [x] Code editor (monaco editor)
- [ ] Project Templates
- [ ] Swift LSP Support
- [ ] UI to change makefile settings

## What AI did

- Generated the logo
- Helped port some code from [Sideloader](https://github.com/Dadoum/Sideloader)
