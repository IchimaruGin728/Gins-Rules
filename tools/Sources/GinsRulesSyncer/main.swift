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

    let results = try await withThrowingTaskGroup(of: (String, String, [String]).self) { group in
      for src in sources where src.enabled {
        group.addTask {
          print("    Fetching \(src.name)...")
          guard let url = URL(string: src.url) else { return (src.category, src.target, []) }
          let (data, _) = try await URLSession.shared.data(from: url)
          guard let content = String(data: data, encoding: .utf8) else {
            return (src.category, src.target, [])
          }
          return (src.category, src.target, processRules(content: content))
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
    let configPath = root.appending(path: "source/icons.json")
    let outPath = root.appending(path: "compiled/Gins-Icons.json")
    let dashboardPath = root.appending(path: "dashboard/public/icons-catalog.json")

    let configData = try Data(contentsOf: configPath)
    let sources = try JSONDecoder().decode([IconSource].self, from: configData)

    let allIcons = try await withThrowingTaskGroup(of: [NormalizedIcon].self) { group in
      for src in sources where src.enabled {
        group.addTask {
          print("    Fetching icons from \(src.name)...")
          guard let url = URL(string: src.url) else { return [] }
          do {
            let (data, _) = try await URLSession.shared.data(from: url)

            // Try to decode as array first
            if let raw = try? JSONDecoder().decode([RawIcon].self, from: data) {
              return raw.map {
                NormalizedIcon(
                  name: $0.name ?? $0.tag ?? "icon", url: $0.url ?? "", source: src.name,
                  theme: src.theme)
              }
            }

            // Try to decode as dictionary with common keys
            if let dict = try? JSONSerialization.jsonObject(with: data) as? [String: Any] {
              let keys = ["icons", "items", "iconList", "list", "tubiao"]
              for key in keys {
                if let list = dict[key] as? [[String: Any]] {
                  return list.compactMap { item in
                    let name = (item["name"] as? String) ?? (item["tag"] as? String) ?? "icon"
                    let iconUrl = (item["url"] as? String) ?? ""
                    return NormalizedIcon(
                      name: name, url: iconUrl, source: src.name, theme: src.theme)
                  }
                }
              }
            }
          } catch {
            print("    [ERROR] Failed to fetch \(src.name): \(error)")
          }
          return []
        }
      }
      var collected: [NormalizedIcon] = []
      for try await icons in group { collected.append(contentsOf: icons) }
      return collected
    }

    let encoder = JSONEncoder()
    encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
    let finalData = try encoder.encode(
      allIcons.sorted(by: { $0.source == $1.source ? $0.name < $1.name : $0.source < $1.source }))

    try? FileManager.default.createDirectory(
      at: outPath.deletingLastPathComponent(), withIntermediateDirectories: true)
    try finalData.write(to: outPath)

    try? FileManager.default.createDirectory(
      at: dashboardPath.deletingLastPathComponent(), withIntermediateDirectories: true)
    try finalData.write(to: dashboardPath)
    print("  [SUCCESS] Written \(allIcons.count) icons.")
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
  var name, url, category, target: String
  var enabled: Bool
}

struct IconSource: Codable, Sendable {
  var name, url, theme: String
  var enabled: Bool
}

struct NormalizedIcon: Codable, Sendable {
  var name, url, source, theme: String
}

struct RawIcon: Codable, Sendable {
  var name, url, tag: String?
}
