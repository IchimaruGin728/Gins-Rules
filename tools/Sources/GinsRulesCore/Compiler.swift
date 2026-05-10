import Foundation
import Yams

public enum Compiler {
  public static func compileSingBoxJSON(name: String, rules: Rules, to outDir: URL) throws -> URL {
    let rs = SingBoxRuleSet(
      version: 1,
      rules: [
        SingBoxRule(
          domainSuffix: rules.domainSuffix.isEmpty ? nil : Array(rules.domainSuffix).sorted(),
          domain: rules.domain.isEmpty ? nil : Array(rules.domain).sorted(),
          domainKeyword: rules.domainKeyword.isEmpty ? nil : Array(rules.domainKeyword).sorted(),
          domainRegex: rules.domainRegex.isEmpty ? nil : Array(rules.domainRegex).sorted(),
          ipCidr: rules.ipCidr.isEmpty ? nil : Array(rules.ipCidr).sorted(),
          processName: rules.processName.isEmpty ? nil : Array(rules.processName).sorted(),
          userAgent: rules.userAgent.isEmpty ? nil : Array(rules.userAgent).sorted()
        )
      ])

    let encoder = JSONEncoder()
    encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
    let data = try encoder.encode(rs)
    let fileURL = outDir.appending(path: "\(name).json")
    try data.write(to: fileURL)
    return fileURL
  }

  public static func compileMihomoYAML(
    name: String, rules: Rules, to outDir: URL, mode: MihomoRuleMode
  ) throws {
    var payload: [String] = []

    if mode.behavior == "classical" {
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
    } else if mode.behavior == "ipcidr" {
      payload.append(contentsOf: rules.ipCidr.sorted())
    } else {
      // domain behavior
      var domains = rules.domainSuffix
      domains.formUnion(rules.domain)
      payload.append(
        contentsOf: domains.filter { $0.components(separatedBy: ".").count <= 5 }.sorted())
    }

    let yamlData: [String: Any] = [
      "payload": payload
    ]

    let header = "# Gins-Rules: \(name)\n# Auto-generated, do not edit\n\n"
    let yamlString = try Yams.dump(object: yamlData)
    let finalString = header + yamlString

    let fileURL = outDir.appending(path: "\(name).yaml")
    try finalString.write(to: fileURL, atomically: true, encoding: .utf8)
  }

  public static func detectMihomoRuleMode(rules: Rules, isIPCategory: Bool) -> MihomoRuleMode {
    let hasIP = !rules.ipCidr.isEmpty
    let hasASN = !rules.ipAsn.isEmpty
    let hasDomain =
      !rules.domainSuffix.isEmpty || !rules.domain.isEmpty || !rules.domainKeyword.isEmpty
      || !rules.domainRegex.isEmpty
    let hasOther = !rules.processName.isEmpty || !rules.userAgent.isEmpty
    let hasClassicalOnly = !rules.domainKeyword.isEmpty || !rules.domainRegex.isEmpty

    if hasOther || hasASN || (hasIP && hasDomain) || hasClassicalOnly {
      return MihomoRuleMode(behavior: "classical")
    }
    if isIPCategory || (hasIP && !hasDomain) {
      return MihomoRuleMode(behavior: "ipcidr")
    }
    return MihomoRuleMode(behavior: "domain")
  }

  public static func compileTextList(name: String, rules: Rules, to outDir: URL, isIP: Bool) throws
  {
    var lines: [String] = []
    lines.append(contentsOf: rules.domainSuffix.sorted().map { "DOMAIN-SUFFIX,\($0)" })
    lines.append(contentsOf: rules.domain.sorted().map { "DOMAIN,\($0)" })
    lines.append(contentsOf: rules.domainKeyword.sorted().map { "DOMAIN-KEYWORD,\($0)" })
    lines.append(
      contentsOf: rules.ipCidr.sorted().map { "\($0.contains(":") ? "IP-CIDR6" : "IP-CIDR"),\($0)" }
    )
    lines.append(contentsOf: rules.ipAsn.sorted().map { "IP-ASN,\($0)" })

    let content = lines.joined(separator: "\n") + "\n"
    let ext = isIP ? ".ip.list" : ".list"
    let fileURL = outDir.appending(path: "\(name)\(ext)")
    try content.write(to: fileURL, atomically: true, encoding: .utf8)
  }

  public static func compileQuanXList(
    name: String, rules: Rules, to outDir: URL, isIP: Bool, category: String
  ) throws {
    let policy =
      switch category {
      case "direct": "Direct"
      case "reject": "Reject"
      default: "Proxy"
      }

    var lines: [String] = []
    lines.append(contentsOf: rules.domainSuffix.sorted().map { "HOST-SUFFIX,\($0),\(policy)" })
    lines.append(contentsOf: rules.domain.sorted().map { "HOST,\($0),\(policy)" })
    lines.append(contentsOf: rules.domainKeyword.sorted().map { "HOST-KEYWORD,\($0),\(policy)" })
    lines.append(
      contentsOf: rules.ipCidr.sorted().map {
        "\($0.contains(":") ? "IP6-CIDR" : "IP-CIDR"),\($0),\(policy)"
      })
    lines.append(contentsOf: rules.userAgent.sorted().map { "USER-AGENT,\($0),\(policy)" })

    let content = lines.joined(separator: "\n") + "\n"
    let ext = isIP ? ".ip.list" : ".list"
    let fileURL = outDir.appending(path: "\(name)\(ext)")
    try content.write(to: fileURL, atomically: true, encoding: .utf8)
  }
}
