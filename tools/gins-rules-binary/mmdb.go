package main

import (
	"fmt"
	"net"
	"os"
	"path/filepath"
	"sort"
	"strings"

	"github.com/maxmind/mmdbwriter"
	"github.com/maxmind/mmdbwriter/mmdbtype"
)

// generateAllMMDB generates MMDB files from ASN data
func generateAllMMDB(categories map[string]map[string]RuleSet, output string) int {
	count := 0
	outDir := filepath.Join(output, "mmdb")
	if err := ensureDir(outDir); err != nil {
		fmt.Fprintf(os.Stderr, "  ❌ mkdir %s: %v\n", outDir, err)
		return 0
	}

	// Generate gins-asn.mmdb from ASN sources
	if asnTargets, ok := categories["asn"]; ok {
		outPath := filepath.Join(outDir, "gins-asn.mmdb")
		if err := generateASNMMDB(asnTargets, outPath); err != nil {
			fmt.Fprintf(os.Stderr, "  ❌ gins-asn.mmdb: %v\n", err)
		} else {
			count++
		}
	}

	return count
}

// generateASNMMDB generates a MaxMind DB v2 format file from ASN data
func generateASNMMDB(targets map[string]RuleSet, outPath string) error {
	// Create a new MMDB writer
	writer, err := mmdbwriter.New(mmdbwriter.Options{
		DatabaseType: "Gins-ASN",
		Description: map[string]string{
			"en": "Gins-Rules ASN Database",
		},
		RecordSize: 24,
	})
	if err != nil {
		return fmt.Errorf("create MMDB writer: %w", err)
	}

	// Process each ASN target
	names := sortedKeys(targets)
	for _, name := range names {
		rs := targets[name]
		if len(rs.IPAsn) == 0 && len(rs.IPCidr) == 0 {
			continue
		}

		// Extract ASN number from name (e.g., "asn-google" -> "AS15169")
		asn := extractASN(name, rs)
		if asn == "" {
			continue
		}

		// Get all CIDRs for this ASN
		cidrs := make([]string, 0, len(rs.IPCidr))
		cidrs = append(cidrs, rs.IPCidr...)
		sort.Strings(cidrs)

		// Insert each CIDR into the MMDB
		for _, cidr := range cidrs {
			_, ipNet, err := net.ParseCIDR(cidr)
			if err != nil {
				fmt.Fprintf(os.Stderr, "  ⚠️  Invalid CIDR %s: %v\n", cidr, err)
				continue
			}

			// Create the data record
			record := mmdbtype.Map{
				"autonomous_system_number":       mmdbtype.String(asn),
				"autonomous_system_organization": mmdbtype.String(name),
			}

			// Insert into MMDB
			if err := writer.Insert(ipNet, record); err != nil {
				fmt.Fprintf(os.Stderr, "  ⚠️  Insert %s: %v\n", cidr, err)
			}
		}
	}

	// Write the MMDB file
	f, err := os.Create(outPath)
	if err != nil {
		return fmt.Errorf("create file: %w", err)
	}
	defer f.Close()

	if _, err := writer.WriteTo(f); err != nil {
		return fmt.Errorf("write MMDB: %w", err)
	}

	return nil
}

// extractASN extracts the ASN number from the target name and rules
func extractASN(name string, rs RuleSet) string {
	// First check if there are explicit ASN entries in the rules
	if len(rs.IPAsn) > 0 {
		// Use the first ASN entry
		asn := rs.IPAsn[0]
		if !strings.HasPrefix(asn, "AS") {
			asn = "AS" + asn
		}
		return asn
	}

	// Try to extract from the name (e.g., "asn-google" -> look for ASN in the name)
	// This is a fallback - ideally ASN should be in the rules
	return ""
}
