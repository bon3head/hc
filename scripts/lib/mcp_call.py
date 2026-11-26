#!/usr/bin/env python3
"""Drive `hc mcp` over stdio for tests: initialize, then one tools/call per argument.

    mcp_call.py HC_BIN REPO 'tool {"json": "args"}' ...
Prints each tool result's text, one block per call.
"""
import json, subprocess, sys

hc, repo, calls = sys.argv[1], sys.argv[2], sys.argv[3:]
p = subprocess.Popen([hc, "mcp", "--repo", repo], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
def send(msg):
    p.stdin.write(json.dumps(msg) + "\n"); p.stdin.flush()
def recv(want):
    for line in p.stdout:
        msg = json.loads(line)
        if msg.get("id") == want:
            return msg
send({"jsonrpc": "2.0", "id": 0, "method": "initialize", "params": {"protocolVersion": "2025-11-25", "capabilities": {}, "clientInfo": {"name": "t", "version": "0"}}})
recv(0)
send({"jsonrpc": "2.0", "method": "notifications/initialized"})
for i, call in enumerate(calls, 1):
    tool, _, args = call.partition(" ")
    send({"jsonrpc": "2.0", "id": i, "method": "tools/call", "params": {"name": tool, "arguments": json.loads(args)}})
    res = recv(i).get("result", {})
    print("".join(c.get("text", "") for c in res.get("content", [])))
p.stdin.close(); p.wait(timeout=30)
