import { useState, lazy, Suspense } from 'react'
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ThemeProvider } from './components/theme-provider'

// Pages (lazy loaded for code splitting)
const ChatInterface = lazy(() => import('./pages/ChatInterface'))
const Dashboard = lazy(() => import('./pages/Dashboard'))
const WorkflowBuilder = lazy(() => import('./pages/WorkflowBuilder'))
const BiInsights = lazy(() => import('./pages/BiInsights'))
const Settings = lazy(() => import('./pages/Settings'))

// Layout (eager loaded - needed immediately)
import Sidebar from './components/layout/Sidebar'
import Header from './components/layout/Header'
import ErrorBoundary from './components/ErrorBoundary'
import { Toaster } from './components/ui/toaster'

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      refetchOnWindowFocus: false,
      retry: 1,
    },
  },
})

function App() {
  const [sidebarOpen, setSidebarOpen] = useState(true)

  return (
    <ErrorBoundary>
      <ThemeProvider defaultTheme="system" storageKey="judgify-ui-theme">
        <QueryClientProvider client={queryClient}>
          <Router>
            <div className="flex h-screen bg-background">
              {/* Sidebar */}
              <Sidebar isOpen={sidebarOpen} onToggle={() => setSidebarOpen(!sidebarOpen)} />

              {/* Main Content */}
              <div className="flex-1 flex flex-col">
                <Header />

                <main className="flex-1 overflow-auto p-6">
                  <Suspense fallback={<div className="flex items-center justify-center h-full">Loading...</div>}>
                    <Routes>
                      <Route path="/" element={<ChatInterface />} />
                      <Route path="/dashboard" element={<Dashboard />} />
                      <Route path="/workflow" element={<WorkflowBuilder />} />
                      <Route path="/bi" element={<BiInsights />} />
                      <Route path="/settings" element={<Settings />} />
                    </Routes>
                  </Suspense>
                </main>
              </div>
            </div>
          </Router>
          <Toaster />
        </QueryClientProvider>
      </ThemeProvider>
    </ErrorBoundary>
  )
}

export default App
