import ArgumentParser
import Foundation
import GinsRulesCore

@main
struct GinsRulesSyncer: AsyncParsableCommand {
  @Option(name: .shortAndLong, help: "Working directory")
  var root: String = "."

  @Argument(help: "Action: sync, icons")
  var action: String

  mutating func run() async throws {
    let rootURL = URL(filePath: root).standardized
    print("🚀 [Syncer] Running \(action)...")

    switch action {
    case "sync": try await syncRules(root: rootURL)
    case "icons": try await syncIcons(root: rootURL)
    default: throw CleanError("Unsupported action: \(action)")
    }
  }

  private func syncRules(root: URL) async throws {
    let sourcesURL = root.appending(path: "source/sources.json")
    let sources = try JSONDecoder().decode(
      [UpstreamSource].self, from: Data(contentsOf: sourcesURL)
    ).filter(\.enabled)

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
    }
    print("✨ [Syncer] Rules updated successfully.")
  }

  private func syncIcons(root: URL) async throws {
    let configURL = root.appending(path: "source/icons.json")
    let sources = try JSONDecoder().decode([IconSource].self, from: Data(contentsOf: configURL))
      .filter(\.enabled)

    let allIcons = try await withThrowingTaskGroup(of: [NormalizedIcon].self) { group in
      for src in sources {
        group.addTask {
          guard let (data, _) = try? await URLSession.shared.data(from: URL(string: src.url)!)
          else { return [] }
          if let raw = try? JSONDecoder().decode([RawIcon].self, from: data) {
            return raw.compactMap {
              guard let url = $0.url, !url.isEmpty else { return nil }
              return NormalizedIcon(
                name: $0.name ?? $0.tag ?? "icon", url: url, source: src.name, theme: src.theme)
            }
          }
          return []
        }
      }
      var collected: [NormalizedIcon] = []
      for try await batch in group { collected.append(contentsOf: batch) }
      return collected
    }

    let encoder = JSONEncoder()
    encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
    let data = try encoder.encode(allIcons.sorted(by: { $0.name < $1.name }))
    try data.write(to: root.appending(path: "compiled/Gins-Icons.json"))
    print("✨ [Syncer] Icon hub synchronized.")
  }
}

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
struct CleanError: LocalizedError {
  let message: String
  init(_ message: String) { self.message = message }
  var errorDescription: String? { message }
}
