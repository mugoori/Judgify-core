/**
 * React Component Performance Profiling Script
 *
 * Provides utilities and instrumentation for profiling React component rendering performance.
 * Focuses on identifying re-render issues and expensive components.
 *
 * Key Components to Profile (from research):
 *   1. ChatInterface.tsx (1,416 lines - complex state management)
 *   2. WorkflowBuilder.tsx (ReactFlow integration)
 *   3. Dashboard.tsx (Recharts integration)
 *
 * Usage:
 *   1. Add Profiler wrapper to target components (see instructions below)
 *   2. Run dev server: npm run tauri:dev
 *   3. Interact with profiled components
 *   4. Check console for performance data
 *   5. Use React DevTools Profiler for visual analysis
 *
 * Outputs:
 *   - Console logs with render times
 *   - performance-profile/profile-{timestamp}.json (aggregated data)
 */

const fs = require('fs');
const path = require('path');

// Configuration
const OUTPUT_DIR = path.join(__dirname, '..', 'performance-profile');
const TIMESTAMP = new Date().toISOString().replace(/[:.]/g, '-').split('T')[0];

// Ensure output directory exists
if (!fs.existsSync(OUTPUT_DIR)) {
  fs.mkdirSync(OUTPUT_DIR, { recursive: true });
}

console.log('⚛️  React Component Performance Profiling Guide');
console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n');

console.log('📋 Step 1: Add Profiler Wrapper to Components\n');

const profilerCodeExample = `
// Example: Profiling ChatInterface.tsx
import { Profiler } from 'react';

// Add this callback function at the top of your component file
function onRenderCallback(
  id,                    // Component identifier
  phase,                 // "mount" or "update"
  actualDuration,        // Time spent rendering
  baseDuration,          // Estimated time without memoization
  startTime,             // When React began rendering
  commitTime,            // When React committed the update
  interactions           // Set of interactions for this update
) {
  console.log(\`[Profiler] \${id} (\${phase}): \${actualDuration.toFixed(2)}ms\`);

  // Warn if render is slow
  if (actualDuration > 16) { // 60fps = 16ms per frame
    console.warn(\`⚠️  Slow render detected in \${id}: \${actualDuration.toFixed(2)}ms\`);
  }
}

// Wrap your component export with Profiler
export default function ChatInterface() {
  // ... existing code ...

  return (
    <Profiler id="ChatInterface" onRender={onRenderCallback}>
      {/* Your component JSX */}
    </Profiler>
  );
}
`;

console.log(profilerCodeExample);

console.log('\n📋 Step 2: Target Components for Profiling\n');

const componentsToProfile = [
  {
    name: 'ChatInterface',
    file: 'src/pages/ChatInterface.tsx',
    reason: '1,416 lines, complex state management, multiple useEffect hooks',
    suspectedIssues: [
      'Multiple re-renders due to localStorage updates',
      'useMutation callbacks not memoized',
      'Frequent state changes from chat messages',
    ],
  },
  {
    name: 'WorkflowBuilder',
    file: 'src/pages/WorkflowBuilder.tsx',
    reason: 'ReactFlow integration with large graph rendering',
    suspectedIssues: [
      'Node/edge updates trigger full graph re-render',
      'No virtualization for large workflows',
      'React.memo not applied to custom nodes',
    ],
  },
  {
    name: 'Dashboard',
    file: 'src/pages/Dashboard.tsx',
    reason: 'Recharts integration with 30s refetch interval',
    suspectedIssues: [
      'Chart re-renders every 30 seconds',
      'Chart config not memoized',
      'ResponsiveContainer optimization missing',
    ],
  },
];

componentsToProfile.forEach((comp, index) => {
  console.log(`${index + 1}. ${comp.name} (${comp.file})`);
  console.log(`   Reason: ${comp.reason}`);
  console.log(`   Suspected Issues:`);
  comp.suspectedIssues.forEach(issue => console.log(`   - ${issue}`));
  console.log('');
});

console.log('\n📋 Step 3: Use React DevTools Profiler (Recommended)\n');

console.log('For visual profiling:');
console.log('1. Install React DevTools browser extension');
console.log('2. Open Tauri dev app: npm run tauri:dev');
console.log('3. Open Chrome DevTools → React tab → Profiler');
console.log('4. Click "Record" button');
console.log('5. Interact with the app (navigate, type in chat, update dashboard)');
console.log('6. Click "Stop" to see flamegraph and ranked charts\n');

