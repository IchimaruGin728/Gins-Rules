import Foundation

/// A high-performance, thread-safe rule container using Swift Sets for O(1) deduplication.
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

  /// Total count of all unique rules across all categories.
  public var count: Int {
    domainSuffix.count + domain.count + domainKeyword.count + domainRegex.count + ipCidr.count
      + ipAsn.count + processName.count + userAgent.count
  }

  public var isEmpty: Bool { count == 0 }

  /// Merges another set of rules into this one efficiently.
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

  /// Returns a new Rules instance with sensitive/unwanted rules removed.
  public func sanitized() -> Rules {
    let restricted: Set<String> = [
      "tiktok.com", "tiktokv.com", "tiktokcdn.com", "byteoversea.com",
      "ibyteimg.com", "ibytedtos.com", "ipstatp.com", "muscdn.com",
      "musical.ly", "tik-tokapi.com",
    ]

    let isRestricted = { (d: String) -> Bool in
      if d.localizedStandardContains("tiktok") || d.localizedStandardContains("tik-tok") {
        return true
      }
      return restricted.contains { d == $0 || d.hasSuffix(".\($0)") }
    }

    var copy = self
    copy.domain = Set(copy.domain.filter { !isRestricted($0) })
    copy.domainSuffix = Set(copy.domainSuffix.filter { !isRestricted($0) })
    return copy
  }
}

// --- Platform Specific Rule Models ---

public struct MihomoRuleMode: Codable, Sendable {
  public let behavior: String
  public init(behavior: String) { self.behavior = behavior }
}

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
