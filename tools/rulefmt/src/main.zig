const std = @import("std");

pub fn main(init: std.process.Init) !void {
    const allocator = init.gpa;

    var args = std.process.Args.Iterator.init(init.minimal.args);
    defer args.deinit();
    _ = args.next();

    var seen = std.StringHashMap(void).init(allocator);
    defer seen.deinit();

    var lines = std.ArrayList([]const u8).empty;
    defer {
        for (lines.items) |line| allocator.free(line);
        lines.deinit(allocator);
    }

    var had_file = false;
    while (args.next()) |arg| {
        had_file = true;
        try ingestFile(init.io, allocator, arg, &seen, &lines);
    }

    std.mem.sort([]const u8, lines.items, {}, lessThan);

    var stdout_buffer: [4096]u8 = undefined;
    var stdout_writer = std.Io.File.stdout().writer(init.io, &stdout_buffer);
    const stdout = &stdout_writer.interface;
    for (lines.items) |line| {
        try stdout.print("{s}\n", .{line});
    }
    try stdout.flush();
}

fn ingestFile(
    io: std.Io,
    allocator: std.mem.Allocator,
    path: []const u8,
    seen: *std.StringHashMap(void),
    lines: *std.ArrayList([]const u8),
) !void {
    const data = try std.Io.Dir.cwd().readFileAlloc(io, path, allocator, .limited(1024 * 1024 * 512));
    defer allocator.free(data);
    try ingestBuffer(allocator, data, seen, lines);
}

fn ingestBuffer(
    allocator: std.mem.Allocator,
    data: []const u8,
    seen: *std.StringHashMap(void),
    lines: *std.ArrayList([]const u8),
) !void {
    var iter = std.mem.splitScalar(u8, data, '\n');
    while (iter.next()) |raw| {
        const normalized = normalize(raw) orelse continue;
        if (seen.contains(normalized)) continue;

        const owned = try allocator.dupe(u8, normalized);
        try seen.put(owned, {});
        try lines.append(allocator, owned);
    }
}

fn normalize(raw: []const u8) ?[]const u8 {
    var line = std.mem.trim(u8, raw, " \t\r\n");
    if (line.len == 0) return null;
    if (std.mem.startsWith(u8, line, "#") or std.mem.startsWith(u8, line, "//")) return null;

    if (std.mem.indexOf(u8, line, " #")) |idx| {
        line = std.mem.trim(u8, line[0..idx], " \t\r\n");
    }
    if (std.mem.indexOf(u8, line, " //")) |idx| {
        line = std.mem.trim(u8, line[0..idx], " \t\r\n");
    }
    if (line.len == 0) return null;

    if (std.mem.startsWith(u8, line, "+.")) {
        line = line[2..];
    }
    while (std.mem.startsWith(u8, line, ".")) {
        line = line[1..];
    }
    return line;
}

fn lessThan(_: void, a: []const u8, b: []const u8) bool {
    return std.mem.order(u8, a, b) == .lt;
}
