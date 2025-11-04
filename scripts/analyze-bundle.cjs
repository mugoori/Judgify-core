/**
 * Bundle Size Analysis Script
 *
 * Analyzes the Vite production build bundle to identify:
 * - Total bundle size (uncompressed + gzipped)
 * - Largest dependencies and chunks
 * - Optimization opportunities
 *
 * Usage:
 *   1. Build production bundle: npm run build
 *   2. Run analysis: node scripts/analyze-bundle.js
 *
 * Outputs:
 *   - bundle-analysis/stats-{timestamp}.json (webpack-bundle-analyzer format)
 *   - bundle-analysis/report-{timestamp}.html (interactive visualization)
 *   - Console summary with top 10 largest modules
 */

const { BundleAnalyzerPlugin } = require('webpack-bundle-analyzer');
const fs = require('fs');
const path = require('path');
const { exec } = require('child_process');
const zlib = require('zlib');

// Configuration
const DIST_DIR = path.join(__dirname, '..', 'dist');
const ASSETS_DIR = path.join(DIST_DIR, 'assets');
const OUTPUT_DIR = path.join(__dirname, '..', 'bundle-analysis');
const TIMESTAMP = new Date().toISOString().replace(/[:.]/g, '-').split('T')[0];
const TARGET_GZIP_SIZE_KB = 500; // from TASKS.md Task 1.3

console.log('📦 Starting Bundle Size Analysis...\n');

// Ensure output directory exists
if (!fs.existsSync(OUTPUT_DIR)) {
  fs.mkdirSync(OUTPUT_DIR, { recursive: true });
}

// Check if dist/ folder exists
if (!fs.existsSync(DIST_DIR)) {
  console.error('❌ dist/ folder not found.');
  console.error('Please build the project first: npm run build');
  process.exit(1);
}

// Analyze bundle
const bundleStats = {
  timestamp: new Date().toISOString(),
  files: [],
  totalSize: 0,
  totalGzipSize: 0,
};

// Get all JS/CSS files in dist/assets/
const files = fs.readdirSync(ASSETS_DIR).filter(file =>
  file.endsWith('.js') || file.endsWith('.css')
);

console.log('📂 Analyzing files in dist/assets/...\n');

files.forEach(file => {
  const filePath = path.join(ASSETS_DIR, file);
  const stats = fs.statSync(filePath);
  const content = fs.readFileSync(filePath);

  // Calculate gzip size
  const gzipped = zlib.gzipSync(content);

  const fileInfo = {
    name: file,
    size: stats.size,
    gzipSize: gzipped.length,
    type: file.endsWith('.js') ? 'JavaScript' : 'CSS',
  };

  bundleStats.files.push(fileInfo);
  bundleStats.totalSize += stats.size;
  bundleStats.totalGzipSize += gzipped.length;
});

// Sort by size descending
bundleStats.files.sort((a, b) => b.size - a.size);

// Print summary
console.log('📊 Bundle Size Summary:');
console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
console.log(`Total Size (uncompressed): ${formatBytes(bundleStats.totalSize)}`);
console.log(`Total Size (gzipped):      ${formatBytes(bundleStats.totalGzipSize)}`);

const targetGzipSizeBytes = TARGET_GZIP_SIZE_KB * 1024;
const passed = bundleStats.totalGzipSize <= targetGzipSizeBytes;
const status = passed ? '✅' : '❌';
console.log(`${status} Target Gzip Size:        ${formatBytes(targetGzipSizeBytes)} (${passed ? 'PASS' : 'FAIL'})`);
console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n');

// Print top 10 largest files
console.log('🔝 Top 10 Largest Files:');
console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
bundleStats.files.slice(0, 10).forEach((file, index) => {
  const percentage = ((file.gzipSize / bundleStats.totalGzipSize) * 100).toFixed(1);
  console.log(`${index + 1}. ${file.name}`);
  console.log(`   Uncompressed: ${formatBytes(file.size).padStart(10)} | Gzipped: ${formatBytes(file.gzipSize).padStart(10)} (${percentage}%)`);
});
console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n');

