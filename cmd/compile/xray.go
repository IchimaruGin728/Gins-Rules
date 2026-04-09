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
	"github.com/xtls/xray-core/app/router"
	"google.golang.org/protobuf/proto"
)

// ASNMap mirrors source/asn-map.json for MMDB generation
type ASNMapDef struct {
	Services map[string]struct {
		ASNs []int  `json:"asns"`
		Org  string `json:"org"`
	} `json:"services"`
}

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

func compileMMDB(allRules map[string]map[string]Rules, outDir string) error {
	root := findRoot()
	asnMapDef := loadASNMap(root)

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
					writerIP.Insert(network, mmdbtype.Map{
						"country": mmdbtype.Map{
							"iso_code": mmdbtype.String(tag),
						},
					})
				}
			}

			if category == "asn" {
				// name is e.g. "asn-telegram" — strip prefix to get service name
				svcName := strings.TrimPrefix(strings.ToLower(name), "asn-")

				// Look up ASN numbers from asn-map.json
				svcDef, hasDef := asnMapDef.Services[svcName]

				for _, cidr := range rules.IPCIDR {
					_, network, err := net.ParseCIDR(cidr)
					if err != nil {
						continue
					}

					// Write GeoIP entry (tag-based, for geoip:asn-telegram usage)
					writerIP.Insert(network, mmdbtype.Map{
						"country": mmdbtype.Map{
							"iso_code": mmdbtype.String(tag),
						},
					})

					// Write ASN entries — one per ASN number in the map
					if hasDef && len(svcDef.ASNs) > 0 {
						for _, asnNum := range svcDef.ASNs {
							writerASN.Insert(network, mmdbtype.Map{
								"autonomous_system_number":       mmdbtype.Uint32(uint32(asnNum)),
								"autonomous_system_organization": mmdbtype.String(svcDef.Org),
							})
						}
					} else {
						// Fallback: write with org name only, ASN = 0
						fmt.Printf("  [WARN] No ASN mapping for service '%s', writing org-only entry\n", svcName)
						writerASN.Insert(network, mmdbtype.Map{
							"autonomous_system_number":       mmdbtype.Uint32(0),
							"autonomous_system_organization": mmdbtype.String(tag),
						})
					}
				}
			}
		}
	}

	// Write files
	outIP, err := os.Create(filepath.Join(outDir, "geoip.mmdb"))
	if err == nil {
		defer outIP.Close()
		writerIP.WriteTo(outIP)
	}

	outASN, err := os.Create(filepath.Join(outDir, "geoasn.mmdb"))
	if err == nil {
		defer outASN.Close()
		writerASN.WriteTo(outASN)
	}

	fmt.Printf("  [MMDB] Created geoip.mmdb, geoasn.mmdb\n")
	return nil
}


func compileXrayDAT(allRules map[string]map[string]Rules, outDir string) error {
	geositeList := &router.GeoSiteList{}
	geoipList := &router.GeoIPList{}

	for category, rulesMap := range allRules {
		for name, rules := range rulesMap {
			// Xray tag: geosite:apple or geoip:cn
			tag := strings.ToLower(name)

			// 1. Geosite (Domains)
			if category != "ip" && category != "asn" {
				site := &router.GeoSite{
					CountryCode: strings.ToUpper(tag),
				}
				for _, d := range rules.DomainSuffix {
					site.Domain = append(site.Domain, &router.Domain{Type: router.Domain_Domain, Value: d})
				}
				for _, d := range rules.Domain {
					site.Domain = append(site.Domain, &router.Domain{Type: router.Domain_Full, Value: d})
				}
				for _, d := range rules.DomainKeyword {
					site.Domain = append(site.Domain, &router.Domain{Type: router.Domain_Plain, Value: d})
				}
				for _, d := range rules.DomainRegex {
					site.Domain = append(site.Domain, &router.Domain{Type: router.Domain_Regex, Value: d})
				}

				if len(site.Domain) > 0 {
					geositeList.Entry = append(geositeList.Entry, site)
				}
			}

			// 2. GeoIP (IP CIDRs)
			if (category == "ip" || category == "asn") && len(rules.IPCIDR) > 0 {
				geoIP := &router.GeoIP{
					CountryCode: strings.ToUpper(tag),
				}
				for _, cidr := range rules.IPCIDR {
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

					geoIP.Cidr = append(geoIP.Cidr, &router.CIDR{
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
