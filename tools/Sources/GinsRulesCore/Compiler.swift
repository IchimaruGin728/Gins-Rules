import Foundation
import Yams

public enum RuleCompiler {
  public enum Format: String, CaseIterable {
    case singbox, mihomo, text, quantumultx, surge, loon, shadowrocket
  }

  public static func compile(_ format: Format, name: String, rules: Rules, outURL: URL, isIP: Bool)
    throws
  {
    try FileManager.default.createDirectory(at: outURL, withIntermediateDirectories: true)

    switch format {
    case .singbox:
      try toSingBoxJSON(name: name, rules: rules, outURL: outURL)
    case .mihomo:
      let behavior = detectBehavior(rules: rules, isIP: isIP)
      try toMihomoYAML(name: name, rules: rules, outURL: outURL, behavior: behavior)
    case .text:
      try toPlainList(name: name, rules: rules, outURL: outURL, isIP: isIP)
    case .quantumultx:
      try toSimpleList(
        name: name, rules: rules, outURL: outURL, extension: ".list", prefix: "HOST-SUFFIX")
    case .surge:
      try toSimpleList(
        name: name, rules: rules, outURL: outURL, extension: ".list", prefix: "DOMAIN-SUFFIX")
    case .loon:
      try toSimpleList(
        name: name, rules: rules, outURL: outURL, extension: ".lsr", prefix: "DOMAIN-SUFFIX")
    case .shadowrocket:
      try toSimpleList(
        name: name, rules: rules, outURL: outURL, extension: ".list", prefix: "DOMAIN-SUFFIX")
    }
  }

  private static func toSingBoxJSON(name: String, rules: Rules, outURL: URL) throws {
    let rs = SingBoxRuleSet(
      version: 1,
      rules: [
        SingBoxRule(
          domainSuffix: rules.domainSuffix.isEmpty ? nil : rules.domainSuffix.sorted(),
          domain: rules.domain.isEmpty ? nil : rules.domain.sorted(),
          domainKeyword: rules.domainKeyword.isEmpty ? nil : rules.domainKeyword.sorted(),
          domainRegex: rules.domainRegex.isEmpty ? nil : rules.domainRegex.sorted(),
          ipCidr: rules.ipCidr.isEmpty ? nil : rules.ipCidr.sorted(),
          processName: rules.processName.isEmpty ? nil : rules.processName.sorted(),
          userAgent: rules.userAgent.isEmpty ? nil : rules.userAgent.sorted()
        )
      ])
    let data = try JSONEncoder().encode(rs)
    try data.write(to: outURL.appending(path: "\(name).json"))
  }

  private static func toMihomoYAML(name: String, rules: Rules, outURL: URL, behavior: String) throws
  {
    var payload: [String] = []
    if behavior == "classical" {
      payload.append(contentsOf: rules.domainSuffix.sorted().map { "DOMAIN-SUFFIX,\($0)" })
      payload.append(contentsOf: rules.domain.sorted().map { "DOMAIN,\($0)" })
      payload.append(contentsOf: rules.domainKeyword.sorted().map { "DOMAIN-KEYWORD,\($0)" })
      payload.append(contentsOf: rules.domainRegex.sorted().map { "DOMAIN-REGEXP,\($0)" })
      payload.append(
        contentsOf: rules.ipCidr.sorted().map {
          "\($0.contains(":") ? "IP-CIDR6" : "IP-CIDR"),\($0)"
        })
      payload.append(contentsOf: rules.ipAsn.sorted().map { "IP-ASN,\($0)" })
      payload.append(contentsOf: rules.processName.sorted().map { "PROCESS-NAME,\($0)" })
      payload.append(contentsOf: rules.userAgent.sorted().map { "USER-AGENT,\($0)" })
    } else if behavior == "ipcidr" {
      payload.append(contentsOf: rules.ipCidr.sorted())
    } else {
      payload.append(contentsOf: rules.domainSuffix.union(rules.domain).sorted())
    }
    let yaml = try Yams.dump(object: ["payload": payload])
    try yaml.write(to: outURL.appending(path: "\(name).yaml"), atomically: true, encoding: .utf8)
  }

  private static func toPlainList(name: String, rules: Rules, outURL: URL, isIP: Bool) throws {
    var lines: [String] = []
    lines.append(contentsOf: rules.domainSuffix.sorted().map { "DOMAIN-SUFFIX,\($0)" })
    lines.append(contentsOf: rules.domain.sorted().map { "DOMAIN,\($0)" })
    lines.append(
      contentsOf: rules.ipCidr.sorted().map { "\($0.contains(":") ? "IP-CIDR6" : "IP-CIDR"),\($0)" }
    )
    try (lines.joined(separator: "\n") + "\n").write(
      to: outURL.appending(path: "\(name)\(isIP ? ".ip.list" : ".list")"), atomically: true,
      encoding: .utf8)
  }

  private static func toSimpleList(
    name: String, rules: Rules, outURL: URL, extension ext: String, prefix: String
  ) throws {
    var lines: [String] = []
    lines.append(contentsOf: rules.domainSuffix.sorted().map { "\(prefix),\($0)" })
    lines.append(contentsOf: rules.domain.sorted().map { "DOMAIN,\($0)" })
    lines.append(
      contentsOf: rules.ipCidr.sorted().map { "\($0.contains(":") ? "IP-CIDR6" : "IP-CIDR"),\($0)" }
    )
    try (lines.joined(separator: "\n") + "\n").write(
      to: outURL.appending(path: "\(name)\(ext)"), atomically: true, encoding: .utf8)
  }

  public static func detectBehavior(rules: Rules, isIP: Bool) -> String {
    if !rules.domainKeyword.isEmpty || !rules.domainRegex.isEmpty || !rules.processName.isEmpty
      || !rules.userAgent.isEmpty
    {
      return "classical"
    }
    return isIP ? "ipcidr" : "domain"
  }
}
