import Foundation

public enum Parser {
  public static func parseSource(at url: URL) throws -> Rules {
    let content = try String(contentsOf: url, encoding: .utf8)
    return parse(content: content)
  }

  public static func parse(content: String) -> Rules {
    var rules = Rules()
    let lines = content.components(separatedBy: .newlines)

    for line in lines {
      let trimmed = line.trimmingCharacters(in: .whitespaces)
      if trimmed.isEmpty || trimmed.hasPrefix("#") {
        continue
      }

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
        let domain = trimmed.trimmingCharacters(in: CharacterSet(charactersIn: "+."))
        rules.domainSuffix.insert(domain)
      }
    }
    return rules
  }
}

extension Rules {
  public func sanitized() -> Rules {
    let forceProxy: Set<String> = [
      "tiktok.com", "tiktokv.com", "tiktokcdn.com", "byteoversea.com",
      "ibyteimg.com", "ibytedtos.com", "ipstatp.com", "muscdn.com",
      "musical.ly", "tik-tokapi.com",
    ]

    let isForce = { (d: String) -> Bool in
      if d.contains("tiktok") || d.contains("tik-tok") {
        return true
      }
      return forceProxy.contains { d == $0 || d.hasSuffix(".\($0)") }
    }

    var newRules = self
    newRules.domain = newRules.domain.filter { !isForce($0) }
    newRules.domainSuffix = newRules.domainSuffix.filter { !isForce($0) }
    return newRules
  }
}
