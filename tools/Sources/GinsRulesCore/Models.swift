import Foundation

/// A high-performance, thread-safe rule container for proxy rules.
/// Optimized for Swift 6.3 structured concurrency.
public struct Rules: Codable, Sendable {
  public var domainSuffix: Set<String> = []
  public var domain: Set<String> = []
  public var domainKeyword: Set<String> = []
  public var domainRegex: Set<String> = []
  public var ipCidr: Set<String> = []
  public var ipAsn: Set<String> = []
  public var processName: Set<String> = []
  public var userAgent: Set<String> = []

  public init() {}

  /// Merges another ruleset into this one efficiently.
  public mutating func merge(with other: Rules) {
    domainSuffix.formUnion(other.domainSuffix)
    domain.formUnion(other.domain)
    domainKeyword.formUnion(other.domainKeyword)
    domainRegex.formUnion(other.domainRegex)
    ipCidr.formUnion(other.ipCidr)
    ipAsn.formUnion(other.ipAsn)
    processName.formUnion(other.processName)
    userAgent.formUnion(other.userAgent)
  }

  /// Total count of unique rules.
  public var count: Int {
    domainSuffix.count + domain.count + domainKeyword.count + domainRegex.count + ipCidr.count
      + ipAsn.count + processName.count + userAgent.count
  }

  public var isEmpty: Bool { count == 0 }

  /// Returns a version of these rules filtered for regional/sensitive restrictions.
  public func sanitized() -> Rules {
    let restricted: Set<String> = [
      "tiktok.com", "tiktokv.com", "tiktokcdn.com", "byteoversea.com",
      "ibyteimg.com", "ibytedtos.com", "ipstatp.com", "muscdn.com",
      "musical.ly", "tik-tokapi.com",
    ]
    var copy = self
    let isRestricted = { (d: String) -> Bool in
      if d.localizedCaseInsensitiveContains("tiktok")
        || d.localizedCaseInsensitiveContains("tik-tok")
      {
        return true
      }
      return restricted.contains { d == $0 || d.hasSuffix(".\($0)") }
    }
    copy.domain = Set(copy.domain.filter { !isRestricted($0) })
    copy.domainSuffix = Set(copy.domainSuffix.filter { !isRestricted($0) })
    return copy
  }
}

// --- Rule Serialization Models ---

public struct SingBoxRuleSet: Codable, Sendable {
  public let version: Int
  public let rules: [SingBoxRule]
  public init(version: Int, rules: [SingBoxRule]) {
    self.version = version
    self.rules = rules
  }
}

public struct SingBoxRule: Codable, Sendable {
  public var domainSuffix: [String]?
  public var domain: [String]?
  public var domainKeyword: [String]?
  public var domainRegex: [String]?
  public var ipCidr: [String]?
  public var processName: [String]?
  public var userAgent: [String]?

  enum CodingKeys: String, CodingKey {
    case domainSuffix = "domain_suffix"
    case domain
    case domainKeyword = "domain_keyword"
    case domainRegex = "domain_regex"
    case ipCidr = "ip_cidr"
    case processName = "process_name"
    case userAgent = "user_agent"
  }

  public init(
    domainSuffix: [String]? = nil, domain: [String]? = nil, domainKeyword: [String]? = nil,
    domainRegex: [String]? = nil, ipCidr: [String]? = nil, processName: [String]? = nil,
    userAgent: [String]? = nil
  ) {
    self.domainSuffix = domainSuffix
    self.domain = domain
    self.domainKeyword = domainKeyword
    self.domainRegex = domainRegex
    self.ipCidr = ipCidr
    self.processName = processName
    self.userAgent = userAgent
  }
}
