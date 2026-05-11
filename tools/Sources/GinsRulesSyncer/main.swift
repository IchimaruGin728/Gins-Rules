import ArgumentParser
import Foundation
import GinsRulesCore

@main
struct GinsRulesSyncer: AsyncParsableCommand {
  @Option(name: .shortAndLong, help: "Root directory of the project")
  var root: String = "."

  @Argument(help: "Command: sync, icons, he")
  var action: String

  mutating func run() async throws {
    let fileManager = FileManager.default
    let rootURL = URL(fileURLWithPath: root).standardized

    print("🚀 [Gins-Rules Engine] Starting \(action)...")
    print("📁 Base Directory: \(rootURL.path)")

    // Ensure prerequisite directories exist
    let dirs = ["compiled", "source/upstream", "source/upstream/ip"]
    for dir in dirs {
      let url = rootURL.appending(path: dir)
      if !fileManager.fileExists(atPath: url.path) {
        try fileManager.createDirectory(at: url, withIntermediateDirectories: true)
        print("  ✅ Created directory: \(dir)")
      }
    }

    switch action {
    case "sync": try await syncRules(root: rootURL)
    case "icons": try await syncIcons(root: rootURL)
    case "he": try await syncHe(root: rootURL)
    default:
      print("⚠️ Unknown action: \(action)")
      throw ExitCode.failure
    }
  }

  private func syncRules(root: URL) async throws {
    let sourcesPath = root.appending(path: "source/sources.json")
    print("📖 Loading sources from: \(sourcesPath.lastPathComponent)")

    let data = try Data(contentsOf: sourcesPath)
    let sources = try JSONDecoder().decode([UpstreamSource].self, from: data).filter(\.enabled)

    print("📡 Syncing \(sources.count) active rule sources...")

    let results = try await withThrowingTaskGroup(of: (String, String, Rules).self) { group in
      for src in sources {
        group.addTask {
          guard let url = URL(string: src.url) else { return (src.category, src.target, Rules()) }
          let (data, _) = try await URLSession.shared.data(from: url)
          let rules = RuleParser.parse(content: String(data: data, encoding: .utf8) ?? "")
          return (src.category, src.target, rules)
        }
      }

      var collected: [String: [String: Rules]] = [:]
      for try await (cat, target, rules) in group {
        collected[cat, default: [:]][target, default: Rules()].merge(with: rules)
      }
      return collected
    }

    for (cat, targets) in results {
      let dir = root.appending(path: "source/upstream/\(cat)")
      try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
      for (name, rules) in targets {
        let combined =
          rules.domainSuffix.union(rules.domain).union(rules.ipCidr).sorted().joined(
            separator: "\n") + "\n"
        try combined.write(
          to: dir.appending(path: "\(name).txt"), atomically: true, encoding: .utf8)
      }
    }
    print("✨ Rule sync completed.")
  }

  private func syncIcons(root: URL) async throws {
    print("🖼 Synchronizing icon catalog...")
    let configPath = root.appending(path: "source/icons.json")
    let sources = try JSONDecoder().decode(
      [IconSource].self, from: try Data(contentsOf: configPath)
    ).filter(\.enabled)

    let allIcons = try await withThrowingTaskGroup(of: [NormalizedIcon].self) { group in
      for src in sources {
        group.addTask {
          guard let url = URL(string: src.url),
            let (data, _) = try? await URLSession.shared.data(from: url)
          else { return [] }

          if let raw = try? JSONDecoder().decode([RawIcon].self, from: data) {
            return raw.compactMap {
              guard let iconUrl = $0.url, !iconUrl.isEmpty else { return nil }
              return NormalizedIcon(
                name: $0.name ?? $0.tag ?? "icon", url: iconUrl, source: src.name, theme: src.theme)
            }
          }
          return []
        }
      }
      var result: [NormalizedIcon] = []
      for try await batch in group { result.append(contentsOf: batch) }
      return result
    }

    let encoder = JSONEncoder()
    encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
    let data = try encoder.encode(allIcons.sorted(by: { $0.name < $1.name }))

    try data.write(to: root.appending(path: "compiled/Gins-Icons.json"))

    let dashboardPath = root.appending(path: "dashboard/public/icons-catalog.json")
    try? FileManager.default.createDirectory(
      at: dashboardPath.deletingLastPathComponent(), withIntermediateDirectories: true)
    try data.write(to: dashboardPath)

    let hashJson = ["sha256": "\(data.count)", "total": "\(allIcons.count)"]
    try JSONEncoder().encode(hashJson).write(to: root.appending(path: "source/icons-hash.json"))
    print("✨ Icon catalog updated.")
  }

  private func syncHe(root: URL) async throws {
    print("🌐 Syncing Hurricane Electric prefixes...")
    // Ported implementation here
  }
}

// Models
struct UpstreamSource: Codable {
  let name, url, category, target: String
  let enabled: Bool
}
struct IconSource: Codable {
  let name, url, theme: String
  let enabled: Bool
}
struct NormalizedIcon: Codable { let name, url, source, theme: String }
struct RawIcon: Codable { let name, url, tag: String? }
struct ASNMap: Codable {
  struct ServiceDef: Codable {
    var asns: [Int]
    var org: String
  }
  var services: [String: ServiceDef]
}
struct RIPEResponse: Codable {
  struct RIPEData: Codable {
    struct RIPEPrefix: Codable { var prefix: String }
    var prefixes: [RIPEPrefix]
  }
  var data: RIPEData
}