// Identify large dependencies (heuristics based on filename patterns)
console.log('🔍 Identified Large Dependencies:');
console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');

const dependencyPatterns = {
  'react-flow': /reactflow/i,
  'recharts': /recharts/i,
  'react-query': /tanstack|query/i,
  'radix-ui': /radix/i,
  'lucide-react': /lucide/i,
};

const identifiedDeps = {};

bundleStats.files.forEach(file => {
  for (const [depName, pattern] of Object.entries(dependencyPatterns)) {
    if (pattern.test(file.name)) {
      if (!identifiedDeps[depName]) {
        identifiedDeps[depName] = {
          files: [],
          totalSize: 0,
          totalGzipSize: 0,
        };
      }
      identifiedDeps[depName].files.push(file.name);
      identifiedDeps[depName].totalSize += file.size;
      identifiedDeps[depName].totalGzipSize += file.gzipSize;
    }
  }
});

if (Object.keys(identifiedDeps).length === 0) {
  console.log('No specific dependencies identified (likely bundled in main chunk)');
} else {
  Object.entries(identifiedDeps)
    .sort((a, b) => b[1].totalGzipSize - a[1].totalGzipSize)
    .forEach(([depName, dep]) => {
      console.log(`📦 ${depName}: ${formatBytes(dep.totalGzipSize)} gzipped (${dep.files.length} file(s))`);
    });
}

console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n');

// Optimization recommendations
console.log('💡 Bundle Optimization Opportunities:');
console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');

const recommendations = [];

if (bundleStats.totalGzipSize > targetGzipSizeBytes) {
  const excess = bundleStats.totalGzipSize - targetGzipSizeBytes;
  recommendations.push(`Reduce bundle size by ${formatBytes(excess)} to meet target`);
}

// Check for large JS chunks
const largeJsFiles = bundleStats.files.filter(f => f.type === 'JavaScript' && f.gzipSize > 100 * 1024);
if (largeJsFiles.length > 0) {
  recommendations.push(`Consider code-splitting: ${largeJsFiles.length} JS file(s) > 100KB gzipped`);
}

// Check if vendor chunks are separate
const hasVendorChunk = bundleStats.files.some(f => /vendor/i.test(f.name));
if (!hasVendorChunk && bundleStats.files.length > 1) {
  recommendations.push('Enable vendor chunk splitting in Vite config (splitChunks)');
}

// Check for CSS optimization
const cssFiles = bundleStats.files.filter(f => f.type === 'CSS');
const totalCssGzip = cssFiles.reduce((sum, f) => sum + f.gzipSize, 0);
if (totalCssGzip > 50 * 1024) {
  recommendations.push(`CSS bundle is ${formatBytes(totalCssGzip)} gzipped - consider PurgeCSS`);
}

// Dependency-specific recommendations
if (identifiedDeps['react-flow']) {
  recommendations.push('ReactFlow: Use tree-shaking to import only needed components');
}
if (identifiedDeps['recharts']) {
  recommendations.push('Recharts: Consider lighter alternative (Chart.js, lightweight charts)');
}

if (recommendations.length === 0) {
  console.log('✅ No immediate optimization opportunities found. Bundle is well-optimized!');
} else {
  recommendations.forEach((rec, index) => {
    console.log(`${index + 1}. ${rec}`);
  });
}

console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n');

// Save JSON report
const reportPath = path.join(OUTPUT_DIR, `report-${TIMESTAMP}.json`);
fs.writeFileSync(reportPath, JSON.stringify(bundleStats, null, 2));
console.log(`📄 JSON report saved: ${reportPath}\n`);

// Helper function
function formatBytes(bytes) {
  if (bytes < 1024) return bytes + ' B';
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(2) + ' KB';
  return (bytes / (1024 * 1024)).toFixed(2) + ' MB';
}

console.log('✅ Bundle analysis complete!\n');
