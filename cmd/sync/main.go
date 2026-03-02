package main

import (
	"bufio"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"strings"
)

type UpstreamSource struct {
	Name     string `json:"name"`
	URL      string `json:"url"`
	Category string `json:"category"` // proxy, direct, reject
	Target   string `json:"target"`   // The filename in source/upstream/ (e.g., apple, google)
	Enabled  bool   `json:"enabled"`
}

func main() {
	root := findRoot()
	configPath := filepath.Join(root, "source", "sources.json")
	
	configData, err := os.ReadFile(configPath)
	if err != nil {
		fmt.Printf("  [ERROR] Failed to read %s: %v\n", configPath, err)
		os.Exit(1)
	}

	var sources []UpstreamSource
	if err := json.Unmarshal(configData, &sources); err != nil {
		fmt.Printf("  [ERROR] Failed to parse %s: %v\n", configPath, err)
		os.Exit(1)
	}

	upstreamDir := filepath.Join(root, "source", "upstream")

	fmt.Println("============================================================")
	fmt.Println("  Gins-Rules Upstream Syncer (Go)")
	fmt.Println("============================================================")

	// Map to store merged rules by Category -> TargetName
	mergedResults := make(map[string]map[string][]string)

	for _, src := range sources {
		if !src.Enabled {
			fmt.Printf("  Skipping %s (disabled)\n", src.Name)
			continue
		}
		fmt.Printf("  Fetching %s (targeting %s)...\n", src.URL, src.Target)

		content, err := fetchURL(src.URL)
		if err != nil {
			fmt.Printf("  [ERROR] Failed to fetch %s: %v\n", src.URL, err)
			continue
		}

		rules := processRules(content)
		
		if mergedResults[src.Category] == nil {
			mergedResults[src.Category] = make(map[string][]string)
		}
		mergedResults[src.Category][src.Target] = append(mergedResults[src.Category][src.Target], rules...)
	}

	for cat, targets := range mergedResults {
		outDir := filepath.Join(upstreamDir, cat)
		os.MkdirAll(outDir, 0755)

		for name, rules := range targets {
			outPath := filepath.Join(outDir, name+".txt")
			err := os.WriteFile(outPath, []byte(strings.Join(rules, "\n")+"\n"), 0644)
			if err != nil {
				fmt.Printf("  [ERROR] Failed to write %s: %v\n", outPath, err)
				continue
			}
			fmt.Printf("  [SUCCESS] Written %d rules to %s/%s.txt\n", len(rules), cat, name)
		}
	}

	fmt.Println("\n  Sync complete!")
	fmt.Println("============================================================")
}

func fetchURL(url string) (string, error) {
	resp, err := http.Get(url)
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return "", fmt.Errorf("bad status: %s", resp.Status)
	}

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return "", err
	}
	return string(body), nil
}

func processRules(content string) []string {
	var rules []string
	scanner := bufio.NewScanner(strings.NewReader(content))
	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())
		if line == "" || strings.HasPrefix(line, "#") || strings.HasPrefix(line, ";") || strings.HasPrefix(line, "//") {
			continue
		}
		parts := strings.Split(line, ",")
		if len(parts) >= 2 {
			ruleType := strings.ToUpper(strings.TrimSpace(parts[0]))
			val := strings.TrimSpace(parts[1])
			switch ruleType {
			case "DOMAIN-SUFFIX", "HOST-SUFFIX":
				rules = append(rules, val)
			case "DOMAIN", "HOST":
				rules = append(rules, "full:"+val)
			case "DOMAIN-KEYWORD", "HOST-KEYWORD":
				rules = append(rules, "keyword:"+val)
			case "IP-CIDR", "IP-CIDR6":
				rules = append(rules, val)
				if !strings.Contains(line, ",") {
					rules = append(rules, line)
				}
			}
		} else if !strings.Contains(line, ",") {
			rules = append(rules, line)
		}
	}
	return rules
}

func findRoot() string {
	wd, _ := os.Getwd()
	for d := wd; d != "/"; d = filepath.Dir(d) {
		if _, err := os.Stat(filepath.Join(d, "go.mod")); err == nil {
			return d
		}
	}
	return "."
}
