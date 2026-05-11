import ArgumentParser
import Foundation
import GinsRulesCore

@main
struct GinsRulesCompiler: ParsableCommand {
  @Option(name: .shortAndLong, help: "Root directory")
  var root: String = "."

  @Option(name: .shortAndLong, help: "Output directory")
  var output: String = "compiled"

  mutating func run() throws {
    let rootURL = URL(filePath: root).standardized
    let outDir = rootURL.appending(path: output).appending(path: "ruleset")
    let binDir = rootURL.appending(path: "bin")
    print("🚀 [Compiler] Building rule matrix (SRS/MRS Active)...")

    let categories = ["proxy", "direct", "reject", "ip", "asn"]
    var aiAggregateRules = Rules()

    for cat in categories {
      print("  📁 Category: \(cat)")
      let localDir = rootURL.appending(path: "source/\(cat)")
      let upstreamDir = rootURL.appending(path: "source/upstream/\(cat)")

      var ruleNames: Set<String> = []
      [localDir, upstreamDir].forEach { dir in
        if let files = try? FileManager.default.contentsOfDirectory(
          at: dir, includingPropertiesForKeys: nil)
        {
          files.filter { $0.pathExtension == "txt" }.forEach {
            ruleNames.insert($0.deletingPathExtension().lastPathComponent)
          }
        }
      }

      var categoryAggregateRules = Rules()

      for name in ruleNames.sorted() {
        var rules = Rules()
        [localDir, upstreamDir].forEach { dir in
          if let r = try? RuleParser.parse(url: dir.appending(path: "\(name).txt")) {
            rules.merge(with: r)
          }
        }

        if cat != "proxy" { rules = rules.sanitized() }
        if rules.isEmpty { continue }
        
        categoryAggregateRules.merge(with: rules)
        
        if isAIRuleName(name) {
          aiAggregateRules.merge(with: rules)
        }

        for format in RuleCompiler.Format.allCases {
          let dirName: String
          switch format {
          case .singbox, .srs: dirName = "singbox"
          case .mihomo, .mrs: dirName = "mihomo"
          case .stash: dirName = "stash"
          default: dirName = format.rawValue
          }

          let targetURL = outDir.appending(path: "\(dirName)/\(cat)")
          try RuleCompiler.compile(
            format, name: name, rules: rules, outURL: targetURL, isIP: cat == "ip", binDir: binDir)
        }
      }

      // Compile aggregate bundle for the category
      if !categoryAggregateRules.isEmpty {
        print("    📦 Compiling aggregate bundle: \(cat)")
        for format in RuleCompiler.Format.allCases {
          let dirName: String
          switch format {
          case .singbox, .srs: dirName = "singbox"
          case .mihomo, .mrs: dirName = "mihomo"
          case .stash: dirName = "stash"
          default: dirName = format.rawValue
          }

          let targetURL = outDir.appending(path: "\(dirName)/\(cat)")
          try RuleCompiler.compile(
            format, name: cat, rules: categoryAggregateRules, outURL: targetURL, isIP: cat == "ip", binDir: binDir)
        }
      }
    }

    // Compile AI aggregate bundle
    if !aiAggregateRules.isEmpty {
      print("    🤖 Compiling AI aggregate bundle: ai")
      for format in RuleCompiler.Format.allCases {
        let dirName: String
        switch format {
        case .singbox, .srs: dirName = "singbox"
        case .mihomo, .mrs: dirName = "mihomo"
        case .stash: dirName = "stash"
        default: dirName = format.rawValue
        }

        let targetURL = outDir.appending(path: "\(dirName)/ai")
        try RuleCompiler.compile(
          format, name: "ai", rules: aiAggregateRules, outURL: targetURL, isIP: false, binDir: binDir)
      }
    }

    print("✨ [Compiler] All formats generated successfully.")
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