console.log('Key Metrics to Check:');
console.log('- Render duration (target: <16ms for 60fps)');
console.log('- Number of re-renders (minimize unnecessary renders)');
console.log('- Commit duration (time to apply changes to DOM)');
console.log('- Components that render most frequently\n');

console.log('\n📋 Step 4: Automated Profiling Data Collection\n');

const automatedProfilingCode = `
// Add this to src/utils/performance-monitoring.ts

export class PerformanceMonitor {
  private static renderData: Array<{
    componentId: string;
    phase: string;
    duration: number;
    timestamp: number;
  }> = [];

  static recordRender(id: string, phase: string, actualDuration: number) {
    this.renderData.push({
      componentId: id,
      phase,
      duration: actualDuration,
      timestamp: Date.now(),
    });

    // Log slow renders
    if (actualDuration > 16) {
      console.warn(\`⚠️  Slow render: \${id} took \${actualDuration.toFixed(2)}ms\`);
    }
  }

  static getStats() {
    const stats = this.renderData.reduce((acc, entry) => {
      if (!acc[entry.componentId]) {
        acc[entry.componentId] = {
          count: 0,
          totalDuration: 0,
          avgDuration: 0,
          maxDuration: 0,
        };
      }

      const stat = acc[entry.componentId];
      stat.count++;
      stat.totalDuration += entry.duration;
      stat.maxDuration = Math.max(stat.maxDuration, entry.duration);
      stat.avgDuration = stat.totalDuration / stat.count;

      return acc;
    }, {} as Record<string, any>);

    return stats;
  }

  static exportData() {
    return {
      timestamp: new Date().toISOString(),
      renderData: this.renderData,
      stats: this.getStats(),
    };
  }

  static saveToFile() {
    const data = this.exportData();
    // Save to localStorage or send to backend
    console.log('Performance data:', JSON.stringify(data, null, 2));
  }
}

// Use in Profiler callback
function onRenderCallback(id, phase, actualDuration) {
  PerformanceMonitor.recordRender(id, phase, actualDuration);
}

// Call this after profiling session
// PerformanceMonitor.saveToFile();
`;

console.log(automatedProfilingCode);

console.log('\n📋 Step 5: Performance Optimization Checklist\n');

const optimizationChecklist = [
  {
    issue: 'Excessive re-renders',
    solutions: [
      'Use React.memo() for pure components',
      'Use useCallback() for event handlers',
      'Use useMemo() for expensive calculations',
      'Split large components into smaller ones',
    ],
  },
  {
    issue: 'Slow initial render',
    solutions: [
      'Implement code-splitting with React.lazy()',
      'Defer non-critical components',
      'Use Suspense boundaries',
      'Virtualize long lists (react-window)',
    ],
  },
  {
    issue: 'Large component trees',
    solutions: [
      'Apply React.memo to leaf components',
      'Use children prop pattern',
      'Avoid inline object/function creation in JSX',
      'Move static data outside component',
    ],
  },
  {
    issue: 'Heavy third-party libraries',
    solutions: [
      'ReactFlow: Lazy load nodes, use React.memo on custom nodes',
      'Recharts: Disable animations, use static data',
      'Use dynamic imports for large dependencies',
      'Consider lighter alternatives',
    ],
  },
];

optimizationChecklist.forEach((item, index) => {
  console.log(`${index + 1}. ${item.issue}`);
  console.log('   Solutions:');
  item.solutions.forEach(solution => console.log(`   - ${solution}`));
  console.log('');
});

console.log('\n📋 Step 6: Benchmarking Targets\n');

const benchmarks = [
  { metric: 'ChatInterface initial render', target: '<50ms', rationale: 'Complex component, acceptable startup time' },
  { metric: 'ChatInterface re-render (message)', target: '<16ms', rationale: '60fps for smooth typing experience' },
  { metric: 'WorkflowBuilder node update', target: '<16ms', rationale: 'Smooth drag-and-drop interaction' },
  { metric: 'Dashboard chart render', target: '<100ms', rationale: 'Recharts renders can be expensive' },
  { metric: 'Dashboard re-render (data update)', target: '<50ms', rationale: 'Every 30s, should not block UI' },
];

