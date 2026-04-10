package main

import (
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"sync"
)

type IconSource struct {
	Name    string `json:"name"`
	URL     string `json:"url"`
	Theme   string `json:"theme"`
	Enabled bool   `json:"enabled"`
}

type NormalizedIcon struct {
	Name   string `json:"name"`
	URL    string `json:"url"`
	Source string `json:"source"`
	Theme  string `json:"theme"`
}

type RawIcon struct {
	Name string `json:"name"`
	URL  string `json:"url"`
	Tag  string `json:"tag"` // Some sources use tag instead of name
}

func main() {
	root := findRoot()
	configPath := filepath.Join(root, "source", "icons.json")
	outPath := filepath.Join(root, "compiled", "Gins-Icons.json")
	hashPath := filepath.Join(root, "source", "icons-hash.json")
	dashboardPath := filepath.Join(root, "dashboard", "public", "icons-catalog.json")

	os.MkdirAll(filepath.Dir(outPath), 0755)
	os.MkdirAll(filepath.Dir(dashboardPath), 0755)

	configData, err := os.ReadFile(configPath)
	if err != nil {
		fmt.Printf("[ERROR] Failed to read %s: %v\n", configPath, err)
		os.Exit(1)
	}

	var sources []IconSource
	if err := json.Unmarshal(configData, &sources); err != nil {
		fmt.Printf("[ERROR] Failed to parse %s: %v\n", configPath, err)
		os.Exit(1)
	}

	var allIcons []NormalizedIcon
	var mu sync.Mutex
	var wg sync.WaitGroup

	fmt.Println("============================================================")
	fmt.Println("  Gins-Icons Advanced Aggregator (30+ Sources)")
	fmt.Println("============================================================")

	for _, src := range sources {
		if !src.Enabled {
			continue
		}
		wg.Add(1)
		go func(s IconSource) {
			defer wg.Done()
			fmt.Printf(" [FETCH] %s...\n", s.Name)
			icons, err := fetchAndNormalize(s)
			if err != nil {
				fmt.Printf(" [ERROR] %s: %v\n", s.Name, err)
				return
			}
			mu.Lock()
			allIcons = append(allIcons, icons...)
			mu.Unlock()
			fmt.Printf(" [SUCCESS] Got %d icons from %s\n", len(icons), s.Name)
		}(src)
	}

	wg.Wait()

	// Sort by Source first, then by Name
	sort.Slice(allIcons, func(i, j int) bool {
		if allIcons[i].Source != allIcons[j].Source {
			return allIcons[i].Source < allIcons[j].Source
		}
		return strings.ToLower(allIcons[i].Name) < strings.ToLower(allIcons[j].Name)
	})

	finalData, _ := json.MarshalIndent(allIcons, "", "  ")

	// 1. Save distribution & UI catalog
	os.WriteFile(outPath, finalData, 0644)
	os.WriteFile(dashboardPath, finalData, 0644)

	// 2. Generate Change Fingerprint (SHA256)
	hash := sha256.Sum256(finalData)
	hashStr := hex.EncodeToString(hash[:])
	
	hashJSON, _ := json.Marshal(map[string]string{
		"sha256":    hashStr,
		"total":     fmt.Sprintf("%d", len(allIcons)),
		"timestamp": fmt.Sprintf("%v", os.Getenv("GITHUB_RUN_ID")), // Hook into CI if available
	})
	os.WriteFile(hashPath, hashJSON, 0644)

	fmt.Printf("\n [DONE] Total icons: %d | Fingerprint: %s\n", len(allIcons), hashStr[:8])
	fmt.Println("============================================================")
}

func fetchAndNormalize(src IconSource) ([]NormalizedIcon, error) {
	resp, err := http.Get(src.URL)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("status %d", resp.StatusCode)
	}

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, err
	}

	// Dynamic recursive parsing for various JSON dialects
	var rawList []RawIcon
	rawList = extractIcons(body)

	var results []NormalizedIcon
	seen := make(map[string]bool)

	for _, icon := range rawList {
		if icon.URL == "" {
			continue
		}
		name := icon.Name
		if name == "" {
			name = icon.Tag
		}
		if name == "" {
			// Extract name from URL if missing
			parts := strings.Split(icon.URL, "/")
			name = strings.TrimSuffix(parts[len(parts)-1], ".png")
		}

		// Cleanup common name noise
		name = strings.ReplaceAll(name, "_", " ")
		name = strings.ReplaceAll(name, "-", " ")

		id := icon.URL
		if seen[id] {
			continue
		}
		seen[id] = true

		results = append(results, NormalizedIcon{
			Name:   name,
			URL:    icon.URL,
			Source: src.Name,
			Theme:  src.Theme,
		})
	}

	return results, nil
}

// extractIcons attempts to find icon arrays in various nested structures
func extractIcons(data []byte) []RawIcon {
	var result []RawIcon

	// 1. Try if it's a direct array
	if err := json.Unmarshal(data, &result); err == nil && len(result) > 0 {
		if result[0].URL != "" { return result }
	}

	// 2. Try generic map search
	var m map[string]interface{}
	if err := json.Unmarshal(data, &m); err == nil {
		// Common keys in icon JSONs
		keys := []string{"icons", "items", "iconList", "list", "tubiao"}
		for _, k := range keys {
			if val, ok := m[k]; ok {
				raw, _ := json.Marshal(val)
				var list []RawIcon
				if err := json.Unmarshal(raw, &list); err == nil {
					return list
				}
			}
		}
	}
	
	return nil
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
