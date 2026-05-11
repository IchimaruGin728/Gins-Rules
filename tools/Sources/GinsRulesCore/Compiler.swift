import Foundation
import Yams

public enum RuleCompiler {
  public enum Format: String, CaseIterable {
    case singbox, srs, mihomo, mrs, stash, surge, loon, shadowrocket, quantumultx, surfboard,
      exclave, anywhere, egern, text
  }

  /// Entry point for compiling a ruleset into a specific format.
  public static func compile(
    _ format: Format, name: String, rules: Rules, outURL: URL, isIP: Bool, binDir: URL? = nil
  ) throws {
    try FileManager.default.createDirectory(at: outURL, withIntermediateDirectories: true)

    switch format {
    case .singbox: try toSingBoxJSON(name: name, rules: rules, outURL: outURL)
    case .srs: try toSingBoxBinary(name: name, rules: rules, outURL: outURL, binDir: binDir)
    case .mihomo: try toMihomoYAML(name: name, rules: rules, outURL: outURL, isIP: isIP)
    case .mrs, .stash:
      // Stash and MRS both need binary format.
      // Stash also gets YAML via the .stash case mapping to same folder in main.swift?
      // No, let's just make stash call both if needed, or rely on the loop.
      try toMihomoBinary(name: name, rules: rules, outURL: outURL, isIP: isIP, binDir: binDir)
      if format == .stash {
        try toMihomoYAML(name: name, rules: rules, outURL: outURL, isIP: isIP)
      }
    case .surge: try toSurgeRocket(name: name, rules: rules, outURL: outURL, ext: "domainset")
    case .shadowrocket: try toSurgeRocket(name: name, rules: rules, outURL: outURL, ext: "txt")
    case .loon: try toLoon(name: name, rules: rules, outURL: outURL)
    case .quantumultx: try toQuantumultX(name: name, rules: rules, outURL: outURL)
    case .surfboard: try toSurfboard(name: name, rules: rules, outURL: outURL)
    case .exclave: try toExclave(name: name, rules: rules, outURL: outURL)
    case .anywhere: try toAnywhere(name: name, rules: rules, outURL: outURL)
    case .egern: try toEgern(name: name, rules: rules, outURL: outURL)
    case .text: try toPlainList(name: name, rules: rules, outURL: outURL, isIP: isIP)
    }
  }

  // MARK: - Format Implementations (Aligned with 2026 Docs)

  private static func toSingBoxJSON(name: String, rules: Rules, outURL: URL) throws {
    let rs = SingBoxRuleSet(
      version: 1,
      rules: [
        SingBoxRule(
          domainSuffix: rules.domainSuffix.isEmpty ? nil : rules.domainSuffix.sorted(),
          domain: rules.domain.isEmpty ? nil : rules.domain.sorted(),
          domainKeyword: rules.domainKeyword.isEmpty ? nil : rules.domainKeyword.sorted(),
          domainRegex: rules.domainRegex.isEmpty ? nil : rules.domainRegex.sorted(),
          ipCidr: rules.ipCidr.isEmpty ? nil : rules.ipCidr.sorted()
        )
      ])
    try JSONEncoder().encode(rs).write(to: outURL.appending(path: "\(name).json"))
  }

  private static func toSingBoxBinary(name: String, rules: Rules, outURL: URL, binDir: URL?) throws
  {
    let tempJsonURL = outURL.appending(path: "\(name).compile.tmp.json")

    // Generate a temporary JSON for sing-box to compile
    let rs = SingBoxRuleSet(
      version: 1,
      rules: [
        SingBoxRule(
          domainSuffix: rules.domainSuffix.isEmpty ? nil : rules.domainSuffix.sorted(),
          domain: rules.domain.isEmpty ? nil : rules.domain.sorted(),
          domainKeyword: rules.domainKeyword.isEmpty ? nil : rules.domainKeyword.sorted(),
          domainRegex: rules.domainRegex.isEmpty ? nil : rules.domainRegex.sorted(),
          ipCidr: rules.ipCidr.isEmpty ? nil : rules.ipCidr.sorted()
        )
      ])
    try JSONEncoder().encode(rs).write(to: tempJsonURL)

    if let bin = binDir?.appending(path: "sing-box") {
      if FileManager.default.fileExists(atPath: bin.path) {
        try shell(
          bin.path,
          [
            "rule-set", "compile", tempJsonURL.path, "-o",
            outURL.appending(path: "\(name).srs").path,
          ])
      } else {
        print("  ⚠️ Sing-box not found at: \(bin.path)")
      }
    }
    try? FileManager.default.removeItem(at: tempJsonURL)
  }

