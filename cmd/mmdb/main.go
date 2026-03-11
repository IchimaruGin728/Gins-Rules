package main

import (
	"bufio"
	"fmt"
	"io"
	"net/http"
	"net/netip"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"time"

	"github.com/oschwald/maxminddb-golang"
)

// MMDB sources from xream/geoip — two providers for multi-source merge
var mmdbSources = []struct {
	Name string
	URL  string
	Type string // "country" or "asn"
}{
	// Full Country MMDB
	{
		Name: "ipinfo.country",
		URL:  "https://github.com/xream/geoip/releases/latest/download/ipinfo.country.mmdb",
		Type: "country",
	},
	{
		Name: "ip2location.country",
		URL:  "https://github.com/xream/geoip/releases/latest/download/ip2location.country.mmdb",
		Type: "country",
	},
	// ASN MMDB (already full)
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
	{"asn-disney", []uint{19679}},
	{"asn-oracle", []uint{31898}},
	{"asn-akamai", []uint{16625, 20940, 3131, 33905, 34164, 34850, 43639, 53235, 54104}},
	{"asn-twitch", []uint{46489}},
	{"asn-alibaba", []uint{37963, 45102, 132335}},
	{"asn-tencent", []uint{132203, 132591, 133478, 133543}},
	{"asn-bytedance", []uint{138690}},
	{"asn-baidu", []uint{55967, 134177}},
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
			count := parseASNMMDB(db, asnCIDRs)
			fmt.Printf("  [OK] %s: extracted %d networks\n", src.Name, count)
		}

		db.Close()
	}

	// Write country files
	fmt.Println("\n  Writing country IP lists...")
	
	notCN := make(map[string]bool)
	commonRegions := map[string]bool{
		"CN": true, "SG": true, "TW": true, "JP": true, "US": true,
	}

	for code, cidrs := range countryCIDRs {
		// Collect all non-CN for !cn.txt (Complete Global Fallback)
		if code != "CN" {
			for cidr := range cidrs {
				notCN[cidr] = true
			}
		}

		// Only write separate files for common regions to keep dashboard clean
		if commonRegions[code] {
			filename := strings.ToLower(code) + ".txt"
			outPath := filepath.Join(ipDir, filename)
			if err := writeIPList(outPath, cidrs); err != nil {
				fmt.Printf("  [ERROR] Failed to write ip/%s: %v\n", filename, err)
			} else {
				fmt.Printf("  [WRITE] ip/%s: %d CIDRs\n", filename, len(cidrs))
			}
		}
	}

	// Clear country mapping as it's no longer needed
	countryCIDRs = nil

	// Write !cn.txt
	if len(notCN) > 0 {
		outPath := filepath.Join(ipDir, "!cn.txt")
		if err := writeIPList(outPath, notCN); err != nil {
			fmt.Printf("  [ERROR] Failed to write ip/!cn.txt: %v\n", err)
		} else {
			fmt.Printf("  [WRITE] ip/!cn.txt: %d CIDRs (All non-CN countries)\n", len(notCN))
		}
		notCN = nil // Clear for memory
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
		outPath := filepath.Join(ipDir, target.Name+".txt")
		if err := writeIPList(outPath, merged); err != nil {
			fmt.Printf("  [ERROR] Failed to write ip/%s.txt: %v\n", target.Name, err)
		} else {
			fmt.Printf("  [WRITE] ip/%s.txt: %d CIDRs\n", target.Name, len(merged))
		}
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

func parseASNMMDB(db *maxminddb.Reader, asnCIDRs map[uint]map[string]bool) int {
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
	client := &http.Client{
		Timeout: 5 * time.Minute,
	}
	resp, err := client.Get(url)
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

func writeIPList(filePath string, cidrSet map[string]bool) error {
	f, err := os.Create(filePath)
	if err != nil {
		return err
	}
	defer f.Close()

	writer := bufio.NewWriter(f)
	defer writer.Flush()

	// 1. Convert strings to netip.Prefix
	var prefixes []netip.Prefix
	for s := range cidrSet {
		p, err := netip.ParsePrefix(s)
		if err == nil {
			prefixes = append(prefixes, p)
		}
	}

	// 2. Aggregate/Merge for "Perfection"
	aggregated := aggregatePrefixes(prefixes)

	// 3. Sort for deterministic output
	sort.Slice(aggregated, func(i, j int) bool {
		if aggregated[i].Addr().Compare(aggregated[j].Addr()) != 0 {
			return aggregated[i].Addr().Compare(aggregated[j].Addr()) < 0
		}
		return aggregated[i].Bits() < aggregated[j].Bits()
	})

	for _, p := range aggregated {
		if _, err := writer.WriteString(p.String() + "\n"); err != nil {
			return err
		}
	}
	return nil
}

// aggregatePrefixes merges adjacent or overlapping prefixes into a minimal set.
func aggregatePrefixes(prefixes []netip.Prefix) []netip.Prefix {
	if len(prefixes) == 0 {
		return nil
	}

	// Build ranges
	type addrRange struct {
		from, to netip.Addr
	}
	var ranges []addrRange
	for _, p := range prefixes {
		ranges = append(ranges, addrRange{p.Masked().Addr(), lastAddr(p)})
	}

	// Sort ranges
	sort.Slice(ranges, func(i, j int) bool {
		return ranges[i].from.Compare(ranges[j].from) < 0
	})

	// Merge ranges
	var merged []addrRange
	if len(ranges) > 0 {
		curr := ranges[0]
		for i := 1; i < len(ranges); i++ {
			// If overlapping or adjacent
			if curr.to.Compare(ranges[i].from) >= 0 || isAdjacent(curr.to, ranges[i].from) {
				if ranges[i].to.Compare(curr.to) > 0 {
					curr.to = ranges[i].to
				}
			} else {
				merged = append(merged, curr)
				curr = ranges[i]
			}
		}
		merged = append(merged, curr)
	}

	// Convert ranges back to prefixes
	var result []netip.Prefix
	for _, r := range merged {
		result = append(result, partitionRange(r.from, r.to)...)
	}

	return result
}

func lastAddr(p netip.Prefix) netip.Addr {
	p = p.Masked()
	addr := p.Addr()
	bits := p.Bits()
	totalBits := 32
	if addr.Is6() {
		totalBits = 128
	}

	// Get raw bytes
	b := addr.AsSlice()
	
	// Set remaining bits to 1
	for i := totalBits - 1; i >= bits; i-- {
		byteIdx := i / 8
		bitIdx := 7 - (i % 8)
		b[byteIdx] |= (1 << bitIdx)
	}

	res, _ := netip.AddrFromSlice(b)
	return res
}

func isAdjacent(a, b netip.Addr) bool {
	if a.Is4() != b.Is4() {
		return false
	}
	return a.Next() == b
}

func partitionRange(from, to netip.Addr) []netip.Prefix {
	var res []netip.Prefix
	for from.Compare(to) <= 0 {
		// Find largest prefix that starts at 'from' and ends at or before 'to'
		bits := 32
		if from.Is6() {
			bits = 128
		}
		
		maxBits := 0
		if from.Is4() {
			// Find max trailing zeros
			ip4 := from.As4()
			u := uint32(ip4[0])<<24 | uint32(ip4[1])<<16 | uint32(ip4[2])<<8 | uint32(ip4[3])
			if u == 0 {
				maxBits = 32
			} else {
				for (u & 1) == 0 {
					maxBits++
					u >>= 1
				}
			}
		} else {
			// IPv6 fallback: simpler logic
			maxBits = 0 // Keep it simple for now or implement full
		}
		
		// Adjust for 'to'
		finalBits := bits
		for b := maxBits; b >= 0; b-- {
			p := netip.PrefixFrom(from, bits-b)
			if p.Masked() == p && lastAddr(p).Compare(to) <= 0 {
				finalBits = bits - b
				break
			}
		}
		
		p := netip.PrefixFrom(from, finalBits)
		res = append(res, p)
		
		last := lastAddr(p)
		if last == to {
			break
		}
		from = last.Next()
		if from == (netip.Addr{}) { // overflow
			break
		}
	}
	return res
}

func sortedKeys(m map[string]bool) []string {
	var keys []string
	for k := range m {
		keys = append(keys, k)
	}
	sort.Strings(keys)
	return keys
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

