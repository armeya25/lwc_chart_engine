import { execSync } from 'child_process';
import fs from 'fs';
import path from 'path';

/**
 * 🛠 Cross-Platform Frontend Build Script
 * Ensures directory creation and file copying work on Windows, Linux, and macOS.
 */

const DIST_DIR = path.join('src-frontend', 'dist');

function run() {
    console.log('🚀 Starting cross-platform frontend build...');

    // 1. Ensure dist directory exists
    if (!fs.existsSync(DIST_DIR)) {
        console.log(`📁 Creating directory: ${DIST_DIR}`);
        fs.mkdirSync(DIST_DIR, { recursive: true });
    } else {
        console.log(`🧹 Cleaning old artifacts in: ${DIST_DIR}`);
        const files = fs.readdirSync(DIST_DIR);
        for (const file of files) {
            fs.unlinkSync(path.join(DIST_DIR, file));
        }
    }

    // 2. Run esbuild for JS
    console.log('📦 Bundling JavaScript with esbuild...');
    execSync('npx esbuild src-frontend/js/entry.js --bundle --minify --outfile=src-frontend/dist/app.min.js', { stdio: 'inherit' });

    // 3. Run esbuild for CSS
    console.log('🎨 Bundling and minifying CSS with esbuild...');
    execSync('npx esbuild src-frontend/terminal.css --bundle --minify --outfile=src-frontend/dist/terminal.css', { stdio: 'inherit' });

    // 4. Copy static assets
    console.log('📄 Copying static assets...');
    fs.copyFileSync(path.join('src-frontend', 'index.dist.html'), path.join(DIST_DIR, 'index.html'));
    fs.copyFileSync(path.join('src-frontend', 'lightweight-charts.js'), path.join(DIST_DIR, 'lightweight-charts.js'));

    console.log('✨ Frontend build complete!');
}

try {
    run();
} catch (error) {
    console.error('❌ Build failed:', error);
    process.exit(1);
}