  private static func toMihomoYAML(name: String, rules: Rules, outURL: URL, isIP: Bool) throws {
    let behavior = detectMihomoBehavior(rules: rules, isIP: isIP)
    var payload: [String] = []

    if behavior == "classical" {
      payload += rules.domainSuffix.sorted().map { "DOMAIN-SUFFIX,\($0)" }
      payload += rules.domain.sorted().map { "DOMAIN,\($0)" }
      payload += rules.domainKeyword.sorted().map { "DOMAIN-KEYWORD,\($0)" }
      payload += rules.ipCidr.sorted().map { "\($0.contains(":") ? "IP-CIDR6" : "IP-CIDR"),\($0)" }
      payload += rules.ipAsn.sorted().map { "IP-ASN,\($0)" }
    } else if behavior == "ipcidr" {
      payload = rules.ipCidr.sorted()
    } else {
      payload = rules.domainSuffix.sorted().map { ".\($0)" } + rules.domain.sorted()
    }

    let yaml = try Yams.dump(object: ["payload": payload])
    try ("# Gins-Rules: \(name)\n" + yaml).write(
      to: outURL.appending(path: "\(name).yaml"), atomically: true, encoding: .utf8)
  }

  private static func toMihomoBinary(
    name: String, rules: Rules, outURL: URL, isIP: Bool, binDir: URL?
  ) throws {
    let behavior = detectMihomoBehavior(rules: rules, isIP: isIP)
    let yamlURL = outURL.appending(path: "\(name).tmp.yaml")
    let mrsURL = outURL.appending(path: "\(name).mrs")

    // We avoid 'classical' for MRS to prevent hangs in this environment.
    // Instead we split or choose the best single-type behavior.
    let targetBehavior = (behavior == "classical") ? (isIP ? "ipcidr" : "domain") : behavior

    var payload: [String] = []
    if targetBehavior == "domain" {
      payload = rules.domainSuffix.sorted().map { ".\($0)" } + rules.domain.sorted()
    } else if targetBehavior == "ipcidr" {
      payload = rules.ipCidr.sorted()
    }

    if payload.isEmpty { return }

    let yaml = try Yams.dump(object: ["payload": payload])
    try ("# Gins-Rules: \(name)\n" + yaml).write(to: yamlURL, atomically: true, encoding: .utf8)

    if let bin = binDir?.appending(path: "mihomo"), FileManager.default.fileExists(atPath: bin.path)
    {
      try shell(bin.path, ["convert-ruleset", targetBehavior, "yaml", yamlURL.path, mrsURL.path])
    }
    try? FileManager.default.removeItem(at: yamlURL)
  }

  private static func toSurgeRocket(name: String, rules: Rules, outURL: URL, ext: String) throws {
    // Aligned with Surge 5.x / Shadowrocket latest docs
    // 1. Standard List
    var list: [String] = []
    list += rules.domainSuffix.sorted().map { "DOMAIN-SUFFIX,\($0)" }
    list += rules.domain.sorted().map { "DOMAIN,\($0)" }
    list += rules.domainKeyword.sorted().map { "DOMAIN-KEYWORD,\($0)" }
    list += rules.ipCidr.sorted().map {
      "\($0.contains(":") ? "IP-CIDR6" : "IP-CIDR"),\($0),no-resolve"
    }
    list += rules.ipAsn.sorted().map { "IP-ASN,\($0),no-resolve" }
    try (list.joined(separator: "\n") + "\n").write(
      to: outURL.appending(path: "\(name).list"), atomically: true, encoding: .utf8)

    // 2. High-performance Domainset (if domains exist)
    if !rules.domainSuffix.isEmpty || !rules.domain.isEmpty {
      let ds = rules.domainSuffix.sorted().map { ".\($0)" } + rules.domain.sorted()
      try (ds.joined(separator: "\n") + "\n").write(
        to: outURL.appending(path: "\(name).\(ext)"), atomically: true, encoding: .utf8)
    }
  }

  private static func toLoon(name: String, rules: Rules, outURL: URL) throws {
    var list: [String] = []
    list += rules.domainSuffix.sorted().map { "DOMAIN-SUFFIX,\($0)" }
    list += rules.domain.sorted().map { "DOMAIN,\($0)" }
    list += rules.domainKeyword.sorted().map { "DOMAIN-KEYWORD,\($0)" }
    list += rules.ipCidr.sorted().map { "\($0.contains(":") ? "IP-CIDR6" : "IP-CIDR"),\($0)" }
    list += rules.ipAsn.sorted().map { "IP-ASN,\($0)" }
    try (list.joined(separator: "\n") + "\n").write(
      to: outURL.appending(path: "\(name).lsr"), atomically: true, encoding: .utf8)
  }

  private static func toQuantumultX(name: String, rules: Rules, outURL: URL) throws {
    var list: [String] = []
    list += rules.domainSuffix.sorted().map { "HOST-SUFFIX,\($0),Proxy" }
    list += rules.domain.sorted().map { "HOST,\($0),Proxy" }
    list += rules.ipCidr.sorted().map { "\($0.contains(":") ? "IP6-CIDR" : "IP-CIDR"),\($0),Proxy" }
    try (list.joined(separator: "\n") + "\n").write(
      to: outURL.appending(path: "\(name).list"), atomically: true, encoding: .utf8)
  }

