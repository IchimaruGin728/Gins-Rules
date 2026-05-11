import Foundation

public enum RuleParser {
  /// Parses a rule source file at the given local URL.
  public static func parse(url: URL) throws -> Rules {
    let content = try String(contentsOf: url, encoding: .utf8)
    return parse(content: content)
  }

  /// Parses rule content from a raw string using high-speed string processing.
  public static func parse(content: String) -> Rules {
    var rules = Rules()

    content.enumerateLines { line, _ in
      let trimmed = line.trimmingCharacters(in: .whitespaces)
      guard !trimmed.isEmpty, !trimmed.hasPrefix("#"), !trimmed.hasPrefix(";"),
        !trimmed.hasPrefix("//")
      else { return }

      if trimmed.hasPrefix("full:") {
        rules.domain.insert(String(trimmed.dropFirst(5)))
      } else if trimmed.hasPrefix("keyword:") {
        rules.domainKeyword.insert(String(trimmed.dropFirst(8)))
      } else if trimmed.hasPrefix("regexp:") {
        rules.domainRegex.insert(String(trimmed.dropFirst(7)))
      } else if trimmed.hasPrefix("process:") {
        rules.processName.insert(String(trimmed.dropFirst(8)))
      } else if trimmed.hasPrefix("user-agent:") {
        rules.userAgent.insert(String(trimmed.dropFirst(11)))
      } else if trimmed.hasPrefix("asn:") {
        rules.ipAsn.insert(String(trimmed.dropFirst(4)))
      } else if trimmed.contains("/") {
        rules.ipCidr.insert(trimmed)
      } else {
        // Default to domain suffix, stripping leading control characters
        let domain = trimmed.trimmingCharacters(in: CharacterSet(charactersIn: "+."))
        rules.domainSuffix.insert(domain)
      }
    }

    return rules
  }
}
