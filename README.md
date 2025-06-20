# YCode

iOS Development IDE for linux

Coming soon...

## How it works

- [Theos](https://theos.dev/) is used to build the project into an IPA.
- [apple-private-apis](https://github.com/SideStore/apple-private-apis) is used to login to the Apple Account. Heavy additions have been made to support actually accessing the Developer APIs
- [ZSign](https://github.com/zhlynn/zsign) (will be) used to sign the IPA with the certificate retrieved from the Apple Account.
- [idevice](https://github.com/jkcoxson/idevice) (will be) used to install the IPA on the device.

- [Sideloader](https://github.com/Dadoum/Sideloader) has been heavily used as a reference for the implementation of the Apple Developer APIs and sideloading process.

## Progress

**Installing App**

- [x] Login to Apple Account
- [x] Create lockdown connection with device (retrives name)
- [x] Register Device as a development devices
- [ ] Create/Save Certificate for YCode
- [ ] Create an App ID for the app
- [ ] Create & manage an application group for the app
- [ ] Sign the app
- [ ] Install the app!

**Coding App**

- [x] Rudimentary File Browser
- [x] Code editor (monaco editor)
- [ ] Project Templates
- [ ] Swift LSP Support
- [ ] UI to change makefile settings
