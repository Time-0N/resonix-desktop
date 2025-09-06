import { defineConfig } from 'vite';

export default defineConfig({
    root: './ui',
    build: {
        outDir: 'dist',
        rollupOptions: {
            input: {
                main: './ui/index.html',
            },
        },
    },
    resolve: {
        alias: {
            '@': '/src',
            '@components': '/src/components',
            '@services': '/src/services',
            '@stores': '/src/stores',
            '@utils': '/src/utils',
            '@types': '/src/types',
        },
    },
});