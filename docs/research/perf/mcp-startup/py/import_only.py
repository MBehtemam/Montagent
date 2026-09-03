# B: interpreter + SDK import, nothing else.
import sys
import mcp.server.lowlevel  # noqa: F401
import mcp.server.stdio  # noqa: F401
sys.stdout.write('{"ready":true}\n')
