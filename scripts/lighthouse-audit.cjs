/**
 * Lighthouse Performance Audit Script
 *
 * Automates Lighthouse CLI to audit the Tauri desktop app's frontend performance.
 * Targets the Vite dev server at http://localhost:1420
 *
 * Usage:
 *   1. Start dev server: npm run tauri:dev
 *   2. In separate terminal: node scripts/lighthouse-audit.js
 *
 * Outputs:
 *   - lighthouse/report-{timestamp}.html (detailed report)
 *   - lighthouse/report-{timestamp}.json (raw data)
 */

const { exec } = require('child_process');
const fs = require('fs');
const path = require('path');

// Configuration
const TARGET_URL = 'http://localhost:1420';
const OUTPUT_DIR = path.join(__dirname, '..', 'lighthouse');
const TIMESTAMP = new Date().toISOString().replace(/[:.]/g, '-').split('T')[0];

// Performance targets (from TASKS.md Task 1.3)
const TARGETS = {
  performance: 90,          // Lighthouse Performance Score
  fcp: 1500,                // First Contentful Paint (ms)
  tti: 3000,                // Time to Interactive (ms)
  tbt: 200,                 // Total Blocking Time (ms)
  cls: 0.1,                 // Cumulative Layout Shift
};

// Ensure output directory exists
if (!fs.existsSync(OUTPUT_DIR)) {
  fs.mkdirSync(OUTPUT_DIR, { recursive: true });
}

// Lighthouse CLI command
const outputPath = path.join(OUTPUT_DIR, `report-${TIMESTAMP}`);
const lighthouseCmd = `npx lighthouse ${TARGET_URL} \
  --output html \
  --output json \
  --output-path ${outputPath} \
  --chrome-flags="--headless --disable-gpu" \
  --only-categories=performance \
  --throttling-method=simulate \
  --preset=desktop`;

console.log('🚀 Starting Lighthouse Performance Audit...');
console.log(`📍 Target: ${TARGET_URL}`);
console.log(`📂 Output: ${outputPath}.html\n`);

// Check if dev server is running
const http = require('http');
http.get(TARGET_URL, (res) => {
  if (res.statusCode === 200) {
    console.log('✅ Dev server is running. Starting audit...\n');
    runLighthouse();
  } else {
    console.error('❌ Dev server returned status code:', res.statusCode);
    console.error('Please ensure "npm run tauri:dev" is running.');
    process.exit(1);
  }
}).on('error', (err) => {
  console.error('❌ Cannot connect to dev server at', TARGET_URL);
  console.error('Please start the dev server first: npm run tauri:dev');
  process.exit(1);
});

function runLighthouse() {
  exec(lighthouseCmd, { maxBuffer: 1024 * 1024 * 10 }, (error, stdout, stderr) => {
    if (error) {
      console.error('❌ Lighthouse audit failed:', error.message);
      process.exit(1);
    }

    console.log(stdout);

    // Parse JSON report
    const jsonReportPath = `${outputPath}.report.json`;
    if (fs.existsSync(jsonReportPath)) {
      const report = JSON.parse(fs.readFileSync(jsonReportPath, 'utf8'));

      // Extract key metrics
      const performanceScore = Math.round(report.categories.performance.score * 100);
      const audits = report.audits;

      const fcp = Math.round(audits['first-contentful-paint'].numericValue);
      const tti = Math.round(audits['interactive'].numericValue);
      const tbt = Math.round(audits['total-blocking-time'].numericValue);
      const cls = audits['cumulative-layout-shift'].numericValue.toFixed(3);

      console.log('\n📊 Performance Metrics:');
      console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
      printMetric('Performance Score', performanceScore, TARGETS.performance, '%', 'higher');
      printMetric('First Contentful Paint', fcp, TARGETS.fcp, 'ms', 'lower');
      printMetric('Time to Interactive', tti, TARGETS.tti, 'ms', 'lower');
      printMetric('Total Blocking Time', tbt, TARGETS.tbt, 'ms', 'lower');
      printMetric('Cumulative Layout Shift', parseFloat(cls), TARGETS.cls, '', 'lower');
      console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n');

      // Check pass/fail
      const passed =
        performanceScore >= TARGETS.performance &&
        fcp <= TARGETS.fcp &&
        tti <= TARGETS.tti &&
        tbt <= TARGETS.tbt &&
        parseFloat(cls) <= TARGETS.cls;

      if (passed) {
        console.log('✅ All performance targets met!');
      } else {
        console.log('⚠️  Some performance targets not met. See recommendations in HTML report.');
      }

      console.log(`\n📄 Full report: ${outputPath}.report.html`);
      console.log(`📄 JSON data: ${outputPath}.report.json`);
    } else {
      console.error('⚠️  JSON report not found at:', jsonReportPath);
    }
  });
}

function printMetric(name, value, target, unit, direction) {
  const passed = direction === 'higher' ? value >= target : value <= target;
  const status = passed ? '✅' : '❌';
  const valueStr = unit === '%' ? `${value}${unit}` : `${value}${unit}`;
  const targetStr = unit === '%' ? `${target}${unit}` : `${target}${unit}`;

  console.log(`${status} ${name.padEnd(30)} ${valueStr.padStart(10)} (target: ${targetStr})`);
}
