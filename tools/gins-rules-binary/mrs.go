package main

import (
	"fmt"
	"io"
	"os"
	"os/exec"
	"path/filepath"
)

// generateAllMRS generates MRS files for all categories using mihomo CLI
func generateAllMRS(categories map[string]map[string]RuleSet, output string, mihomoBin string) (int, int) {
	success := 0
	total := 0
	for cat, targets := range categories {
		mihomoDir := filepath.Join(output, "mihomo", cat)
		stashDir := filepath.Join(output, "stash", cat)

		for _, dir := range []string{mihomoDir, stashDir} {
			if err := ensureDir(dir); err != nil {
				fmt.Fprintf(os.Stderr, "  ❌ mkdir %s: %v\n", dir, err)
				continue
			}
			// Clean up existing .mrs files
			files, _ := filepath.Glob(filepath.Join(dir, "*.mrs"))
			for _, f := range files {
				os.Remove(f)
			}
		}

		for name, rs := range targets {
			if ruleSetIsEmpty(rs) {
				continue
			}

			domainLines := domainPayload(rs)
			ipLines := ipcidrPayload(rs)

			hasDomains := len(domainLines) > 0
			hasIPs := len(ipLines) > 0

			if !hasDomains && !hasIPs {
				continue
			}

			if hasDomains && hasIPs {
				// CASE 1: Mixed content - split into domain (name.mrs) and ipcidr (name-ip.mrs)
				total++
				mrsPath := filepath.Join(mihomoDir, name+".mrs")
				if err := generateMrs(mihomoBin, "domain", domainLines, mrsPath); err != nil {
					fmt.Fprintf(os.Stderr, "  ❌ MRS domain %s/%s: %v\n", cat, name, err)
				} else {
					success++
					// Copy to stash
					if err := copyFile(mrsPath, filepath.Join(stashDir, name+".mrs")); err != nil {
						fmt.Fprintf(os.Stderr, "  ❌ Copy MRS to stash %s/%s: %v\n", cat, name, err)
					}
				}

				total++
				ipMrsPath := filepath.Join(mihomoDir, name+"-ip.mrs")
				if err := generateMrs(mihomoBin, "ipcidr", ipLines, ipMrsPath); err != nil {
					fmt.Fprintf(os.Stderr, "  ❌ MRS ipcidr %s/%s: %v\n", cat, name, err)
				} else {
					success++
					// Copy to stash
					if err := copyFile(ipMrsPath, filepath.Join(stashDir, name+"-ip.mrs")); err != nil {
						fmt.Fprintf(os.Stderr, "  ❌ Copy MRS-ip to stash %s/%s: %v\n", cat, name, err)
					}
				}
			} else if hasDomains {
				// CASE 2: Domain only - single domain ruleset (name.mrs)
				total++
				mrsPath := filepath.Join(mihomoDir, name+".mrs")
				if err := generateMrs(mihomoBin, "domain", domainLines, mrsPath); err != nil {
					fmt.Fprintf(os.Stderr, "  ❌ MRS domain %s/%s: %v\n", cat, name, err)
				} else {
					success++
					// Copy to stash
					if err := copyFile(mrsPath, filepath.Join(stashDir, name+".mrs")); err != nil {
						fmt.Fprintf(os.Stderr, "  ❌ Copy MRS to stash %s/%s: %v\n", cat, name, err)
					}
				}
			} else if hasIPs {
				// CASE 3: IP only - single ipcidr ruleset (name.mrs)
				total++
				mrsPath := filepath.Join(mihomoDir, name+".mrs")
				if err := generateMrs(mihomoBin, "ipcidr", ipLines, mrsPath); err != nil {
					fmt.Fprintf(os.Stderr, "  ❌ MRS ipcidr %s/%s: %v\n", cat, name, err)
				} else {
					success++
					// Copy to stash
					if err := copyFile(mrsPath, filepath.Join(stashDir, name+".mrs")); err != nil {
						fmt.Fprintf(os.Stderr, "  ❌ Copy MRS to stash %s/%s: %v\n", cat, name, err)
					}
				}

				// If not in IP/ASN category, also generate name-ip.mrs for backward compatibility
				if cat != "ip" && cat != "asn" {
					total++
					ipMrsPath := filepath.Join(mihomoDir, name+"-ip.mrs")
					if err := generateMrs(mihomoBin, "ipcidr", ipLines, ipMrsPath); err != nil {
						fmt.Fprintf(os.Stderr, "  ❌ MRS ipcidr fallback %s/%s: %v\n", cat, name, err)
					} else {
						success++
						// Copy to stash
						if err := copyFile(ipMrsPath, filepath.Join(stashDir, name+"-ip.mrs")); err != nil {
							fmt.Fprintf(os.Stderr, "  ❌ Copy MRS-ip fallback to stash %s/%s: %v\n", cat, name, err)
						}
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

// copyFile copies a file from src to dst
func copyFile(src, dst string) error {
	in, err := os.Open(src)
	if err != nil {
		return err
	}
	defer in.Close()

	out, err := os.Create(dst)
	if err != nil {
		return err
	}
	defer out.Close()

	_, err = io.Copy(out, in)
	if err != nil {
		return err
	}
	return out.Sync()
}
