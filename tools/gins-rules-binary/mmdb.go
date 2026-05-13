package main

import (
	"fmt"
	"io"
	"net"
	"net/http"
	"os"
	"path/filepath"
	"strings"
	"sync"

	"github.com/maxmind/mmdbwriter"
	"github.com/maxmind/mmdbwriter/mmdbtype"
	"github.com/oschwald/maxminddb-golang"
)

// UpstreamMMDBSource represents an upstream MMDB data source
type UpstreamMMDBSource struct {
	Name string
	URL  string
	Type string // "country" or "asn"
}

// Define upstream MMDB sources
var upstreamCountrySources = []UpstreamMMDBSource{
	{Name: "ipinfo.country", URL: "https://github.com/xream/geoip/releases/latest/download/ipinfo.country.mmdb", Type: "country"},
	{Name: "ip2location.country", URL: "https://github.com/xream/geoip/releases/latest/download/ip2location.country.mmdb", Type: "country"},
	{Name: "GeoLite2-Country", URL: "https://raw.githubusercontent.com/Loyalsoldier/geoip/release/GeoLite2-Country.mmdb", Type: "country"},
	{Name: "Country-without-asn", URL: "https://raw.githubusercontent.com/Loyalsoldier/geoip/release/Country-without-asn.mmdb", Type: "country"},
}

var upstreamASNSources = []UpstreamMMDBSource{
	{Name: "GeoLite2-ASN", URL: "https://raw.githubusercontent.com/Loyalsoldier/geoip/release/GeoLite2-ASN.mmdb", Type: "asn"},
	{Name: "ipinfo.asn", URL: "https://github.com/xream/geoip/releases/latest/download/ipinfo.asn.mmdb", Type: "asn"},
	{Name: "ip2location.asn", URL: "https://github.com/xream/geoip/releases/latest/download/ip2location.asn.mmdb", Type: "asn"},
}

// generateAllMMDB generates MMDB files from ASN data and upstream sources
func generateAllMMDB(categories map[string]map[string]RuleSet, output string) int {
	count := 0
	outDir := filepath.Join(output, "mmdb")
	if err := ensureDir(outDir); err != nil {
		fmt.Fprintf(os.Stderr, "  ❌ mkdir %s: %v\n", outDir, err)
		return 0
	}

	// Create cache directory for downloaded MMDB files
	cacheDir := filepath.Join(output, ".mmdb-cache")
	if err := ensureDir(cacheDir); err != nil {
		fmt.Fprintf(os.Stderr, "  ❌ mkdir cache %s: %v\n", cacheDir, err)
		return 0
	}

	// Download upstream MMDB files
	fmt.Println("  📥 Downloading upstream MMDB files...")
	downloadUpstreamMMDBs(cacheDir)

	// Generate geoip.mmdb by merging upstream country MMDBs and our IP data
	if err := generateGeoIPMMDB(categories, cacheDir, filepath.Join(outDir, "geoip.mmdb")); err != nil {
		fmt.Fprintf(os.Stderr, "  ❌ geoip.mmdb: %v\n", err)
	} else {
		count++
	}

	// Generate geoasn.mmdb by merging upstream ASN MMDBs and our ASN data
	if err := generateGeoASNMMDB(categories, cacheDir, filepath.Join(outDir, "geoasn.mmdb")); err != nil {
		fmt.Fprintf(os.Stderr, "  ❌ geoasn.mmdb: %v\n", err)
	} else {
		count++
	}

	return count
}

// downloadUpstreamMMDBs downloads all upstream MMDB files
func downloadUpstreamMMDBs(cacheDir string) {
	var wg sync.WaitGroup
	sem := make(chan struct{}, 3) // Limit concurrent downloads

	for _, src := range append(upstreamCountrySources, upstreamASNSources...) {
		wg.Add(1)
		go func(s UpstreamMMDBSource) {
			defer wg.Done()
			sem <- struct{}{}
			defer func() { <-sem }()

			outPath := filepath.Join(cacheDir, s.Name+".mmdb")
			if _, err := os.Stat(outPath); err == nil {
				fmt.Printf("    ✓ %s (cached)\n", s.Name)
				return
			}

			fmt.Printf("    ⬇ %s...\n", s.Name)
			if err := downloadFile(s.URL, outPath); err != nil {
				fmt.Fprintf(os.Stderr, "    ❌ %s: %v\n", s.Name, err)
			}
		}(src)
	}
	wg.Wait()
}

// downloadFile downloads a file from URL to path
func downloadFile(url, path string) error {
	resp, err := http.Get(url)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return fmt.Errorf("HTTP %d", resp.StatusCode)
	}

	out, err := os.Create(path)
	if err != nil {
		return err
	}
	defer out.Close()

	_, err = io.Copy(out, resp.Body)
	return err
}

