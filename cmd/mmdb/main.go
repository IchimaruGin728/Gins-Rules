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

	"github.com/oschwald/maxminddb-golang"
)

// MMDB sources from xream/geoip — two providers for multi-source merge
var mmdbSources = []struct {
	Name string
	URL  string
	Type string // "country" or "asn"
}{
	// Country MMDB (CN + Private only — smaller, faster)
	{
		Name: "ipinfo.country-only-cn-private",
		URL:  "https://github.com/xream/geoip/releases/latest/download/ipinfo.country-only-cn-private.mmdb",
		Type: "country",
	},
	{
		Name: "ip2location.country-only-cn-private",
		URL:  "https://github.com/xream/geoip/releases/latest/download/ip2location.country-only-cn-private.mmdb",
		Type: "country",
	},
	// ASN MMDB
	{
		Name: "ipinfo.asn",
		URL:  "https://github.com/xream/geoip/releases/latest/download/ipinfo.asn.mmdb",
		Type: "asn",
	},
	{
		Name: "ip2location.asn",
		URL:  "https://github.com/xream/geoip/releases/latest/download/ip2location.asn.mmdb",
		Type: "asn",
	},
}

// ASN targets: ASN number(s) -> output filename
var asnTargets = []struct {
	Name string
	ASNs []uint // autonomous_system_number
}{
	{"asn-cloudflare", []uint{13335}},
	{"asn-google", []uint{15169, 396982}},
	{"asn-microsoft", []uint{8075}},
	{"asn-amazon", []uint{16509, 14618}},
	{"asn-facebook", []uint{32934}},
	{"asn-telegram", []uint{62041, 62014, 59930, 44907}},
	{"asn-netflix", []uint{2906}},
	{"asn-github", []uint{36459}},
	{"asn-twitter", []uint{13414}},
	{"asn-apple", []uint{714, 6185}},
	{"asn-discord", []uint{49544}},
	{"asn-spotify", []uint{8403}},
	{"asn-steam", []uint{32590}},
}

// Country MMDB record structure
type CountryRecord struct {
	Country struct {
		ISOCode string `maxminddb:"iso_code"`
	} `maxminddb:"country"`
}

// ASN MMDB record structure (both ipinfo and ip2location use this format)
type ASNRecord struct {
	AutonomousSystemNumber       uint   `maxminddb:"autonomous_system_number"`
	AutonomousSystemOrganization string `maxminddb:"autonomous_system_organization"`
}

func main() {
	root := findRoot()
	ipDir := filepath.Join(root, "source", "ip")
	tmpDir := filepath.Join(root, ".mmdb-cache")

	os.MkdirAll(ipDir, 0o755)
	os.MkdirAll(tmpDir, 0o755)
	defer os.RemoveAll(tmpDir)

	fmt.Println("============================================================")
	fmt.Println("  Gins-Rules MMDB Parser (Multi-Source)")
	fmt.Println("============================================================")

	// Country CIDR collection: country_code -> set of CIDRs
	countryCIDRs := make(map[string]map[string]bool)
	// ASN CIDR collection: asn_number -> set of CIDRs
	asnCIDRs := make(map[uint]map[string]bool)

	for _, src := range mmdbSources {
		fmt.Printf("\n  Downloading %s...\n", src.Name)
		localPath := filepath.Join(tmpDir, src.Name+".mmdb")

		if err := downloadFile(src.URL, localPath); err != nil {
			fmt.Printf("  [ERROR] Failed to download %s: %v\n", src.Name, err)
			continue
		}

		db, err := maxminddb.Open(localPath)
		if err != nil {
			fmt.Printf("  [ERROR] Failed to open %s: %v\n", src.Name, err)
			continue
		}

		switch src.Type {
		case "country":
			count := parseCountryMMDB(db, countryCIDRs)
			fmt.Printf("  [OK] %s: extracted %d networks\n", src.Name, count)
		case "asn":
			count := parseASNMMDB(db, asnCIDRs, src.Name)
			fmt.Printf("  [OK] %s: extracted %d networks\n", src.Name, count)
		}

		db.Close()
	}

	// Write country files
	fmt.Println("\n  Writing country IP lists...")
	for code, cidrs := range countryCIDRs {
		filename := strings.ToLower(code) + ".txt"
		sorted := sortedKeys(cidrs)
		outPath := filepath.Join(ipDir, filename)
		os.WriteFile(outPath, []byte(strings.Join(sorted, "\n")+"\n"), 0o644)
		fmt.Printf("  [WRITE] ip/%s: %d CIDRs (merged from %d sources)\n", filename, len(sorted), countSources(countryCIDRs, code))
	}

	// Write ASN files
	fmt.Println("\n  Writing ASN IP lists...")
	for _, target := range asnTargets {
		merged := make(map[string]bool)
		for _, asn := range target.ASNs {
			if cidrs, ok := asnCIDRs[asn]; ok {
				for cidr := range cidrs {
					merged[cidr] = true
				}
			}
		}
		if len(merged) == 0 {
			continue
		}
		sorted := sortedKeys(merged)
		outPath := filepath.Join(ipDir, target.Name+".txt")
		os.WriteFile(outPath, []byte(strings.Join(sorted, "\n")+"\n"), 0o644)
		fmt.Printf("  [WRITE] ip/%s.txt: %d CIDRs\n", target.Name, len(sorted))
	}

	fmt.Println("\n============================================================")
	fmt.Println("  MMDB extraction complete!")
	fmt.Println("============================================================")
}

