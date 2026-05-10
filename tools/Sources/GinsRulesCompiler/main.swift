import ArgumentParser
import Foundation
import GinsRulesCore

@main
struct GinsRulesCompiler: ParsableCommand {
  @Option(name: .shortAndLong, help: "Root directory of the project")
  var root: String = "."

  @Option(name: .shortAndLong, help: "Output directory for compiled rules")
  var output: String = "compiled"

  mutating func run() throws {
    let rootURL = URL(fileURLWithPath: root).standardized
    let outputURL = rootURL.appendingPathComponent(output)

    print("============================================================")
    print("  Gins-Rules Compiler (Swift Version)")
    print("============================================================")

    let outputCategories = ["proxy", "direct", "reject", "ip", "asn", "ai"]
    let formatDirs = [
      "singbox", "mihomo", "text", "quantumultx", "egern", "loon",
      "stash", "shadowrocket", "surfboard", "exclave", "surge", "anywhere",
    ]

    let rulesetDir = outputURL.appendingPathComponent("ruleset")

    // Setup directories
    for fmt in formatDirs {
      let fmtURL = rulesetDir.appendingPathComponent(fmt)
      if FileManager.default.fileExists(atPath: fmtURL.path) {
        try FileManager.default.removeItem(at: fmtURL)
      }
      try FileManager.default.createDirectory(at: fmtURL, withIntermediateDirectories: true)
      for cat in outputCategories {
        try FileManager.default.createDirectory(
          at: fmtURL.appendingPathComponent(cat), withIntermediateDirectories: true)
      }
    }

    let categories = ["proxy", "direct", "reject", "ip", "asn"]
    var categoryMergedRules: [String: Rules] = [:]
    for cat in outputCategories {
      categoryMergedRules[cat] = Rules()
    }

    for category in categories {
      print("  [Diagnostic] Processing category: \(category)")
      let localDir = rootURL.appendingPathComponent("source").appendingPathComponent(category)
      let upstreamDir = rootURL.appendingPathComponent("source").appendingPathComponent("upstream")
        .appendingPathComponent(category)

      var ruleNames: Set<String> = []

      let searchDirs =
        (category == "asn")
        ? [
          rootURL.appendingPathComponent("source").appendingPathComponent("ip"),
          rootURL.appendingPathComponent("source").appendingPathComponent("upstream")
            .appendingPathComponent("ip"),
        ] : [localDir, upstreamDir]

      for dir in searchDirs {
        if let entries = try? FileManager.default.contentsOfDirectory(
          at: dir, includingPropertiesForKeys: nil)
        {
          for entry in entries where entry.pathExtension == "txt" {
            let name = entry.deletingPathExtension().lastPathComponent
            if category == "ip" && name.hasPrefix("asn-") { continue }
            if category == "asn" && !name.hasPrefix("asn-") { continue }
            ruleNames.insert(name)
          }
        }
      }

      for name in ruleNames.sorted() {
        var rules = Rules()
        let localPath = (category == "asn" ? searchDirs[0] : localDir).appendingPathComponent(
          "\(name).txt")
        let upstreamPath = (category == "asn" ? searchDirs[1] : upstreamDir).appendingPathComponent(
          "\(name).txt")

        if FileManager.default.fileExists(atPath: localPath.path) {
          let localRules = try Parser.parseSource(at: localPath)
          rules.merge(with: localRules)
        }
        if FileManager.default.fileExists(atPath: upstreamPath.path) {
          let upstreamRules = try Parser.parseSource(at: upstreamPath)
          rules.merge(with: upstreamRules)
        }

        if category != "proxy" {
          rules = rules.sanitized()
        }
        // rules.unique() // Set is already unique
        if !rules.isEmpty {
          try compileToAllFormats(
            name: name, category: category, rules: rules, rulesetDir: rulesetDir)

          categoryMergedRules[category]?.merge(with: rules)

          if isAIRuleName(name) {
            categoryMergedRules["ai"]?.merge(with: rules)
            try compileToAllFormats(
              name: name, category: "ai", rules: rules, rulesetDir: rulesetDir)
          }
        }
      }
    }

    // Process aggregate bundles
    for category in outputCategories {
      guard let rules = categoryMergedRules[category], !rules.isEmpty else { continue }
      let finalRules = rules
      // finalRules.unique()
      try compileToAllFormats(
        name: category, category: category, rules: finalRules, rulesetDir: rulesetDir)
    }

    print("  [SUCCESS] Compilation complete.")
  }

  func compileToAllFormats(name: String, category: String, rules: Rules, rulesetDir: URL) throws {
    let isIP = (category == "ip" || category == "asn")

    // Sing-box
    let sbDir = rulesetDir.appendingPathComponent("singbox").appendingPathComponent(category)
    _ = try Compiler.compileSingBoxJSON(name: name, rules: rules, to: sbDir)

    // Mihomo
    let mode = Compiler.detectMihomoRuleMode(rules: rules, isIPCategory: isIP)
    let mhDir = rulesetDir.appendingPathComponent("mihomo").appendingPathComponent(category)
    try Compiler.compileMihomoYAML(name: name, rules: rules, to: mhDir, mode: mode)

    // Stash / Fallback
    let stashDir = rulesetDir.appendingPathComponent("stash").appendingPathComponent(category)
    try Compiler.compileMihomoYAML(
      name: name, rules: rules, to: stashDir, mode: MihomoRuleMode(behavior: "classical"))

    // Text / QuanX
    try Compiler.compileTextList(
      name: name, rules: rules,
      to: rulesetDir.appendingPathComponent("text").appendingPathComponent(category), isIP: isIP)
    try Compiler.compileQuanXList(
      name: name, rules: rules,
      to: rulesetDir.appendingPathComponent("quantumultx").appendingPathComponent(category),
      isIP: isIP, category: category)
  }

  func isAIRuleName(_ name: String) -> Bool {
    let aiKeywords = [
      "openai", "claude", "gemini", "copilot", "ai-other",
      "apple-intelligence", "mistral", "deepseek", "character",
      "perplexity", "groq", "anthropic",
    ]
    return aiKeywords.contains { name.contains($0) }
  }
}