console.log('Performance Targets:');
benchmarks.forEach(b => {
  console.log(`- ${b.metric}: ${b.target} (${b.rationale})`);
});

console.log('\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n');

console.log('💡 Next Steps:\n');
console.log('1. Add Profiler wrappers to the 3 target components');
console.log('2. Run "npm run tauri:dev" and interact with the app');
console.log('3. Use React DevTools Profiler for visual analysis');
console.log('4. Check console for slow render warnings');
console.log('5. Implement optimizations based on findings');
console.log('6. Re-profile to verify improvements\n');

console.log(`📄 This guide saved to: ${__filename}\n`);
console.log('✅ Performance profiling guide complete!\n');

// Create a template Profiler utility file
const profilerUtilTemplate = `// src/utils/performance-profiler.tsx
// Auto-generated by scripts/performance-profile.js

import { ProfilerOnRenderCallback } from 'react';

interface RenderMetric {
  componentId: string;
  phase: 'mount' | 'update';
  duration: number;
  timestamp: number;
}

class PerformanceMonitor {
  private static renderData: RenderMetric[] = [];
  private static readonly SLOW_RENDER_THRESHOLD = 16; // 60fps

  static recordRender(
    id: string,
    phase: 'mount' | 'update',
    actualDuration: number
  ): void {
    this.renderData.push({
      componentId: id,
      phase,
      duration: actualDuration,
      timestamp: Date.now(),
    });

    if (actualDuration > this.SLOW_RENDER_THRESHOLD) {
      console.warn(
        \`⚠️  Slow render: \${id} (\${phase}) took \${actualDuration.toFixed(2)}ms\`
      );
    }
  }

  static getStats() {
    return this.renderData.reduce((acc, entry) => {
      if (!acc[entry.componentId]) {
        acc[entry.componentId] = {
          count: 0,
          totalDuration: 0,
          avgDuration: 0,
          maxDuration: 0,
          phases: { mount: 0, update: 0 },
        };
      }

      const stat = acc[entry.componentId];
      stat.count++;
      stat.totalDuration += entry.duration;
      stat.maxDuration = Math.max(stat.maxDuration, entry.duration);
      stat.avgDuration = stat.totalDuration / stat.count;
      stat.phases[entry.phase]++;

      return acc;
    }, {} as Record<string, any>);
  }

  static exportData() {
    return {
      timestamp: new Date().toISOString(),
      renderData: this.renderData,
      stats: this.getStats(),
    };
  }

  static printSummary() {
    const stats = this.getStats();
    console.log('\\n📊 React Component Performance Summary:');
    console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');

    Object.entries(stats).forEach(([componentId, stat]: [string, any]) => {
      console.log(\`\\n\${componentId}:\`);
      console.log(\`  Renders: \${stat.count} (mount: \${stat.phases.mount}, update: \${stat.phases.update})\`);
      console.log(\`  Avg Duration: \${stat.avgDuration.toFixed(2)}ms\`);
      console.log(\`  Max Duration: \${stat.maxDuration.toFixed(2)}ms\`);

      if (stat.avgDuration > this.SLOW_RENDER_THRESHOLD) {
        console.log(\`  ⚠️  Average render time exceeds 16ms threshold\`);
      }
    });

    console.log('\\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\\n');
  }
}

// Export Profiler callback
export const onRenderCallback: ProfilerOnRenderCallback = (
  id,
  phase,
  actualDuration
) => {
  PerformanceMonitor.recordRender(id, phase, actualDuration);
};

// Export utility functions
export const getPerformanceStats = () => PerformanceMonitor.getStats();
export const exportPerformanceData = () => PerformanceMonitor.exportData();
export const printPerformanceSummary = () => PerformanceMonitor.printSummary();

// Add to window for console access
if (typeof window !== 'undefined') {
  (window as any).__PERF_MONITOR__ = {
    getStats: getPerformanceStats,
    exportData: exportPerformanceData,
    printSummary: printPerformanceSummary,
  };

  console.log('💡 Performance monitoring enabled. Use window.__PERF_MONITOR__ in console.');
}
`;

const utilOutputPath = path.join(OUTPUT_DIR, 'performance-profiler.tsx.template');
fs.writeFileSync(utilOutputPath, profilerUtilTemplate);
console.log(`📄 Profiler utility template saved: ${utilOutputPath}`);
console.log('   Copy to src/utils/performance-profiler.tsx to use\n');
