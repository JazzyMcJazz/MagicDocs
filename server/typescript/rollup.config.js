import typescript from '@rollup/plugin-typescript';
import { nodeResolve } from '@rollup/plugin-node-resolve';
import terser from '@rollup/plugin-terser';

const dev = process.env.DEV === 'true';

export default {
    input: {
        htmx: 'src/htmx.js',
        global: 'src/global.ts',
        'browser-sync-client': 'src/browser-sync-client.ts',
    },
    output: {
        dir: '../static/js',  // Output directory for all bundles
        format: 'esm',  // Use ES module format
        sourcemap: dev,  // Enable source maps for debugging
        entryFileNames: '[name].js',  // Use the property names from input as filenames
    },
    plugins: [
        nodeResolve(),
        typescript({
            tsconfig: 'tsconfig.json',
            outputToFilesystem: true,
        }),
        !dev && terser({
            format: { comments: false },
            compress: {
                drop_console: false,
                dead_code: true,
            },
            mangle: { toplevel: true ,}
        })
    ],
    external: ['htmx']
};
