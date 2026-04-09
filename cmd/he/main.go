package main

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"time"
)

// ─── Config ────────────────────────────────────────────────────────────────

type ASNMap struct {
	Services map[string]ServiceDef `json:"services"`
}

type ServiceDef struct {
	ASNs []int  `json:"asns"`
	Org  string `json:"org"`
}

// ─── RIPE Stat API response ─────────────────────────────────────────────────

type RIPEResponse struct {
	Data struct {
		Prefixes []struct {
			Prefix  string `json:"prefix"`
			Timings struct {
				StartTime string `json:"starttime"`
			} `json:"timings"`
		} `json:"prefixes"`
	} `json:"data"`
	Status string `json:"status"`
}

// ─── Official CIDR sources (Plan B) ─────────────────────────────────────────

type OfficialSource struct {
	Service  string
	URL      string
	ParseFn  func(body string) []string
}

var officialSources = []OfficialSource{
	{
		Service: "telegram",
		URL:     "https://core.telegram.org/resources/cidr.txt",
		ParseFn: parsePlainCIDR,
	},
	{
		Service: "cloudflare",
		URL:     "https://www.cloudflare.com/ips-v4",
		ParseFn: parsePlainCIDR,
	},
	{
		Service: "cloudflare",
		URL:     "https://www.cloudflare.com/ips-v6",
		ParseFn: parsePlainCIDR,
	},
	{
		Service: "google",
		URL:     "https://www.gstatic.com/ipranges/goog.json",
		ParseFn: parseGoogleJSON,
	},
	{
		Service: "amazon",
		URL:     "https://ip-ranges.amazonaws.com/ip-ranges.json",
		ParseFn: parseAWSJSON,
	},
	{
		Service: "github",
		URL:     "https://api.github.com/meta",
		ParseFn: parseGitHubJSON,
	},
}

// ─── Main ───────────────────────────────────────────────────────────────────

func main() {
	root := findRoot()
	asnMapPath := filepath.Join(root, "source", "asn-map.json")
	outDir := filepath.Join(root, "source", "upstream", "ip")
	os.MkdirAll(outDir, 0o755)

	fmt.Println("============================================================")
	fmt.Println("  Gins-Rules HE/RIPE BGP + Official CIDR Syncer")
	fmt.Println("============================================================")

	// Load ASN map
	asnMapData, err := os.ReadFile(asnMapPath)
	if err != nil {
		fmt.Printf("  [ERROR] Cannot read asn-map.json: %v\n", err)
		os.Exit(1)
	}
	var asnMap ASNMap
	if err := json.Unmarshal(asnMapData, &asnMap); err != nil {
		fmt.Printf("  [ERROR] Cannot parse asn-map.json: %v\n", err)
		os.Exit(1)
	}

	// Collect CIDRs per service
	results := make(map[string]map[string]bool)
	for svc := range asnMap.Services {
		results[svc] = make(map[string]bool)
	}

	// ── Plan A: RIPE Stat BGP prefix lookup per ASN ────────────────────────
	fmt.Println("\n  [Plan A] Fetching RIPE Stat BGP prefixes...")
	for svcName, svcDef := range asnMap.Services {
		for _, asn := range svcDef.ASNs {
			prefixes, err := fetchRIPEPrefixes(asn)
			if err != nil {
				fmt.Printf("  [WARN] RIPE AS%d (%s): %v\n", asn, svcName, err)
				continue
			}
			for _, p := range prefixes {
				results[svcName][p] = true
			}
			fmt.Printf("  [RIPE] AS%d → %s: +%d prefixes\n", asn, svcName, len(prefixes))
			time.Sleep(200 * time.Millisecond) // be polite to RIPE API
		}
	}

	// ── Plan B: Official CIDR lists ────────────────────────────────────────
	fmt.Println("\n  [Plan B] Fetching official CIDR sources...")
	for _, src := range officialSources {
		body, err := fetchURL(src.URL)
		if err != nil {
			fmt.Printf("  [WARN] Official %s (%s): %v\n", src.Service, src.URL, err)
			continue
		}
		prefixes := src.ParseFn(body)
		for _, p := range prefixes {
			if results[src.Service] == nil {
				results[src.Service] = make(map[string]bool)
			}
			results[src.Service][p] = true
		}
		fmt.Printf("  [Official] %s (%s): +%d prefixes\n", src.Service, src.URL, len(prefixes))
	}

	// ── Write upstream files ───────────────────────────────────────────────
	fmt.Println("\n  [Write] Saving upstream asn-*.txt files...")
	for svcName, cidrSet := range results {
		if len(cidrSet) == 0 {
			fmt.Printf("  [SKIP] asn-%s: no prefixes found\n", svcName)
			continue
		}

		var sorted []string
		for cidr := range cidrSet {
			sorted = append(sorted, cidr)
		}
		sort.Strings(sorted)

		outPath := filepath.Join(outDir, "asn-"+svcName+".txt")
		content := strings.Join(sorted, "\n") + "\n"
		if err := os.WriteFile(outPath, []byte(content), 0o644); err != nil {
			fmt.Printf("  [ERROR] Write asn-%s.txt: %v\n", svcName, err)
			continue
		}
		fmt.Printf("  [OK] asn-%s.txt → %d CIDRs\n", svcName, len(sorted))
	}

	fmt.Println("\n============================================================")
	fmt.Println("  HE/RIPE sync complete!")
	fmt.Println("============================================================")
}

