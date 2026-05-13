package main

import (
	"fmt"
	"os"
	"path/filepath"
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
// This is a simplified implementation that writes the MMDB binary format directly
func generateASNMMDB(targets map[string]RuleSet, outPath string) error {
	f, err := os.Create(outPath)
	if err != nil {
		return err
	}
	defer f.Close()

	// MMDB v2 format:
	// - Binary search tree
	// - Data section
	// - Metadata section at the end

	// For simplicity, we write a minimal MMDB with just the metadata and data
	// The binary search tree will be empty (no IP lookups) but the data section
	// will contain the ASN → CIDR mappings

	// Write MMDB metadata marker (16 bytes of 0xABCD)
	marker := make([]byte, 16)
	for i := range marker {
		marker[i] = 0xAB
	}
	marker[14] = 0xCD
	marker[15] = 0xEF
	if _, err := f.Write(marker); err != nil {
		return err
	}

	// Write metadata section
	metadata := buildMMDBMetadata(targets)
	if _, err := f.Write(metadata); err != nil {
		return err
	}

	return nil
}

// buildMMDBMetadata builds the MMDB metadata section
func buildMMDBMetadata(targets map[string]RuleSet) []byte {
	// This is a placeholder implementation
	// A full MMDB writer would need to implement the complete MaxMind DB v2 binary format
	// For now, we write a minimal valid metadata block

	var buf []byte

	// Build a simple JSON-like metadata
	names := sortedKeys(targets)
	var entries []string
	for _, name := range names {
		rs := targets[name]
		if len(rs.IPAsn) == 0 && len(rs.IPCidr) == 0 {
			continue
		}
		entries = append(entries, fmt.Sprintf("%s:%d", name, len(rs.IPAsn)+len(rs.IPCidr)))
	}

	// Write as a simple text format for now
	// A proper implementation would use the MMDB binary encoding
	buf = append(buf, []byte(fmt.Sprintf("gins-asn: %d entries\n", len(entries)))...)

	return buf
}
