package main

import (
	"encoding/json"
	"fmt"
	"net"
	"os"
	"path/filepath"
	"strings"

	"github.com/maxmind/mmdbwriter"
	"github.com/maxmind/mmdbwriter/mmdbtype"
	"github.com/xtls/xray-core/common/geodata"
	"google.golang.org/protobuf/proto"
)

// ASNMap mirrors source/asn-map.json for MMDB generation
type ASNMapDef struct {
	Services map[string]struct {
		ASNs []int  `json:"asns"`
		Org  string `json:"org"`
	} `json:"services"`
}

type ASNPrefixRecord struct {
	ASN  uint32 `json:"asn"`
	CIDR string `json:"cidr"`
	Org  string `json:"org,omitempty"`
}

type ASNPrefixIndex map[string][]ASNPrefixRecord

func loadASNMap(root string) ASNMapDef {
	var m ASNMapDef
	data, err := os.ReadFile(filepath.Join(root, "source", "asn-map.json"))
	if err != nil {
		fmt.Printf("  [WARN] asn-map.json not found, ASN MMDB will have no ASN numbers\n")
		return m
	}
	_ = json.Unmarshal(data, &m)
	return m
}

func loadASNPrefixIndex(root string) ASNPrefixIndex {
	for _, path := range []string{
		filepath.Join(root, "compiled", "asn-prefix-index.json"),
		filepath.Join(root, "source", "asn-prefix-index.json"),
	} {
		data, err := os.ReadFile(path)
		if err != nil {
			continue
		}
		var index ASNPrefixIndex
		if err := json.Unmarshal(data, &index); err != nil {
			fmt.Printf("  [WARN] cannot parse %s: %v\n", path, err)
			continue
		}
		return index
	}

	fmt.Printf("  [WARN] asn-prefix-index.json not found, geoasn.mmdb will only use CIDR-backed ASN rules\n")
	return ASNPrefixIndex{}
}

func resolvedASNCIDRs(name string, rules Rules, prefixIndex ASNPrefixIndex) []string {
	seen := make(map[string]bool)
	var cidrs []string
	for _, cidr := range rules.IPCIDR {
		if !seen[cidr] {
			seen[cidr] = true
			cidrs = append(cidrs, cidr)
		}
	}
	for _, record := range prefixIndex[name] {
		if record.CIDR != "" && !seen[record.CIDR] {
			seen[record.CIDR] = true
			cidrs = append(cidrs, record.CIDR)
		}
	}
	return cidrs
}

func compileMMDB(allRules map[string]map[string]Rules, outDir string) error {
	root := findRoot()
	asnMapDef := loadASNMap(root)
	asnPrefixIndex := loadASNPrefixIndex(root)
	insertedASNNetworks := make(map[string]bool)

	// 1. GeoIP MMDB
	writerIP, _ := mmdbwriter.New(mmdbwriter.Options{
		DatabaseType: "GeoLite2-Country",
		RecordSize:   24,
	})

	// 2. GeoASN MMDB
	writerASN, _ := mmdbwriter.New(mmdbwriter.Options{
		DatabaseType: "GeoLite2-ASN",
		RecordSize:   24,
	})

	for category, rulesMap := range allRules {
		for name, rules := range rulesMap {
			tag := strings.ToUpper(name)

			if category == "ip" {
				for _, cidr := range rules.IPCIDR {
					_, network, err := net.ParseCIDR(cidr)
					if err != nil {
						continue
					}
					if err := writerIP.Insert(network, mmdbtype.Map{
						"country": mmdbtype.Map{
							"iso_code": mmdbtype.String(tag),
						},
					}); err != nil {
						fmt.Printf("  [WARN] geoip(mmdb) insert failed for %s: %v\n", cidr, err)
					}
				}
			}

			if category == "asn" {
				// name is e.g. "asn-telegram" — strip prefix to get service name
				svcName := strings.TrimPrefix(strings.ToLower(name), "asn-")

				// Look up ASN numbers from asn-map.json
				svcDef, hasDef := asnMapDef.Services[svcName]

				for _, cidr := range resolvedASNCIDRs(name, rules, asnPrefixIndex) {
					_, network, err := net.ParseCIDR(cidr)
					if err != nil {
						continue
					}

					// Write GeoIP entry (tag-based, for geoip:asn-telegram usage)
					if err := writerIP.Insert(network, mmdbtype.Map{
						"country": mmdbtype.Map{
							"iso_code": mmdbtype.String(tag),
						},
					}); err != nil {
						fmt.Printf("  [WARN] geoip(mmdb) insert failed for %s: %v\n", cidr, err)
					}
				}

				records := asnPrefixIndex[name]
				if len(records) == 0 && len(rules.IPASN) > 0 {
					fmt.Printf("  [WARN] geoasn(mmdb) has ASN declarations for %s but no expanded prefix index\n", name)
				}

				for _, record := range records {
					if record.ASN == 0 || record.CIDR == "" {
						continue
					}
					if insertedASNNetworks[record.CIDR] {
						continue
					}
					_, network, err := net.ParseCIDR(record.CIDR)
					if err != nil {
						continue
					}
					org := record.Org
					if org == "" && hasDef {
						org = svcDef.Org
					}
					if org == "" {
						org = fmt.Sprintf("AS%d", record.ASN)
					}
					if err := writerASN.Insert(network, mmdbtype.Map{
						"autonomous_system_number":       mmdbtype.Uint32(record.ASN),
						"autonomous_system_organization": mmdbtype.String(org),
					}); err != nil {
						fmt.Printf("  [WARN] geoasn(mmdb) insert failed for AS%d %s: %v\n", record.ASN, record.CIDR, err)
						continue
					}
					insertedASNNetworks[record.CIDR] = true
				}
			}
		}
	}

	// Write files
	outIP, err := os.Create(filepath.Join(outDir, "geoip.mmdb"))
	if err == nil {
		defer outIP.Close()
		if _, werr := writerIP.WriteTo(outIP); werr != nil {
			fmt.Printf("  [WARN] writing geoip.mmdb failed: %v\n", werr)
		}
	}

	outASN, err := os.Create(filepath.Join(outDir, "geoasn.mmdb"))
	if err == nil {
		defer outASN.Close()
		if _, werr := writerASN.WriteTo(outASN); werr != nil {
			fmt.Printf("  [WARN] writing geoasn.mmdb failed: %v\n", werr)
		}
	}

	fmt.Printf("  [MMDB] Created geoip.mmdb, geoasn.mmdb\n")
	return nil
}

