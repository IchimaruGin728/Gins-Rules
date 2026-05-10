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

    let results = try await withThrowingTaskGroup(of: (String, String, [String]).self) { group in
      for src in sources where src.enabled {
        group.addTask {
          print("    Fetching \(src.name)...")
          guard let url = URL(string: src.url) else {
            return (src.category, src.target, [])
          }
          let (data, _) = try await session.data(from: url)
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

  func runIcons(root: URL) async throws {
    print("  [Syncer] Icons sync started (Swift Version)...")
    // Basic implementation to satisfy the workflow for now
    let configPath = root.appending(path: "source/icons.json")
    if FileManager.default.fileExists(atPath: configPath.path) {
      print("    Icons config found, processing...")
    }
  }

  func runHe(root: URL) async throws { print("  [Syncer] HE sync placeholder") }
  func runGeo(root: URL) async throws { print("  [Syncer] Geo sync placeholder") }

  func processRules(content: String) -> [String] {
    content.components(separatedBy: .newlines).compactMap { line in
      let trimmed = line.trimmingCharacters(in: .whitespaces)
      if trimmed.isEmpty || trimmed.hasPrefix("#") || trimmed.hasPrefix(";")
        || trimmed.hasPrefix("//")
      {
        return nil
      }

      // Handle common formats (classical, list, etc.)
      let clean = trimmed.trimmingCharacters(in: CharacterSet(charactersIn: "-'\" "))
      let parts = clean.split(separator: ",").map { $0.trimmingCharacters(in: .whitespaces) }

      if parts.count >= 2 {
        let type = parts[0].uppercased()
        let value = parts[1].split(separator: "//")[0].trimmingCharacters(
          in: CharacterSet(charactersIn: "'\" "))

        switch type {
        case "DOMAIN-SUFFIX", "HOST-SUFFIX": return value
        case "DOMAIN", "HOST": return "full:\(value)"
        case "DOMAIN-KEYWORD", "HOST-KEYWORD": return "keyword:\(value)"
        case "DOMAIN-REGEX", "URL-REGEX": return "regexp:\(value)"
        case "PROCESS-NAME": return "process:\(value)"
        case "USER-AGENT": return "user-agent:\(value)"
        case "IP-CIDR", "IP-CIDR6": return value
        case "IP-ASN": return "asn:\(value)"
        default: return nil
        }
      } else {
        let rule = clean.trimmingCharacters(in: CharacterSet(charactersIn: "+."))
        if rule.hasPrefix("domain:") { return String(rule.dropFirst(7)) }
        return rule
      }
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