  private static func toSurfboard(name: String, rules: Rules, outURL: URL) throws {
    // 1. Standard List (Same as Surge)
    var list: [String] = []
    list += rules.domainSuffix.sorted().map { "DOMAIN-SUFFIX,\($0)" }
    list += rules.domain.sorted().map { "DOMAIN,\($0)" }
    list += rules.domainKeyword.sorted().map { "DOMAIN-KEYWORD,\($0)" }
    list += rules.ipCidr.sorted().map { "\($0.contains(":") ? "IP-CIDR6" : "IP-CIDR"),\($0)" }
    list += rules.ipAsn.sorted().map { "IP-ASN,\($0)" }
    try (list.joined(separator: "\n") + "\n").write(
      to: outURL.appending(path: "\(name).list"), atomically: true, encoding: .utf8)

    // 2. Optimized for Surfboard performance
    if !rules.domainSuffix.isEmpty || !rules.domain.isEmpty {
      let ds = rules.domainSuffix.sorted().map { ".\($0)" } + rules.domain.sorted()
      try (ds.joined(separator: "\n") + "\n").write(
        to: outURL.appending(path: "\(name).txt"), atomically: true, encoding: .utf8)
    }
  }

  private static func toExclave(name: String, rules: Rules, outURL: URL) throws {
    let combined =
      rules.domainSuffix.map { "+.\($0)" } + rules.domain.sorted() + rules.ipCidr.sorted()
    try (combined.joined(separator: "\n") + "\n").write(
      to: outURL.appending(path: "\(name).list"), atomically: true, encoding: .utf8)
  }

  private static func toAnywhere(name: String, rules: Rules, outURL: URL) throws {
    let anyRules =
      rules.domainSuffix.map { AnywhereRule(type: 2, value: $0) }
      + rules.domain.map { AnywhereRule(type: 3, value: $0) }
    try JSONEncoder().encode(anyRules).write(to: outURL.appending(path: "\(name).json"))
  }

  private static func toEgern(name: String, rules: Rules, outURL: URL) throws {
    var egernRules: [[String: [String: String]]] = []
    rules.domainSuffix.sorted().forEach {
      egernRules.append(["domain_suffix": ["match": $0, "policy": "Proxy"]])
    }
    rules.domain.sorted().forEach {
      egernRules.append(["domain": ["match": $0, "policy": "Proxy"]])
    }
    rules.domainKeyword.sorted().forEach {
      egernRules.append(["domain_keyword": ["match": $0, "policy": "Proxy"]])
    }
    rules.domainRegex.sorted().forEach {
      egernRules.append(["domain_regex": ["match": $0, "policy": "Proxy"]])
    }
    rules.ipCidr.sorted().forEach {
      egernRules.append([
        ($0.contains(":") ? "ip_cidr6" : "ip_cidr"): [
          "match": $0, "policy": "Proxy", "no_resolve": "true",
        ]
      ])
    }
    rules.ipAsn.sorted().forEach {
      egernRules.append([
        "asn": ["match": $0.replacingOccurrences(of: "AS", with: ""), "policy": "Proxy"]
      ])
    }

    let yaml = try Yams.dump(object: ["rules": egernRules])
    try ("# Gins-Rules: \(name)\n" + yaml).write(
      to: outURL.appending(path: "\(name).yaml"), atomically: true, encoding: .utf8)
  }

  private static func toPlainList(name: String, rules: Rules, outURL: URL, isIP: Bool) throws {
    let list = rules.domainSuffix.union(rules.domain).union(rules.ipCidr).sorted()
    try (list.joined(separator: "\n") + "\n").write(
      to: outURL.appending(path: "\(name)\(isIP ? ".ip.txt" : ".txt")"), atomically: true,
      encoding: .utf8)
  }

  // MARK: - Shared Utilities

  public static func detectMihomoBehavior(rules: Rules, isIP: Bool) -> String {
    if isIP { return "ipcidr" }
    if !rules.domainKeyword.isEmpty || !rules.domainRegex.isEmpty || !rules.processName.isEmpty
      || !rules.userAgent.isEmpty || !rules.ipAsn.isEmpty
    {
      return "classical"
    }
    if !rules.ipCidr.isEmpty && (!rules.domain.isEmpty || !rules.domainSuffix.isEmpty) {
      return "classical"
    }
    return "domain"
  }

  private static func shell(_ path: String, _ args: [String]) throws {
    let process = Process()
    process.executableURL = URL(filePath: path)
    process.arguments = args
    try process.run()
    process.waitUntilExit()
    if process.terminationStatus != 0 {
      throw RuleCompilerError.shellError(Int(process.terminationStatus))
    }
  }
}
