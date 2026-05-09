import { WorkflowEntrypoint } from "cloudflare:workers";
import type { WorkflowEvent, WorkflowStep } from "cloudflare:workers";

interface Env {
  GINS_RULES_ASSETS: Fetcher;
  GINS_RULES_R2_STORAGE: R2Bucket;
  GINS_RULES_KV_HOT: KVNamespace;
  GINS_RULES_KV_METADATA: KVNamespace;
  GINS_RULES_WORKFLOW_BUILD: Workflow;
  GINS_RULES_QUEUE_NOTIFY: Queue;
  GINS_RULES_ANALYTICS_HITS: AnalyticsEngineDataset;
  GINS_RULES_AI: Ai;
  TELEGRAM_BOT_TOKEN: string;
  TELEGRAM_CHAT_ID: string;
  DISCORD_WEBHOOK_URL: string;
  WORKFLOW_SECRET: string;
}

interface BuildStats {
  services: number;
  rules: number;
  ipRules: number;
  srs: number;
  mrs: number;
  timestamp: string;
  topHits?: string;
}

interface NotifyMessage {
  text: string;
  telegram: boolean;
  discord: boolean;
}

export default {
  async fetch(
    request: Request,
    env: Env,
    ctx: ExecutionContext,
  ): Promise<Response> {
    const url = new URL(request.url);
    const path = url.pathname;

    if (request.method === "OPTIONS") {
      return new Response(null, { headers: corsHeaders() });
    }

    if (path.startsWith("/ruleset/")) {
      return handleFeed(request, path, url, env, ctx);
    }

    const parsers = [
      "/QX-Resource-Parser.js",
      "/Loon-Resource-Parser.js",
      "/geo_location_checker.js",
    ];
    if (parsers.includes(path)) {
      return handleParser(request, path, env);
    }

    if (path === "/Gins-Icons.json") {
      return handleIconsCatalog(request, env);
    }

    const assetResp = await env.GINS_RULES_ASSETS.fetch(
      request.clone() as Request,
    );
    if (assetResp.status !== 404) return assetResp;

    if (path === "/health" || path === "/api/health") {
      return json({
        status: "ok",
        service: "gins-rules",
        timestamp: Date.now(),
      });
    }

    if (path === "/api/build-summary")
      return handleR2JSON(env, "ruleset/build-summary.json");
    if (path === "/api/asn-prefix-index")
      return handleR2JSON(env, "asn-prefix-index.json");
    if (path === "/api/ruleset-manifest")
      return handleRulesetManifest(url, env);
    if (path === "/workflow/build-complete" && request.method === "POST")
      return handleBuildComplete(request, env);

    return env.GINS_RULES_ASSETS.fetch(
      new Request(new URL("/", request.url).toString(), request as Request),
    );
  },

  async queue(batch: MessageBatch<NotifyMessage>, env: Env): Promise<void> {
    for (const msg of batch.messages) {
      const { text, telegram, discord } = msg.body;
      const sections = text
        .split("---SPLIT---")
        .map((s: string) => s.trim())
        .filter(Boolean);
      for (const content of sections) {
        if (telegram) await sendTelegram(env, content);
        if (discord) await sendDiscord(env, content);
      }
      msg.ack();
    }
  },
};

export class BuildWorkflow extends WorkflowEntrypoint<Env, BuildStats> {
  async run(event: WorkflowEvent<BuildStats>, step: WorkflowStep) {
    const stats = event.payload;

    await step.do("health-check", async () => {
      const sample = await this.env.GINS_RULES_R2_STORAGE.head(
        "ruleset/singbox/proxy/apple.srs",
      );
      if (!sample) throw new Error("R2 Validation Failed");
    });

    await step.do("cache-bust", async () => {
      await this.env.GINS_RULES_KV_METADATA.put(
        "latest_build_id",
        stats.timestamp,
      );
    });

    const topHits = await step.do("aggregate-traffic", async () => {
      const list = await this.env.GINS_RULES_KV_METADATA.list({
        prefix: "hit:",
      });
      const hits = [];
      for (const key of list.keys) {
        const count = await this.env.GINS_RULES_KV_METADATA.get(key.name);
        hits.push({
          name: key.name.replace("hit:", ""),
          count: parseInt(count || "0"),
        });
        await this.env.GINS_RULES_KV_METADATA.delete(key.name);
      }
      return (
        hits
          .sort((a, b) => b.count - a.count)
          .slice(0, 5)
          .map((h) => `· ${h.name} (${h.count} hits)`)
          .join("\n") || "No traffic data"
      );
    });

    const summary = await step.do("generate-summary", async () => {
      return await generateDailySummary(this.env, { ...stats, topHits });
    });

    await step.do("notify", async () => {
      await this.env.GINS_RULES_QUEUE_NOTIFY.send({
        text: summary,
        telegram: true,
        discord: true,
      });
    });

    await step.do("warm-kv", async () => {
      await populateHotRulesKV(this.env);
    });
  }
}

