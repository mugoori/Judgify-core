import { useState } from 'react'
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ThemeProvider } from './components/theme-provider'

// Pages
import ChatInterface from './pages/ChatInterface'
import Dashboard from './pages/Dashboard'
import WorkflowBuilder from './pages/WorkflowBuilder'
import BiInsights from './pages/BiInsights'
import Settings from './pages/Settings'

// Layout
import Sidebar from './components/layout/Sidebar'
import Header from './components/layout/Header'

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
                <Routes>
                  <Route path="/" element={<ChatInterface />} />
                  <Route path="/dashboard" element={<Dashboard />} />
                  <Route path="/workflow" element={<WorkflowBuilder />} />
                  <Route path="/bi" element={<BiInsights />} />
                  <Route path="/settings" element={<Settings />} />
                </Routes>
              </main>
            </div>
          </div>
        </Router>
      </QueryClientProvider>
    </ThemeProvider>
  )
}

export default App
