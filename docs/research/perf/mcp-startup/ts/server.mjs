// C/D: a minimal stdio MCP server advertising one tool.
// SCHEMA=big  -> the full element schema (41 KB minified)
// SCHEMA=small -> a trivial two-field schema, to isolate schema cost from startup
import { readFileSync } from 'node:fs';
import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import { ListToolsRequestSchema, CallToolRequestSchema } from '@modelcontextprotocol/sdk/types.js';

const big = process.env.SCHEMA !== 'small';
const elementSchema = big
    ? JSON.parse(readFileSync(new URL('../element-schema.json', import.meta.url), 'utf8'))
    : { type: 'object', properties: { type: { type: 'string' }, start: { type: 'number' } } };

const server = new Server({ name: 'montaget', version: '0.0.0' }, { capabilities: { tools: {} } });

server.setRequestHandler(ListToolsRequestSchema, () => ({
    tools: [
        {
            name: 'add_element',
            description: 'Append one complete element to a project file.',
            inputSchema: {
                type: 'object',
                required: ['path', 'element'],
                properties: { path: { type: 'string' }, element: elementSchema }
            }
        }
    ]
}));
server.setRequestHandler(CallToolRequestSchema, () => ({ content: [{ type: 'text', text: 'ok' }] }));

await server.connect(new StdioServerTransport());
