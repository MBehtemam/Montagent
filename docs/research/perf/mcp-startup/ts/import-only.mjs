// B: runtime + SDK import, nothing else. Isolates import cost from protocol work.
await import('@modelcontextprotocol/sdk/server/index.js');
await import('@modelcontextprotocol/sdk/server/stdio.js');
process.stdout.write('{"ready":true}\n');