// generateGeoIPMMDB generates geoip.mmdb by merging upstream country MMDBs
func generateGeoIPMMDB(categories map[string]map[string]RuleSet, cacheDir, outPath string) error {
	writer, err := mmdbwriter.New(mmdbwriter.Options{
		DatabaseType: "Gins-GeoIP",
		Description: map[string]string{
			"en": "Gins-Rules GeoIP Database (merged from upstream)",
		},
		RecordSize: 24,
	})
	if err != nil {
		return fmt.Errorf("create writer: %w", err)
	}

	// Merge upstream country MMDBs
	for _, src := range upstreamCountrySources {
		mmdbPath := filepath.Join(cacheDir, src.Name+".mmdb")
		if err := mergeUpstreamMMDB(writer, mmdbPath, src.Name); err != nil {
			fmt.Fprintf(os.Stderr, "    ⚠️  Merge %s: %v\n", src.Name, err)
		}
	}

	// Add our local IP data (e.g., CN IPs)
	if ipTargets, ok := categories["ip"]; ok {
		for name, rs := range ipTargets {
			if len(rs.IPCidr) == 0 {
				continue
			}

			// Use uppercase country code
			code := strings.ToUpper(name)

			for _, cidr := range rs.IPCidr {
				_, ipNet, err := net.ParseCIDR(cidr)
				if err != nil {
					continue
				}

				record := mmdbtype.Map{
					"country": mmdbtype.Map{
						"iso_code": mmdbtype.String(code),
					},
				}

				if err := writer.Insert(ipNet, record); err != nil {
					fmt.Fprintf(os.Stderr, "    ⚠️  Insert %s: %v\n", cidr, err)
				}
			}
		}
	}

	// Write the merged MMDB
	f, err := os.Create(outPath)
	if err != nil {
		return fmt.Errorf("create file: %w", err)
	}
	defer f.Close()

	if _, err := writer.WriteTo(f); err != nil {
		return fmt.Errorf("write MMDB: %w", err)
	}

	fmt.Printf("    ✅ geoip.mmdb\n")
	return nil
}

// generateGeoASNMMDB generates geoasn.mmdb by merging upstream ASN MMDBs
func generateGeoASNMMDB(categories map[string]map[string]RuleSet, cacheDir, outPath string) error {
	writer, err := mmdbwriter.New(mmdbwriter.Options{
		DatabaseType: "Gins-GeoASN",
		Description: map[string]string{
			"en": "Gins-Rules GeoASN Database (merged from upstream)",
		},
		RecordSize: 24,
	})
	if err != nil {
		return fmt.Errorf("create writer: %w", err)
	}

	// Merge upstream ASN MMDBs
	for _, src := range upstreamASNSources {
		mmdbPath := filepath.Join(cacheDir, src.Name+".mmdb")
		if err := mergeUpstreamMMDB(writer, mmdbPath, src.Name); err != nil {
			fmt.Fprintf(os.Stderr, "    ⚠️  Merge %s: %v\n", src.Name, err)
		}
	}

	// Add our local ASN data
	if asnTargets, ok := categories["asn"]; ok {
		for name, rs := range asnTargets {
			if len(rs.IPCidr) == 0 {
				continue
			}

			// Extract ASN number
			asn := extractASN(name, rs)
			if asn == "" {
				continue
			}

			for _, cidr := range rs.IPCidr {
				_, ipNet, err := net.ParseCIDR(cidr)
				if err != nil {
					continue
				}

				record := mmdbtype.Map{
					"autonomous_system_number":       mmdbtype.String(asn),
					"autonomous_system_organization": mmdbtype.String(name),
				}

				if err := writer.Insert(ipNet, record); err != nil {
					fmt.Fprintf(os.Stderr, "    ⚠️  Insert %s: %v\n", cidr, err)
				}
			}
		}
	}

	// Write the merged MMDB
	f, err := os.Create(outPath)
	if err != nil {
		return fmt.Errorf("create file: %w", err)
	}
	defer f.Close()

	if _, err := writer.WriteTo(f); err != nil {
		return fmt.Errorf("write MMDB: %w", err)
	}

	fmt.Printf("    ✅ geoasn.mmdb\n")
	return nil
}

// mergeUpstreamMMDB merges an upstream MMDB file into the writer
func mergeUpstreamMMDB(writer *mmdbwriter.Tree, mmdbPath, sourceName string) error {
	if _, err := os.Stat(mmdbPath); os.IsNotExist(err) {
		return fmt.Errorf("file not found: %s", mmdbPath)
	}

	reader, err := maxminddb.Open(mmdbPath)
	if err != nil {
		return fmt.Errorf("open MMDB: %w", err)
	}
	defer reader.Close()

	networks := reader.Networks()
	count := 0
	for networks.Next() {
		record := make(map[string]interface{})
		ipNet, err := networks.Network(&record)
		if err != nil {
			continue
		}

		// Convert record to mmdbtype
		mmdbRecord := convertToMMDBType(record)
		if err := writer.Insert(ipNet, mmdbRecord); err != nil {
			// Skip conflicting entries
			continue
		}
		count++
	}

	fmt.Printf("    ✓ %s: %d entries\n", sourceName, count)
	return nil
}

// convertToMMDBType converts a map to mmdbtype.Map
func convertToMMDBType(record map[string]interface{}) mmdbtype.Map {
	result := make(mmdbtype.Map)
	for k, v := range record {
		result[mmdbtype.String(k)] = convertValue(v)
	}
	return result
}

// convertValue converts a value to mmdbtype
func convertValue(v interface{}) mmdbtype.DataType {
	switch val := v.(type) {
	case map[string]interface{}:
		return convertToMMDBType(val)
	case []interface{}:
		arr := make(mmdbtype.Slice, len(val))
		for i, item := range val {
			arr[i] = convertValue(item)
		}
		return arr
	case string:
		return mmdbtype.String(val)
	case float64:
		return mmdbtype.Float64(val)
	case bool:
		return mmdbtype.Bool(val)
	case uint16:
		return mmdbtype.Uint16(val)
	case uint32:
		return mmdbtype.Uint32(val)
	case uint64:
		return mmdbtype.Uint64(val)
	default:
		return mmdbtype.String(fmt.Sprintf("%v", v))
	}
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
