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
	"time"
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
			} else {
				fmt.Printf("  [SUCCESS] Written %d rules to %s.txt\n", len(rules), name)
			}
		}
	}

	fmt.Println("\n  Syncing QX-Resource-Parser.js...")
	parserURL := "https://raw.githubusercontent.com/KOP-XIAO/QuantumultX/master/Scripts/resource-parser.js"
	parserContent, err := fetchURL(parserURL)
	if err != nil {
		fmt.Printf("  [ERROR] Failed to fetch QX-Resource-Parser.js: %v\n", err)
	} else {
		parserContent = cleanQXParser(parserContent)
		parserPath := filepath.Join(root, "source", "QX-Resource-Parser.js")
		err = os.WriteFile(parserPath, []byte(parserContent), 0644)
		if err != nil {
			fmt.Printf("  [ERROR] Failed to write QX-Resource-Parser.js: %v\n", err)
		} else {
			fmt.Println("  [SUCCESS] QX-Resource-Parser.js updated and cleaned successfully")
		}
	}

	fmt.Println("\n  Syncing Loon-Resource-Parser.js (Sub-Store)...")
	loonParserURL := "https://github.com/sub-store-org/Sub-Store/releases/latest/download/sub-store-parser.loon.min.js"
	loonParserContent, err := fetchURL(loonParserURL)
	if err != nil {
		fmt.Printf("  [ERROR] Failed to fetch Loon-Resource-Parser.js: %v\n", err)
	} else {
		loonParserPath := filepath.Join(root, "source", "Loon-Resource-Parser.js")
		err = os.WriteFile(loonParserPath, []byte(loonParserContent), 0644)
		if err != nil {
			fmt.Printf("  [ERROR] Failed to write Loon-Resource-Parser.js: %v\n", err)
		} else {
			fmt.Println("  [SUCCESS] Loon-Resource-Parser.js updated successfully")
		}
	}

	fmt.Println("\n  Sync complete!")
	fmt.Println("============================================================")
}

func fetchURL(url string) (string, error) {
	client := &http.Client{Timeout: 30 * time.Second}
	req, err := http.NewRequest("GET", url, nil)
	if err != nil {
		return "", err
	}
	req.Header.Set("User-Agent", "Gins-Rules/1.0 (https://github.com/IchimaruGin728/Gins-Rules)")

	resp, err := client.Do(req)
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
			case "PROCESS-NAME":
				rules = append(rules, "process:"+val)
			case "USER-AGENT":
				rules = append(rules, "user-agent:"+val)
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

func cleanQXParser(content string) string {
	// Remove the original header (first /** ... */ block) and replace it with a clean one
	startIdx := strings.Index(content, "/**")
	endIdx := strings.Index(content, "*/")

	if startIdx != -1 && endIdx != -1 && endIdx > startIdx {
		header := `/** 
 * Gins-Rules QX Resource Parser
 * - Automated Proxy Rule Conversion
 */`
		// Keep everything after the first comment block
		return header + "\n" + content[endIdx+2:]
	}
	return content
}
