#!/usr/bin/env node

/**
 * Criterion.rs Benchmark Report Generator
 *
 * Parses Criterion benchmark results and generates:
 * 1. JSON summary for GitHub Actions
 * 2. Regression detection (>10% slowdown)
 * 3. Performance improvements tracking
 */

const fs = require('fs');
const path = require('path');

const REGRESSION_THRESHOLD = 10.0; // 10% slowdown triggers warning
const IMPROVEMENT_THRESHOLD = 5.0; // 5% speedup is notable

/**
 * Parse Criterion's estimates.json files
 */
function parseCriterionResults(criterionDir) {
  const benchmarks = [];

  if (!fs.existsSync(criterionDir)) {
    console.log('No criterion directory found. This is the first run.');
    return benchmarks;
  }

  // Find all benchmark directories
  const findBenchmarks = (dir) => {
    const entries = fs.readdirSync(dir, { withFileTypes: true });

    for (const entry of entries) {
      const fullPath = path.join(dir, entry.name);

      if (entry.isDirectory()) {
        // Check if this directory has estimates
        const newEstimates = path.join(fullPath, 'new', 'estimates.json');
        const baseEstimates = path.join(fullPath, 'base', 'estimates.json');

        if (fs.existsSync(newEstimates)) {
          const newData = JSON.parse(fs.readFileSync(newEstimates, 'utf8'));

          let benchmark = {
            name: entry.name,
            current_mean: formatTime(newData.mean.point_estimate),
            current_mean_ns: newData.mean.point_estimate,
            baseline_mean: null,
            baseline_mean_ns: null,
            change_pct: 0
          };

          // Compare with baseline if available
          if (fs.existsSync(baseEstimates)) {
            const baseData = JSON.parse(fs.readFileSync(baseEstimates, 'utf8'));
            benchmark.baseline_mean = formatTime(baseData.mean.point_estimate);
            benchmark.baseline_mean_ns = baseData.mean.point_estimate;

            // Calculate percentage change
            const changePct = ((newData.mean.point_estimate - baseData.mean.point_estimate) / baseData.mean.point_estimate) * 100;
            benchmark.change_pct = changePct;
          }

          benchmarks.push(benchmark);
        }

        // Recurse into subdirectories
        findBenchmarks(fullPath);
      }
    }
  };

  findBenchmarks(criterionDir);
  return benchmarks;
}

/**
 * Format nanoseconds to human-readable time
 */
function formatTime(ns) {
  if (ns < 1000) {
    return `${ns.toFixed(2)} ns`;
  } else if (ns < 1000000) {
    return `${(ns / 1000).toFixed(2)} µs`;
  } else if (ns < 1000000000) {
    return `${(ns / 1000000).toFixed(2)} ms`;
  } else {
    return `${(ns / 1000000000).toFixed(2)} s`;
  }
}

/**
 * Detect regressions and improvements
 */
function analyzeBenchmarks(benchmarks) {
  const regressions = benchmarks.filter(b => b.change_pct > REGRESSION_THRESHOLD);
  const improvements = benchmarks.filter(b => b.change_pct < -IMPROVEMENT_THRESHOLD);

  return {
    regressions,
    improvements,
    total: benchmarks.length,
    has_regressions: regressions.length > 0
  };
}

/**
 * Main execution
 */
function main() {
  const criterionDir = path.join(process.cwd(), 'src-tauri', 'target', 'criterion');

  console.log('📊 Parsing Criterion benchmark results...');
  const benchmarks = parseCriterionResults(criterionDir);

  if (benchmarks.length === 0) {
    console.log('⚠️ No benchmarks found. This might be the first run.');

    // Create minimal summary
    const summary = {
      benchmarks: [],
      regressions: [],
      improvements: [],
      total: 0,
      has_regressions: false
    };

    fs.writeFileSync('benchmark-summary.json', JSON.stringify(summary, null, 2));
    return;
  }

  console.log(`✅ Found ${benchmarks.length} benchmarks`);

  const analysis = analyzeBenchmarks(benchmarks);

  // Generate summary
  const summary = {
    benchmarks,
    regressions: analysis.regressions,
    improvements: analysis.improvements,
    total: analysis.total,
    has_regressions: analysis.has_regressions
  };

  fs.writeFileSync('benchmark-summary.json', JSON.stringify(summary, null, 2));
  console.log('📄 Summary written to benchmark-summary.json');

  // Report to console
  if (analysis.regressions.length > 0) {
    console.log('\n⚠️ Performance Regressions Detected:');
    analysis.regressions.forEach(r => {
      console.log(`  - ${r.name}: ${r.change_pct.toFixed(2)}% slower (${r.baseline_mean} → ${r.current_mean})`);
    });

    // Create flag file for CI to detect
    fs.writeFileSync('regression-detected.flag', '1');
  }

  if (analysis.improvements.length > 0) {
    console.log('\n🚀 Performance Improvements:');
    analysis.improvements.forEach(i => {
      console.log(`  - ${i.name}: ${Math.abs(i.change_pct).toFixed(2)}% faster (${i.baseline_mean} → ${i.current_mean})`);
    });
  }

  if (analysis.regressions.length === 0 && analysis.improvements.length === 0) {
    console.log('\n✅ No significant performance changes detected.');
  }
}

main();
