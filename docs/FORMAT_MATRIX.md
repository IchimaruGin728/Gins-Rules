# Rule Format Matrix

This is the migration contract for the Rust rulekit. The Go compiler remains the comparison baseline until Rust emits every format here.

| Format | Current output | Upstream syntax target | Must preserve |
| --- | --- | --- | --- |
| sing-box | `compiled/ruleset/singbox/**/{name}.json`, `.srs` | Source rule-set JSON v2, compiled with `sing-box rule-set compile` | `domain_suffix`, `domain`, `domain_keyword`, `domain_regex`, `ip_cidr`; ASN rules must be resolved to CIDR before SRS |
| mihomo | `compiled/ruleset/mihomo/**/{name}.yaml`, `.mrs` | rule-provider payload with `domain`, `ipcidr`, or `classical` behavior | `DOMAIN`, `DOMAIN-SUFFIX`, `DOMAIN-KEYWORD`, `DOMAIN-REGEX`, `IP-CIDR`, `IP-CIDR6`, `IP-ASN`; MRS behavior must match payload type |
| Stash | `compiled/ruleset/stash/**/{name}.yaml`, `.mrs` | Clash/Mihomo compatible rule-provider payload | Same as mihomo unless Stash diverges |
| text | `compiled/ruleset/text/**/{name}.list` | Portable comma rule list | All supported rule classes in a readable fallback artifact |
| Quantumult X | `compiled/ruleset/quantumultx/**/{name}.list` | `host-suffix`, `host`, `host-keyword`, `ip-cidr`, `ip6-cidr`, `ip-asn`, `USER-AGENT` | QX resource parser must be synced from upstream each scheduled run |
| Loon | `compiled/ruleset/loon/**/{name}.lsr` | Loon rule list | Loon Sub-Store parser must be synced from upstream each scheduled run |
| Surge | `compiled/ruleset/surge/**/{name}.list`, `.domainset` | Surge rule list and domain-set | `DOMAIN-*`, `IP-CIDR*`, `IP-ASN`, process/user-agent where supported by list output; domainset contains suffix/exact domains only |
| Shadowrocket | `compiled/ruleset/shadowrocket/**/{name}.list`, `.txt` | Shadowrocket rule list and domain-set style text | Same list surface as Surge/Loon; domainset-style text for suffix/exact domains |
| Surfboard | `compiled/ruleset/surfboard/**/{name}.list`, `.txt` | Surfboard profile rule list | Domain/IP core rules; avoid emitting unsupported special rules in Surfboard list |
| Egern | `compiled/ruleset/egern/**/{name}.yaml` | Egern rule-set YAML | `domain_suffix_set`, `domain_set`, `domain_keyword_set`, `domain_regex_set`, `ip_cidr_set`, `ip_cidr6_set`, `ip_asn_set`, `user_agent_set` |
| Exclave | `compiled/ruleset/exclave/**/{name}.list` | Exclave route list | `domain:`, `full:`, `keyword:`, `regexp:`, `ip:`, `asn:` |
| Xray | `compiled/ruleset/xray/*` | DAT assets | `geoip`, `geosite`, and `geoasn` tags generated from our source categories |
| MMDB | `compiled/ruleset/xray/*`, `compiled/asn-prefix-index.json` | MaxMind-compatible IP/ASN assets | `asn-cn` and all `asn-*` files must become real ASN-derived CIDR/prefix records, not just tag aliases |

## Parser Inputs

Rust sync must accept all input shapes already handled by Go and the newer upstream formats:

- Plain domain/IP lines, including `+.example.com` and `.example.com`.
- v2ray-domain-list style prefixes: `domain:`, `full:`, `keyword:`, `regexp:`.
- Clash/Mihomo/Loon/Surge/QX comma rules: `DOMAIN`, `DOMAIN-SUFFIX`, `DOMAIN-KEYWORD`, `DOMAIN-REGEX`, `URL-REGEX`, `PROCESS-NAME`, `USER-AGENT`, `IP-CIDR`, `IP-CIDR6`, `IP-ASN`.
- YAML payload lines such as `- DOMAIN-SUFFIX,example.com,Proxy`.
- sing-box source JSON rule-sets.

## Documentation Sources

- mihomo route rules: https://wiki.metacubex.one/en/config/rules/
- sing-box source rule-set format: https://sing-box.sagernet.org/configuration/rule-set/source-format/
- Loon docs: https://nsloon.app/docs/intro/
- Surge docs index: https://nssurge.com/llms.txt
- Stash rule-set docs: https://stash.wiki/en/rules/rule-set
- Quantumult X repository/wiki references: https://github.com/crossutility/Quantumult-X and https://qx.atlucky.me/shi-yong-fang-fa/pei-zhi-wen-jian-xiang-jie
- Egern config example: https://egernapp.com/docs/configuration/example/
- Surfboard profile docs: https://getsurfboard.com/docs/profile-format/overview/
- Exclave config wiki: https://github.com/dyhkwong/Exclave/wiki/Configuration
- Shadowrocket docs: https://lowertop.github.io/Shadowrocket/#配置文件
