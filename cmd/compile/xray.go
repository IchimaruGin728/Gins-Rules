package main

import (
	"fmt"
	"net"
	"os"
	"path/filepath"
	"strings"

	"github.com/v2fly/v2ray-core/v5/app/router/routercommon"
	"google.golang.org/protobuf/proto"
)

func compileXrayDAT(allRules map[string]map[string]Rules, outDir string) error {
	geositeList := &routercommon.GeoSiteList{}
	geoipList := &routercommon.GeoIPList{}

	for category, rulesMap := range allRules {
		for name, rules := range rulesMap {
			// Xray tag: geosite:apple or geoip:cn
			tag := strings.ToLower(name)

			// 1. Geosite (Domains)
			if category != "ip" && category != "asn" {
				site := &routercommon.GeoSite{
					CountryCode: strings.ToUpper(tag),
				}
				for _, d := range rules.DomainSuffix {
					site.Domain = append(site.Domain, &routercommon.Domain{Type: routercommon.Domain_Subdomain, Value: d})
				}
				for _, d := range rules.Domain {
					site.Domain = append(site.Domain, &routercommon.Domain{Type: routercommon.Domain_Full, Value: d})
				}
				for _, d := range rules.DomainKeyword {
					site.Domain = append(site.Domain, &routercommon.Domain{Type: routercommon.Domain_Keyword, Value: d})
				}
				for _, d := range rules.DomainRegex {
					site.Domain = append(site.Domain, &routercommon.Domain{Type: routercommon.Domain_Regex, Value: d})
				}

				if len(site.Domain) > 0 {
					geositeList.Site = append(geositeList.Site, site)
				}
			}

			// 2. GeoIP (IP CIDRs)
			if (category == "ip" || category == "asn") && len(rules.IPCIDR) > 0 {
				geoIP := &routercommon.GeoIP{
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

					geoIP.Cidr = append(geoIP.Cidr, &routercommon.CIDR{
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

	fmt.Printf("\n  [Xray] geosite.dat: %d tags, geoip.dat: %d tags\n", len(geositeList.Site), len(geoipList.Entry))
	return nil
}
