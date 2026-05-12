import ArgumentParser
import Foundation
import GinsRulesCore

struct BuildSummary: Encodable {
  var services: Int = 12
  var rules: Int = 0
  var ipRules: Int = 0
  var srs: Int = 0
  var mrs: Int = 0
  var timestamp: String = {
    let formatter = ISO8601DateFormatter()
    formatter.formatOptions = [.withInternetDateTime, .withDashSeparatorInDate, .withColonSeparatorInTime]
    return formatter.string(from: Date())
  }()
}

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
    
    var summary = BuildSummary()
    // Using global collectors for an elegant counting approach
    var allRules = Rules()
    var allRuleNames = Set<String>()
    var srsCount = 0
    var mrsCount = 0

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
        
        // Collect stats gracefully
        allRules.merge(with: rules)
        allRuleNames.insert(name)
        
        if isAIRuleName(name) {
          aiAggregateRules.merge(with: rules)
        }

        for format in RuleCompiler.Format.allCases {
          let dirName: String
          switch format {
          case .singbox, .srs: 
            dirName = "singbox"
            if format == .srs { srsCount += 1 }
          case .mihomo, .mrs: 
            dirName = "mihomo"
            if format == .mrs { mrsCount += 1 }
          case .stash: 
            dirName = "stash"
            mrsCount += 1 // Stash also generates MRS and YAML
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
          case .singbox, .srs: 
            dirName = "singbox"
            if format == .srs { srsCount += 1 }
          case .mihomo, .mrs: 
            dirName = "mihomo"
            if format == .mrs { mrsCount += 1 }
          case .stash: 
            dirName = "stash"
            mrsCount += 1
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
        case .singbox, .srs: 
          dirName = "singbox"
          if format == .srs { srsCount += 1 }
        case .mihomo, .mrs: 
          dirName = "mihomo"
          if format == .mrs { mrsCount += 1 }
        case .stash: 
          dirName = "stash"
          mrsCount += 1
        default: dirName = format.rawValue
        }

        let targetURL = outDir.appending(path: "\(dirName)/ai")
        try RuleCompiler.compile(
          format, name: "ai", rules: aiAggregateRules, outURL: targetURL, isIP: false, binDir: binDir)
      }
    }
    
    summary.services = allRuleNames.count
    summary.rules = allRules.count
    summary.ipRules = allRules.ipCidr.count + allRules.ipAsn.count
    summary.srs = srsCount
    summary.mrs = mrsCount
    
    let summaryURL = outDir.appending(path: "build-summary.json")
    try JSONEncoder().encode(summary).write(to: summaryURL)

    print("✨ [Compiler] All formats generated successfully.")
    print("📊 [Summary] Services: \(summary.services), Rules: \(summary.rules), IP Rules: \(summary.ipRules), SRS: \(summary.srs), MRS: \(summary.mrs)")
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
