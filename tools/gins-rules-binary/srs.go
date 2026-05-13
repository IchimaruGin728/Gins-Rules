package main

import (
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
)

// generateAllSRS generates SRS files for all categories using sing-box CLI
func generateAllSRS(categories map[string]map[string]RuleSet, output string, singboxBin string) (int, int) {
	success := 0
	total := 0
	for cat, targets := range categories {
		for name, rs := range targets {
			if ruleSetIsEmpty(rs) {
				continue
			}
			total++
			outDir := filepath.Join(output, "singbox", cat)
			if err := ensureDir(outDir); err != nil {
				fmt.Fprintf(os.Stderr, "  ❌ mkdir %s: %v\n", outDir, err)
				continue
			}

			// Generate SRS using sing-box CLI
			outPath := filepath.Join(outDir, name+".srs")
			if err := generateSrs(singboxBin, rs, outPath); err != nil {
				fmt.Fprintf(os.Stderr, "  ❌ SRS %s/%s: %v\n", cat, name, err)
			} else {
				success++
			}
		}
	}
	return success, total
}

// generateSrs writes a sing-box JSON source file and calls sing-box rule-set compile
func generateSrs(singboxBin string, rs RuleSet, outPath string) error {
	// Build sing-box source JSON
	rule := singboxJSON(rs)
	if len(rule) == 0 {
		return nil
	}

	source := map[string]interface{}{
		"version": 3,
		"rules":   []interface{}{rule},
	}

	jsonData, err := json.Marshal(source)
	if err != nil {
		return fmt.Errorf("marshal JSON: %w", err)
	}

	// Write temporary JSON file
	tmpFile, err := os.CreateTemp("", "gins-srs-*.json")
	if err != nil {
		return fmt.Errorf("create temp file: %w", err)
	}
	defer os.Remove(tmpFile.Name())

	if _, err := tmpFile.Write(jsonData); err != nil {
		tmpFile.Close()
		return fmt.Errorf("write temp file: %w", err)
	}
	tmpFile.Close()

	// Call sing-box rule-set compile
	cmd := exec.Command(singboxBin, "rule-set", "compile", "--output", outPath, tmpFile.Name())
	cmd.Stderr = os.Stderr
	if err := cmd.Run(); err != nil {
		return fmt.Errorf("sing-box rule-set compile: %w", err)
	}

	return nil
}