func parseCountryMMDB(db *maxminddb.Reader, countryCIDRs map[string]map[string]bool) int {
	count := 0
	networks := db.Networks(maxminddb.SkipAliasedNetworks)
	for networks.Next() {
		var record CountryRecord
		subnet, err := networks.Network(&record)
		if err != nil {
			continue
		}
		code := record.Country.ISOCode
		if code == "" {
			continue
		}
		if countryCIDRs[code] == nil {
			countryCIDRs[code] = make(map[string]bool)
		}
		countryCIDRs[code][subnet.String()] = true
		count++
	}
	return count
}

func parseASNMMDB(db *maxminddb.Reader, asnCIDRs map[uint]map[string]bool, sourceName string) int {
	count := 0

	networks := db.Networks(maxminddb.SkipAliasedNetworks)
	for networks.Next() {
		var record ASNRecord
		subnet, err := networks.Network(&record)
		if err != nil {
			continue
		}

		asn := record.AutonomousSystemNumber
		if asn == 0 || subnet == nil {
			continue
		}

		if asnCIDRs[asn] == nil {
			asnCIDRs[asn] = make(map[string]bool)
		}
		asnCIDRs[asn][subnet.String()] = true
		count++
	}
	return count
}

func downloadFile(url string, dest string) error {
	resp, err := http.Get(url)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return fmt.Errorf("bad status: %s", resp.Status)
	}

	out, err := os.Create(dest)
	if err != nil {
		return err
	}
	defer out.Close()

	written, err := io.Copy(out, resp.Body)
	if err != nil {
		return err
	}
	fmt.Printf("  Downloaded %s (%.1f MB)\n", filepath.Base(dest), float64(written)/1024/1024)
	return nil
}

func sortedKeys(m map[string]bool) []string {
	var keys []string
	for k := range m {
		keys = append(keys, k)
	}
	sort.Strings(keys)
	return keys
}

func countSources(_ map[string]map[string]bool, _ string) int {
	// We always merge from 2 MMDB sources (ipinfo + ip2location)
	return 2
}

func findRoot() string {
	// Try GitHub Actions workspace first
	if ws := os.Getenv("GITHUB_WORKSPACE"); ws != "" {
		return ws
	}

	wd, _ := os.Getwd()
	for d := wd; d != "/"; d = filepath.Dir(d) {
		if _, err := os.Stat(filepath.Join(d, "go.mod")); err == nil {
			return d
		}
	}
	return "."
}

// getLatestReleaseTag fetches the latest release tag from xream/geoip
// This is kept for potential future use if we need versioned URLs
func getLatestReleaseTag() (string, error) {
	resp, err := http.Get("https://api.github.com/repos/xream/geoip/releases/latest")
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()
	var release struct {
		TagName string `json:"tag_name"`
	}
	if err := json.NewDecoder(resp.Body).Decode(&release); err != nil {
		return "", err
	}
	return release.TagName, nil
}
