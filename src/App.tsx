import { useState, lazy, Suspense, useEffect } from 'react'
import { HashRouter as Router, Routes, Route, useLocation, useNavigate } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ThemeProvider } from './components/theme-provider'

// Lazy load framer-motion for better initial load performance
import { AnimatePresence, motion } from 'framer-motion'

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
import { Skeleton } from './components/ui/skeleton'
import OfflineDetector from './components/OfflineDetector'

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      refetchOnWindowFocus: false,
      retry: (failureCount, error) => {
        // Retry up to 3 times for network errors
        if (error instanceof Error && error.message.includes('Network')) {
          return failureCount < 3;
        }
        // Don't retry for other errors (4xx, 5xx)
        return false;
      },
      retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),
      refetchOnReconnect: true, // Auto-refetch when back online
      staleTime: 30000, // 30 seconds
    },
  },
})

// Animation variants for page transitions
const pageVariants = {
  initial: {
    opacity: 0,
    y: 20,
  },
  animate: {
    opacity: 1,
    y: 0,
  },
  exit: {
    opacity: 0,
    y: -20,
  },
}

const pageTransition = {
  type: 'tween' as const,
  ease: 'anticipate' as const,
  duration: 0.3,
}

// Wrapper component to use useLocation hook
function AnimatedRoutes() {
  const location = useLocation()
  const navigate = useNavigate()

  // System Tray "설정" 메뉴 클릭 이벤트 리스너
  useEffect(() => {
    const listenForTrayEvents = async () => {
      const { listen } = await import('@tauri-apps/api/event')
      const unlisten = await listen('navigate-to-settings', () => {
        navigate('/settings')
      })
      return unlisten
    }

    let unlisten: (() => void) | null = null
    listenForTrayEvents().then((fn) => { unlisten = fn })

    return () => {
      if (unlisten) unlisten()
    }
  }, [navigate])

  return (
    <AnimatePresence mode="wait">
      <Routes location={location} key={location.pathname}>
        <Route
          path="/"
          element={
            <motion.div
              initial="initial"
              animate="animate"
              exit="exit"
              variants={pageVariants}
              transition={pageTransition}
            >
              <ChatInterface />
            </motion.div>
          }
        />
        <Route
          path="/dashboard"
          element={
            <motion.div
              initial="initial"
              animate="animate"
              exit="exit"
              variants={pageVariants}
              transition={pageTransition}
            >
              <Dashboard />
            </motion.div>
          }
        />
        <Route
          path="/workflow"
          element={
            <motion.div
              initial="initial"
              animate="animate"
              exit="exit"
              variants={pageVariants}
              transition={pageTransition}
            >
              <WorkflowBuilder />
            </motion.div>
          }
        />
        <Route
          path="/bi"
          element={
            <motion.div
              initial="initial"
              animate="animate"
              exit="exit"
              variants={pageVariants}
              transition={pageTransition}
            >
              <BiInsights />
            </motion.div>
          }
        />
        <Route
          path="/settings"
          element={
            <motion.div
              initial="initial"
              animate="animate"
              exit="exit"
              variants={pageVariants}
              transition={pageTransition}
            >
              <Settings />
            </motion.div>
          }
        />
      </Routes>
    </AnimatePresence>
  )
}

function App() {
  const [sidebarOpen, setSidebarOpen] = useState(true)

  return (
    <ErrorBoundary>
      <ThemeProvider defaultTheme="system" storageKey="judgify-ui-theme">
        <QueryClientProvider client={queryClient}>
          <Router>
            {/* Skip to content link for keyboard navigation */}
            <a
              href="#main-content"
              className="sr-only focus:not-sr-only focus:absolute focus:top-4 focus:left-4 focus:z-50 focus:px-4 focus:py-2 focus:bg-primary focus:text-primary-foreground focus:rounded-md focus:outline-none focus:ring-2 focus:ring-ring"
            >
              메인 콘텐츠로 건너뛰기
            </a>

            <div className="flex h-screen bg-background">
              {/* Sidebar */}
              <Sidebar isOpen={sidebarOpen} onToggle={() => setSidebarOpen(!sidebarOpen)} />

              {/* Main Content */}
              <div className="flex-1 flex flex-col">
                <Header />

                <main id="main-content" className="flex-1 overflow-auto p-6">
                  <Suspense fallback={
                    <div className="flex flex-col gap-4">
                      <Skeleton className="h-12 w-full" />
                      <Skeleton className="h-64 w-full" />
                      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <Skeleton className="h-32 w-full" />
                        <Skeleton className="h-32 w-full" />
                      </div>
                    </div>
                  }>
                    <AnimatedRoutes />
                  </Suspense>
                </main>
              </div>
            </div>
          </Router>
          <Toaster />
          <OfflineDetector />
        </QueryClientProvider>
      </ThemeProvider>
    </ErrorBoundary>
  )
}

export default App
