import ArgumentParser
import Foundation
import GinsRulesCore

@main
struct GinsRulesSyncer: AsyncParsableCommand {
  @Option(name: .shortAndLong, help: "Root directory")
  var root: String = "."

  @Argument(help: "Command: sync, icons, he, geo")
  var command: String

  mutating func run() async throws {
    let rootURL = URL(fileURLWithPath: root).standardized
    print("🚀 [Gins-Rules] Engine v6.3.1 Active")

    try FileManager.default.createDirectory(
      at: rootURL.appending(path: "compiled"), withIntermediateDirectories: true)
    try FileManager.default.createDirectory(
      at: rootURL.appending(path: "source/upstream/ip"), withIntermediateDirectories: true)

    switch command {
    case "sync": try await performSync(root: rootURL)
    case "icons": try await performIcons(root: rootURL)
    case "he": try await performHe(root: rootURL)
    case "geo": try await performGeo(root: rootURL)
    default: print("⚠️ Unrecognized task")
    }
  }

  private func performSync(root: URL) async throws {
    let configPath = root.appending(path: "source/sources.json")
    let upstreamDir = root.appending(path: "source/upstream")
    let sources = try JSONDecoder().decode(
      [UpstreamSource].self, from: Data(contentsOf: configPath)
    ).filter { $0.enabled }

    print("📡 Syncing \(sources.count) rule sources...")
    try await withThrowingTaskGroup(of: (String, String, Rules).self) { group in
      for src in sources {
        group.addTask {
          let (data, _) = try await URLSession.shared.data(from: URL(string: src.url)!)
          let rules = RuleParser.parse(content: String(data: data, encoding: .utf8) ?? "")
          return (src.category, src.target, rules)
        }
      }
      var results: [String: [String: Rules]] = [:]
      for try await (cat, target, rules) in group {
        results[cat, default: [:]][target, default: Rules()].merge(with: rules)
      }
      for (cat, targets) in results {
        let catDir = upstreamDir.appending(path: cat)
        try FileManager.default.createDirectory(at: catDir, withIntermediateDirectories: true)
        for (name, rules) in targets {
          let content =
            Array(rules.domainSuffix.union(rules.domain).union(rules.ipCidr)).sorted().joined(
              separator: "\n") + "\n"
          try content.write(
            to: catDir.appending(path: "\(name).txt"), atomically: true, encoding: .utf8)
        }
      }
    }
    print("✅ Rule sync successful")
  }

  private func performIcons(root: URL) async throws {
    print("🎨 Aggregating icons...")
    let configPath = root.appending(path: "source/icons.json")
    let sources = try JSONDecoder().decode([IconSource].self, from: Data(contentsOf: configPath))
      .filter { $0.enabled }

    let allIcons = try await withThrowingTaskGroup(of: [NormalizedIcon].self) { group in
      for src in sources {
        group.addTask {
          guard let (data, _) = try? await URLSession.shared.data(from: URL(string: src.url)!)
          else { return [] }
          if let raw = try? JSONDecoder().decode([RawIcon].self, from: data) {
            return raw.map {
              NormalizedIcon(
                name: $0.name ?? $0.tag ?? "icon", url: $0.url ?? "", source: src.name,
                theme: src.theme)
            }
          }
          return []
        }
      }
      var collected: [NormalizedIcon] = []
      for try await batch in group { collected.append(contentsOf: batch) }
      return collected
    }

    let outPath = root.appending(path: "compiled/Gins-Icons.json")
    let dashboardPath = root.appending(path: "dashboard/public/icons-catalog.json")
    let encoder = JSONEncoder()
    encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
    let data = try encoder.encode(allIcons.sorted(by: { $0.name < $1.name }))
    try data.write(to: outPath)
    try? FileManager.default.createDirectory(
      at: dashboardPath.deletingLastPathComponent(), withIntermediateDirectories: true)
    try data.write(to: dashboardPath)
    print("✅ Icon hub updated")
  }

  private func performHe(root: URL) async throws {
    print("🌐 Syncing HE Announced Prefixes...")
    let asnMapPath = root.appending(path: "source/asn-map.json")
    let asnMap = try JSONDecoder().decode(ASNMap.self, from: Data(contentsOf: asnMapPath))
    let outDir = root.appending(path: "source/upstream/ip")

    for (svc, def) in asnMap.services {
      var prefixes: Set<String> = []
      for asn in def.asns {
        let url = URL(
          string: "https://stat.ripe.net/data/announced-prefixes/data.json?resource=AS\(asn)")!
        if let (data, _) = try? await URLSession.shared.data(from: url),
          let resp = try? JSONDecoder().decode(RIPEResponse.self, from: data)
        {
          prefixes.formUnion(resp.data.prefixes.map { $0.prefix })
        }
      }
      if !prefixes.isEmpty {
        try (prefixes.sorted().joined(separator: "\n") + "\n").write(
          to: outDir.appending(path: "asn-\(svc).txt"), atomically: true, encoding: .utf8)
        print("  ✅ AS\(svc) synced")
      }
    }
  }

  private func performGeo(root: URL) async throws {
    print("🌍 Geo processing active")
    // Implementation for MMDB processing could be added here if swift-mmdb is stable
  }
}

// Support structures
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
