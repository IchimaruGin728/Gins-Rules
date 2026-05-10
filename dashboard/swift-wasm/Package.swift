// swift-tools-version: 6.3
import PackageDescription

let package = Package(
  name: "GinsRulesWasm",
  platforms: [.macOS(.v14)],
  products: [
    .executable(name: "GinsRulesWasm", targets: ["GinsRulesWasm"])
  ],
  dependencies: [
    .package(url: "https://github.com/swiftwasm/JavaScriptKit", from: "0.52.0")
  ],
  targets: [
    .executableTarget(
      name: "GinsRulesWasm",
      dependencies: [
        .product(name: "JavaScriptKit", package: "JavaScriptKit")
      ]
    )
  ]
)
