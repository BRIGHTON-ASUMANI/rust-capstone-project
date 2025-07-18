// Polyfill fetch for Node.js 18
import { TextEncoder, TextDecoder } from 'util';
import fetch from 'node-fetch';

// Make fetch available globally
(global as any).fetch = fetch;
(global as any).TextEncoder = TextEncoder;
(global as any).TextDecoder = TextDecoder; 