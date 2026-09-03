# C/D: a minimal stdio MCP server advertising one tool.
# SCHEMA=big   -> the full element schema (41 KB minified)
# SCHEMA=small -> a trivial two-field schema, to isolate schema cost from startup
import json
import os
import pathlib

import anyio
import mcp.server.stdio
import mcp.types as types
from mcp.server.lowlevel import Server

BIG = os.environ.get("SCHEMA") != "small"
_here = pathlib.Path(__file__).resolve().parent
ELEMENT = (
    json.loads((_here.parent / "element-schema.json").read_text())
    if BIG
    else {"type": "object", "properties": {"type": {"type": "string"}, "start": {"type": "number"}}}
)

TOOLS = [
    types.Tool(
        name="add_element",
        description="Append one complete element to a project file.",
        inputSchema={
            "type": "object",
            "required": ["path", "element"],
            "properties": {"path": {"type": "string"}, "element": ELEMENT},
        },
    )
]


async def on_list_tools(ctx, params):
    return types.ListToolsResult(tools=TOOLS)


async def on_call_tool(ctx, params):
    return types.CallToolResult(content=[types.TextContent(type="text", text="ok")])


server = Server(
    "montaget",
    version="0.0.0",
    on_list_tools=on_list_tools,
    on_call_tool=on_call_tool,
)


async def main():
    async with mcp.server.stdio.stdio_server() as (read, write):
        await server.run(read, write, server.create_initialization_options())


anyio.run(main)
