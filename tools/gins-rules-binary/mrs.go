package main

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
)

// generateAllMRS generates MRS files for all categories using mihomo CLI
func generateAllMRS(categories map[string]map[string]RuleSet, output string, mihomoBin string) (int, int) {
	success := 0
	total := 0
	for cat, targets := range categories {
		for name, rs := range targets {
			if ruleSetIsEmpty(rs) {
				continue
			}
			outDir := filepath.Join(output, "mihomo", cat)
			if err := ensureDir(outDir); err != nil {
				fmt.Fprintf(os.Stderr, "  ❌ mkdir %s: %v\n", outDir, err)
				continue
			}

			hasComplex := len(rs.DomainKeyword) > 0 || len(rs.DomainRegex) > 0 ||
				len(rs.DomainWildcard) > 0 || len(rs.IPCidr) > 0 ||
				len(rs.IPAsn) > 0 || len(rs.ProcessName) > 0 || len(rs.UserAgent) > 0

			if hasComplex {
				// Domain MRS
				domainLines := domainPayload(rs)
				if len(domainLines) > 0 {
					total++
					if err := generateMrs(mihomoBin, "domain", domainLines, filepath.Join(outDir, name+".mrs")); err != nil {
						fmt.Fprintf(os.Stderr, "  ❌ MRS domain %s/%s: %v\n", cat, name, err)
					} else {
						success++
					}
				}

				// IP MRS
				ipLines := ipcidrPayload(rs)
				if len(ipLines) > 0 {
					total++
					if err := generateMrs(mihomoBin, "ipcidr", ipLines, filepath.Join(outDir, name+"-ip.mrs")); err != nil {
						fmt.Fprintf(os.Stderr, "  ❌ MRS ipcidr %s/%s: %v\n", cat, name, err)
					} else {
						success++
					}
				}
			} else {
				// Simple domain-only: generate domain MRS
				domainLines := domainPayload(rs)
				if len(domainLines) > 0 {
					total++
					if err := generateMrs(mihomoBin, "domain", domainLines, filepath.Join(outDir, name+".mrs")); err != nil {
						fmt.Fprintf(os.Stderr, "  ❌ MRS domain %s/%s: %v\n", cat, name, err)
					} else {
						success++
					}
				}
			}
		}
	}
	return success, total
}

// generateMrs writes a temporary text file and calls mihomo convert-ruleset
func generateMrs(mihomoBin, behavior string, lines []string, outPath string) error {
	// Write temporary input file
	tmpFile, err := os.CreateTemp("", "gins-mrs-*.txt")
	if err != nil {
		return fmt.Errorf("create temp file: %w", err)
	}
	defer os.Remove(tmpFile.Name())

	if err := writeLines(tmpFile.Name(), lines); err != nil {
		tmpFile.Close()
		return fmt.Errorf("write temp file: %w", err)
	}
	tmpFile.Close()

	// Call mihomo convert-ruleset
	cmd := exec.Command(mihomoBin, "convert-ruleset", behavior, "text", tmpFile.Name(), outPath)
	cmd.Stderr = os.Stderr
	if err := cmd.Run(); err != nil {
		return fmt.Errorf("mihomo convert-ruleset: %w", err)
	}

	return nil
}
