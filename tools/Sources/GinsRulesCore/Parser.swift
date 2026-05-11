import Foundation

public enum RuleParser {
  /// High-performance line-based parser using modern Swift string primitives.
  public static func parse(content: String) -> Rules {
    var rules = Rules()

    content.enumerateLines { line, _ in
      let trimmed = line.trimmingCharacters(in: .whitespaces)
      guard !trimmed.isEmpty, !trimmed.hasPrefix("#"), !trimmed.hasPrefix(";"),
        !trimmed.hasPrefix("//")
      else { return }

      // Handle Clash/Surge style rules (e.g. DOMAIN-SUFFIX,google.com,Proxy)
      let parts = trimmed.split(separator: ",").map { $0.trimmingCharacters(in: .whitespaces) }
      if parts.count >= 2 {
        let type = parts[0].uppercased()
        let val = parts[1]

        // Skip complex rules
        if type == "AND" || type == "OR" || type == "NOT" { return }

        switch type {
        case "DOMAIN", "DOMAIN-SUFFIX", "DOMAIN-KEYWORD", "DOMAIN-REGEX", "IP-CIDR", "IP-CIDR6",
          "IP6-CIDR", "IP-ASN", "PROCESS-NAME", "USER-AGENT", "HOST", "HOST-SUFFIX", "HOST-KEYWORD":
          switch type {
          case "DOMAIN", "HOST": rules.domain.insert(val)
          case "DOMAIN-SUFFIX", "HOST-SUFFIX": rules.domainSuffix.insert(val)
          case "DOMAIN-KEYWORD", "HOST-KEYWORD": rules.domainKeyword.insert(val)
          case "DOMAIN-REGEX": rules.domainRegex.insert(val)
          case "IP-CIDR", "IP-CIDR6", "IP6-CIDR": rules.ipCidr.insert(val)

          case "IP-ASN": rules.ipAsn.insert(val)
          case "PROCESS-NAME": rules.processName.insert(val)
          case "USER-AGENT": rules.userAgent.insert(val)
          default: break
          }
          return
        default:
          break  // Fallthrough to legacy parsing if not a standard type
        }
      }

      // Legacy/Simple parsing
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
        // Default to domain suffix, stripping leading control characters (+.)
        rules.domainSuffix.insert(trimmed.trimmingCharacters(in: CharacterSet(charactersIn: "+.")))
      }
    }

    return rules
  }

  /// Parses a local file.
  public static func parse(url: URL) throws -> Rules {
    let content = try String(contentsOf: url, encoding: .utf8)
    return parse(content: content)
  }
}
