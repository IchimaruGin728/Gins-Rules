// swift-tools-version: 6.3.1
import PackageDescription

let package = Package(
  name: "GinsRules",
  platforms: [
    .macOS(.v15)
  ],
  products: [
    .library(name: "GinsRulesCore", targets: ["GinsRulesCore"]),
    .executable(name: "gins-rules-compiler", targets: ["GinsRulesCompiler"]),
    .executable(name: "gins-rules-syncer", targets: ["GinsRulesSyncer"]),
  ],
  dependencies: [
    .package(url: "https://github.com/apple/swift-argument-parser.git", from: "1.7.1"),
    .package(url: "https://github.com/jpsim/Yams.git", from: "6.2.1"),
    .package(url: "https://github.com/apple/swift-protobuf.git", from: "1.37.0"),
  ],
  targets: [
    .target(
      name: "GinsRulesCore",
      dependencies: [
        .product(name: "Yams", package: "Yams"),
        .product(name: "SwiftProtobuf", package: "swift-protobuf"),
      ]
    ),
    .executableTarget(
      name: "GinsRulesCompiler",
      dependencies: [
        "GinsRulesCore",
        .product(name: "ArgumentParser", package: "swift-argument-parser"),
      ]
    ),
    .executableTarget(
      name: "GinsRulesSyncer",
      dependencies: [
        "GinsRulesCore",
        .product(name: "ArgumentParser", package: "swift-argument-parser"),
      ]
    ),
  ]
)
