import ArgumentParser
import Foundation
import GinsRulesCore

@main
struct GinsRulesSyncer: AsyncParsableCommand {
  @Option(name: .shortAndLong, help: "Root directory of the project")
  var root: String = "."

  @Argument(help: "Command to run (sync, icons, he, geo)")
  var command: String

  mutating func run() async throws {
    let rootURL = URL(fileURLWithPath: root).standardized

    switch command {
    case "sync":
      try await runSync(root: rootURL)
    case "icons":
      try await runIcons(root: rootURL)
    case "he":
      try await runHe(root: rootURL)
    case "geo":
      try await runGeo(root: rootURL)
    default:
      print("Unknown command: \(command)")
    }
  }

  func runSync(root: URL) async throws {
    let configPath = root.appending(path: "source/sources.json")
    let upstreamDir = root.appending(path: "source/upstream")

    let configData = try Data(contentsOf: configPath)
    let sources = try JSONDecoder().decode([UpstreamSource].self, from: configData)

    print("  [Syncer] Starting parallel sync of \(sources.filter { $0.enabled }.count) sources...")

    let session = URLSession.shared

    // Use a TaskGroup for parallel fetching
    let results = try await withThrowingTaskGroup(of: (String, String, [String]).self) { group in
      for src in sources where src.enabled {
        group.addTask {
          print("    Fetching \(src.name)...")
          let (data, _) = try await session.data(from: URL(string: src.url)!)
          guard let content = String(data: data, encoding: .utf8) else {
            return (src.category, src.target, [])
          }
          let rules = processRules(content: content)
          return (src.category, src.target, rules)
        }
      }

      var collected: [String: [String: [String]]] = [:]
      for try await (category, target, rules) in group {
        collected[category, default: [:]][target, default: []].append(contentsOf: rules)
      }
      return collected
    }

    for (cat, targets) in results {
      let outDir = upstreamDir.appending(path: cat)
      try FileManager.default.createDirectory(at: outDir, withIntermediateDirectories: true)
      for (name, rules) in targets {
        let content = Set(rules).sorted().joined(separator: "\n") + "\n"
        try content.write(
          to: outDir.appending(path: "\(name).txt"), atomically: true, encoding: .utf8)
        print("  [SUCCESS] Written \(rules.count) rules to \(name).txt")
      }
    }
  }

  func runIcons(root: URL) async throws { print("Icons command not fully ported yet") }
  func runHe(root: URL) async throws { print("He command not fully ported yet") }
  func runGeo(root: URL) async throws { print("Geo command not fully ported yet") }

  func processRules(content: String) -> [String] {
    content.components(separatedBy: .newlines).compactMap { line in
      let trimmed = line.trimmingCharacters(in: .whitespaces)
      if trimmed.isEmpty || trimmed.hasPrefix("#") || trimmed.hasPrefix(";")
        || trimmed.hasPrefix("//")
      {
        return nil
      }
      return trimmed
    }
  }
}

struct UpstreamSource: Codable, Sendable {
  var name: String
  var url: String
  var category: String
  var target: String
  var enabled: Bool
}