// ─── RIPE Stat fetcher ───────────────────────────────────────────────────────

func fetchRIPEPrefixes(asn int) ([]string, error) {
	url := fmt.Sprintf("https://stat.ripe.net/data/announced-prefixes/data.json?resource=AS%d", asn)
	body, err := fetchURL(url)
	if err != nil {
		return nil, err
	}

	var resp RIPEResponse
	if err := json.Unmarshal([]byte(body), &resp); err != nil {
		return nil, fmt.Errorf("JSON parse: %w", err)
	}
	if resp.Status != "ok" {
		return nil, fmt.Errorf("RIPE status: %s", resp.Status)
	}

	var prefixes []string
	for _, p := range resp.Data.Prefixes {
		if p.Prefix != "" {
			prefixes = append(prefixes, p.Prefix)
		}
	}
	return prefixes, nil
}

// ─── Official source parsers ─────────────────────────────────────────────────

func parsePlainCIDR(body string) []string {
	var out []string
	for _, line := range strings.Split(body, "\n") {
		line = strings.TrimSpace(line)
		if line == "" || strings.HasPrefix(line, "#") {
			continue
		}
		if strings.Contains(line, "/") {
			out = append(out, line)
		}
	}
	return out
}

func parseGoogleJSON(body string) []string {
	var data struct {
		Prefixes []struct {
			IPv4Prefix string `json:"ipv4Prefix"`
			IPv6Prefix string `json:"ipv6Prefix"`
		} `json:"prefixes"`
	}
	if err := json.Unmarshal([]byte(body), &data); err != nil {
		return nil
	}
	var out []string
	for _, p := range data.Prefixes {
		if p.IPv4Prefix != "" {
			out = append(out, p.IPv4Prefix)
		}
		if p.IPv6Prefix != "" {
			out = append(out, p.IPv6Prefix)
		}
	}
	return out
}

func parseAWSJSON(body string) []string {
	var data struct {
		Prefixes []struct {
			IPPrefix string `json:"ip_prefix"`
		} `json:"prefixes"`
		IPv6Prefixes []struct {
			IPv6Prefix string `json:"ipv6_prefix"`
		} `json:"ipv6_prefixes"`
	}
	if err := json.Unmarshal([]byte(body), &data); err != nil {
		return nil
	}
	var out []string
	for _, p := range data.Prefixes {
		if p.IPPrefix != "" {
			out = append(out, p.IPPrefix)
		}
	}
	for _, p := range data.IPv6Prefixes {
		if p.IPv6Prefix != "" {
			out = append(out, p.IPv6Prefix)
		}
	}
	return out
}

func parseGitHubJSON(body string) []string {
	var data struct {
		Web        []string `json:"web"`
		API        []string `json:"api"`
		Git        []string `json:"git"`
		Packages   []string `json:"packages"`
		Actions    []string `json:"actions"`
		Dependabot []string `json:"dependabot"`
	}
	if err := json.Unmarshal([]byte(body), &data); err != nil {
		return nil
	}
	seen := make(map[string]bool)
	var out []string
	for _, list := range [][]string{data.Web, data.API, data.Git, data.Packages, data.Actions, data.Dependabot} {
		for _, cidr := range list {
			if !seen[cidr] {
				seen[cidr] = true
				out = append(out, cidr)
			}
		}
	}
	return out
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

func fetchURL(url string) (string, error) {
	client := &http.Client{Timeout: 30 * time.Second}
	req, err := http.NewRequest("GET", url, nil)
	if err != nil {
		return "", err
	}
	req.Header.Set("User-Agent", "Gins-Rules/1.0 (https://github.com/IchimaruGin/Gins-Rules)")
	resp, err := client.Do(req)
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		return "", fmt.Errorf("HTTP %d", resp.StatusCode)
	}
	body, err := io.ReadAll(resp.Body)
	return string(body), err
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
