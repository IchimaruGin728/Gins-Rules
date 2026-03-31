if ($response.statusCode != 200) {
  $done(null);
}

try {
  var obj = JSON.parse($response.body);

  function getFlagEmoji(cc) {
    if (!cc) return "🏁";
    return String.fromCodePoint(...cc.toUpperCase().split("").map(function(c) { return 127397 + c.charCodeAt(0); }));
  }

  var flag = getFlagEmoji(obj.countryCode);
  var code = obj.countryCode || "??";
  var city = obj.city || "Unknown";
  var region = obj.regionCode || obj.region || "";
  var ip = obj.ip || "N/A";
  var asn = obj.asn ? "AS" + obj.asn : "";
  var org = (obj.asOrganization || "Unknown ISP").replace(/,?\s*(Inc\.|LLC|Corp\.|LTD|Ltd\.|S\.A\.|GmbH)/ig, "").trim();

  var score = obj.fraudScore !== undefined ? obj.fraudScore : -1;
  var riskTag = "";
  if (score < 0) {
    riskTag = "N/A";
  } else if (score > 66) {
    riskTag = "🔴 " + score;
  } else if (score > 33) {
    riskTag = "🟡 " + score;
  } else {
    riskTag = "🟢 " + score;
  }

  var netType = obj.isResidential ? "🏠 Residential" : "🏢 Datacenter";

  // Title:  🇺🇸 US · Los Angeles, CA | Risk 75 🔴
  // Subtitle: 🏢 Datacenter · Cloudflare (AS13335) · 104.28.123.123
  var loc = city + (region ? ", " + region : "");
  var title = flag + " " + code + " · " + loc + " | Risk " + riskTag;
  var subtitle = netType + " · " + org + (asn ? " (" + asn + ")" : "") + " · " + ip;

  $done({ title: title, subtitle: subtitle });

} catch (e) {
  $done({ title: "⚠️ Error", subtitle: e.message || "Parse Failed" });
}
