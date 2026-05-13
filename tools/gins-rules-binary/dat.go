package main

import (
	"encoding/binary"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
)

// generateAllDAT generates geoip.dat, geosite.dat, geoasn.dat
func generateAllDAT(categories map[string]map[string]RuleSet, output string) int {
	count := 0
	outDir := filepath.Join(output, "xray")
	if err := ensureDir(outDir); err != nil {
		fmt.Fprintf(os.Stderr, "  ❌ mkdir %s: %v\n", outDir, err)
		return 0
	}

	// geoip.dat — from ip category
	if ipTargets, ok := categories["ip"]; ok {
		if err := generateGeoIP(ipTargets, filepath.Join(outDir, "geoip.dat")); err != nil {
			fmt.Fprintf(os.Stderr, "  ❌ geoip.dat: %v\n", err)
		} else {
			count++
		}
	}

	// geoasn.dat — from asn category
	if asnTargets, ok := categories["asn"]; ok {
		if err := generateGeoIP(asnTargets, filepath.Join(outDir, "geoasn.dat")); err != nil {
			fmt.Fprintf(os.Stderr, "  ❌ geoasn.dat: %v\n", err)
		} else {
			count++
		}
	}

	// geosite.dat — from proxy, direct, reject categories
	geositeData := make(map[string]RuleSet)
	for _, cat := range []string{"proxy", "direct", "reject"} {
		if targets, ok := categories[cat]; ok {
			for name, rs := range targets {
				geositeData[name] = rs
			}
		}
	}
	// Also add category aggregates
	for _, cat := range []string{"proxy", "direct", "reject"} {
		if targets, ok := categories[cat]; ok {
			aggregate := RuleSet{}
			for _, rs := range targets {
				aggregate.Domain = append(aggregate.Domain, rs.Domain...)
				aggregate.DomainSuffix = append(aggregate.DomainSuffix, rs.DomainSuffix...)
				aggregate.DomainKeyword = append(aggregate.DomainKeyword, rs.DomainKeyword...)
				aggregate.DomainRegex = append(aggregate.DomainRegex, rs.DomainRegex...)
			}
			if len(aggregate.Domain) > 0 || len(aggregate.DomainSuffix) > 0 {
				geositeData[cat] = aggregate
			}
		}
	}
	if len(geositeData) > 0 {
		if err := generateGeoSite(geositeData, filepath.Join(outDir, "geosite.dat")); err != nil {
			fmt.Fprintf(os.Stderr, "  ❌ geosite.dat: %v\n", err)
		} else {
			count++
		}
	}

	return count
}

// generateGeoIP generates a geoip.dat compatible file
// Format: simple binary encoding of country code → CIDR list
func generateGeoIP(targets map[string]RuleSet, outPath string) error {
	f, err := os.Create(outPath)
	if err != nil {
		return err
	}
	defer f.Close()

	// Write entries sorted by name
	names := sortedKeys(targets)
	for _, name := range names {
		rs := targets[name]
	 cidrs := rs.IPCidr
		if len(cidrs) == 0 && len(rs.IPAsn) == 0 {
			continue
		}

		// Use name as country code (uppercase)
		code := strings.ToUpper(strings.TrimPrefix(name, "asn-"))
		code = strings.TrimPrefix(code, "!")

		// Encode: [code_len:u16][code][cidr_count:u32][cidr_len:u16][cidr]...
		codeBytes := []byte(code)
		if err := binary.Write(f, binary.BigEndian, uint16(len(codeBytes))); err != nil {
			return err
		}
		if _, err := f.Write(codeBytes); err != nil {
			return err
		}

		// Write CIDRs
		sort.Strings(cidrs)
		if err := binary.Write(f, binary.BigEndian, uint32(len(cidrs))); err != nil {
			return err
		}
		for _, cidr := range cidrs {
			cidrBytes := []byte(cidr)
			if err := binary.Write(f, binary.BigEndian, uint16(len(cidrBytes))); err != nil {
				return err
			}
			if _, err := f.Write(cidrBytes); err != nil {
				return err
			}
		}

		// Write ASN entries as CIDR-like
		asns := rs.IPAsn
		sort.Strings(asns)
		for _, asn := range asns {
			asnBytes := []byte(asn)
			if err := binary.Write(f, binary.BigEndian, uint16(len(asnBytes))); err != nil {
				return err
			}
			if _, err := f.Write(asnBytes); err != nil {
				return err
			}
		}
	}

	return nil
}

// generateGeoSite generates a geosite.dat compatible file
func generateGeoSite(targets map[string]RuleSet, outPath string) error {
	f, err := os.Create(outPath)
	if err != nil {
		return err
	}
	defer f.Close()

	names := sortedKeys(targets)
	for _, name := range names {
		rs := targets[name]
		if ruleSetIsEmpty(rs) {
			continue
		}

		// Encode: [name_len:u16][name][domain_count:u32][type:u8][domain_len:u16][domain]...
		nameBytes := []byte(name)
		if err := binary.Write(f, binary.BigEndian, uint16(len(nameBytes))); err != nil {
			return err
		}
		if _, err := f.Write(nameBytes); err != nil {
			return err
		}

		// Count total domains
		total := len(rs.Domain) + len(rs.DomainSuffix) + len(rs.DomainKeyword) + len(rs.DomainRegex)
		if err := binary.Write(f, binary.BigEndian, uint32(total)); err != nil {
			return err
		}

		// Write domain entries with type tags
		// Type 0 = plain (exact), 1 = domain (suffix), 2 = keyword, 3 = regex
		writeDomainEntries(f, 1, rs.DomainSuffix)
		writeDomainEntries(f, 0, rs.Domain)
		writeDomainEntries(f, 2, rs.DomainKeyword)
		writeDomainEntries(f, 3, rs.DomainRegex)
	}

	return nil
}

func writeDomainEntries(f *os.File, domainType byte, domains []string) {
	sort.Strings(domains)
	for _, d := range domains {
		dBytes := []byte(d)
		binary.Write(f, binary.BigEndian, domainType)
		binary.Write(f, binary.BigEndian, uint16(len(dBytes)))
		f.Write(dBytes)
	}
}
