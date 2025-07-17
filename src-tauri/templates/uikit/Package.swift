// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "{{projectName}}",
    platforms: [.iOS(.v15)],
    targets: [
        .executableTarget(
            name: "{{projectName}}",
            path: "src"
        ),
    ]
)