func compileXrayDAT(allRules map[string]map[string]Rules, outDir string) error {
	root := findRoot()
	asnPrefixIndex := loadASNPrefixIndex(root)
	geositeList := &geodata.GeoSiteList{}
	geoipList := &geodata.GeoIPList{}

	for category, rulesMap := range allRules {
		for name, rules := range rulesMap {
			// Xray tag: geosite:apple or geoip:cn
			tag := strings.ToLower(name)

			// 1. Geosite (Domains)
			if category != "ip" && category != "asn" {
				site := &geodata.GeoSite{
					Code: strings.ToUpper(tag),
				}
				for _, d := range rules.DomainSuffix {
					site.Domain = append(site.Domain, &geodata.Domain{Type: geodata.Domain_Domain, Value: d})
				}
				for _, d := range rules.Domain {
					site.Domain = append(site.Domain, &geodata.Domain{Type: geodata.Domain_Full, Value: d})
				}
				for _, d := range rules.DomainKeyword {
					site.Domain = append(site.Domain, &geodata.Domain{Type: geodata.Domain_Substr, Value: d})
				}
				for _, d := range rules.DomainRegex {
					site.Domain = append(site.Domain, &geodata.Domain{Type: geodata.Domain_Regex, Value: d})
				}

				if len(site.Domain) > 0 {
					geositeList.Entry = append(geositeList.Entry, site)
				}
			}

			// 2. GeoIP (IP CIDRs)
			cidrs := rules.IPCIDR
			if category == "asn" {
				cidrs = resolvedASNCIDRs(name, rules, asnPrefixIndex)
			}
			if (category == "ip" || category == "asn") && len(cidrs) > 0 {
				geoIP := &geodata.GeoIP{
					Code: strings.ToUpper(tag),
				}
				for _, cidr := range cidrs {
					ip, ipnet, err := net.ParseCIDR(cidr)
					if err != nil {
						fmt.Printf("  [Xray] Skip invalid CIDR: %s\n", cidr)
						continue
					}
					ones, _ := ipnet.Mask.Size()

					// Convert IP to byte slice
					var ipBytes []byte
					if ip4 := ip.To4(); ip4 != nil {
						ipBytes = ip4
					} else {
						ipBytes = ip.To16()
					}

					geoIP.Cidr = append(geoIP.Cidr, &geodata.CIDR{
						Ip:     ipBytes,
						Prefix: uint32(ones),
					})
				}

				if len(geoIP.Cidr) > 0 {
					geoipList.Entry = append(geoipList.Entry, geoIP)
				}
			}
		}
	}

	// Create output directory
	xrayDir := filepath.Join(outDir, "xray")
	os.MkdirAll(xrayDir, 0755)

	// Write geosite.dat
	geositeBytes, err := proto.Marshal(geositeList)
	if err != nil {
		return fmt.Errorf("failed to marshal geosite: %w", err)
	}
	if err := os.WriteFile(filepath.Join(xrayDir, "geosite.dat"), geositeBytes, 0644); err != nil {
		return fmt.Errorf("failed to write geosite.dat: %w", err)
	}

	// Write geoip.dat
	geoipBytes, err := proto.Marshal(geoipList)
	if err != nil {
		return fmt.Errorf("failed to marshal geoip: %w", err)
	}
	if err := os.WriteFile(filepath.Join(xrayDir, "geoip.dat"), geoipBytes, 0644); err != nil {
		return fmt.Errorf("failed to write geoip.dat: %w", err)
	}

	fmt.Printf("\n  [Xray] geosite.dat: %d tags, geoip.dat: %d tags\n", len(geositeList.Entry), len(geoipList.Entry))
	return nil
}
