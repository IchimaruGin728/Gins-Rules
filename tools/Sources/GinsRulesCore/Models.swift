import Foundation

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

  public var count: Int {
    domainSuffix.count + domain.count + domainKeyword.count + domainRegex.count + ipCidr.count
      + ipAsn.count + processName.count + userAgent.count
  }

  public var isEmpty: Bool {
    count == 0
  }

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
}

public struct MihomoRuleMode: Codable, Sendable {
  public var behavior: String

  public init(behavior: String) {
    self.behavior = behavior
  }
}

public struct SingBoxRuleSet: Codable, Sendable {
  public var version: Int
  public var rules: [SingBoxRule]

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
    domainSuffix: [String]? = nil,
    domain: [String]? = nil,
    domainKeyword: [String]? = nil,
    domainRegex: [String]? = nil,
    ipCidr: [String]? = nil,
    processName: [String]? = nil,
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

public struct AnywhereRule: Codable, Sendable {
  public var type: Int
  public var value: String

  public init(type: Int, value: String) {
    self.type = type
    self.value = value
  }
}

public struct AsnPrefixRecord: Codable, Sendable {
  public var asn: UInt32
  public var cidr: String
  public var org: String?

  public init(asn: UInt32, cidr: String, org: String? = nil) {
    self.asn = asn
    self.cidr = cidr
    self.org = org
  }
}
