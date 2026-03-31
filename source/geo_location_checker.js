if ($response.statusCode != 200) {
  $done(null);
}

try {
  var body = $response.body;
  var obj = JSON.parse(body);

  function getFlagEmoji(countryCode) {
      if (!countryCode) return "🏁";
      var codePoints = countryCode
        .toUpperCase()
        .split('')
        .map(char => 127397 + char.charCodeAt(0));
      return String.fromCodePoint(...codePoints);
  }

  var code = obj.countryCode || "UN";
  var flag = getFlagEmoji(code);
  var city = obj.city || "Unknown";
  var org = obj.asOrganization || obj.org || "Unknown ISP";

  // Clean org name
  org = org.replace(/,?\s*(Inc\.|LLC|Corp\.|LTD|Ltd\.|S\.A\.|GmbH)/ig, "");
  org = org.trim();

  var score = obj.fraudScore !== undefined ? obj.fraudScore : 0;
  var riskEmoji = "🔹";
  if (score > 60) {
      riskEmoji = "🔺";
  } else if (score > 30) {
      riskEmoji = "🔸";
  }

  var buildType = obj.isResidential ? "🏠" : "🏢";

  // Style A:
  // Title: 🇺🇸 US | Risk: 75 🔴
  // Subtitle: Los Angeles (🏢) • Cloudflare
  var title = flag + " " + code + " | Risk: " + score + " " + riskEmoji;
  var subtitle = city + " (" + buildType + ") • " + org;

  $done({ title: title, subtitle: subtitle });

} catch (e) {
  $done({ title: "Error", subtitle: "Parse Failed" });
}