async function handleFeed(
  request: Request,
  path: string,
  url: URL,
  env: Env,
  ctx: ExecutionContext,
): Promise<Response> {
  const cache = (caches as unknown as { default: Cache }).default;
  const buildId =
    (await env.GINS_RULES_KV_METADATA.get("latest_build_id")) || "v1";
  const cacheKey = new Request(
    `${url.origin}${path}?v=${buildId}`,
    request as Request,
  );

  let response = await cache.match(cacheKey);
  if (response) return response;

  let assetPath: string;
  let ext: string = "";

  const isGeo = path.includes(".dat") || path.includes(".mmdb");
  if (isGeo) {
    assetPath = path.endsWith(".mmdb")
      ? path.replace(/^\//, "")
      : path
          .replace(/^\/(ruleset)\/(v2ray|xray)\//, "$1/xray/")
          .replace(/^\//, "");
    ext = path.split(".").pop() || "";
  } else {
    const parts = path.replace("/ruleset/", "").split("/");
    let app: string | null = null;
    let category: string;
    let name: string;

    if (parts.length === 1) {
      const dotIdx = parts[0].lastIndexOf(".");
      if (dotIdx === -1)
        return new Response("Invalid filename", { status: 400 });
      category = parts[0].slice(0, dotIdx);
      name = category;
      ext = parts[0].slice(dotIdx + 1);
    } else if (parts.length === 2) {
      const apps = [
        "singbox",
        "mihomo",
        "stash",
        "surge",
        "quantumultx",
        "quanx",
        "loon",
        "egern",
        "shadowrocket",
        "surfboard",
        "exclave",
        "anywhere",
      ];
      if (apps.includes(parts[0])) {
        app = parts[0];
        const dotIdx = parts[1].lastIndexOf(".");
        if (dotIdx === -1)
          return new Response("Invalid filename", { status: 400 });
        category = parts[1].slice(0, dotIdx);
        name = category;
        ext = parts[1].slice(dotIdx + 1);
      } else {
        category = parts[0];
        const dotIdx = parts[1].lastIndexOf(".");
        if (dotIdx === -1)
          return new Response("Invalid filename", { status: 400 });
        name = parts[1].slice(0, dotIdx);
        ext = parts[1].slice(dotIdx + 1);
      }
    } else if (parts.length === 3) {
      app = parts[0];
      category = parts[1];
      const dotIdx = parts[2].lastIndexOf(".");
      if (dotIdx === -1)
        return new Response("Invalid filename", { status: 400 });
      name = parts[2].slice(0, dotIdx);
      ext = parts[2].slice(dotIdx + 1);
    } else {
      return new Response("Not Found", { status: 404 });
    }

    const appToDir: Record<string, string> = {
      singbox: "singbox",
      mihomo: "mihomo",
      clash: "mihomo",
      stash: "stash",
      surge: "surge",
      quantumultx: "quantumultx",
      quanx: "quantumultx",
      loon: "loon",
      egern: "egern",
      shadowrocket: "shadowrocket",
      surfboard: "surfboard",
      surfboard_ds: "surfboard",
      exclave: "exclave",
      anywhere: "anywhere",
      v2ray: "xray",
    };
    const extMap: Record<string, string> = {
      lsr: "loon",
      yaml: "egern",
      srs: "singbox",
      mrs: "mihomo",
    };
    const dir = app ? appToDir[app] : (extMap[ext] ?? ext);
    if (!dir) return new Response("Invalid app or extension", { status: 400 });

    let targetFile = `${name}.${ext}`;
    if (app === "loon" && ext === "list") targetFile = `${name}.lsr`;
    const isIPLike = category === "ip" || category === "asn";
    if (
      dir === "text" &&
      isIPLike &&
      ext === "list" &&
      !targetFile.includes(".ip.")
    ) {
      targetFile = targetFile.replace(".list", ".ip.list");
    }

    assetPath = `ruleset/${dir}/${category}/${targetFile}`;
    env.GINS_RULES_ANALYTICS_HITS.writeDataPoint({
      blobs: [app || "unknown", category, name, ext],
      indexes: [name],
    });
  }

  ctx.waitUntil(
    env.GINS_RULES_KV_METADATA.get(`hit:${assetPath}`)
      .then((c) =>
        env.GINS_RULES_KV_METADATA.put(
          `hit:${assetPath}`,
          (parseInt(c || "0") + 1).toString(),
        ),
      )
      .catch(() => {}),
  );

  const kvStream = await env.GINS_RULES_KV_HOT.get(assetPath, "stream");
  let body: ReadableStream | null = null;
  let source = "R2";
  const headers = new Headers({
    "Access-Control-Allow-Origin": "*",
    "Cache-Control": "public, max-age=86400",
  });

  if (kvStream) {
    body = kvStream;
    source = "KV";
  } else {
    const r2Obj = await env.GINS_RULES_R2_STORAGE.get(assetPath);
    if (!r2Obj) return new Response("Not Found", { status: 404 });
    body = r2Obj.body;
    source = "R2";
    r2Obj.writeHttpMetadata(headers);
  }

  headers.set("X-Cache-Source", source);
  headers.set("X-Cache-Status", "MISS-Edge");

  if (ext === "srs" || ext === "mrs" || isGeo)
    headers.set("Content-Type", "application/octet-stream");
  else if (ext === "json") headers.set("Content-Type", "application/json");
  else headers.set("Content-Type", "text/plain; charset=utf-8");

  const finalResponse = new Response(body, { headers });
  ctx.waitUntil(cache.put(cacheKey, finalResponse.clone()));
  return finalResponse;
}

async function handleParser(
  request: Request,
  path: string,
  env: Env,
): Promise<Response> {
  const key = path.replace(/^\//, "");
  const kv = await env.GINS_RULES_KV_HOT.get(key, "stream");
  if (kv) {
    return new Response(kv, {
      headers: {
        "Content-Type": "application/javascript",
        "Cache-Control": "public, max-age=3600",
      },
    });
  }
  return env.GINS_RULES_ASSETS.fetch(request as Request);
}

async function populateHotRulesKV(env: Env) {
  const hotTargets = [
    "proxy",
    "direct",
    "reject",
    "ai",
    "apple",
    "apple-cdn",
    "apple-intelligence",
    "apple-music",
    "appletv",
  ];
  const formats = [
    "singbox",
    "mihomo",
    "stash",
    "surge",
    "quantumultx",
    "loon",
    "egern",
    "shadowrocket",
    "surfboard",
    "exclave",
    "anywhere",
    "text",
  ];
  const extMap: Record<string, string> = {
    singbox: "srs",
    mihomo: "mrs",
    stash: "mrs",
    surge: "list",
    quantumultx: "list",
    loon: "lsr",
    egern: "yaml",
    shadowrocket: "list",
    surfboard: "list",
    exclave: "list",
    anywhere: "json",
    text: "list",
  };

  const promises: Promise<any>[] = [];

  for (const format of formats) {
    for (const target of hotTargets) {
      let category = "proxy";
      if (target === "direct" || target.includes("cdn")) category = "direct";
      else if (target === "reject") category = "reject";
      else if (target === "ai" || target === "apple-intelligence")
        category = "ai";

      const path = `ruleset/${format}/${category}/${target}.${extMap[format]}`;
      promises.push(
        env.GINS_RULES_R2_STORAGE.get(path)
          .then(async (obj) => {
            if (obj)
              await env.GINS_RULES_KV_HOT.put(path, await obj.arrayBuffer());
          })
          .catch(() => {}),
      );
    }
  }

  const geoFiles = [
    "ruleset/xray/geoip.dat",
    "ruleset/xray/geosite.dat",
    "ruleset/geoip.mmdb",
    "ruleset/geoasn.mmdb",
    "QX-Resource-Parser.js",
    "Loon-Resource-Parser.js",
    "geo_location_checker.js",
  ];
  for (const file of geoFiles) {
    promises.push(
      env.GINS_RULES_R2_STORAGE.get(file)
        .then(async (obj) => {
          if (obj)
            await env.GINS_RULES_KV_HOT.put(file, await obj.arrayBuffer());
        })
        .catch(() => {}),
    );
  }

  const batchSize = 10;
  for (let i = 0; i < promises.length; i += batchSize) {
    await Promise.all(promises.slice(i, i + batchSize));
  }
}

async function generateDailySummary(
  env: Env,
  stats: BuildStats,
): Promise<string> {
  const date = new Date().toLocaleDateString("zh-CN", {
    month: "2-digit",
    day: "2-digit",
    timeZone: "Asia/Singapore",
  });
  const domainRules = Math.max(0, stats.rules - (stats.ipRules || 0));
  const prompt = `Write build report for project Gins-Rules. Services: ${stats.services}, Rules: ${domainRules} domain + ${stats.ipRules} IP. Build Time: ${stats.timestamp}. Top Hits: ${stats.topHits}. Use ---SPLIT--- for CN/EN. Updated ${date}`;
  try {
    const res = await env.GINS_RULES_AI.run("@cf/meta/llama-3.1-8b-instruct", {
      messages: [{ role: "user", content: prompt }],
    });
    return (res as any).response || "Build Success";
  } catch (e) {
    return "Build Success";
  }
}

async function handleBuildComplete(
  request: Request,
  env: Env,
): Promise<Response> {
  const auth = request.headers.get("Authorization");
  if (auth !== `Bearer ${env.WORKFLOW_SECRET}`)
    return new Response("Unauthorized", { status: 401 });
  const stats = (await request.json()) as BuildStats;
  await env.GINS_RULES_WORKFLOW_BUILD.create({
    id: `build-${Date.now()}`,
    params: stats,
  });
  return json({ status: "ok" });
}

async function handleIconsCatalog(
  request: Request,
  env: Env,
): Promise<Response> {
  return env.GINS_RULES_ASSETS.fetch(
    new Request(
      new URL("/icons-catalog.json", request.url).toString(),
      request as Request,
    ),
  );
}

async function handleR2JSON(env: Env, key: string): Promise<Response> {
  const obj = await env.GINS_RULES_R2_STORAGE.get(key);
  if (!obj) return new Response("Not Found", { status: 404 });
  return new Response(obj.body, {
    headers: {
      "Content-Type": "application/json",
      "Access-Control-Allow-Origin": "*",
    },
  });
}

async function handleRulesetManifest(url: URL, env: Env): Promise<Response> {
  const f = url.searchParams.get("format") || "";
  const c = url.searchParams.get("category") || "";
  return handleR2JSON(env, `ruleset/${f}/${c}/manifest.json`);
}

async function sendTelegram(env: Env, text: string) {
  await fetch(
    `https://api.telegram.org/bot${env.TELEGRAM_BOT_TOKEN}/sendMessage`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        chat_id: env.TELEGRAM_CHAT_ID,
        text,
        parse_mode: "Markdown",
      }),
    },
  );
}

async function sendDiscord(env: Env, text: string) {
  await fetch(env.DISCORD_WEBHOOK_URL, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ content: text }),
  });
}

function json(d: any) {
  return new Response(JSON.stringify(d), {
    headers: { "Content-Type": "application/json" },
  });
}
function corsHeaders() {
  return {
    "Access-Control-Allow-Origin": "*",
    "Access-Control-Allow-Methods": "GET, POST, OPTIONS",
    "Access-Control-Allow-Headers": "Content-Type, Authorization",
  };
}
