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
  var city = obj.city || "Unknown";
  var region = obj.regionCode || obj.region || "";
  var postal = obj.postalCode || "";
  var ip = obj.ip || "N/A";
  var asn = obj.asn ? "AS" + obj.asn : "";
  var org = (obj.asOrganization || "Unknown ISP").replace(/,?\s*(Inc\.|LLC|Corp\.|LTD|Ltd\.|S\.A\.|GmbH)/ig, "").trim();

  var score = obj.fraudScore !== undefined ? obj.fraudScore : -1;
  var riskBar = score < 0 ? "N/A" : (function() {
    var f = Math.round(score / 10);
    var bar = "";
    for (var i = 0; i < 10; i++) bar += i < f ? "▓" : "░";
    return bar + " " + score;
  })();

  // Quality verdict: isResidential + isBroadcast + fraudScore
  var typeIcon, typeLabel, verdict;
  if (obj.isBroadcast) {
    typeIcon = "📡"; typeLabel = "Broadcast";
  } else if (obj.isResidential) {
    typeIcon = "🏠"; typeLabel = "Native";
  } else {
    typeIcon = "🏢"; typeLabel = "DC";
  }

  if (score < 0 || score <= 33) {
    verdict = "Clean";
  } else if (score <= 66) {
    verdict = "Caution";
  } else {
    verdict = "Risky";
  }

  var loc = city + (region ? ", " + region : "") + (postal ? " " + postal : "");
  var title = flag + " " + loc + " ║ " + typeIcon + " " + typeLabel + " · " + verdict;
  var subtitle = "⚡ " + org + (asn ? " (" + asn + ")" : "") + " · " + ip + " · " + riskBar;

  $done({ title: title, subtitle: subtitle });

} catch (e) {
  $done({ title: "⚠️ Error", subtitle: e.message || "Parse Failed" });
}
